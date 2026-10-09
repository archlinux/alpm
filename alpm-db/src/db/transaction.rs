//! Transaction representation in [alpm-db].
//!
//! A transactio describes a set of additions and/or removals, that are executed in sequence.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{fmt::Display, fs::remove_dir_all, path::Path};

use alpm_types::EntryName;
use fluent_i18n::t;
use log::debug;
use uuid::Uuid;

use crate::db::{Error, WritableEntry};

/// A transaction unit in an [alpm-db].
///
/// This may be an addition or a removal.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    /// A package is added.
    Add(EntryName),

    /// A package is removed.
    Remove(EntryName),
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

    /// Returns the the actions of this transaction.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Returns a reference to the UUID of this transaction.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

/// A transaction unit with data in an [alpm-db].
///
/// This may be an addition or a removal.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub enum ActionWithData {
    /// A package is added.
    Add {
        /// The name and version of the entry that is added.
        entry: EntryName,

        /// An implementation of WritableEntry.
        data: Box<dyn WritableEntry>,
    },

    /// A package is removed.
    Remove(EntryName),
}

impl ActionWithData {
    /// Executes the action in the root `path` of an [alpm-db].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the addition of an entry fails (see [`WritableEntry::write_to_db_root`] for details)
    /// - the removal of an entry fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    fn execute(&self, path: impl AsRef<Path>) -> Result<(), crate::Error> {
        let path = path.as_ref();
        debug!("Execute the action in database root directory {path:?}: {self}");

        match self {
            Self::Add { data, .. } => data.write_to_db_root(path)?,
            Self::Remove(entry) => {
                let entry_dir = path.join(entry.to_string());

                if !entry_dir.exists() {
                    return Err(Error::EntryDoesNotExist {
                        name: entry.clone(),
                    }
                    .into());
                }

                remove_dir_all(&entry_dir).map_err(|source| crate::Error::IoPath {
                    path: entry_dir,
                    context: t!("error-io-path-db-entry-remove"),
                    source,
                })?
            }
        }

        Ok(())
    }
}

impl Display for ActionWithData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (action, entry) = match self {
            Self::Add { entry, .. } => ("Add", entry),
            Self::Remove(entry) => ("Remove", entry),
        };

        write!(f, "{action} {entry}")
    }
}

impl From<&ActionWithData> for Action {
    fn from(value: &ActionWithData) -> Self {
        match value {
            ActionWithData::Add { entry, .. } => Self::Add(entry.clone()),
            ActionWithData::Remove(entry) => Self::Remove(entry.clone()),
        }
    }
}

/// A transaction and accompanying data for an [alpm-db].
///
/// A transaction consists of one more more [`Action`]s.
/// Each [`Action::Add`] is paired with an implementation of [`WritableEntry`], which provides
/// access to the data about to be written to disk.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub struct TransactionWithData {
    uuid: Uuid,
    actions: Vec<ActionWithData>,
}

impl TransactionWithData {
    /// Creates a new [`Transaction`] from a list of [`Action`] items.
    ///
    /// # Errors
    ///
    /// Returns an error, if `actions` is empty.
    pub fn new(actions: Vec<ActionWithData>) -> Result<Self, crate::Error> {
        let uuid = Uuid::now_v7();
        if actions.is_empty() {
            return Err(Error::TransactionHasNoAction { uuid }.into());
        }

        Ok(Self { uuid, actions })
    }

    /// Returns the the actions of this transaction.
    pub fn actions(&self) -> &[ActionWithData] {
        &self.actions
    }

    /// Returns a reference to the UUID of this transaction.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Executes all actions of this transaction in the [alpm-db] root directory `path`.
    ///
    /// # Errors
    ///
    /// Returns an error, if an action of the transaction fails.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub(crate) fn execute(&self, path: impl AsRef<Path>) -> Result<(), crate::Error> {
        let path = path.as_ref();

        for action in self.actions.iter() {
            action.execute(path)?
        }

        Ok(())
    }
}

impl From<&TransactionWithData> for Transaction {
    fn from(value: &TransactionWithData) -> Self {
        Self {
            uuid: value.uuid,
            actions: value.actions.iter().map(Into::into).collect(),
        }
    }
}
