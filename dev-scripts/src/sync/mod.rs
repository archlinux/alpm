use strum::{Display, EnumIter};

/// The [mirror] module contains all logic to download data from an Arch Linux package mirror.
/// This includes the database files or packages.
pub mod mirror;

/// All Arch Linux package repositories we may want to test.
#[derive(EnumIter, Display, Debug, PartialEq)]
pub enum PackageRepositories {
    #[strum(to_string = "core")]
    Core,
    #[strum(to_string = "extra")]
    Extra,
    #[strum(to_string = "multilib")]
    Multilib,
}
