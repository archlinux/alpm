# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] - 2025-12-17

### Fixed

- Make the `PackageVersion` parser allow character set from the spec
- Fix clippy warnings for Rust 1.92.0
- Correctly handle tabs in while deriving schema from str
- Correctly handle tabs in error messages

### Other

- Prevent CLI from parsing .SRCINFO twice

## [0.6.0] - 2025-11-15

### Added

- Localize error messages for alpm-srcinfo
- [**breaking**] Rename `RelativePath` to `RelativeFilePath`
- Support for CRC-32/CKSUM algorithm and `cksums` field

### Other

- *(readme)* Remove `cli` feature enabled by default
- Add `CRC-32/CKSUM` hash function support to specification
- Update `validpgpkeys` spec to allow whitespace separators
- Fix spelling of SRCINFO

## [0.5.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`
- Better debug output for generate_srcinfo.bash helper script

### Fixed

- Better error message for too long checksums
- Trailing whitespace handling
- Error when file ends without newline

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Hide cli module documentation
- Cleanup srcinfo modules, dependencies and feature flags
- Split package srcinfo parsing logic
- Better error for missing pkgname section
- (non)-trailing whitespaces scenarios

## [0.4.0] - 2025-10-07

### Added

- [**breaking**] Make `MergedPackage`'s merging methods private
- Add constructors for Package and PackageBase
- Remove srcinfo linting logic

### Fixed

- Correct architecture in the output of alpm-srcinfo
- Remove unused snapshots
- Remove trailing newline from the `alpm-srcinfo create` output
- Update to rstest v0.26.1 and fix lints

### Other

- Fix mangled Package::architectures docstring
- Fix comment in srcinfo
- Fix violations of MD034
- Remove alpm-srcinfo lint tests
- Fix violations of MD012
- Fix violations of MD007
- Fix violations of MD029
- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag

## [0.3.0] - 2025-07-24

### Added

- [**breaking**] Use `FullVersion` for `MergedPackage` and `PackageBase`

### Fixed

- Add bash shebang and shellcheck rules for test pkgbuild files

### Other

- Move BridgeOutput to SourceInfo conversion to alpm-srcinfo
- Make SourceInfo architecture_suffix parser public
- Convert all srcinfo tests to tabs
- Parse error for invalid architecture
- Simplify the information on `alpm-package-base` related values
- *(deps)* Move `pretty_assertions` to workspace dependencies

## [0.2.0] - 2025-06-16

### Added

- Add SRCINFO file writer
- Add PartialEq to SourceInfo
- *(cargo)* Use the workspace linting rules
- Derive `Clone` and `Debug` for `MergedSource`
- Derive `Clone` and `Debug` for `MergedPackagesIterator`
- Add format command to alpm-srcinfo
- Enforce PackageDescription invariants
- [**breaking**] Fully validate makepkg's BUILDENV and OPTIONS
- *(srcinfo)* Type to represent package overrides
- *(types)* Implement Deserialize for all srcinfo types
- Rely on `SourceInfo` when parsing SRCINFO data
- Add `SourceInfo` as entry point for reading SRCINFO data
- Add `SourceInfoSchema` to track SRCINFO schemas

### Fixed

- Make noextract not architecture specific
- Use correct type aliases for alpm-types
- Don't create blanket architecture specific properties
- Use new option wrapper type in sourceinfo
- *(architecture)* Serialize architecture as lowercase
- SourceInfo Architecture urls
- *(srcinfo)* Package versioning representation
- *(cargo)* Use the package's README instead of the workspace README

### Other

- Noextract may not be architecture specific
- Move architecture parser into own function
- Add missing documentation for all public items
- Cleanup unneeded return statements
- Update package description specification
- Change architectures to Vec
- *(justfile)* Add cargo-sort-derives
- Move RelationOrSoname to alpm_types
- Move keyword parsers to keyword enum types
- Add srcinfo cli tests
- Add helper macros for parse error contexts
- Use winnow's new error context functions
- Fix typos in README.md of file formats
- Srcinfo bin check command for now
- *(parsers)* Add winnow parser for SkippableChecksum
- *(types)* Properly type PackageRelease version data
- *(srcinfo)* Restructure files hierarchy
- Rename `SourceInfo` to `SourceInfoV1` and move to own module
- Improve parser code
- *(parser)* Add OptionalDependency winnow parser
- *(parser)* Add winnow parser for PackageRelation
- *(parser)* Add winnow parsers for PackageVersion, Epoch, PackageRelease
- *(parser)* Swap from regex-based parser to winnow for Name
- *(cargo)* Consolidate and sort package section

## [0.1.0] - 2025-02-28

### Added

- Add `SonameV1::Basic` support for `depends` and `provides`
- *(srcinfo)* Add format command for MergedPackage representation
- *(srcinfo)* Merged package representation
- *(srcinfo)* SourceInfo struct representation
- *(srcinfo)* Add srcinfo parser

### Other

- Consolidate keywords in the the `SEE ALSO` section
- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- *(SRCINFO)* Fix indentation of some links in NOTES section
- *(ARCHITECTURE.md)* Link to latest Rust docs instead of docs.rs
- *(README)* Link to rendered website for architecture documentation
- *(mtree)* Happy path parsing
- *(srcinfo)* Add ARCHITECTURE.md
- *(srcinfo)* Parse errors
- *(srcinfo)* Lint errors
- *(srcinfo)* README
- Add specification for SRCINFO file format
- *(README)* Add missing link target for alpm-pkginfo
- *(README)* Add information on releases and OpenPGP verification
- Add alpm-pkginfo to mdbook setup and top-level README
- *(readme)* Mention the official announcement in top-level documentation
- Move listing of specifications to mdbook setup
- Add/ Update links to latest project documentation
- *(mtree)* Update main README for new MTREE specs
- *(README)* Update components section with current libraries
- *(README)* Add information on specs and implementations
- *(README)* Add visualization providing a project overview
- *(README)* Add links to all current specifications
- Add initial project README
