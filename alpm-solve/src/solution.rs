use std::{
    fmt,
    fmt::{Display, Formatter},
};
use resolvo::SolvableId;
use alpm_common::GenericInstalledPackageMetadata;
use alpm_types::{FullVersion, Name};
use crate::error::Error;
use crate::MetadataSource;
use crate::provider::ArchDependencyProvider;
use crate::types::{PackageRecord, RelationName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageResolutionAction {
    /// A new package has to be installed.
    Install {
        name: Name,
        version: FullVersion,
        source: MetadataSource,
    },
    /// An existing package has to be upgraded or downgraded.
    Change {
        name: Name,
        from_version: FullVersion,
        to_version: FullVersion,
        source: MetadataSource,
    },
    /// An existing package has to be removed.
    Remove(Name),
    /// No action is required for the package.
    NoAction(Name),
}

impl PackageResolutionAction {
    pub fn name(&self) -> &Name {
        match self {
            PackageResolutionAction::Install { name, .. } => name,
            PackageResolutionAction::Remove(name) => name,
            PackageResolutionAction::Change { name, .. } => name,
            PackageResolutionAction::NoAction(name) => name,
        }
    }

    pub fn action_required(&self) -> bool {
        !matches!(self, PackageResolutionAction::NoAction(_))
    }
}

impl Display for PackageResolutionAction {
    /// Todo - i18n
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PackageResolutionAction::Install {
                name,
                version,
                source,
            } => {
                write!(
                    f,
                    "[+] {:>10} {:<25} from {:<14} {}",
                    "install:",
                    name.to_string(),
                    source.to_string(),
                    version
                )
            }
            PackageResolutionAction::Remove(name) => {
                write!(f, "[-] {:>10} {}", "remove:", name)
            }
            PackageResolutionAction::Change {
                name,
                from_version,
                to_version,
                source,
            } => {
                if from_version < to_version {
                    write!(
                        f,
                        "[^] {:>10} {:<25} from {:<14} {:─<16}► {}",
                        "upgrade:",
                        name.to_string(),
                        source.to_string(),
                        from_version.to_string(),
                        to_version
                    )
                } else {
                    write!(
                        f,
                        "[⌄] {:>10} {:<25} from {:<14} {:─<16}► {}",
                        "downgrade:",
                        name.to_string(),
                        source.to_string(),
                        from_version.to_string(),
                        to_version
                    )
                }
            }
            PackageResolutionAction::NoAction(name) => {
                // todo: probably should be locked behind some verbose flag? This is pure clutter.
                write!(f, "[=] {:>10} {}", "no action:", name)
            }
        }
    }
}

/// Represents a solution for dependency resolution problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    /// Actions to be performed for package resolution.
    pub actions: Vec<PackageResolutionAction>,
}

impl From<Vec<PackageResolutionAction>> for Solution {
    fn from(actions: Vec<PackageResolutionAction>) -> Self {
        Solution { actions }
    }
}

impl Solution {
    /// Constructs a new [`Solution`] from the given initial state and raw solver output.
    pub fn new<I>(
        provider: &ArchDependencyProvider,
        initial_state: Vec<I>,
        raw_solution: Vec<SolvableId>,
    ) -> Result<Solution, Error>
    where
        I: GenericInstalledPackageMetadata + Clone,
    {
        let mut packages = initial_state.clone();
        let mut actions = Vec::new();

        for solvable_id in raw_solution {
            let solvable = provider.pool.resolve_solvable(solvable_id);
            let name = provider.pool.resolve_package_name(solvable.name);
            // We only care about real packages.
            // The solution already includes providers of all virtual packages.
            if let PackageRecord::Real{ version, source, ..} = solvable.record.clone()
                // All real packages are relations
                && let RelationName::Relation(name) = name
            {
                let new_version: FullVersion = version.try_into()?;
                let idx = match packages
                    .iter()
                    .enumerate()
                    .find(|(_, installed_pkg)| installed_pkg.get_name() == name)
                {
                    Some((idx, installed_pkg)) => {
                        let installed_version = installed_pkg.get_version();
                        if new_version != *installed_version {
                            actions.push(PackageResolutionAction::Change {
                                name: name.clone(),
                                from_version: installed_version.clone(),
                                to_version: new_version,
                                source,
                            });
                        } else {
                            actions.push(PackageResolutionAction::NoAction(name.clone()));
                        }
                        Some(idx)
                    }
                    None => {
                        actions.push(PackageResolutionAction::Install {
                            name: name.clone(),
                            version: new_version,
                            source,
                        });
                        None
                    }
                };
                if let Some(idx) = idx {
                    packages.remove(idx);
                }
            }
        }

        // Packages left in the `packages` are to be removed.
        for pkg in packages.iter() {
            actions.push(PackageResolutionAction::Remove(pkg.get_name().clone()));
        }

        actions.sort_by_key(|action| action.name().clone());

        Ok(Self::from(actions))
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for action in self.actions.iter().filter(|a| a.action_required()) {
            writeln!(f, ":: {}", action)?;
        }
        Ok(())
    }
}
