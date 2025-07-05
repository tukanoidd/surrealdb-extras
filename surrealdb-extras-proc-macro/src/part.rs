use crate::SurrealTableOverwrite;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_attribute_collector(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let mut attributes = vec!["id".to_string()];
    attributes.append(
        &mut {
            match &mut input.data {
                Data::Struct(data_struct) => match &mut data_struct.fields {
                    Fields::Named(fields_named) => fields_named.named.iter_mut().map(|field| {
                        let renamed: Option<SurrealTableOverwrite> =
                            deluxe::parse_attributes(field).ok();

                        renamed
                            .and_then(|v| v.rename.clone())
                            .unwrap_or(field.ident.as_ref().unwrap().to_string())
                    }),
                    _ => unimplemented!("AttributeCollector only supports structs."),
                },
                _ => unimplemented!("AttributeCollector only supports structs."),
            }
        }
        .collect::<Vec<_>>(),
    );
    let struct_impl = &input.ident;
    let gen_ = quote! {
        impl surrealdb_extras::SurrealSelectInfo for #struct_impl {
            fn keys() -> &'static [&'static str]{
                &[#( #attributes ),*]
            }
        }
    };

    gen_.into()
}
