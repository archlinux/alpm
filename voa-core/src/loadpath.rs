//! VOA load path handling

use std::path::PathBuf;

/// A VOA load path.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadPath {
    pub(crate) path: PathBuf,
    ephemeral: bool,
}

impl From<(&str, bool)> for LoadPath {
    fn from(value: (&str, bool)) -> Self {
        LoadPath {
            path: value.0.into(),
            ephemeral: value.1,
        }
    }
}

impl From<(PathBuf, bool)> for LoadPath {
    fn from(value: (PathBuf, bool)) -> Self {
        LoadPath {
            path: value.0,
            ephemeral: value.1,
        }
    }
}

/// A list of load paths.
#[derive(Debug)]
pub struct LoadPathList {
    pub(crate) paths: Vec<LoadPath>,
}

impl LoadPathList {
    /// A set of load paths into which symlinks may point from `current`, for this LoadPathList.
    ///
    /// This returns a subset of [LoadPath]s in this list, starting with `current` and encompassing
    /// all load paths with lower priority. Ephemeral LoadPaths are excluded from the result.
    ///
    /// If `current` is not contained in `self.paths`, an empty list is returned.
    #[allow(dead_code)]
    pub fn legal_link_load_paths(&self, current: &LoadPath) -> Vec<&LoadPath> {
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
}

pub(crate) fn load_paths_sys() -> LoadPathList {
    /// Load paths for "system mode" operation of VOA.
    /// Pairs of path name and a flag for emphemerality.
    const LOAD_PATHS_SYSTEM_MODE: &[(&str, bool)] = &[
        ("/etc/voa/", false),
        ("/run/voa/", true),
        ("/usr/local/share/voa/", false),
        ("/usr/share/voa/", false),
    ];

    let paths = LOAD_PATHS_SYSTEM_MODE
        .iter()
        .map(|&path| path.into())
        .collect();

    LoadPathList { paths }
}

pub(crate) fn load_paths_user() -> LoadPathList {
    let mut paths = vec![];

    if let Some(proj_dirs) = directories::ProjectDirs::from("voa", "VOA", "VOA") {
        // 1. $XDG_CONFIG_HOME/voa/
        paths.push((proj_dirs.config_dir().to_path_buf(), false).into());

        // 2. the ./voa/ directory in each directory defined in $XDG_CONFIG_DIRS
        let xdg = xdg::BaseDirectories::with_prefix("voa");

        xdg.get_config_dirs()
            .into_iter()
            .for_each(|dir| paths.push((dir, false).into()));

        // 3. $XDG_RUNTIME_DIR/voa/
        if let Some(runtime_dir) = proj_dirs.runtime_dir() {
            paths.push(LoadPath {
                path: runtime_dir.into(),
                ephemeral: true,
            });
        }

        // 4. $XDG_DATA_HOME/voa/
        paths.push(LoadPath {
            path: proj_dirs.data_dir().into(),
            ephemeral: false,
        });

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
            .for_each(|dir| paths.push((dir, false).into()));
    }

    LoadPathList { paths }
}

#[test]
fn load_path_foo() {
    let paths = load_paths_user();
    eprintln!("user load paths: {paths:#?}");
}
