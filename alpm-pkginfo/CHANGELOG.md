# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2025-12-17

### Other

- *(deps)* Remove unused direct dependency on `winnow`
- [**breaking**] Unify RelationOrSoname in alpm-types

## [0.5.0] - 2025-11-15

### Added

- Localize error messages for alpm-pkginfo
- [**breaking**] Remove `PackageInfoV1` and `PackageInfoV2` `new` constructors
- [**breaking**] Replace `Vec<ExtraData>` with `ExtraData` newtype

### Other

- *(readme)* Remove `cli` feature enabled by default
- *(cargo)* Move `serde_with` to workspace dependencies

## [0.4.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Hide cli module documentation
- Cleanup pkginfo modules, dependencies and feature flags

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
- *(deps)* Move `pretty_assertions` to workspace dependencies

## [0.2.0] - 2025-06-16

### Added

- *(cargo)* Use the workspace linting rules
- Derive `Debug` for `PackageInfoInput`
- Enforce PackageDescription invariants
- Derive `PartialEq` for `PackageInfo` and `PackageInfoV1`/V2
- Rely on `PackageInfo` when parsing PKGINFO data
- Add `PackageInfo` as entry point for reading PKGINFO data
- Add `PkgInfoSchema` to track PKGINFO data schemas

### Fixed

- Wrong pkginfo tests
- *(cargo)* Use the package's README instead of the workspace README

### Other

- Add missing documentation for all public items
- Cleanup unneeded return statements
- Update package description specification
- *(justfile)* Add cargo-sort-derives
- Fix typos in README.md of file formats
- Enum for pkginfo write testing
- *(parser)* Add winnow parser for PackageRelation
- Move `PackageInfoV1` and `PackageInfoV2` to a module
- *(cargo)* Consolidate and sort package section

## [0.1.0] - 2025-02-28

### Added

- Add `SonameV1` and `SonameV2` support for `depend` and `provides`
- *(srcinfo)* Add srcinfo parser
- *(pkginfo)* Add cli feature flag
- *(pkginfo)* Add CLI for writing and parsing of PKGINFO files
- *(pkginfo)* Create structs for PKGINFO v1 and v2

### Fixed

- Sanitize `cargo-insta` snapshot names
- *(tests)* Replace testdir with tempfile
- *(pkginfo)* Do not reuse the same test directory

### Other

- Consolidate keywords in the the `SEE ALSO` section
- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- Streamline wording around keyword assignments
- *(README)* Sort components alphabetically
- *(README)* Sort links alphabetically
- *(srcinfo)* README
- *(format)* Merge imports
- *(types)* Rename PkgInfo -> PackageInfo
- *(types)* Rename OptDepend -> OptionalDependency
- *(types)* Rename PkgType -> PackageType
- *(types)* Rename PkgDesc -> PackageDescription
- *(README)* Add missing link target for alpm-pkginfo
- *(README)* Add information on releases and OpenPGP verification
- Add alpm-pkginfo to mdbook setup and top-level README
- *(pkginfo)* Clean up API surface
- *(pkginfo)* Assert the output of commands in README.md
- *(pkginfo)* Add integration tests for writing and parsing PKGINFO
- *(pkginfo)* Add documentation for alpm-pkginfo crate
- *(readme)* Mention the official announcement in top-level documentation
- Move listing of specifications to mdbook setup
- Add/ Update links to latest project documentation
- *(mtree)* Update main README for new MTREE specs
- More precisely distinguish `pkgver` from `alpm-pkgver`
- *(README)* Update components section with current libraries
- *(README)* Add information on specs and implementations
- *(README)* Add visualization providing a project overview
- *(README)* Add links to all current specifications
- *(PKGINFO)* Add specification for PKGINFOv2
- *(PKGINFO)* Add specification for PKGINFOv1
- Add initial project README
