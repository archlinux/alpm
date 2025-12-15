//! Solution representation for dependency resolution.

use std::{
    collections::HashMap,
    fmt,
    fmt::{Display, Formatter},
};

use alpm_common::GenericInstalledPackageMetadata;
use alpm_types::{FullVersion, Name, PackageRelation};
use resolvo::SolvableId;

use crate::{
    PackageMetadataOrigin,
    provider::ALPMDependencyProvider,
    types::{PackageRecord, RelationName, Replaces},
    utils::into_package_relation,
};

/// Represents an action to be performed for dependency resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencyResolutionAction {
    /// A new package has to be installed.
    Install {
        name: Name,
        version: FullVersion,
        source: PackageMetadataOrigin,
    },
    /// An existing package has to be upgraded or downgraded.
    Change {
        name: Name,
        from_version: FullVersion,
        to_version: FullVersion,
        source: PackageMetadataOrigin,
    },
    /// An existing package has to be removed due to conflicts.
    Remove {
        name: Name,
        conflicts_with: HashMap<PackageRelation, Replaces>,
    },
    /// An existing package can be (optionally) removed as it is no longer required.
    NotRequired(Name),
    /// No action is required for the package.
    NoAction(Name),
}

impl DependencyResolutionAction {
    /// Returns the name of the package associated with this action.
    pub fn name(&self) -> &Name {
        match self {
            DependencyResolutionAction::Install { name, .. } => name,
            DependencyResolutionAction::Change { name, .. } => name,
            DependencyResolutionAction::NotRequired(name) => name,
            DependencyResolutionAction::Remove { name, .. } => name,
            DependencyResolutionAction::NoAction(name) => name,
        }
    }

    /// Returns whether this action requires any changes to be made.
    pub fn is_required(&self) -> bool {
        !matches!(
            self,
            DependencyResolutionAction::NoAction(_) | DependencyResolutionAction::NotRequired(_)
        )
    }
}

impl Display for DependencyResolutionAction {
    /// Todo - i18n
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DependencyResolutionAction::Install {
                name,
                version,
                source,
            } => {
                write!(
                    f,
                    "[+] {:>12} {:<25} from {:<14} {}",
                    "install:",
                    name.to_string(),
                    source.to_string(),
                    version
                )
            }
            DependencyResolutionAction::Change {
                name,
                from_version,
                to_version,
                source,
            } => {
                if from_version < to_version {
                    write!(
                        f,
                        "[^] {:>12} {:<25} from {:<14} {:─<16}► {}",
                        "upgrade:",
                        name.to_string(),
                        source.to_string(),
                        from_version.to_string(),
                        to_version
                    )
                } else {
                    write!(
                        f,
                        "[⌄] {:>12} {:<25} from {:<14} {:─<16}► {}",
                        "downgrade:",
                        name.to_string(),
                        source.to_string(),
                        from_version.to_string(),
                        to_version
                    )
                }
            }
            DependencyResolutionAction::Remove {
                name,
                conflicts_with,
            } => {
                writeln!(f, "[-] {:>12} {}", "remove:", name)?;
                let mut conflicts = conflicts_with.iter().peekable();
                while let Some((conflict, replaces)) = conflicts.next() {
                    let (tree_char, newline) = if conflicts.peek().is_some() {
                        ("├", "\n")
                    } else {
                        ("└", "")
                    };
                    let replaces = if *replaces {
                        "replacement:"
                    } else {
                        "conflict:"
                    };
                    write!(f, " {}{:─>14} {}{}", tree_char, replaces, conflict, newline)?;
                }
                Ok(())
            }
            DependencyResolutionAction::NotRequired(name) => {
                write!(
                    f,
                    "[?] {:>12} {:<25} is no longer required",
                    "remove?",
                    name.to_string()
                )
            }
            DependencyResolutionAction::NoAction(name) => {
                write!(f, "[=] {:>12} {}", "no action:", name)
            }
        }
    }
}

/// Represents a solution for dependency resolution problem.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Solution {
    /// Actions to be performed for package resolution.
    actions: Vec<DependencyResolutionAction>,
}

impl From<Vec<DependencyResolutionAction>> for Solution {
    /// Creates a [`Solution`] from a vector of [`DependencyResolutionAction`]s.
    fn from(actions: Vec<DependencyResolutionAction>) -> Self {
        Solution { actions }
    }
}

impl Solution {
    /// Creates a new [`Solution`] from the given initial state and raw solver output.
    ///
    /// Uses `initial_state` for diffing against `raw_solution`, `provider` is used to extract data
    /// based on interned solvables and for detecting conflicts and replacements suggested by the
    /// solver.
    pub(crate) fn new<I>(
        provider: &ALPMDependencyProvider,
        initial_state: Vec<I>,
        raw_solution: Vec<SolvableId>,
    ) -> Self
    where
        I: GenericInstalledPackageMetadata + Clone,
    {
        let mut packages = initial_state.clone();
        let mut actions = Vec::new();
        let mut new_state = Vec::new();

        for solvable_id in raw_solution {
            let solvable = provider.pool.resolve_solvable(solvable_id);
            let name = provider.pool.resolve_package_name(solvable.name);

            // We only care about real packages.
            // The solution already includes providers of all virtual components and sonames.
            let (
                PackageRecord::Real {
                    version: new_version,
                    source,
                    ..
                },
                // All real packages are relations
                RelationName::Relation(name),
            ) = (solvable.record.clone(), name)
            else {
                continue;
            };

            let remove_idx = match packages
                .iter()
                .enumerate()
                .find(|(_, installed_pkg)| installed_pkg.get_name() == name)
            {
                Some((package_idx, installed_pkg)) => {
                    let installed_version = installed_pkg.get_version();
                    if new_version != *installed_version {
                        actions.push(DependencyResolutionAction::Change {
                            name: name.clone(),
                            from_version: installed_version.clone(),
                            to_version: new_version,
                            source,
                        });
                    } else {
                        actions.push(DependencyResolutionAction::NoAction(name.clone()));
                    }
                    Some(package_idx)
                }
                None => {
                    actions.push(DependencyResolutionAction::Install {
                        name: name.clone(),
                        version: new_version,
                        source,
                    });
                    new_state.push(name.clone());
                    None
                }
            };
            if let Some(idx) = remove_idx {
                packages.remove(idx);
            }
        }

        // Check whether remaining packages _have to_ be removed due to conflict
        // or are just no longer needed.
        for pkg in packages {
            let relation = into_package_relation(&pkg);
            let relation_unversioned = PackageRelation {
                name: relation.name.clone(),
                version_requirement: None,
            };

            // reverse conflicts can be unversioned so we have to check that too
            let conflicts_unversioned = provider
                .conflicts_map
                .get(&relation_unversioned)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            let mut conflicts = provider
                .conflicts_map
                .get(&relation)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            conflicts.extend(conflicts_unversioned);

            let conflicts = conflicts
                .into_iter()
                .filter(|(conflict, _)| new_state.contains(&conflict.name))
                .collect::<HashMap<PackageRelation, Replaces>>();

            if conflicts.is_empty() {
                actions.push(DependencyResolutionAction::NotRequired(
                    pkg.get_name().clone(),
                ));
            } else {
                actions.push(DependencyResolutionAction::Remove {
                    name: pkg.get_name().clone(),
                    conflicts_with: conflicts,
                });
            }
        }

        actions.sort_by_key(|action| action.name().clone());

        Self::from(actions)
    }
}

impl AsRef<[DependencyResolutionAction]> for Solution {
    /// Returns a list of [`DependencyResolutionAction`]s to satisfy requirements.
    fn as_ref(&self) -> &[DependencyResolutionAction] {
        &self.actions
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for action in self.actions.iter().filter(|a| a.is_required()) {
            writeln!(f, "{}", action)?;
        }
        Ok(())
    }
}
