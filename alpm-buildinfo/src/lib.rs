// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later

#![doc = include_str!("../README.md")]

mod buildinfo_v1;
pub use crate::buildinfo_v1::BuildInfoV1;

pub mod cli;

mod common;

mod error;
pub use crate::error::Error;
