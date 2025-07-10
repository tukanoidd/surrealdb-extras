use std::fmt::Display;

use darling::{
    FromDeriveInput, FromField,
    ast::Data,
    util::{Flag, Ignored},
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    AngleBracketedGenericArguments, Expr, Ident, LitStr, PathSegment, Type, TypeParen, TypePath,
    spanned::Spanned,
};

use crate::util::DeriveInputUtil;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct SurrealSelect {
    ident: Ident,
    data: Data<Ignored, SurrealSelectTableField>,
}

impl DeriveInputUtil for SurrealSelect {
    fn gen_(&self) -> manyhow::Result<TokenStream> {
        let Self { ident, data } = self;

        let fields = match data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let keys = [Ident::new("id", Span::mixed_site())].into_iter().chain(
            fields
                .iter()
                .filter(|&f| (!f.exclude.is_present()))
                .map(|f| f.field_name().clone()),
        );

        Ok(quote! {
                impl surrealdb_extras::SurrealSelectInfo for #ident {
                fn keys()-> &'static [&'static str] {
                    &[#( stringify!(#keys) ),*]
                }
            }
        })
    }
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(table))]
pub struct SurrealTable {
    ident: Ident,
    data: Data<Ignored, SurrealSelectTableField>,

    sql: Option<Vec<LitStr>>,
    db: Ident,
}

impl DeriveInputUtil for SurrealTable {
    fn gen_(&self) -> manyhow::Result<TokenStream> {
        let Self {
            ident,
            data,

            sql,
            db,
        } = self;

        let keys = SurrealSelect {
            ident: ident.clone(),
            data: data.clone(),
        }
        .gen_()?;

        let fields = match data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let exc = fields
            .iter()
            .filter(|&f| f.exclude.is_present())
            .map(SurrealSelectTableField::field_name);

        let mut err_emitter = manyhow::Emitter::new();

        let sql = sql
            .iter()
            .flatten()
            .map(|x| quote!(surrealdb_extras::sql!(#x).into()));

        let define_field_queries = fields
            .iter()
            .map(|f| {
                let name = f.field_name();
                let ty = f
                    .db_type
                    .as_ref()
                    .map(|t| manyhow::Result::Ok(t.to_token_stream()))
                    .unwrap_or_else(|| -> manyhow::Result<_> {
                        let ty = f.surreal_ty()?;
                        Ok(ty.to_token_stream())
                    })?;

                let lit_sql_str = LitStr::new(
                    &format!("DEFINE FIELD {name} ON TABLE {db} TYPE {ty}"),
                    name.span(),
                );

                Ok(quote!(surrealdb_extras::sql!(#lit_sql_str).into()))
            })
            .collect::<manyhow::Result<Vec<_>>>()?;

        err_emitter.into_result()?;

        let attr = [{
            let str = LitStr::new(&format!("DEFINE TABLE {db}"), db.span());
            quote!(surrealdb_extras::sql!(#str).into())
        }]
        .into_iter()
        .chain(define_field_queries)
        .chain(sql);

        Ok(quote! {
            #keys

            impl surrealdb_extras::SurrealTableInfo for #ident {
                fn name() -> &'static str {
                    stringify!(#db)
                }

                fn path() -> &'static str {
                    std::any::type_name::<#ident>()
                }

                fn exclude() -> &'static [&'static str] {
                    &[#( #exc ),*]
                }

                fn funcs() ->  Vec<String>{
                    vec![#( #attr ),*]
                }
            }

            impl #ident {
                pub fn add<'a: 'b, 'b, D: surrealdb::Connection>(
                    self,
                    conn: &'a surrealdb::Surreal<D>
                )-> surrealdb::method::Content<'b, D, Option<surrealdb_extras::RecordData<#ident>>> {
                    conn.create(stringify!(#db)).content(self)
                }

                pub fn insert<'a: 'b, 'b, D: surrealdb::Connection>(
                    self,
                    conn: &'a surrealdb::Surreal<D>,
                    id: surrealdb::RecordIdKey
                )-> surrealdb::method::Content<'b, D, Option<surrealdb_extras::RecordData<#ident>>> {
                    conn.create((stringify!(#db), id)).content(self)
                }
            }
        })
    }
}

#[derive(Clone, FromField)]
#[darling(attributes(opt))]
struct SurrealSelectTableField {
    ident: Option<Ident>,
    ty: Type,

    rename: Option<Ident>,
    // TODO: Support more complex types
    db_type: Option<Ident>,
    exclude: Flag,
}

impl SurrealSelectTableField {
    fn field_name(&self) -> &Ident {
        self.rename
            .as_ref()
            .unwrap_or_else(|| self.ident.as_ref().unwrap())
    }

    fn surreal_ty(&self) -> manyhow::Result<SurrealTy> {
        match &self.db_type {
            Some(db_ty) => Ok(db_ty.clone().into()),
            None => Self::from_ty_to_surreal_ty(&self.ty, true),
        }
    }

    fn from_ty_to_surreal_ty(ty: &Type, primary: bool) -> manyhow::Result<SurrealTy> {
        match ty {
            Type::Paren(TypeParen { elem, .. }) => Self::from_ty_to_surreal_ty(elem, primary),
            Type::Path(TypePath { path, .. }) => {
                let PathSegment { ident, arguments } = path
                    .segments
                    .last()
                    .ok_or_else(|| manyhow::error_message!(ty.span(), "Empty path!"))?;

                let gen_tys = match arguments {
                    syn::PathArguments::None => None,
                    syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args,
                        ..
                    }) => Some(
                        args.iter()
                            .map(|arg| match arg {
                                syn::GenericArgument::Type(ty) => {
                                    Ok(SurrealTableFieldTypeArg::Type(Self::from_ty_to_surreal_ty(
                                        ty, false,
                                    )?))
                                }
                                syn::GenericArgument::Const(expr) => {
                                    Ok(SurrealTableFieldTypeArg::Const(expr))
                                }
                                _ => manyhow::bail!(arg.span(), "Argument type not supported!"),
                            })
                            .collect::<manyhow::Result<Vec<_>>>()?,
                    ),
                    syn::PathArguments::Parenthesized(_) => {
                        manyhow::bail!(arguments.span(), "Unsupported arguments!")
                    }
                };

                let primary_ty = match ident.to_string().as_str() {
                    "Vec" => match gen_tys.as_ref().and_then(|tys| {
                        tys.first()
                            .and_then(|ty| match ty {
                                SurrealTableFieldTypeArg::Const(_) => None,
                                SurrealTableFieldTypeArg::Type(ident) => Some(ident),
                            })
                            .map(|ident| ident.to_string())
                    }) {
                        Some(ty) => match ty.as_str() {
                            "u8" => return Ok(Ident::new("bytes", ident.span()).into()),
                            _ => Ident::new("array", ident.span()),
                        },
                        None => Ident::new("array", ident.span()),
                    },
                    "bool" => return Ok(Ident::new("bool", ident.span()).into()),
                    "DateTime" => return Ok(Ident::new("datetime", ident.span()).into()),
                    "Duration" => return Ok(Ident::new("duration", ident.span()).into()),
                    "f128" => return Ok(Ident::new("decimal", ident.span()).into()),
                    "f16" | "f32" | "f64" => return Ok(Ident::new("float", ident.span()).into()),
                    "i8" | "i16" | "i32" | "i64" | "u16" | "u32" => {
                        return Ok(Ident::new("int", ident.span()).into());
                    }
                    "i128" | "u64" | "u128" => return Ok(Ident::new("number", ident.span()).into()),
                    "u8" => {
                        return Ok(Ident::new(
                            match primary {
                                true => "int",
                                false => "u8",
                            },
                            ident.span(),
                        )
                        .into());
                    }
                    "Option" => Ident::new("option", ident.span()),
                    // TODO: add type which record
                    "RecordId" => Ident::new("record", ident.span()),
                    "HashSet" => Ident::new("set", ident.span()),
                    "String" | "PathBuf" => return Ok(Ident::new("string", ident.span()).into()),
                    // TODO: geometry
                    _ => return Ok(Ident::new("object", ident.span()).into()),
                };

                let gens = gen_tys.map(|gen_tys| {
                    quote! {
                        <#(#gen_tys),*>
                    }
                });

                Ok(SurrealTy::Combined(quote!(#primary_ty #gens)))
            }
            _ => manyhow::bail!(ty.span(), "Unsupported type!"),
        }
    }
}

enum SurrealTy {
    Ident(Ident),
    Combined(TokenStream),
}

impl Display for SurrealTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Self::Combined(token_stream) => token_stream.fmt(f),
        }
    }
}

impl From<Ident> for SurrealTy {
    fn from(value: Ident) -> Self {
        Self::Ident(value)
    }
}

impl ToTokens for SurrealTy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Combined(token_stream) => token_stream.to_tokens(tokens),
        }
    }
}

enum SurrealTableFieldTypeArg<'a> {
    Const(&'a Expr),
    Type(SurrealTy),
}

impl<'a> ToTokens for SurrealTableFieldTypeArg<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Const(expr) => expr.to_tokens(tokens),
            Self::Type(ident) => ident.to_tokens(tokens),
        }
    }
}
