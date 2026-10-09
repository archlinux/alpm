//! Handling of [alpm-db] database structures.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

mod database;
mod entry;
mod error;
mod schema;
mod transaction;

pub use database::{Database, DatabaseBuilder, DatabaseLock};
pub use entry::Entry;
pub use error::Error;
pub use schema::DbSchema;
pub use transaction::{Action, Transaction};
