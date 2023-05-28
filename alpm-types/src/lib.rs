// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later

#![doc = include_str!("../README.md")]

mod checksum;
pub use checksum::Md5Sum;

mod date;
pub use date::BuildDate;

mod env;
pub use env::BuildEnv;
pub use env::BuildOption;
pub use env::PackageOption;

mod error;
pub use error::Error;

mod macros;
use macros::regex_once;

mod name;
pub use name::BuildTool;
pub use name::Name;

mod path;
pub use path::AbsolutePath;
pub use path::BuildDir;

mod pkg;
pub use pkg::Packager;
pub use pkg::PkgType;

mod size;
pub use size::CompressedSize;
pub use size::InstalledSize;

mod system;
pub use system::Architecture;

mod version;
pub use version::BuildToolVer;
pub use version::Epoch;
pub use version::Pkgrel;
pub use version::Pkgver;
pub use version::SchemaVersion;
pub use version::Version;
