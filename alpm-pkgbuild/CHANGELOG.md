# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2025-10-07

### Other

- Fix violations of MD022 and MD032
- Hide winnow_debug feature flag

## [0.1.0] - 2025-07-24

### Added

- Add source_info_v1_from_pkgbuild function
- Add pkgbuild srcinfo comparison command
- BridgeOutput to SourceInfo conversion
- PKGBUILD bridge parser
- Use bridge script to export PKGBUILD variables

### Other

- Move BridgeOutput to SourceInfo conversion to alpm-srcinfo
- Clean up, and feature gate API surface
- Move alpm-pkgbuild srcinfo-compare function to dev-tools
- BridgeOutput to SourceInfo conversion errors
- Add minimalistic PKGBUILD test
- Add empty value test
- Add run_bridge_script error cases
- Add project documentation
