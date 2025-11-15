# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-11-15

### Added

- Localize error messages for alpm-soname
- Replace the use of `clap-verbosity` with `clap-verbosity-flag`
- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors
- [**breaking**] Remove `PackageInfoV1` and `PackageInfoV2` `new` constructors

### Other

- *(readme)* Remove `cli` feature enabled by default

## [0.2.0] - 2025-10-30

### Added

- [**breaking**] Replace `Option<CompressionSettings>` with `CompressionSettings`
- [**breaking**] Replace usages of `alpm_package::compression` with `alpm-compress`

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Hide cli module documentation
- Split up soname integration tests and integrated rust script
- Cleanup soname modules, dependencies and feature flags
- Make clap-verbosity-flag a workspace dependency

## [0.1.0] - 2025-10-07

### Added

- Add tests for alpm-soname
- Add alpm-soname crate for soname provision/dependency lookup

### Fixed

- Use crate README instead of workspace README
- *(deps)* Update Rust crate goblin to 0.10.0

### Other

- Add documentation for alpm-soname crate
