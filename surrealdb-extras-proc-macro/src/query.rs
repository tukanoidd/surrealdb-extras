use std::str::FromStr;

use darling::{
    FromDeriveInput, FromField,
    ast::{Data, Style},
    util::{Flag, Ignored},
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Generics, Ident, LitStr, Path, Type, Visibility, spanned::Spanned};

use crate::DeriveInputUtil;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named, struct_tuple), attributes(query))]
pub struct SurrealQuery {
    ident: Ident,
    generics: Generics,
    data: Data<Ignored, DBQueryField>,

    output: Option<LitStr>,
    check: Flag,
    error: Option<Path>,
    sql: LitStr,
}

impl SurrealQuery {
    fn build_query(&self) -> TokenStream {
        let Self { data, check, .. } = self;

        let fields = match data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let binds = DBQueryField::build_query_binds(&fields.fields);

        let res = quote! {
            db.query(Self::QUERY_STR)
                #(#binds)*
                .await?
        };

        match check.is_present() {
            true => quote! {
                #res.check()?;
                Ok(())
            },
            false => quote!(Ok(#res.take::<Self::Output>(0)?)),
        }
    }

    fn build_query_str(&self) -> LitStr {
        let fields = match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let query_str = fields.iter().fold(self.sql.value(), |str, field| {
            let ident = field.ident.as_ref().unwrap();
            str.replace(&format!("{{{ident}}}"), &format!("${ident}"))
        });

        LitStr::new(&query_str, self.sql.span())
    }
}

impl DeriveInputUtil for SurrealQuery {
    fn gen_(&self) -> manyhow::Result<TokenStream> {
        let Self {
            ident,
            generics,
            data,

            output,
            error,
            ..
        } = self;

        let output = output
            .as_ref()
            .map(|ty| {
                TokenStream::from_str(&ty.value())
                    .map_err(|err| manyhow::error_message!(ty.span(), "{err}"))
            })
            .transpose()?
            .unwrap_or_else(|| quote!(()));

        let error = error
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or_else(|| quote!(surrealdb::Error));

        let (impl_gen, ty_gen, where_gen) = generics.split_for_impl();

        let fields = match data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let query_str = self.build_query_str();
        let query = self.build_query();

        let field_names = fields.fields.iter().enumerate().map(|(ind, field)| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| Ident::new(&format!("var{ind}"), field.span()))
        });

        let self_unwrapped = {
            let fields = match fields.style {
                Style::Tuple => quote!((#(#field_names),*)),
                Style::Struct => quote!({#(#field_names),*}),
                Style::Unit => unreachable!(),
            };

            quote!(let Self #fields = self;)
        };

        Ok(quote! {
            impl #impl_gen surrealdb_extras::SurrealQuery for #ident #ty_gen #where_gen {
                type Output = #output;
                type Error = #error;

                const QUERY_STR: &'static str = surrealdb_extras::sql!(#query_str);

                async fn execute<C>(
                    self,
                    db: std::sync::Arc<surrealdb::Surreal<C>>
                ) -> Result<Self::Output, Self::Error> where C: surrealdb::Connection {
                    #self_unwrapped
                    #query
                }
            }
        })
    }
}

#[derive(FromField)]
struct DBQueryField {
    vis: Visibility,
    ident: Option<Ident>,
    ty: Type,
}

impl DBQueryField {
    fn span(&self) -> Span {
        match &self.vis {
            Visibility::Public(pub_) => pub_.span(),
            Visibility::Restricted(vis_restricted) => vis_restricted.span(),
            Visibility::Inherited => match &self.ident {
                Some(ident) => ident.span(),
                None => self.ty.span(),
            },
        }
    }

    fn build_query_binds(list: &[Self]) -> impl Iterator<Item = TokenStream> {
        list.iter().enumerate().map(|(ind, Self { ident, .. })| {
            let ident_str = ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| format!("var{ind}"));
            let ident_lit_str = LitStr::new(&ident_str, ident.span());

            quote!(.bind((#ident_lit_str, #ident)))
        })
    }
}
