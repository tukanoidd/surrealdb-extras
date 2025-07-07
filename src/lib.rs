#![doc=include_str!( "../readme.md")]

pub mod thing;

mod define;
mod does_imp;

mod records;
mod surreal_table;

use serde::de::DeserializeOwned;

pub use surrealdb_extras_proc_macro::*;

pub use define::use_ns_db;
pub use records::{Record, RecordData};
pub use surreal_table::SurrealTableInfo;
pub use thing::{RecordIdFunc, RecordIdType};

/// SELECT {keys} IN db
pub trait SurrealSelectInfo: DeserializeOwned {
    /// all attributes
    fn keys() -> &'static [&'static str];
}
