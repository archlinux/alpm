# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.0] - 2025-12-17

### Added

- Add RelativePath type to alpm-types
- Add `SonameV1::shared_object_name` returning the `SharedObjectName`
- Add `OptionalDependency::package_relation`
- Add `InstalledPackage::to_package_relation`
- Derive `Hash` for `SharedObjectName`
- Add `VersionRequirement::is_intersection`
- Add `Base64OpenPGPSignature` newtype

### Fixed

- Make `is_satisfied_by` ignore pkgrel if not present in requirement
- Make the `PackageVersion` parser allow character set from the spec
- Fix clippy warnings for Rust 1.92.0
- *(deps)* Update Rust crate spdx to 0.13.0

### Other

- Only allow "equal to" comparison in alpm-package-relation provision
- Implement Hash for RelativeFilePath type
- Allow dot as the first character in `pkgver`
- Disallow '<', '>', and '=' in `pkgver`
- Split up the `alpm_types::relation` module further
- Move alpm-repo-desc spec to `alpm-repo-db`
- Use the correct license for alpm-types
- Split alpm-files specifications
- Update examples for alpm-repo-descv2
- [**breaking**] Unify RelationOrSoname in alpm-types

## [0.10.0] - 2025-11-15

### Added

- Implement `Default` for `PackageInstallReason`
- Implement `Default` for `PackageDescription`
- Localize error messages for alpm-types
- [**breaking**] Replace `Vec<ExtraData>` with `ExtraData` newtype
- [**breaking**] Rename `RelativePath` to `RelativeFilePath`
- Support for CRC-32/CKSUM algorithm and `cksums` field
- Implement Deserialize for Package and ExtraData types
- Add PackageValidation to alpm-types
- Add PackageInstallReason to alpm-types
- Support whitespace separated PGP key fingerprint format

### Fixed

- Correct error message when passing a directory to `RelativePath`

### Other

- Move `alpm-files` specification to `alpm-files` crate
- Add `CRC-32/CKSUM` hash function support to specification
- Move alpm-db-desc specifications to alpm-db crate
- Add information on different `alpm-files` styles

## [0.9.0] - 2025-10-30

### Added

- [**breaking**] Reimplement `Architecture`

### Fixed

- Better error message for too long checksums

### Other

- Remove link to non-existent/unused license file
- Remove redundant lint attributes

## [0.8.0] - 2025-10-07

### Added

- Make url module public
- Implement Ord/PartialOrd for soname types
- Implement De/Serialize for soname types
- Add SonameLookupDirectory to alpm-types
- Add CheckSumAlgorithm type
- Add SkippableChecksum::is_skip helper

### Fixed

- Clippy lints
- Duplicate makepkg option
- *(deps)* Update Rust crate spdx to 0.12.0
- *(deps)* Update Rust crate spdx to 0.11.1
- Return specific error for deprecated SPDX license IDs
- *(deps)* Update Rust crate spdx to 0.11.0

### Other

- Change version comparison to MIT/APACHE-2.0 and clarify
- Make semver a workspace dependency
- Rename mentions of `alpm-repo-database` to `alpm-repo-db`
- Fix violations of MD034
- Fix ASCII figures for correct rendering with `lowdown`
- Add specification for alpm-source-repo
- Expand test cases for deprecated SPDX licenses
- Fix violations of MD001
- Fix violations of MD033
- Fix violations of MD007
- Fix violations of MD040
- Fix violations of MD029
- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag
- Fix clippy error regarding mismatched lifetime syntaxes
- Fix typo in alpm-sonamev2.7.md
- Fix copy-paste error in alpm-package-source specification

## [0.7.0] - 2025-07-24

### Added

- Impl `From<PackageFileName>` for InstalledPackage
- Support constructing `CompressionAlgorithmFileExtension` from path
- [**breaking**] Use `FullVersion`, not `Version` in `InstalledPackage`
- [**breaking**] Use `FullVersion`, not `Version` in `PackageFileName`
- Add `MinimalVersion` for the minimal `alpm-package-version` format
- Add `FullVersion` for the full `alpm-package-version` format

### Fixed

- Allow tabs at start of optdepends description
- Adjust license of `PackageVersion` comparison to LGPL-2.0-or-later

### Other

- Add version comparison algorithm documentation
- Add winnow parser for InstalledPackage
- Clean up PackageFileName parser
- Better architecture parsing error text
- Describe `alpm`
- Use winnow for packager info parsing
- *(cargo)* Set a new license identifier, independent of the workspace
- Add information on the use of LGPL-2.0-or-later
- [**breaking**] Remove `Version::with_pkgrel`
- [**breaking**] Turn `BuildToolVersion` into an enum
- Point to `FullVersion` and `MinimalVersion` use-cases in `Version`
- Split the various version handling types into further modules
- Move version module to dedicated module directory
- *(alpm-db-desc)* Generalize that `BASE` may be the same as `NAME`
- Add specification for `alpm-package-base`
- Add missing documentation for variants of `Error`
- *(specification)* Describe the `alpm-repo-desc` file format

## [0.6.0] - 2025-06-16

### Added

- Make Name hashable
- *(cargo)* Use the workspace linting rules
- Derive `Debug` for `VersionSegments`
- Enforce PackageDescription invariants
- Add `Eq`, `Ord` and `PartialOrd` impls for `Digest`
- Add `PackageFileName` to describe valid package file names
- Add `FileTypeIdentifier` to distinguish ALPM file types
- Add `CompressionAlgorithmFileExtension` for compressed files
- Add MakepkgOption wrapper
- Add buildflags and makeflags to BuildEnvironmentOption
- Add abstractions for file names related to packages
- [**breaking**] Fully validate makepkg's BUILDENV and OPTIONS
- *(types)* Implement Deserialize for all alpm-types
- *(docs)* Add note about newlines in optdepends description
- *(parser)* Implement From for winnow error types for Error
- Add public re-export for `semver::Version` used by `SchemaVersion`

### Fixed

- OptionalDependency ToString
- Derive sort formatting
- *(architecture)* Serialize architecture as lowercase
- Make Digest type Send
- *(cargo)* Use the package's README instead of the workspace README

### Other

- Rename `alpm-local-desc` to `alpm-db-desc`
- Replace the use of `alpm-sync-desc` with `alpm-repo-desc`
- Ensure `alpm-repo-files` and `alpm-db-files` man pages are created
- Move architecture parser into own function
- *(package)* Add alpm-local-desc specification
- Add missing documentation for all public items
- Fix clippy error regarding uninlined format args
- *(justfile)* Add cargo-sort-derives
- Update the doc comment for InvalidPackageFileNameVersion error
- Rewrite PackageOption and BuildEnvironmentOption parsers
- Sort option match statements
- Move RelationOrSoname to alpm_types
- Add helper macros for parse error contexts
- Use winnow's new error context functions
- *(testing-tool)* Shared function to parse rsync output
- *(package)* Add alpm-files specification
- Remove all non-exhaustive enum markers
- Pkgver comparison algorithm
- *(types)* Extend and clean up pkgver comparison test cases
- Mention alpm-state-repo in version spec
- *(parsers)* Add winnow parser for SkippableChecksum
- *(parsers)* [**breaking**] Add winnow parser for Checksum
- *(types)* Properly type PackageRelease version data
- Improve parser code
- *(parser)* Add OptionalDependency winnow parser
- *(parser)* Add winnow parser for PackageRelation
- *(parser)* Add winnow parser for VersionRequirement
- *(parser)* Replace EnumString parser with winnow for VersionComparison
- *(parser)* Add Version composite winnow parser
- *(parser)* Add winnow parsers for PackageVersion, Epoch, PackageRelease
- *(parser)* Swap from regex-based parser to winnow for Name
- *(cargo)* Consolidate and sort package section

## [0.5.0] - 2025-02-28

### Added

- Derive `serde::Serialize` for types related to `SonameV1`
- *(types)* Implement SonameV2 type
- *(types)* Implement SonameV1 type
- *(types)* Use SourceUrl in Source type
- *(types)* SourceUrl type
- *(types)* Add winnow dependency
- *(types)* Add SkippableChecksum type
- *(alpm-types)* Use internal tagging for structural enums
- *(types)* Rename BuildEnv -> BuildEnvironmentOption
- *(types)* Support versioned optional dependencies
- *(types)* Make all alpm-types serializable
- *(types)* Implement OpenPGPIdentifier type
- *(types)* Implement OpenPGPKeyId type
- *(types)* Add i386 architecture
- *(types)* Re-export Md5 digest and Digest trait
- *(types)* Implement Changelog type
- *(mtree)* Format subcommand and json serialization
- *(types)* Use Display impl for Checksum Debug
- *(types)* Implement ExtraData type
- *(types)* Implement Install type
- *(types)* Implement Backup and RelativePath types
- *(types)* Implement Group type
- *(types)* Implement OpenPGPv4Fingerprint type
- *(types)* Implement Url type
- *(types)* Implement PkgBase type
- *(types)* Implement License type
- *(alpm)* implement OptDepend type
- *(types)* Implement PkgDesc type
- Add `PackageRelation` to track alpm-package-relations
- Implement `Display` for `VersionRequirement`
- Sort `VersionComparison` variants based on length
- Simplify `VersionComparison` by relying on strum
- *(types)* Add regex_type to RegexDoesNotMatch Error
- *(types)* Add value to RegexDoesNotMatch Error
- *(alpm-types)* UTF-8 capable version segment iterator
- *(parsers)* implement the custom INI parser

### Fixed

- [**breaking**] Adjust version handling for `VersionOrSoname` and `SonameV1`
- *(types)* Use untagged enum for serialization
- *(types)* Allow uppercase characters for package name
- *(types)* Allow `0` as value for `Pkgrel`
- *(alpm-types)* Make BuildTool version architecture optional
- *(test)* allow underscore in build option
- Adapt documentation links to workspace locations
- Use automatic instead of bare links to documentation
- Properly export macro to ensure visibility
- Insert empty line after list in documentation

### Other

- *(url)* Simplify the FromStr implementation of SourceUrl
- Consolidate keywords in the the `SEE ALSO` section
- *(package-relation)* Update specification to include soname dependencies
- *(types)* Add alpm-sonamev2 specification
- *(types)* Add alpm-sonamev1 specification
- Switch to rustfmt style edition 2024
- *(cargo)* Declare `rust-version` in the workspace not per crate
- Streamline wording around keyword assignments
- Add specification for ALPM package source checksum
- Add specification for ALPM package source
- Extend package relations with architecture specific examples
- Add specification for split packages
- *(format)* Merge imports
- *(types)* Rename MakePkgOption -> MakepkgOption
- *(types)* Rename Pkgver -> PackageVersion
- *(types)* Rename Pkgrel -> PackageRelease
- *(types)* Rename OptDepend -> OptionalDependency
- *(types)* Rename PkgType -> PackageType
- *(types)* Rename PkgDesc -> PackageDescription
- *(types)* Rename PkgBase -> PackageBaseName
- *(types)* Rename StartDir -> StartDirectory
- *(types)* Rename BuildDir -> BuildDirectory
- *(package-relation)* Update specification about versioned optional dependencies
- *(install)* Add specification for .INSTALL files
- *(types)* Change Name::new parameter from String to &str
- *(types)* Use consistent 'Errors' section in doc comments
- *(types)* Avoid unwrapping in code examples
- *(types)* Link to makepkg.conf manpage for MakePkgOption
- *(types)* Add missing documentation
- *(types)* Update the Checksum documentation
- *(types)* Remove incomplete type examples from README.md
- *(types)* Allow easier conversion from ParseIntError to alpm_types::Error
- *(types)* Use consistent doctests and comments for path module
- Add/ Update links to latest project documentation
- Make strum a workspace dependency
- *(alpm-types)* Document Version::from_str()
- *(types)* Move Packager type to openpgp module
- *(deps)* Move testresult to workspace dependencies
- *(name)* implement `AsRef<str>` for Name
- Add specification for package groups
- Add specification for meta packages
- *(types)* Use lowercase serialization for Architecture
- *(types)* Add type aliases for checksum digests
- *(types)* Provide a better API for creating package sources
- *(types)* Derive `Copy` and `Eq` where possible
- *(types)* Use consistent FromStr parameter
- *(types)* Use consistent constructors
- *(types)* Rename BuildToolVer to BuildToolVersion
- *(types)* Add type aliases for AbsolutePath
- *(types)* Add type aliases for MakePkgOption
- *(types)* Add type aliases for i64
- *(version)* Split buildtoolver tests
- *(version)* Split version tests
- *(version)* Split pkgrel tests
- *(version)* Split pkgver tests
- *(version)* Split version requirement tests
- *(version)* Split version comparison tests
- *(error)* remove MD5 checksum type
- *(error)* use more generalized error types
- *(error)* document the error variants
- *(error)* add newline between variants
- *(specification)* Add specification for package versions
- *(specification)* Add specification for package names
- *(specification)* Add specification for package relations
- *(specification)* Add specification for architecture definitions
- *(specification)* Add specification for comparison operators
- *(pkgversion)* Simplify tests
- *(alpm-types)* Pkgver comparison
- *(alpm-types)* Merge Imports
- Use strum macros via feature
- *(alpmy-types)* Document the version cmp
- *(workspace)* update deployed documentation links
- *(workspace)* use shared workspace metadata
- Add specification for `epoch` as `alpm-epoch`
- Add specification for `pkgrel` as `alpm-pkgrel`
- Add specification for `pkgver` as `alpm-pkgver`
- *(workspace)* move more dependencies to workspace
- Bump MSRV to 1.70.0
- Unify and bump workspace dependencies
- Do not run doc test for private method
- Add cargo-machete metadata for md-5 crate
- Apply rustfmt configuration to codebase
- Adapt alpm-types cargo configuration to workspace
- *(license)* Relicense project as Apache-2.0 OR MIT
- Parse version number with differing components

- - -

## 0.4.0 - 2023-11-17

### Bug Fixes

- **(Cargo.toml)** Update MSRV to 1.67.1 - (66d3e47) - David Runge
- **(deps)** update rust crate regex to 1.10.2 - (bf3423b) - renovate
- **(deps)** update rust crate strum_macros to 0.25.2 - (47f9071) - renovate
- **(deps)** update rust crate strum to 0.25.0 - (e988113) - renovate
- Increase the MSRV to 1.65.0 as let..else is in use - (21bd1ca) - David Runge
- make version types' fields public - (3fe4b5d) - Xiretza
- make *Size field public - (302362c) - Xiretza
- Epoch: take &str instead of String - (df875ae) - Xiretza
- do not allow arbitrary first characters in version strings - (ce1e923) - Xiretza
- simplify BuildOption parsing - (e07b675) - Xiretza
- derive PartialEq implementation for Version - (0cc94e8) - Xiretza
- simplify Version parsing - (959a694) - Xiretza
- avoid unnecessary string allocations in Version Display - (6813580) - Xiretza
- Relicense README under the terms of GFDL-1.3-or-later. - (58494dc) - David Runge

### Continuous Integration

- Verify that the advertised MSRV can be used. - (cd08b09) - David Runge
- Add renovate.json - (9adf80a) - renovate
- Actually publish the documentation. - (483a19d) - David Runge
- Publish development documentation on archlinux.page - (220c487) - David Runge
- Do not run semver check if commits lead to no change in version - (980cafa) - David Runge
- Do not store artifacts for build job - (0b7e894) - David Runge
- Split checks into separate jobs and do not rely on script anymore. - (d888106) - David Runge
- Use default before_script instead of extending from .prepare job - (b51bbf6) - David Runge
- Only run `cargo semver-checks` if there are commits requiring a new version - (ae15fc0) - David Runge

### Documentation

- Add information on where to find documentation. - (78d6271) - David Runge
- Clarify licensing of documentation contribution. - (ffdb0f0) - David Runge
- Add GFDL-1.3-or-later license - (b74f1fd) - David Runge
- Add links to mailing list and IRC channel to contributing guidelines - (7ba5841) - David Runge
- Add security policy - (3cf22d2) - David Runge

### Features

- add #![forbid(unsafe_code)] - (7451249) - Xiretza
- add more BuildOption tests - (08c22a5) - Xiretza

### Miscellaneous Chores

- **(deps)** update rust crate proptest to 1.4.0 - (0ac0208) - renovate
- **(deps)** update rust crate rstest to 0.18.1 - (61e083f) - renovate
- Upgrade dependencies - (9b3c2b2) - David Runge

### Refactoring

- Replace chrono with time - (e3b8922) - Óscar García Amor

- - -

## 0.3.0 - 2023-06-11

### Continuous Integration

- Enable releasing to crates.io via CI - (e74334a) - David Runge

### Documentation

- Add example for Filename, Source and SourceLocation to README - (e3df355) - David Runge
- Add example for VersionComparison and VersionRequirement to README - (b9ef3c5) - David Runge
- No longer manually break long lines in README and contributing guidelines - (af3fea2) - David Runge

### Features

- Derive Clone for BuildTool - (32d9315) - David Runge
- Derive Clone for PkgType - (83bbed5) - David Runge
- Derive Clone for Installed - (8968d7b) - David Runge
- Derive Clone for SchemaVersion - (679f03d) - David Runge
- Derive Clone for BuildToolVer - (05a510f) - David Runge
- Derive Clone for Architecture - (75a50c0) - David Runge
- Add from strum::ParseError for Error - (0b682e1) - David Runge
- Add default Error variant for generic issues. - (e6f6a64) - David Runge
- add Source type - (8853d34) - Xiretza
- add VersionComparison and VersionRequirement types - (1f493ae) - Xiretza
- make Version Clone - (67b5fcc) - Xiretza
- Add Checksum type to generically support checksum algorithms - (f1a6b57) - David Runge

### Miscellaneous Chores

- Deprecate Md5Sum in favor of `Checksum<Md5>` - (50f6f74) - David Runge

### Tests

- Guard against breaking semver using cargo-semver-checks - (757ac72) - David Runge

- - -

## 0.2.0 - 2023-06-01

### Bug Fixes

- **(SchemaVersion)** Use semver:Version as SemverVersion to prevent name clash - (1725d10) - David Runge
- Sort Error variants alphabetically - (19ba3ed) - David Runge
- Use String for initialization where possible - (b693cfc) - David Runge
- Remove implementations of Deref - (1011148) - David Runge
- Apply NewType pattern for all types wrapping one other type - (883526f) - David Runge

### Documentation

- **(BuildDir)** Add example in README. - (a0eee64) - David Runge
- Fix all code examples in README. - (1b87592) - David Runge
- Split examples into sections based on modules - (f4e929a) - David Runge
- Add documentation for Error::InvalidVersion and fix for SchemaVersion - (ad7eaac) - David Runge
- Reference 'deny' at the CONTRIBUTING.md - (15c7352) - Leonidas Spyropoulos

### Features

- **(Version)** Add method to create Version with Pkgrel - (25b1001) - David Runge
- Add StartDir type - (c2e02b9) - David Runge
- Add Installed type - (9b3c92b) - David Runge
- Implement BuildToolVer type - (6276f82) - David Runge
- Derive Architecture from Ord and PartialOrd to allow comparison. - (d9eae8d) - David Runge
- Include README.md as top-level documentation - (ab8d882) - David Runge
- Add Version type - (967cdc8) - David Runge
- Implement BuildDir type - (b50c34e) - Leonidas Spyropoulos
- Use cargo deny instead of only cargo audit in CI and tests - (c28c48f) - David Runge
- Add BuildOption, BuildEnv and PackageOption types - (a22506b) - David Runge
- Add BuildTool type to describe a buildtool name - (a67b54f) - David Runge
- Use Newtype pattern for Name type and use Ord and PartialOrd macros - (66e744a) - David Runge
- Add Packager type - (be30773) - David Runge
- Add SchemaVersion type - (10fc69a) - David Runge

### Miscellaneous Chores

- **(lib)** Sort imports by std/external/alphabetically. - (55dfadf) - David Runge

### Refactoring

- Move environmen related types to separate module - (5442732) - David Runge
- Move package related types to separate module - (860ecb6) - David Runge
- Move system related types to separate module - (28b3662) - David Runge
- Move checksum related types to separate module - (1eec013) - David Runge
- Move date related types to separate module - (a15dafb) - David Runge
- Move size related types to separate module - (e194bc1) - David Runge
- Move name related types to separate module - (9314901) - David Runge
- Move path related types to separate module - (b14ba8b) - David Runge
- Move version related types to separate module - (078c77b) - David Runge

- - -

## 0.1.0 - 2023-04-04

### Continuous Integration

- Add check scripts and Gitlab CI integration - (a301b04) - David Runge

### Documentation

- correct path for quick-check.sh - (06c36ee) - Leonidas Spyropoulos

### Features

- Limit chrono features to avoid audit RUSTSEC-2020-0071 - (a32127f) - Leonidas Spyropoulos
- Implement Md5sum type - (6ab68a8) - Leonidas Spyropoulos
- Increase MSRV to 1.60.0 - (150c878) - David Runge
- Implement Name type - (335d13c) - David Runge
- Implement PkgType - (540746d) - David Runge
- Use rstest to parametrize tests - (44b7644) - David Runge
- Use thiserror to remove Error boilerplate - (14620dd) - David Runge
- Replace enum boilerplate with strum - (d6fc661) - David Runge
- Add initial types (Architecture, BuildDate, CompressedSize, InstalledSize) - (2deba0f) - David Runge

### Miscellaneous Chores

- Publish to crates.io locally (not from CI) - (a0e6b54) - David Runge
- Change CI scripts to LGPL-3.0-or-later - (8995c51) - David Runge

- - -

