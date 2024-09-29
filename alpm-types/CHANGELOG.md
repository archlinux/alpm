- - -
## 0.4.0 - 2023-11-17
#### Bug Fixes
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
#### Continuous Integration
- Verify that the advertised MSRV can be used. - (cd08b09) - David Runge
- Add renovate.json - (9adf80a) - renovate
- Actually publish the documentation. - (483a19d) - David Runge
- Publish development documentation on archlinux.page - (220c487) - David Runge
- Do not run semver check if commits lead to no change in version - (980cafa) - David Runge
- Do not store artifacts for build job - (0b7e894) - David Runge
- Split checks into separate jobs and do not rely on script anymore. - (d888106) - David Runge
- Use default before_script instead of extending from .prepare job - (b51bbf6) - David Runge
- Only run `cargo semver-checks` if there are commits requiring a new version - (ae15fc0) - David Runge
#### Documentation
- Add information on where to find documentation. - (78d6271) - David Runge
- Clarify licensing of documentation contribution. - (ffdb0f0) - David Runge
- Add GFDL-1.3-or-later license - (b74f1fd) - David Runge
- Add links to mailing list and IRC channel to contributing guidelines - (7ba5841) - David Runge
- Add security policy - (3cf22d2) - David Runge
#### Features
- add #![forbid(unsafe_code)] - (7451249) - Xiretza
- add more BuildOption tests - (08c22a5) - Xiretza
#### Miscellaneous Chores
- **(deps)** update rust crate proptest to 1.4.0 - (0ac0208) - renovate
- **(deps)** update rust crate rstest to 0.18.1 - (61e083f) - renovate
- Upgrade dependencies - (9b3c2b2) - David Runge
#### Refactoring
- Replace chrono with time - (e3b8922) - Óscar García Amor

- - -

## 0.3.0 - 2023-06-11
#### Continuous Integration
- Enable releasing to crates.io via CI - (e74334a) - David Runge
#### Documentation
- Add example for Filename, Source and SourceLocation to README - (e3df355) - David Runge
- Add example for VersionComparison and VersionRequirement to README - (b9ef3c5) - David Runge
- No longer manually break long lines in README and contributing guidelines - (af3fea2) - David Runge
#### Features
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
#### Miscellaneous Chores
- Deprecate Md5Sum in favor of Checksum<Md5> - (50f6f74) - David Runge
#### Tests
- Guard against breaking semver using cargo-semver-checks - (757ac72) - David Runge

- - -

## 0.2.0 - 2023-06-01
#### Bug Fixes
- **(SchemaVersion)** Use semver:Version as SemverVersion to prevent name clash - (1725d10) - David Runge
- Sort Error variants alphabetically - (19ba3ed) - David Runge
- Use String for initialization where possible - (b693cfc) - David Runge
- Remove implementations of Deref - (1011148) - David Runge
- Apply NewType pattern for all types wrapping one other type - (883526f) - David Runge
#### Documentation
- **(BuildDir)** Add example in README. - (a0eee64) - David Runge
- Fix all code examples in README. - (1b87592) - David Runge
- Split examples into sections based on modules - (f4e929a) - David Runge
- Add documentation for Error::InvalidVersion and fix for SchemaVersion - (ad7eaac) - David Runge
- Reference 'deny' at the CONTRIBUTING.md - (15c7352) - Leonidas Spyropoulos
#### Features
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
#### Miscellaneous Chores
- **(lib)** Sort imports by std/external/alphabetically. - (55dfadf) - David Runge
#### Refactoring
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
#### Continuous Integration
- Add check scripts and Gitlab CI integration - (a301b04) - David Runge
#### Documentation
- correct path for quick-check.sh - (06c36ee) - Leonidas Spyropoulos
#### Features
- Limit chrono features to avoid audit RUSTSEC-2020-0071 - (a32127f) - Leonidas Spyropoulos
- Implement Md5sum type - (6ab68a8) - Leonidas Spyropoulos
- Increase MSRV to 1.60.0 - (150c878) - David Runge
- Implement Name type - (335d13c) - David Runge
- Implement PkgType - (540746d) - David Runge
- Use rstest to parametrize tests - (44b7644) - David Runge
- Use thiserror to remove Error boilerplate - (14620dd) - David Runge
- Replace enum boilerplate with strum - (d6fc661) - David Runge
- Add initial types (Architecture, BuildDate, CompressedSize, InstalledSize) - (2deba0f) - David Runge
#### Miscellaneous Chores
- Publish to crates.io locally (not from CI) - (a0e6b54) - David Runge
- Change CI scripts to LGPL-3.0-or-later - (8995c51) - David Runge

- - -

