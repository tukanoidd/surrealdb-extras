#![allow(async_fn_in_trait)]

use std::sync::Arc;

use surrealdb::{Connection, Surreal};

pub trait SurrealQuery {
    type Output;
    type Error: From<surrealdb::Error>;

    const QUERY_STR: &'static str;

    async fn execute<C>(self, db: Arc<Surreal<C>>) -> Result<Self::Output, Self::Error>
    where
        C: Connection;
}
