//! Transaction representation in [alpm-db].
//!
//! A transactio describes a set of additions and/or removals, that are executed in sequence.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use alpm_types::{FullVersion, Name};
use uuid::Uuid;

use crate::db::Error;

/// A transaction unit in an [alpm-db].
///
/// This may be an addition, change or removal.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    /// A package is added.
    Addition {
        /// The name of the package that is added.
        name: Name,

        /// The version of the package that is added.
        version: FullVersion,
    },

    /// A package is changed (upgraded or downgraded).
    Change {
        /// The name of the package that is changed.
        from_name: Name,

        /// The version of the package that is added.
        from_version: FullVersion,

        /// The version of the package that is added.
        to_version: FullVersion,
    },

    /// A package is removed.
    Removal {
        /// The name of the package that is removed.
        name: Name,
    },
}

/// A transaction in an [alpm-db].
///
/// A transaction consists of one more more [`Action`]s.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Transaction {
    uuid: Uuid,
    actions: Vec<Action>,
}

impl Transaction {
    /// The default location for transactions, that are currently executed (or about to be
    /// executed).
    pub const CURRENT_DATA_LOCATION: &str = "/var/lib/alpm/alpm-db/current";

    /// The default location for past transactions, that have been executed successfully.
    pub const PAST_DATA_LOCATION: &str = "/var/lib/alpm/alpm-db/past";

    /// Creates a new [`Transaction`] from a list of [`Action`] items.
    ///
    /// # Errors
    ///
    /// Returns an error, if `actions` is empty.
    pub fn new(actions: Vec<Action>) -> Result<Self, crate::Error> {
        let uuid = Uuid::now_v7();
        if actions.is_empty() {
            return Err(Error::TransactionHasNoAction { uuid }.into());
        }

        Ok(Self { uuid, actions })
    }
}
