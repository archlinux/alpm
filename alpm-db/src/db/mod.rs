//! Handling of [alpm-db] database structures.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

mod database;
mod entry;
mod error;
mod schema;
mod traits;
mod transaction;

pub use database::{Database, DatabaseBuilder, DatabaseLock};
pub use entry::Entry;
pub use error::Error;
pub use schema::DbSchema;
pub use traits::WritableEntry;
pub use transaction::{Action, ActionWithData, Transaction, TransactionWithData};
