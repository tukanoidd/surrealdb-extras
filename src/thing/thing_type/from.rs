use std::str::FromStr;

use surrealdb::types::{RecordId, RecordIdKey};

use crate::{Record, RecordData, RecordIdFunc, RecordIdType, SurrealTableInfo};

impl<T: SurrealTableInfo> From<RecordIdFunc> for RecordIdType<T> {
    fn from(value: RecordIdFunc) -> Self {
        Self::new(value)
    }
}

impl<T: SurrealTableInfo> From<RecordId> for RecordIdType<T> {
    fn from(value: RecordId) -> Self {
        Self::new_thing(value)
    }
}

impl<T: SurrealTableInfo> From<Record> for RecordIdType<T> {
    fn from(value: Record) -> Self {
        Self::from(value.id)
    }
}

impl<T: SurrealTableInfo> From<RecordData<T>> for RecordIdType<T> {
    fn from(value: RecordData<T>) -> Self {
        Self::from(value.id)
    }
}

impl<T: SurrealTableInfo> From<(&str, RecordIdKey)> for RecordIdType<T> {
    fn from((table, id): (&str, RecordIdKey)) -> Self {
        Self::from(RecordId::new(table, id))
    }
}

impl<T: SurrealTableInfo> From<(String, RecordIdKey)> for RecordIdType<T> {
    fn from((table, id): (String, RecordIdKey)) -> Self {
        Self::from(RecordId::new(table, id))
    }
}

impl<T: SurrealTableInfo> From<(String, String)> for RecordIdType<T> {
    fn from((table, id): (String, String)) -> Self {
        Self::from(RecordId::new(table, id))
    }
}

impl<T: SurrealTableInfo> From<(&str, &str)> for RecordIdType<T> {
    fn from((table, id): (&str, &str)) -> Self {
        Self::from(RecordId::new(table, id))
    }
}

impl<T: SurrealTableInfo> FromStr for RecordIdType<T> {
    type Err = surrealdb::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(RecordId::parse_simple(s)?))
    }
}

impl<T: SurrealTableInfo> TryFrom<String> for RecordIdType<T> {
    type Error = surrealdb::Error;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        Ok(Self::from(RecordId::parse_simple(v.as_str())?))
    }
}

impl<T: SurrealTableInfo> TryFrom<&str> for RecordIdType<T> {
    type Error = surrealdb::Error;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(RecordId::parse_simple(v)?))
    }
}
