mod query;
mod table;
mod util;

use proc_macro2::TokenStream;
use quote::ToTokens;
use surrealdb_core::dbs::{Capabilities, capabilities::Targets};
use syn::LitStr;

use crate::{
    query::SurrealQuery,
    table::{SurrealSelect, SurrealTable},
    util::DeriveInputUtil,
};

/// implements SurrealSelectInfo, SurrealTableInfo, add and insert
#[manyhow::manyhow]
#[proc_macro_derive(SurrealTable, attributes(table))]
pub fn table(input: TokenStream) -> manyhow::Result<TokenStream> {
    let table = SurrealTable::parse(input)?;
    table.gen_()
}

/// implements SurrealSelectInfo
#[manyhow::manyhow]
#[proc_macro_derive(SurrealSelect)]
pub fn select(input: TokenStream) -> manyhow::Result<TokenStream> {
    let select = SurrealSelect::parse(input)?;
    select.gen_()
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

#[manyhow::manyhow]
#[proc_macro_derive(SurrealQuery, attributes(query, var))]
pub fn query(input: TokenStream) -> manyhow::Result<TokenStream> {
    let query = SurrealQuery::parse(input)?;
    let res = query.gen_()?;

    Ok(res)
}
