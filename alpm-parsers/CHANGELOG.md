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

- [**breaking**] Remove `BuildInfoV1` and `BuildInfoV2` `new` constructors

## [0.3.1] - 2025-10-30

### Other

- Remove redundant lint attributes

## [0.3.0] - 2025-10-07

### Other

- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag

## [0.2.0] - 2025-06-16

### Added

- *(cargo)* Use the workspace linting rules
- Derive `Clone` and `Debug` for `ItemDeserializer`
- [**breaking**] Fully validate makepkg's BUILDENV and OPTIONS

### Fixed

- *(cargo)* Use the package's README instead of the workspace README

### Other

- Add missing documentation for all public items
- *(justfile)* Add cargo-sort-derives
- Add helper macros for parse error contexts
- *(cargo)* Consolidate and sort package section

## [0.1.0] - 2025-02-28

### Added

- *(srcinfo)* Add srcinfo parser
- winnow-debug flags for winnow debugging
- *(alpm-parser)* Use winnow for custom-ini parsing
- *(parsers)* implement the custom INI parser

### Fixed

- Sanitize `cargo-insta` snapshot names
- *(parser)* Allow comments in custom INI parser
- *(clippy)* Remove needless lifetimes

### Other

- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- *(README)* Sort components alphabetically
- *(README)* Sort links alphabetically
- *(srcinfo)* README
- *(format)* Merge imports
- *(deps)* Bump winnow
- *(README)* Add missing link target for alpm-pkginfo
- *(README)* Add information on releases and OpenPGP verification
- Add alpm-pkginfo to mdbook setup and top-level README
- *(readme)* Mention the official announcement in top-level documentation
- *(custom_ini)* Add test case for struct flattening
- Move listing of specifications to mdbook setup
- Add/ Update links to latest project documentation
- *(alpm-parsers)* custom-ini error messages
- Add initial project README
