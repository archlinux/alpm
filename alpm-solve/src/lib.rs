#![doc = include_str!("../README.md")]

mod error;
mod provider;
mod solution;
mod solver;
mod types;
mod utils;

pub use error::Error;
pub use provider::ALPMDependencyProvider;
pub use solution::{DependencyResolutionAction, Solution};
pub use solver::Solver;
pub use types::{MetadataSourcePriority, PackageMetadataOrigin};
