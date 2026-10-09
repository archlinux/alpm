//! Handling of [alpm-db] database structures.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

mod database;
mod entry;
mod error;
mod schema;
mod traits;
mod transaction;
mod utils;

pub use database::{Database, DatabaseConfig};
pub use entry::Entry;
pub use error::Error;
pub use schema::DbSchema;
pub use traits::EntrySource;
pub use transaction::Action;
pub(crate) use transaction::{Transaction, TransactionPlan};
pub(crate) use utils::{DIR_MODE, permissions_world_writable};
