mod from;
mod impl_;

use serde::{Serialize, de::DeserializeOwned};
use surrealdb::{
    Connection, Error, Surreal,
    method::{Content, Delete, Merge, Patch, Select},
    opt::PatchOp,
    types::{RecordId, RecordIdKey, SurrealValue},
};

use crate::{Record, RecordData, SurrealSelectInfo};

#[derive(Clone, Debug, PartialEq, PartialOrd, SurrealValue)]
/// some usefull functions for Thing
/// ```
/// #[derive(surrealdb_extras::SurrealTable, serde::Serialize, serde::Deserialize)]
/// #[db("test_table")]
/// struct Test {
///     name: String,
///     /// a refrence to another table entry
///     refr: surrealdb_extras::RecordIdFunc
/// }
/// ```
pub struct RecordIdFunc(pub RecordId);

impl RecordIdFunc {
    /// From Thing
    pub fn new(thing: RecordId) -> Self {
        Self(thing)
    }

    /// deletes from db and return value
    pub fn delete<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Delete<'_, C, Option<T>> {
        conn.delete(self.0)
    }

    /// gets from db
    pub fn get<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Select<'_, C, Option<T>> {
        conn.select(self.0)
    }

    /// Replaces the current document / record data with the specified data
    pub fn replace<
        R: SurrealValue + DeserializeOwned,
        C: Connection,
        D: SurrealValue + Serialize + 'static,
    >(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Content<'_, C, Option<R>> {
        conn.update(self.0).content(data)
    }

    /// Merges the current document / record data with the specified data
    pub fn merge<T: SurrealValue + DeserializeOwned, C: Connection, D: SurrealValue + Serialize>(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Merge<'_, C, D, Option<T>> {
        conn.update(self.0).merge(data)
    }

    /// Patches the current document / record data with the specified JSON Patch data
    pub fn patch<T: SurrealValue + DeserializeOwned, C: Connection>(
        self,
        conn: &'_ Surreal<C>,
        data: PatchOp,
    ) -> Patch<'_, C, Option<T>> {
        conn.update(self.0).patch(data)
    }

    /// deletes from db and return success
    pub async fn delete_s<C: Connection>(self, conn: &Surreal<C>) -> Result<bool, Error> {
        let r: Option<Record> = conn.delete(self.0).await?;
        Ok(r.is_some())
    }

    /// gets part from db
    pub async fn get_part<C: Connection, T: SurrealValue + SurrealSelectInfo>(
        self,
        conn: &Surreal<C>,
    ) -> Result<Option<RecordData<T>>, Error> {
        conn.query(format!("SELECT {} FROM {}", T::keys().join(", "), self))
            .await?
            .take(0)
    }

    /// returns table
    pub fn tb(&self) -> &str {
        self.0.table.as_str()
    }

    /// returns id
    pub fn id(&self) -> &RecordIdKey {
        &self.0.key
    }
}
