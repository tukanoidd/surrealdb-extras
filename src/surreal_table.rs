#![allow(async_fn_in_trait)]

use serde::Serialize;
use serde_content::Serializer;
use surrealdb::{Connection, RecordIdKey, Surreal, sql::Value};

use crate::{Record, RecordData, SurrealSelectInfo};

type F1 = fn() -> &'static str;
type F3 = fn() -> Vec<String>;

pub type Register = (F1, F1, F3);

/// usefull functions for db
/// will be created by proc macro
pub trait SurrealTableInfo: Serialize + SurrealSelectInfo + Clone + 'static {
    /// db name
    fn name() -> &'static str;
    /// path to struct
    fn path() -> &'static str;
    /// defines what attr to exclude in check_if_exists
    fn exclude() -> &'static [&'static str];
    /// register attr
    fn funcs() -> Vec<String>;

    /// checks if item exists in table and returns the result
    async fn check_if_exists<'a, C: Connection>(
        &'a self,
        db: &'a Surreal<C>,
    ) -> Result<Option<Record>, surrealdb::Error> {
        let ignore = Self::exclude();
        let value: Value = Serializer::new()
            .serialize(self)
            .map_err(|e| {
                surrealdb::Error::Api(surrealdb::error::Api::SerializeValue(e.to_string()))
            })?
            .try_into()?;

        let mut query = vec![];

        if let Value::Object(obj) = value {
            for (key, item) in obj.0 {
                if !ignore.contains(&key.as_str()) {
                    query.push(format!("{key} = {item}"));
                }
            }
        } else {
            unimplemented!()
        }
        let v = format!(
            "SELECT id FROM {} WHERE {} LIMIT 1;",
            Self::name(),
            query.join(" AND ")
        );
        let mut t: Vec<Record> = db.query(v).await?.take(0)?;
        Ok(if !t.is_empty() {
            Some(t.remove(0))
        } else {
            None
        })
    }

    /// adds itself to the db and returns Record
    async fn add_i<D: Connection>(self, conn: &Surreal<D>) -> Result<Record, surrealdb::Error> {
        let r: Option<Record> = conn.create(Self::name()).content(self).await?;

        r.ok_or(surrealdb::Error::Api(surrealdb::error::Api::InternalError(
            "No return value".to_owned(),
        )))
    }

    /// checks if item exists(adds to db if its not in db) and returns id
    async fn get_or_insert<C: Connection>(
        self,
        db: &Surreal<C>,
    ) -> Result<Record, surrealdb::Error> {
        let check = self.check_if_exists(db).await?;
        if let Some(check) = check {
            Ok(check)
        } else {
            self.add_i(db).await
        }
    }

    /// search db
    async fn search<T: SurrealSelectInfo + serde::de::DeserializeOwned, C: Connection>(
        conn: &Surreal<C>,
        query: Option<String>,
    ) -> Result<Vec<RecordData<T>>, surrealdb::Error> {
        let keys = T::keys();
        let query = format!(
            "SELECT {} FROM {}{};",
            keys.join(", "),
            Self::name(),
            match query {
                Some(v) => format!(" {v}"),
                None => "".to_string(),
            }
        );
        conn.query(query).await?.take(0)
    }

    /// adds itself to the db and returns true if there was a response
    async fn add_s<D: Connection>(self, conn: &Surreal<D>) -> Result<bool, surrealdb::Error> {
        let r: Option<Record> = conn.create(Self::name()).content(self).await?;
        Ok(r.is_some())
    }

    /// inserts itself to the db and returns true if there was a response
    async fn insert_s<D: Connection>(
        self,
        conn: &Surreal<D>,
        id: RecordIdKey,
    ) -> Result<bool, surrealdb::Error> {
        let r: Option<Record> = conn.create((Self::name(), id)).content(self).await?;
        Ok(r.is_some())
    }

    /// returns every item in table
    fn all<T: serde::Serialize, C: Connection>(
        conn: &'_ Surreal<C>,
    ) -> surrealdb::method::Select<'_, C, Vec<T>> {
        conn.select(Self::name())
    }

    /// returns functions for register
    fn register() -> Result<Register, String> {
        // TODO: add check
        // for typ in types {
        //     if does_impl!(typ: surrealdb_extras::SurrealTableInfo) {
        //         return Err(format!("{} is a table", typ.to_string())
        //     }
        // }
        Ok((Self::name, Self::path, Self::funcs))
    }
}
