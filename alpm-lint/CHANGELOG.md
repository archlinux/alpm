# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-11-15

### Added

- Replace the use of `clap-verbosity` with `clap-verbosity-flag`
- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors
- [**breaking**] Remove `PackageInfoV1` and `PackageInfoV2` `new` constructors
- [**breaking**] Replace `Vec<ExtraData>` with `ExtraData` newtype
- Update unsafe checksums lint to warn about `cksums`
- *(lint)* Add `UnknownArchitecture` lint

### Fixed

- Adjust alpm-lint architecture link

### Other

- Fix broken link to alpm-lint API

## [0.1.1] - 2025-10-31

### Fixed

- Set correct name attribute for alpm-lint clap integration

## [0.1.0] - 2025-10-30

### Added

- UnsafeChecksum lint
- UndefinedArchitecture lint
- OpenPGPKeyId lint
- InvalidSPDxLicense lint
- NoArchitecture lint
- DuplicateArchitecture lint
- Add lint impl example
- Lint issue rendering
- Introduce alpm-lint framework

### Fixed

- Remove test_option from invalid_spdx_license lint

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Adjust alpm-lint for new Architecture
- Cleanup lint modules, dependencies and feature flags
- Add CLI integration tests
