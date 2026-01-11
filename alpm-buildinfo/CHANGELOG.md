# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2] - 2026-01-11

### Other

- *(README)* Improve wording and specification links
- *(cargo)* Make crate description more concise

## [0.5.1] - 2025-12-17

### Other

- Use a `mod.rs` for the `alpm_buildinfo::build_info` module

## [0.5.0] - 2025-11-15

### Added

- Localize error messages for alpm-buildinfo
- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors

### Other

- *(readme)* Remove `cli` feature enabled by default
- Clarify `format` keyword description
- *(cargo)* Move `serde_with` to workspace dependencies

## [0.4.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Hide cli module documentation
- Cleanup buildinfo modules, dependencies and feature flags

## [0.3.1] - 2025-10-07

### Fixed

- Return conventional types from getters

### Other

- Fix violations of MD034
- Fix violations of MD029
- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag

## [0.3.0] - 2025-07-24

### Added

- [**breaking**] Use `FullVersion`, not `Version` in `BuildInfo` and `PackageInfo`

### Other

- Simplify the information on `alpm-package-base` related values

## [0.2.0] - 2025-06-16

### Added

- *(cargo)* Use the workspace linting rules
- Derive `PartialEq` for `BuildInfo`, `BuildInfoV1`, `BuildInfoV2`
- [**breaking**] Fully validate makepkg's BUILDENV and OPTIONS
- Implement `alpm_common::MetadataFile` for `BuildInfo`
- Implement `alpm_common::FileFormatSchema` for `BuildInfoSchema`
- Switch to `Schema` version 2 as default
- Rely on `BuildInfo` type when parsing BUILDINFO data
- Add `BuildInfo` as entry point for reading BUILDINFO data

### Fixed

- Remove old buildinfo insta snapshots
- [**breaking**] Make `Schema` enum not `non_exhaustive`
- *(cargo)* Use the package's README instead of the workspace README

### Other

- Add missing documentation for all public items
- Cleanup unneeded return statements
- *(justfile)* Add cargo-sort-derives
- Fix typos in README.md of file formats
- Deduplicate buildinfo tests
- *(deps)* Remove unused version key for alpm-common
- Rely on inner data for `Display` impl of `BuildInfoSchema`
- Improve documentation of `BuildInfoSchema` trait impl blocks
- Rename `Schema` to `BuildInfoSchema`
- Move `BuildInfoV1` and `BuildInfoV2` to a module
- *(cargo)* Consolidate and sort package section

## [0.1.0] - 2025-02-28

### Added

- *(srcinfo)* Add srcinfo parser
- *(buildinfo)* Add cli feature flag
- *(types)* Rename BuildEnv -> BuildEnvironmentOption
- winnow-debug flags for winnow debugging
- *(buildinfo)* Support pretty-printing the parse output
- *(buildinfo)* Add format subcommand
- *(alpm-parser)* Use winnow for custom-ini parsing
- *(scripts)* Add file testing command
- *(types)* Add value to RegexDoesNotMatch Error
- *(buildinfo)* expose validate and create methods
- *(buildinfo)* Extend CLI for buildinfo v2 validation/creation
- *(buildinfo)* Adds buildinfo version 2 struct and schema
- *(buildinfo)* parse files with the custom INI parser
- *(cli)* support piping from stdin
- Expose common module publicly to be able to run doc tests
- Add alpm-buildinfo CLI for validating and creation
- Add library implementation of BuildInfoV1
- Add specification for BUILDINFOv1 as man page

### Fixed

- Sanitize `cargo-insta` snapshot names
- *(tests)* Replace testdir with tempfile
- *(buildinfo)* Use macro for flattened struct generation
- *(alpm-types)* Make BuildTool version architecture optional
- Adapt documentation links to workspace locations
- Use automatic instead of bare links to documentation
- Derive default for Schema enum instead of using an impl block
- *(deps)* update rust crate clap_complete to 4.4.4
- *(deps)* update rust crate clap to 4.4.8
- Change README license to GFDL-1.3-or-later

### Other

- Consolidate keywords in the the `SEE ALSO` section
- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- Streamline wording around keyword assignments
- *(README)* Sort components alphabetically
- *(README)* Sort links alphabetically
- *(srcinfo)* README
- *(format)* Merge imports
- *(types)* Rename StartDir -> StartDirectory
- *(types)* Rename BuildDir -> BuildDirectory
- *(README)* Add missing link target for alpm-pkginfo
- *(README)* Add information on releases and OpenPGP verification
- Add alpm-pkginfo to mdbook setup and top-level README
- *(error)* Use thiserror macro inline to avoid conflicts
- *(buildinfo)* Use v2 format examples in README.md
- *(buildinfo)* Simplify the integration tests for alpm-buildinfo
- *(buildinfo)* Assert the output of commands in README.md
- *(buildinfo)* Merge imports
- *(buildinfo)* Clean up API surface
- *(workspace)* Make testdir and erased-serde workspace dependency
- *(buildinfo)* Fix grammar warning in doc comment
- *(types)* Change Name::new parameter from String to &str
- Make alpm-types and alpm-parser workspace dependencies
- *(types)* Use consistent 'Errors' section in doc comments
- *(readme)* Mention the official announcement in top-level documentation
- *(buildinfo)* Implement Serialize for v1/v2 types
- *(buildinfo)* Use ExitCode for simpler exit handling
- *(buildinfo)* Simplify the error type
- *(buildinfo)* Use TestResult in unit tests
- *(buildinfo)* Avoid unwrapping in doc comments
- Move listing of specifications to mdbook setup
- Add/ Update links to latest project documentation
- Make insta a workspace dependency
- *(mtree)* Update main README for new MTREE specs
- *(deps)* Move testresult to workspace dependencies
- *(types)* Use consistent constructors
- *(types)* Rename BuildToolVer to BuildToolVersion
- *(types)* Add type aliases for MakePkgOption
- *(types)* Add type aliases for i64
- *(buildinfo)* Add integration tests for buildinfo v2
- *(buildinfo)* create a helper function for guessing Schema
- *(buildinfo)* Expose buildinfo v1 fields inside of crate
- [**breaking**] Move `Schema` to its own module
- More precisely distinguish `pkgver` from `alpm-pkgver`
- *(BUILDINFO)* Adapt `buildtoolver` to current use scenarios
- *(BUILDINFO)* Rely on new specifications for `installed` keyword
- *(BUILDINFO)* Sync overlapping keyword definitions with PKGINFO
- *(BUILDINFO)* Use more generic package name value
- *(BUILDINFO)* Add release date for supported BUILDINFO versions
- *(README)* Update components section with current libraries
- *(README)* Add information on specs and implementations
- *(README)* Add visualization providing a project overview
- *(README)* Add links to all current specifications
- *(error)* use more generalized error types
- *(workspace)* update deployed documentation links
- *(buildinfo)* write formal specification for buildinfov2
- *(workspace)* use shared workspace metadata
- auto-fix lint issues in buildinfov1
- *(buildinfo)* test with snapshots
- Replace man page/ completion setup with project-wide approach
- Extend General Format and Keywords section for BUILDINFOv1
- Improve language in description of the BUILDINFOv1 specification
- Move synopsis to description section of BUILDINFOv1 specification
- Remove bugs and authors info from BUILDINFOv1 specification
- Use unversioned BUILDINFO name in BUILDINFOv1 specification
- Add symlink for BUILDINFO specification to set default version
- Move BUILDINFOv1 specification to more generic directory
- *(workspace)* move more dependencies to workspace
- Unify and bump workspace dependencies
- Apply rustfmt configuration to codebase
- Adapt alpm-buildinfo cargo configuration to workspace
- *(license)* Relicense the project as Apache-2.0 OR MIT
- *(Cargo.toml)* [**breaking**] Update minimum required rust-version to 1.70.0
- *(deps)* update rust crate rstest to 0.18.2
- Add information on where to find documentation.
- Add CLI examples to README
- Add information on creating BuildInfoV1 to README
- Add README, contributing guidelines, changelog and licenses
