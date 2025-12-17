# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-12-17

### Added

- Use `alpm_types::SonameV1::shared_object_name` for `SonameV1::name`

### Fixed

- Add `Soname` and `SonameV2` to `__all__`
- Make the `PackageVersion` parser allow character set from the spec

### Other

- [**breaking**] Update Python bindings for RelationOrSoname type

## [0.3.0] - 2025-11-15

### Added

- Support whitespace separated PGP key fingerprint format
- Support for CRC-32/CKSUM algorithm and `cksums` field
- [**breaking**] Rename `RelativePath` to `RelativeFilePath`

### Fixed

- *(python)* Return Iterator instead of Iterable in __iter__

### Other

- *(readme)* Fix links broken on PyPI
- *(deps)* Update dependency pytest to v9

## [0.2.1] - 2025-10-31

### Other

- *(metadata)* Add further PyPI classifiers
- *(metadata)* Add README.md (in sdist and for PyPI)
- *(metadata)* Add project URLs
- *(metadata)* Add relevant keywords for PyPI
- *(metadata)* Add a description for PyPI

## [0.2.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`
- *(git-cliff)* Do not bump major version on breaking change by default

### Fixed

- *(deps)* Update Rust crate pyo3 to 0.27
- Revert "chore: Allow publishing the crate on crates.io"
- *(cargo)* Remove the publish restriction again

### Other

- *(deps)* Update dependency ruff to >=0.14,<0.15
- Add ARCHITECTURE.md
- Use staticmethod instead of classmethod
- *(deps)* Update dependency pdoc to v16
- Add cliff.toml
- *(changelog)* Remove empty `unreleased` section
- Rename `alpm` Python package to `python-alpm`
- *(readme)* Add information about PyPI name conflict

## [0.1.0-alpha.0] - 2025-10-07

### Added

- Use `PathBuf` instead of `RelativePath` for read-only properties
- Add third batch of alpm-srcinfo Python bindings
- *(python)* Add pdoc dev dependency for generating docs
- Add second batch of alpm-srcinfo Python bindings
- Add third batch of alpm-types Python bindings
- *(python)* Turn on PyO3 extension-module feature for manylinux compliance
- *(python)* Add union type aliases
- Add initial batch of Python bindings for alpm-srcinfo
- Add second batch of Python bindings for alpm-types
- Create initial batch of Python bindings for the ALPM project

### Fixed

- *(python)* Add missing commas in test
- *(python)* Add a missing property decorator to `SonameV1.form`
- Clippy lints
- *(python)* Implement a workaround for dotted paths import
- *(python)* Fix Checksum import error
- *(deps)* Update Rust crate pyo3 to 0.26

### Other

- Allow publishing the crate on crates.io
- *(python)* Configure doctest
- *(readme)* Use Python REPL sessions in examples
- *(python)* Unify `__repr__` and `__str__` implementations
- Simplify getters using `vec_convert!`
- Remove unused `Checksum` and `SkippableChecksum` enum unions
- *(python)* Add link to docs and more examples to README.md
- *(python)* Use stricter ruff rules and apply required changes
- *(python)* Use mypy strict mode and fix all reported issues
- *(release)* Set python-alpm version to `0.1.0-alpha.0`
- Implement From trait for PyO3 binding newtypes using a macro
- *(deps)* Update dependency ruff to >=0.13,<0.14
- Simplify python-alpm project layout
- *(python)* Set the minimum supported Python version to 3.10
- Make semver a workspace dependency
- *(python)* Add tests for alpm-types Python bindings
