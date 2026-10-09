//! Handling of [alpm-db] database structures.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

mod entry;
mod error;
mod schema;

pub use entry::{Entry, EntryComponentFileName};
pub use error::Error;
pub use schema::DbSchema;
