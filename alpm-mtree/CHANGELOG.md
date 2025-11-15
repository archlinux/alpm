# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-11-15

### Added

- Localize error messages for alpm-mtree
- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors

### Other

- *(readme)* Remove `cli` feature enabled by default

## [0.2.3] - 2025-10-30

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Hide cli module documentation
- Cleanup mtree modules, dependencies and feature flags

## [0.2.2] - 2025-10-07

### Fixed

- Update to rstest v0.26.1 and fix lints

### Other

- Fix badly fenced code blocks
- Fix violations of MD040
- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag

## [0.2.1] - 2025-07-24

### Other

- Make which a workspace dependency

## [0.2.0] - 2025-06-16

### Added

- *(cargo)* Use the workspace linting rules
- Add `Mtree::validate_paths` to validate file system paths
- Enable comparing ALPM-MTREE paths to file system paths
- Consolidate data types for gid, size, time and uid
- Use a sorted list of `Path` objects in the `Mtree` variants
- Add `Eq`, `Ord` and `PartialOrd` for `Path` and its variants
- Add functions for the creation of ALPM-MTREE files
- Derive `PartialEq` for `Mtree`, `MtreeV1` and `MtreeV2`
- Make all members of structs for Path variants public
- Rely on `Mtree` when parsing ALPM-MTREE data
- Add `Mtree` as entry point for reading ALPM-MTREE data
- Add `MtreeSchema` to track ALPM-MTREE data schemas

### Fixed

- *(deps)* Update Rust crate which to v8
- *(codespell)* Ignore false-positive `ser` using codespell config
- *(Mtree)* [**breaking**] Auto-detect gzip compressed readers and decompress them
- *(mtree)* Ensure that stdin can be used in cli
- *(mtree)* Fix default value for --output-format
- *(cargo)* Use the package's README instead of the workspace README

### Other

- [**breaking**] Rename functions for the creation of ALPM-MTREE files
- *(deps)* Move `flate2` crate to workspace dependencies
- Add missing documentation for all public items
- Cleanup unneeded return statements
- *(justfile)* Add cargo-sort-derives
- Move `simplelog` crate to workspace dependencies
- Move `log` crate to workspace dependencies
- Add mtree cli tests
- Add helper macros for parse error contexts
- Use winnow's new error context functions
- Fix typos in README.md of file formats
- Configure relevant tests with "cli" feature
- Move ALPM-MTREE (v1/v2) handling to separate module
- Move handling of gzip compression to separate module
- *(cargo)* Consolidate and sort package section

## [0.1.0] - 2025-02-28

### Added

- *(mtree)* Add cli feature flag
- winnow-debug flags for winnow debugging
- *(mtree)* Custom UTF-8 decoder
- *(mtree)* Format subcommand and json serialization
- *(mtree)* Build mtree v2 interpreter
- *(mtree)* Add the mtree parser
- *(mtree)* Skeleton mtree crate

### Other

- Consolidate keywords in the the `SEE ALSO` section
- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- *(format)* Merge imports
- *(deps)* Bump winnow
- *(mtree)* Expose helper function for gzip decompression
- *(mtree)* Clean up API surface
- *(mtree)* README examples
- *(mtree)* path decoder unit tests
- *(mtree)* Happy path parsing
- *(mtree)* interpreter errors
- *(mtree)* parser errors
- Make alpm-types and alpm-parser workspace dependencies
- *(mtree)* Extend mtree specification
- *(readme)* Mention the official announcement in top-level documentation
- Move listing of specifications to mdbook setup
- Add/ Update links to latest project documentation
- *(mtree)* Update main README for new MTREE specs
- *(mtree)* MTree v2 Specification
- *(mtree)* MTree v1 Specification
- *(README)* Update components section with current libraries
- *(README)* Add information on specs and implementations
- *(README)* Add visualization providing a project overview
- *(README)* Add links to all current specifications
- Add initial project README
