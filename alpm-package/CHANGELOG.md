# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
