# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2025-12-17

### Fixed

- Fix clippy warnings for Rust 1.92.0

## [0.4.0] - 2025-11-15

### Added

- Localize error messages for alpm-package
- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors
- [**breaking**] Remove `PackageInfoV1` and `PackageInfoV2` `new` constructors

## [0.3.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`
- [**breaking**] Use `alpm-compress::tarball` for building and reading packages
- [**breaking**] Rename `Error::Compression` to `Error::AlpmCompress`
- [**breaking**] Replace `Option<CompressionSettings>` with `CompressionSettings`
- [**breaking**] Add `None` variant to encoder, decoder and settings
- [**breaking**] Remove `compression` module from `alpm-package`
- [**breaking**] Replace usages of `alpm_package::compression` with `alpm-compress`

## [0.2.1] - 2025-10-07

### Fixed

- Metadata files being reported as missing on error
- Wrong lifetime on DataEntry::entry()
- Return conventional types from getters

### Other

- Fix violations of MD034
- Fix links to other man pages in `alpm-package` specification
- Fix violations of MD029
- Fix violations of MD022 and MD032

## [0.2.0] - 2025-07-24

### Added

- Support reading package contents
- Add CompressionDecoder for multi-format decompression
- [**breaking**] Use `FullVersion`, not `Version` in `BuildInfo` and `PackageInfo`
- [**breaking**] Use `FullVersion`, not `Version` in `PackageFileName`

### Fixed

- *(deps)* Update Rust crate bzip2 to 0.6.0

## [0.1.0] - 2025-06-16

### Added

- Add integration for creating package files from input directories
- *(cargo)* Use the workspace linting rules
- Initialize new crate `alpm-package`

### Fixed

- *(cargo)* Use the package's README instead of the workspace README

### Other

- *(README)* Add initial text with top level info on package creation
- Add initial library documentation
- Sort the list of compression algorithm file extensions
- Remove duplicate mention of alpm-package-name
- Add alpm-package specification
