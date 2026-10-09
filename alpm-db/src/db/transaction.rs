//! Transaction representation in [alpm-db].
//!
//! A transactio describes a set of additions and/or removals, that are executed in sequence.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::Display,
    fs::{
        File,
        OpenOptions,
        Permissions,
        copy,
        create_dir,
        create_dir_all,
        remove_dir_all,
        rename,
        set_permissions,
    },
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt, chown},
    path::{Path, PathBuf},
};

use alpm_types::EntryName;
use fluent_i18n::t;
use log::debug;
use nix::unistd::{getegid, geteuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{
    DIR_MODE,
    EntrySource,
    Error,
    entry::{EntryComponent, EntryComponentType},
};

/// An action item in a transaction plan for an [alpm-db].
///
/// This may be an addition or a removal.
///
/// # Note
///
/// An [`ActionItem`] only tracks what database item changed in an action, not the actual data.
/// Refer to [`Action`] for transaction items, that are used for changing an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
enum ActionItem {
    /// A package is added.
    Add(EntryName),

    /// A package is removed.
    Remove(EntryName),
}

impl From<&Action> for ActionItem {
    fn from(value: &Action) -> Self {
        match value {
            Action::Add(entry) => Self::Add(entry.entry_name()),
            Action::Remove(entry) => Self::Remove(entry.clone()),
        }
    }
}

/// The plan of a transaction in an [alpm-db].
///
/// A transaction plan consists of one or more action items and serves as a persisted plan of
/// "what is done in a transaction".
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct TransactionPlan {
    uuid: Uuid,
    actions: Vec<ActionItem>,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl TransactionPlan {
    /// Returns a reference to the UUID of the transaction plan.
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the on-disk file name of the transaction plan.
    pub fn file_name(&self) -> String {
        format!("{}.json", self.uuid)
    }

    /// Writes this transaction plan to a JSON encoded file in the current location of the data dir.
    pub fn write_to_current_dir(&self) -> Result<(), crate::Error> {
        let path = self
            .data_dir
            .join(Transaction::CURRENT_DIR)
            .join(self.file_name());
        let data = serde_json::to_string(&self).map_err(|source| crate::Error::Json {
            context: t!("error-json-transaction-plan", { "uuid" => self.uuid.to_string() }),
            source,
        })?;

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o644)
            .open(&path)
            .map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-open-writable"),
                source,
            })?;

        file.write_all(&data.into_bytes()).map_err(|source|
            crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-write-transaction-plan", { "uuid" => self.uuid.to_string() }),
                source
            })?;

        Ok(())
    }

    /// Moves the persisted, current transaction plan to the previous location of the data dir.
    ///
    /// # Errors
    ///
    /// Returns an error, if the current transaction plan cannot be moved (e.g. because it has not
    /// yet been persisted - see [`Self::write_to_current_dir`]).
    pub fn move_to_previous_dir(&self) -> Result<(), crate::Error> {
        // Move transaction plan from current to previous location.
        rename(
            self.data_dir
                .join(Transaction::CURRENT_DIR)
                .join(self.file_name()),
            self.data_dir
                .join(Transaction::PREVIOUS_DIR)
                .join(self.file_name()),
        )
        .map_err(|source| crate::Error::IoPath {
            path: self
                .data_dir
                .join(Transaction::CURRENT_DIR)
                .join(self.file_name()),
            context: t!("error-io-path-move-transaction-plan", { "uuid" => self.uuid().to_string() }),
            source,
        })
    }
}

impl From<&Transaction> for TransactionPlan {
    fn from(value: &Transaction) -> Self {
        Self {
            uuid: value.uuid,
            actions: value.actions.iter().map(Into::into).collect(),
            data_dir: value.data_dir.clone(),
        }
    }
}

/// Creates a new file handle at `path` for writing an entry component.
///
/// The file mode is compatible with alpm-db entry components.
///
/// # Errors
///
/// Returns an error, if a new file cannot be created for writing at `path`.
fn create_entry_component(path: impl AsRef<Path>) -> Result<File, crate::Error> {
    let path = path.as_ref();
    debug!("Creating alpm-db entry component {path:?}");

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(EntryComponent::FILE_MODE)
        .open(path)
        .map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-db-create-component"),
            source,
        })
}

/// Writes an `entry` to a directory `path`.
///
/// This function is used to create a new entry for the root directory of an [alpm-db].
///
/// # Errors
///
/// Returns an error, if
///
/// - the [alpm-db-desc] data of the [`EntrySource`] implementation cannot be retrieved
/// - the [alpm-db-files] data of the [`EntrySource`] implementation cannot be retrieved
/// - the [ALPM-MTREE] data of the [`EntrySource`] implementation cannot be retrieved
/// - an error occurs while attempting to retrieve the optional [alpm-install-scriptlet] data
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
fn write_entry_to_dir(
    entry: Box<dyn EntrySource>,
    path: impl AsRef<Path>,
) -> Result<(), crate::Error> {
    debug!("Load alpm-db-desc");
    let alpm_db_desc = entry.alpm_db_desc()?;
    debug!("Load alpm-db-files");
    let alpm_db_files = entry.alpm_db_files()?;
    debug!("Load alpm-install-scriptlet");
    let alpm_install_scriptlet = entry.alpm_install_scriptlet()?;
    debug!("Load alpm-mtree");
    let alpm_mtree = entry.alpm_mtree()?;

    let entry_dir = path.as_ref().join(entry.entry_name().to_string());
    if entry_dir.exists() {
        return Err(Error::EntryExists {
            name: entry.entry_name().clone(),
        }
        .into());
    }
    create_dir(&entry_dir).map_err(|source| crate::Error::IoPath {
        path: entry_dir.clone(),
        context: t!("error-io-path-entry-dir-create"),
        source,
    })?;
    set_permissions(&entry_dir, Permissions::from_mode(DIR_MODE)).map_err(|source| {
        crate::Error::IoPath {
            path: entry_dir.clone(),
            context: t!("error-io-path-db-set-permissions-on-dir"),
            source,
        }
    })?;

    let components = {
        let mut components = vec![
            (
                EntryComponentType::DbDesc,
                create_entry_component(entry_dir.join(EntryComponentType::DbDesc.as_ref()))?,
                alpm_db_desc.to_string().into_bytes(),
            ),
            (
                EntryComponentType::DbFiles,
                create_entry_component(entry_dir.join(EntryComponentType::DbFiles.as_ref()))?,
                alpm_db_files.to_string().into_bytes(),
            ),
            (
                EntryComponentType::Mtree,
                create_entry_component(entry_dir.join(EntryComponentType::Mtree.as_ref()))?,
                alpm_mtree.data_ensure_gzip()?,
            ),
        ];

        if let Some(alpm_install_scriptlet) = alpm_install_scriptlet {
            components.push((
                EntryComponentType::InstallScriptlet,
                create_entry_component(
                    entry_dir.join(EntryComponentType::InstallScriptlet.as_ref()),
                )?,
                alpm_install_scriptlet.into_bytes(),
            ))
        }

        components
    };

    for (component_type, mut file, data) in components {
        file.write_all(&data)
            .map_err(|source| crate::Error::IoPath {
                path: entry_dir.clone(),
                context: t!(
                    "error-io-path-db-write-component",
                    { "file_type" => component_type.file_type_name().to_string()}
                ),
                source,
            })?
    }

    Ok(())
}

/// An action in a transaction for an [alpm-db].
///
/// This may be an addition or a removal.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub enum Action {
    /// A package is added.
    ///
    /// Tracks data that is lazy-loaded from an implementation of [`EntrySource`].
    Add(Box<dyn EntrySource>),

    /// A package is removed.
    Remove(EntryName),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (action, entry) = match self {
            Self::Add(entry) => ("Add", &entry.entry_name()),
            Self::Remove(entry) => ("Remove", entry),
        };

        write!(f, "{action} {entry}")
    }
}

/// Creates all directories required for a transaction to function at `path`.
///
/// Returns the canonicalized transaction data directory.
///
/// # Note
///
/// This creates `path` and all required subdirectories.
///
/// # Errors
///
/// Returns an error, if
///
/// - `path` exists and
///   - it cannot be canonicalized
///   - it is not a directory
///   - metadata for it cannot be retrieved
///   - its ownership cannot be changed to the UID/GID of the calling user
///   - its permissions cannot be changed to the default permissions
/// - `path` does not exist and
///   - it cannot be created
///   - its permissions cannot be set to the defaults
/// - any of the required subdirectories
///   - cannot be created
///   - cannot be adjusted to be owned by the calling UID/GID and have default permissions
fn create_transaction_dirs(path: impl AsRef<Path>) -> Result<PathBuf, crate::Error> {
    let path = path.as_ref();

    let path = if path.exists() {
        // If the path exists already, canonicalize it and ensure, that it is valid.
        // The path must be an empty directory with default permissions, owned by the
        // UID/GID of the calling process.

        let path = path.canonicalize().map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-db-transaction-data-dir-canonicalize"),
            source,
        })?;

        // Fail, if the targeted path is not a directory.
        if !&path.is_dir() {
            return Err(Error::TransactionDataDirIsNotADirectory { path }.into());
        }

        let euid = geteuid().as_raw();
        let egid = getegid().as_raw();
        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-metadata"),
                source,
            })?;

        // Change ownership if it doesn't match the calling UID/GID.
        if !(metadata.uid() == euid && metadata.gid() == egid) {
            chown(&path, Some(euid), Some(egid)).map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-chown"),
                source,
            })?;
        }

        // Change permissions if they don't match the defaults.
        if metadata.permissions().mode() != DIR_MODE {
            set_permissions(&path, PermissionsExt::from_mode(DIR_MODE)).map_err(|source| {
                crate::Error::IoPath {
                    path: path.clone(),
                    context: t!("error-io-path-db-transaction-data-dir-set-permissions"),
                    source,
                }
            })?;
        }

        path
    } else {
        // If the path does not yet exist, create it, canonicalize it and set default
        // permissions.

        create_dir_all(path).map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-db-transaction-data-dir-create"),
            source,
        })?;
        let path = path.canonicalize().map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-db-transaction-data-dir-canonicalize"),
            source,
        })?;

        set_permissions(&path, PermissionsExt::from_mode(DIR_MODE)).map_err(|source| {
            crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-db-transaction-data-dir-set-permissions"),
                source,
            }
        })?;

        path
    };

    // Create all required subdirectories (for current and previous transaction plans, as well as
    // temporary transaction data).
    for subdir in [
        Transaction::CURRENT_DIR,
        Transaction::PREVIOUS_DIR,
        Transaction::TEMP_DIR,
    ] {
        let subdir = path.join(subdir);

        if subdir.is_dir() {
            let euid = geteuid().as_raw();
            let egid = getegid().as_raw();
            let metadata = subdir
                .symlink_metadata()
                .map_err(|source| crate::Error::IoPath {
                    path: subdir.clone(),
                    context: t!("error-io-path-metadata"),
                    source,
                })?;

            // Change ownership if it doesn't match the calling UID/GID.
            if !(metadata.uid() == euid && metadata.gid() == egid) {
                chown(&subdir, Some(euid), Some(egid)).map_err(|source| crate::Error::IoPath {
                    path: subdir.clone(),
                    context: t!("error-io-path-chown"),
                    source,
                })?;
            }

            // Change permissions if they don't match the defaults.
            if metadata.permissions().mode() != DIR_MODE {
                set_permissions(&subdir, PermissionsExt::from_mode(DIR_MODE)).map_err(
                    |source| crate::Error::IoPath {
                        path: subdir.clone(),
                        context: t!("error-io-path-db-transaction-data-dir-set-permissions"),
                        source,
                    },
                )?;
            }

            continue;
        }

        create_dir(&subdir).map_err(|source| crate::Error::IoPath {
            path: subdir.to_path_buf(),
            context: t!("error-io-path-db-base-create"),
            source,
        })?;
        set_permissions(&subdir, PermissionsExt::from_mode(DIR_MODE)).map_err(|source| {
            crate::Error::IoPath {
                path: subdir.clone(),
                context: t!("error-io-path-db-transaction-data-dir-set-permissions"),
                source,
            }
        })?;
    }

    Ok(path)
}

/// Removes the temporary directory for this transaction.
///
/// # Errors
///
/// Returns an error, if the temporary directory of the transaction, or any of its children
/// cannot be removed.
fn remove_temp_transaction_dir(path: impl AsRef<Path>) -> Result<(), crate::Error> {
    let path = path.as_ref();

    remove_dir_all(path).map_err(|source| crate::Error::IoPath {
        path: path.to_path_buf(),
        context: t!("error-io-path-db-transaction-temp-dir-remove"),
        source,
    })
}

/// Checks whether `path1` and `path2` are on the same device.
///
/// # Errors
///
/// Returns an error, if the metadata for `path1` or `path2` cannot be retrieved.
fn paths_on_same_device(
    path1: impl AsRef<Path>,
    path2: impl AsRef<Path>,
) -> Result<bool, crate::Error> {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    Ok(path1
        .symlink_metadata()
        .map_err(|source| crate::Error::IoPath {
            path: path1.to_path_buf(),
            context: t!("error-io-path-metadata"),
            source,
        })?
        .dev()
        == path2
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path2.to_path_buf(),
                context: t!("error-io-path-metadata"),
                source,
            })?
            .dev())
}

/// A transaction for an [alpm-db].
///
/// A transaction consists of zero or more more [`Action`]s.
/// Each [`Action::Add`] is paired with an implementation of [`EntrySource`], which provides
/// access to the data about to be written to disk.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub(crate) struct Transaction {
    uuid: Uuid,
    actions: Vec<Action>,
    database_dir: PathBuf,
    data_dir: PathBuf,
}

impl Transaction {
    /// The name of the directory below the data directory in which current transaction plans are
    /// stored.
    pub const CURRENT_DIR: &str = "current";

    /// The name of the directory below the data directory in which previous transaction plans are
    /// stored.
    pub const PREVIOUS_DIR: &str = "previous";

    /// The name of the directory below the data directory in which temporary transaction data is
    /// stored.
    pub const TEMP_DIR: &str = "temp";

    /// Creates a new [`Transaction`].
    ///
    /// # Errors
    ///
    /// Returns an error, if a lock file cannot be created for the transaction.
    pub fn new(
        database_dir: impl AsRef<Path>,
        data_dir: impl AsRef<Path>,
    ) -> Result<Self, crate::Error> {
        let database_dir = database_dir.as_ref().to_path_buf();
        let data_dir = data_dir.as_ref();
        let uuid = Uuid::now_v7();
        debug!("Create transaction {uuid} using data dir {:?}", data_dir,);

        let data_dir = create_transaction_dirs(data_dir)?;

        Ok(Self {
            uuid,
            actions: Vec::new(),
            database_dir,
            data_dir,
        })
    }

    /// Adds a new action to this transaction.
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Returns a reference to the UUID of this transaction.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns a reference to the database dir of this transaction.
    pub fn database_dir(&self) -> &Path {
        &self.database_dir
    }

    /// Returns a reference to the data dir of this transaction.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Creates the temporary directory for this transaction and returns its path.
    ///
    /// # Errors
    ///
    /// Returns an error, if the temporary directory cannot be created.
    fn create_temp_dir(&self) -> Result<PathBuf, crate::Error> {
        let temp_dir = self
            .data_dir
            .join(Transaction::TEMP_DIR)
            .join(self.uuid().to_string());

        // Create the temporary directory for this transaction.
        create_dir(&temp_dir).map_err(|source| crate::Error::IoPath {
            path: temp_dir.clone(),
            context: t!("error-io-path-db-transaction-temp-dir-create"),
            source,
        })?;

        Ok(temp_dir)
    }

    /// Commits all changes of this transaction to the [alpm-db] and consumes [`Self`].
    ///
    /// Committing a transaction involves several steps:
    ///
    /// - creation of a temporary directory for the transaction
    /// - collection of the data of all additions in the temporary directory
    /// - removal of all entries marked for removal from the [alpm-db]
    /// - move/copy of all additions from the temporary directory to the [alpm-db]
    /// - removal of the temporary directory
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - creating the temporary directory for the transaction fails
    /// - checking whether temporary directory and [alpm-db] are on the same device fails
    /// - writing a new entry to the temporary directory fails
    /// - removal of an entry would fail because it does not exist in the [alpm-db]
    /// - removing an entry from the [alpm-db] fails
    /// - moving/copying a new entry from the temporary directory to the [alpm-db] fails
    /// - removing the temporary directory fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn commit(self) -> Result<(), crate::Error> {
        debug!(
            "Committing transaction {} for alpm-db {:?} using data dir {:?}",
            self.uuid(),
            self.database_dir(),
            self.data_dir(),
        );

        let temp_dir = self.create_temp_dir()?;
        let path = self.database_dir().to_path_buf();
        let uuid = *self.uuid();
        // Check whether the temporary location and the database are on the same device.
        let same_device = paths_on_same_device(&path, &temp_dir)?;

        // Write all additions to the temporary directory and collect the removals.
        // Any errors that may occur are collected as well.
        let (removals, errors) = {
            let mut removals = Vec::new();
            let mut errors = Vec::new();
            for action in self.actions {
                match action {
                    Action::Add(entry) => {
                        if let Err(error) = write_entry_to_dir(entry, &temp_dir) {
                            errors.push(error);
                        }
                    }
                    Action::Remove(entry_name) => {
                        if !path.join(entry_name.to_string()).exists() {
                            errors.push(
                                Error::EntryDoesNotExist {
                                    name: entry_name.clone(),
                                }
                                .into(),
                            );
                        }
                        removals.push(entry_name)
                    }
                }
            }
            (removals, errors)
        };

        // Remove the temporary directory, if errors occurred during preparation and return an
        // error.
        if !errors.is_empty() {
            remove_temp_transaction_dir(&temp_dir)?;

            return Err(Error::TransactionTemporaryDataErrors {
                path: path.to_path_buf(),
                uuid,
                errors: errors.iter().map(ToString::to_string).collect(),
            }
            .into());
        }

        // Remove all entries marked for removal from the database.
        for removal in removals {
            remove_dir_all(path.join(removal.to_string())).map_err(|source| {
                crate::Error::IoPath {
                    path: path.join(removal.to_string()),
                    context: t!("error-io-path-db-entry-remove"),
                    source,
                }
            })?
        }

        // Move or copy all entries marked for addition from temporary dir to database.
        if same_device {
            debug!(
                "Temporary transaction data and database are on the same device: Moving entries..."
            );
        } else {
            debug!(
                "Temporary transaction data and database are on separate devices: Copying entries..."
            );
        }

        for entry in temp_dir.read_dir().map_err(|source| crate::Error::IoPath {
            path: temp_dir.clone(),
            context: t!("error-io-path-db-transaction-temp-dir-read"),
            source,
        })? {
            let entry = entry.map_err(|source| crate::Error::IoPath {
                path: temp_dir.clone(),
                context: t!("error-io-path-db-transaction-temp-dir-entry-read"),
                source,
            })?;
            let entry_from = entry.path();
            let entry_to = path.join(entry.file_name());
            if same_device {
                rename(&entry_from, &entry_to).map_err(|source| crate::Error::IoPath {
                    path: entry_from.clone(),
                    context: t!("error-io-path-db-transaction-temp-dir-entry-move"),
                    source,
                })?;
            } else {
                copy(&entry_from, &entry_to).map_err(|source| crate::Error::IoPath {
                    path: entry_from.clone(),
                    context: t!("error-io-path-db-transaction-temp-dir-entry-copy"),
                    source,
                })?;
            }
        }

        // Remove the temporary data.
        remove_temp_transaction_dir(&temp_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::{assert_snapshot, with_settings};
    use log::LevelFilter;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;

    /// Initialize a logger that shows trace messages on stderr.
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }
    }

    /// Ensures, that the JSON representation of a [`TransactionPlan`] remains stable.
    #[test]
    fn transaction_plan_stable_serialize_example() -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let path = temp_dir.path();

        let plan = TransactionPlan {
            uuid: "01a17815-fe9d-77f6-9bb1-387d74e07195".parse()?,
            actions: vec![
                ActionItem::Add("example-1.0.0-1".parse()?),
                ActionItem::Remove("other-example-2.0.0-1".parse()?),
            ],
            data_dir: path.to_path_buf(),
        };

        let json_plan = serde_json::to_string(&plan)?;

        debug!("{json_plan}");
        with_settings!({ description => "Simple, serialized transaction plan example", snapshot_path => "snapshots" }, {
            assert_snapshot!("transaction_plan_stable_serialize_example", json_plan);
        });

        Ok(())
    }
}
