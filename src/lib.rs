#![doc=include_str!( "../readme.md")]

pub mod thing;

mod define;
mod does_imp;
mod r2k;
mod records;
mod surreal_table;

use std::collections::HashMap;

use serde::de::DeserializeOwned;

pub use surrealdb_extras_proc_macro::{SurrealSelect, SurrealTable};

pub use define::use_ns_db;
pub use records::{Record, RecordData};
pub use surreal_table::SurrealTableInfo;
pub use thing::{RecordIdFunc, RecordIdType};

#[doc(hidden)]
/// converts struct structure to the db type
/// is used by SurrealTableInfo
pub fn rust_to_surreal(s: &str, names: &HashMap<&'static str, &'static str>) -> String {
    r2k::to_kind(s, names).to_string()
}

/// SELECT {keys} IN db
pub trait SurrealSelectInfo: DeserializeOwned {
    /// all attributes
    fn keys() -> &'static [&'static str];
}
