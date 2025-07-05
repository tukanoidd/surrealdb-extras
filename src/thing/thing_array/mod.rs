mod from;

use serde::{Serialize, de::DeserializeOwned};
use surrealdb::{
    Connection, Error, RecordId, Surreal, Value,
    method::{Content, Delete, Merge, Patch, Select},
    opt::{IntoResource, PatchOp, Resource},
};

use crate::{Record, RecordData, SurrealSelectInfo};

pub struct ThingArray(pub Vec<RecordId>);

impl<R> IntoResource<Vec<R>> for ThingArray {
    fn into_resource(self) -> surrealdb::Result<Resource> {
        let v = self.0.into_iter().map(Value::from).collect::<Vec<_>>();
        Ok(Resource::Array(v))
    }
}

impl ThingArray {
    /// deletes from db and return value
    pub fn delete<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Delete<'_, C, Vec<T>> {
        conn.delete(self)
    }

    /// gets from db
    pub fn get<T, C: Connection>(self, conn: &'_ Surreal<C>) -> Select<'_, C, Vec<T>> {
        conn.select(self)
    }

    /// Replaces the current document / record data with the specified data
    pub fn replace<T: DeserializeOwned, C: Connection, D: Serialize + 'static>(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Content<'_, C, Vec<T>> {
        conn.update(self).content(data)
    }

    /// Merges the current document / record data with the specified data
    pub fn merge<T: DeserializeOwned, C: Connection, D: Serialize>(
        self,
        conn: &'_ Surreal<C>,
        data: D,
    ) -> Merge<'_, C, D, Vec<T>> {
        conn.update(self).merge(data)
    }

    /// Patches the current document / record data with the specified JSON Patch data
    pub fn patch<T: DeserializeOwned, C: Connection>(
        self,
        conn: &'_ Surreal<C>,
        data: PatchOp,
    ) -> Patch<'_, C, Vec<T>> {
        conn.update(self).patch(data)
    }

    /// deletes from db and return success
    pub async fn delete_s<C: Connection>(self, conn: &Surreal<C>) -> Result<bool, Error> {
        let r: Vec<Record> = conn.delete(self).await?;
        Ok(!r.is_empty())
    }

    /// gets part from db
    pub async fn get_part<C: Connection, T: SurrealSelectInfo>(
        self,
        conn: &Surreal<C>,
    ) -> Result<Vec<RecordData<T>>, Error> {
        conn.query(format!("SELECT {} FROM {}", T::keys().join(", "), self))
            .await?
            .take(0)
    }
}
