# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-01-11

### Fixed

- Initialize `fluent_i18n` support for the `alpm-repo-desc` binary

### Other

- *(README)* Use "command line interface" and add specification links

## [0.1.0] - 2025-12-17

### Added

- Add alpm-repo-files to alpm-repo-db crate
- Add alpm-repo-desc CLI tool
- Add alpm-repo-desc parser and writer
- Initialize bare `alpm-repo-db` crate

### Fixed

- Allow sonames in dependencies and provides

### Other

- Clarify that `%DESC%` and `%URL%` may be omitted
- *(readme)* Expand the README.md
- Add README.md
- Move alpm-repo-desc spec to `alpm-repo-db`
- Move `alpm-repo-files` specification to `alpm-repo-db`
- Improve wording in `alpm-repo-db` specification
- Add specification for `alpm-repo-db`
