use serde::{Deserialize, Serialize, de::DeserializeOwned};
use surrealdb::{
    Connection, Error, Surreal,
    method::{Content, Delete, Merge, Patch, Select},
    opt::PatchOp,
    types::{RecordId, SurrealValue},
};

use crate::{RecordIdFunc, SurrealSelectInfo};

#[derive(Debug, SurrealValue, Serialize, Deserialize)]
/// Deserialize response into id
pub struct Record {
    pub id: RecordIdFunc,
}

impl Record {
    /// From Thing
    pub fn new(id: RecordId) -> Self {
        Self {
            id: RecordIdFunc(id),
        }
    }

    /// deletes from db and return value
    pub fn delete<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Delete<'_, C, Option<T>> {
        self.id.delete(conn)
    }

    /// deletes from db and return success
    pub async fn delete_s<C: Connection>(self, conn: &Surreal<C>) -> Result<bool, Error> {
        self.id.delete_s(conn).await
    }

    /// gets from db
    pub fn get<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Select<'_, C, Option<T>> {
        self.id.get(conn)
    }

    /// Replaces the current document / record data with the specified data
    pub fn replace<
        T: SurrealValue + DeserializeOwned,
        C: Connection,
        D: SurrealValue + Serialize + 'static,
    >(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Content<'_, C, Option<T>> {
        self.id.replace(conn, data)
    }

    /// Merges the current document / record data with the specified data
    pub fn merge<T: SurrealValue + DeserializeOwned, C: Connection, D: SurrealValue + Serialize>(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Merge<'_, C, D, Option<T>> {
        self.id.merge(conn, data)
    }

    /// Patches the current document / record data with the specified JSON Patch data
    pub fn patch<T: SurrealValue + DeserializeOwned, C: Connection>(
        self,
        conn: &'_ Surreal<C>,
        data: PatchOp,
    ) -> Patch<'_, C, Option<T>> {
        self.id.patch(conn, data)
    }

    /// Gets part from db
    pub async fn get_part<C: Connection, T: SurrealValue + SurrealSelectInfo>(
        self,
        conn: &Surreal<C>,
    ) -> Result<Option<RecordData<T>>, Error> {
        self.id.get_part(conn).await
    }
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
/// Deserialize response into id and data
pub struct RecordData<RD>
where
    RD: SurrealValue,
{
    pub id: RecordIdFunc,
    #[serde(flatten)]
    pub data: RD,
}

impl<D> RecordData<D>
where
    D: SurrealValue,
{
    /// deletes from db and return value
    pub fn delete<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Delete<'_, C, Option<T>> {
        self.id.delete(conn)
    }

    /// deletes from db and return success
    pub async fn delete_s<C: Connection>(self, conn: &Surreal<C>) -> Result<bool, Error> {
        self.id.delete_s(conn).await
    }

    /// gets from db
    pub fn get<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Select<'_, C, Option<T>> {
        self.id.get(conn)
    }

    /// Replaces the current document / record data with the specified data
    pub fn replace<
        T: SurrealValue + DeserializeOwned,
        C: Connection,
        ID: SurrealValue + Serialize + 'static,
    >(
        self,
        conn: &'_ Surreal<C>,
        data: ID,
    ) -> Content<'_, C, Option<T>> {
        self.id.replace(conn, data)
    }

    /// Merges the current document / record data with the specified data
    pub fn merge<
        T: SurrealValue + DeserializeOwned,
        C: Connection,
        ID: SurrealValue + Serialize,
    >(
        self,
        conn: &'_ Surreal<C>,
        data: ID,
    ) -> Merge<'_, C, ID, Option<T>> {
        self.id.merge(conn, data)
    }

    /// Patches the current document / record data with the specified JSON Patch data
    pub fn patch<T: SurrealValue + DeserializeOwned, C: Connection>(
        self,
        conn: &'_ Surreal<C>,
        data: PatchOp,
    ) -> Patch<'_, C, Option<T>> {
        self.id.patch(conn, data)
    }

    /// Gets part from db
    pub async fn get_part<C: Connection, T: SurrealValue + SurrealSelectInfo>(
        self,
        conn: &Surreal<C>,
    ) -> Result<Option<RecordData<T>>, Error> {
        self.id.get_part(conn).await
    }
}
