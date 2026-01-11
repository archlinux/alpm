# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-01-11

### Fixed

- Ignore backup entries with null digest in alpm-db-files
- Initialize `fluent_i18n` support for the `alpm-db-desc` binary

### Other

- *(README)* Fix docs.rs link of the crate
- *(README)* Streamline wording and use of "command line interface"
- *(cargo)* Make crate description more generic

## [0.2.0] - 2025-12-17

### Added

- Add alpm-db-files to alpm-db crate
- [**breaking**] Use `FullVersion`, not `Version` in `DbDescFileV1`, `DbDescFileV2`
- Support multiple validation entries in alpm-db-desc
- [**breaking**] Use commas as `value_delimiter` for `--optdepends`
- Localize error messages for alpm-db

### Other

- Mention multiple validation entries in alpm-db-desc specification
- Add required paths for `alpm-db-files` specification example
- Split alpm-files specifications
- *(readme)* Add doctest for creating `alpm-db-desc` using CLI
- Use RelationOrSoname for provides/depends in alpm-db-desc

## [0.1.0] - 2025-11-15

### Added

- [**breaking**] Remove `PackageInfoV1` and `PackageInfoV2` `new` constructors
- [**breaking**] Replace `Vec<ExtraData>` with `ExtraData` newtype
- Add desc module to alpm-db
- Initialize a bare `alpm-db` crate

### Fixed

- Ensure that sections in `alpm-db-desc` adhere to the specification
- Use correct types for `replaces`, `conflicts` and `provides`
- Change winnow error messages to not duplicate `expected`

### Other

- *(cargo)* Adjust the project description to also mention the CLI
- *(cargo)* Use the package README instead of the workspace README
- *(readme)* Remove `cli` feature enabled by default
- Require fewer owned types in integration tests
- Move alpm-db-desc specifications to alpm-db crate
- Add integration tests for alpm-db-desc crate
- Add documentation for alpm-db-desc
- Add specification for `alpm-db`
