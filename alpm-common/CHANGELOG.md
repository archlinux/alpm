# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-06-16

### Added
- *(cargo)* Use the workspace linting rules
- Derive `Clone`, `Copy` and `Debug` for `InputPaths`
- Derive `Clone`, `Copy` and `Debug` for `InputPath`
- Add `InputPath` and `InputPaths` helper structs
- Add functions to get relative file paths from input dirs
- Add `alpm-common` crate for common traits and functionalities

### Other
- Convert cloned reference to slice::from_ref
