//! Library for solving package dependencies in Arch Linux ecosystem.

pub mod error;
pub mod provider;
mod solution;
pub mod solver;
mod types;
mod utils;

pub use solution::Solution;
pub use types::{MetadataSource, MetadataSourcePriority, PackageRepositoryName};
