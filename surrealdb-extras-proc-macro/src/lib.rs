mod table;
mod util;

use proc_macro2::TokenStream;
use quote::ToTokens;
use surrealdb_core::dbs::{Capabilities, capabilities::Targets};
use syn::LitStr;

use crate::{table::SurrealTable, util::DeriveInputUtil};

/// implements SurrealSelectInfo, SurrealTableInfo, add and insert
#[manyhow::manyhow]
#[proc_macro_derive(SurrealTable, attributes(table))]
pub fn table(input: TokenStream) -> manyhow::Result<TokenStream> {
    let table = SurrealTable::parse(input)?;
    table.gen_()
}

#[manyhow::manyhow]
#[proc_macro]
pub fn sql(input: TokenStream) -> manyhow::Result<TokenStream> {
    let sql_lit_str = syn::parse2::<LitStr>(input)?;
    let sql_str = sql_lit_str.value();

    let mut capabilities = Capabilities::all();
    *capabilities.allowed_experimental_features_mut() = Targets::All;

    match surrealdb_core::syn::parse_with_capabilities(&sql_str, &capabilities) {
        Ok(_) => Ok(sql_lit_str.to_token_stream()),
        Err(err) => manyhow::bail!(sql_lit_str.span(), "{err}"),
    }
}
