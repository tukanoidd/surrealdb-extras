use std::collections::HashSet;

use surrealdb::{Connection, Surreal, Value};

use crate::surreal_table::Register;

pub trait SurrealExt {
    /// creates namespace, db, tables and defines the attributes if they do not exist
    fn use_ns_db_checked(
        &self,
        ns: impl AsRef<str>,
        db: impl AsRef<str>,
        register: impl IntoIterator<Item = Register>,
    ) -> impl std::future::Future<Output = surrealdb::Result<()>>;

    fn missing(
        &self,
        query: &str,
        key_value: (&str, &str),
    ) -> impl std::future::Future<Output = bool> + Send;
    fn table_list(&self) -> impl std::future::Future<Output = HashSet<String>> + Send;
}

impl<C> SurrealExt for Surreal<C>
where
    C: Connection,
{
    async fn use_ns_db_checked(
        &self,
        ns: impl AsRef<str>,
        db: impl AsRef<str>,
        register: impl IntoIterator<Item = Register>,
    ) -> surrealdb::Result<()> {
        let ns = ns.as_ref();
        let db = db.as_ref();

        if self.missing("INFO FOR KV", ("namespaces", ns)).await {
            self.query(format!("DEFINE NAMESPACE {ns};")).await?;
        }

        self.use_ns(ns).await?;

        if self.missing("INFO FOR NS", ("databases", db)).await {
            self.query(format!("DEFINE DATABASE {db};")).await?;
        }

        self.use_db(db).await?;

        let tables = self.table_list().await;

        for (name, _, funcs) in register {
            if !tables.contains(name()) {
                for query in funcs() {
                    println!("{query}");
                    self.query(query).await?;
                }
            }
        }

        Ok(())
    }

    async fn missing(&self, query: &str, key_value: (&str, &str)) -> bool {
        self.query(query)
            .await
            .unwrap()
            .take::<Value>(0)
            .unwrap()
            .into_inner()
            .into_json()
            .as_object()
            .unwrap()
            .get_key_value(key_value.0)
            .unwrap()
            .1
            .clone()
            .as_object()
            .unwrap()
            .get_key_value(key_value.1)
            .is_none()
    }

    async fn table_list(&self) -> HashSet<String> {
        self.query("INFO FOR DB")
            .await
            .unwrap()
            .take::<Value>(0)
            .unwrap()
            .into_inner()
            .into_json()
            .as_object()
            .unwrap()
            .get_key_value("tables")
            .unwrap()
            .1
            .clone()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<HashSet<_>>()
    }
}
