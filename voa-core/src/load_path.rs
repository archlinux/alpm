//! VOA load path handling.
//!
//! This module can produce [LoadPathList]s for both system and user mode.
//!
//! See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#load-paths>

use std::path::PathBuf;

/// A VOA load path.
///
/// This type is an internal implementation detail of voa-core, and not exposed publicly.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadPath {
    pub(crate) path: PathBuf,
    ephemeral: bool,
    writable: bool,
}

impl LoadPath {
    /// Construct a new LoadPath from a `path` and the `ephemeral` flag
    pub(crate) fn new(path: impl Into<PathBuf>, ephemeral: bool, writable: bool) -> Self {
        Self {
            path: path.into(),
            ephemeral,
            writable,
        }
    }

    pub(crate) fn writable(&self) -> bool {
        self.writable
    }
}

/// A list of load paths.
#[derive(Debug)]
pub(crate) struct LoadPathList {
    paths: Vec<LoadPath>,
}

impl LoadPathList {
    /// System Mode load paths:
    ///
    /// - `/etc/voa/`
    /// - `/run/voa/`
    /// - `/usr/local/share/voa/`
    /// - `/usr/share/voa/`
    pub(crate) fn load_path_list_system() -> LoadPathList {
        /// Load paths for "system mode" operation of VOA.
        /// Triplets of path name, a flag for emphemerality, and a flag for writability.
        const LOAD_PATHS_SYSTEM_MODE: &[(&str, bool, bool)] = &[
            ("/etc/voa/", false, true),
            ("/run/voa/", true, true),
            ("/usr/local/share/voa/", false, false),
            ("/usr/share/voa/", false, false),
        ];

        let paths = LOAD_PATHS_SYSTEM_MODE
            .iter()
            .map(|(path, ephemeral, writable)| LoadPath::new(path, *ephemeral, *writable))
            .collect();

        LoadPathList { paths }
    }

    /// User Mode load paths:
    ///
    /// - `$XDG_CONFIG_HOME/voa/`
    /// - the `./voa/` directory in each directory defined in `$XDG_CONFIG_DIRS`
    /// - `$XDG_RUNTIME_DIR/voa/`
    /// - `$XDG_DATA_HOME/voa/`
    /// - the `./voa/` directory in each directory defined in `$XDG_DATA_DIRS`
    pub(crate) fn load_path_list_user() -> LoadPathList {
        let mut paths = vec![];

        if let Some(proj_dirs) = directories::ProjectDirs::from("voa", "VOA", "VOA") {
            // 1. $XDG_CONFIG_HOME/voa/
            paths.push(LoadPath::new(
                proj_dirs.config_dir().to_path_buf(),
                false,
                true,
            ));

            // 2. the ./voa/ directory in each directory defined in $XDG_CONFIG_DIRS
            let xdg = xdg::BaseDirectories::with_prefix("voa");

            xdg.get_config_dirs()
                .into_iter()
                .for_each(|dir| paths.push(LoadPath::new(dir, false, false)));

            // 3. $XDG_RUNTIME_DIR/voa/
            if let Some(runtime_dir) = proj_dirs.runtime_dir() {
                paths.push(LoadPath::new(runtime_dir, true, true));
            }

            // 4. $XDG_DATA_HOME/voa/
            paths.push(LoadPath::new(proj_dirs.data_dir(), false, false));

            // 5. the ./voa/ directory in each directory defined in $XDG_DATA_DIRS
            let mut data_dirs = xdg.get_data_dirs();

            // If $XDG_DATA_DIRS is either not set or empty, a value equal to
            // /usr/local/share/:/usr/share/ should be used.
            if data_dirs.is_empty() {
                data_dirs.push("/usr/local/share/voa/".into());
                data_dirs.push("/usr/share/voa/".into());
            }

            data_dirs
                .into_iter()
                .for_each(|dir| paths.push(LoadPath::new(dir, false, false)));
        }

        LoadPathList { paths }
    }

    /// A set of load paths into which symlinks may point from `current`, for this LoadPathList.
    ///
    /// This returns a subset of [LoadPath]s in this [LoadPathList], starting with `current` and
    /// encompassing all load paths with lower priority. *Ephemeral* [LoadPath]s are excluded from
    /// the result.
    ///
    /// If `current` is not contained in `self.paths`, an empty list is returned.
    pub(crate) fn legal_symlink_load_paths(&self, current: &LoadPath) -> Vec<&LoadPath> {
        let mut legal = vec![];

        // We're searching for "source" in self
        let mut searching = true;

        for path in &self.paths {
            if searching {
                if path.path == current.path {
                    searching = false;

                    if !path.ephemeral {
                        legal.push(path);
                    }
                }
            } else if !path.ephemeral {
                legal.push(path);
            }
        }

        legal
    }

    pub(crate) fn paths(&self) -> &[LoadPath] {
        &self.paths
    }
}

// TODO: add unit tests for [LoadPathList::legal_symlink_load_paths]
