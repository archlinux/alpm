# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-10-31

### Fixed

- Set correct name attribute for alpm-lint clap integration

## [0.1.0] - 2025-10-30

### Added

- UnsafeChecksum lint
- UndefinedArchitecture lint
- OpenPGPKeyId lint
- InvalidSPDxLicense lint
- NoArchitecture lint
- DuplicateArchitecture lint
- Add lint impl example
- Lint issue rendering
- Introduce alpm-lint framework

### Fixed

- Remove test_option from invalid_spdx_license lint

### Other

- *(deps)* Update Rust crate assert_cmd to v2.1.1
- Adjust alpm-lint for new Architecture
- Cleanup lint modules, dependencies and feature flags
- Add CLI integration tests
