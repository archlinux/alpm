# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
