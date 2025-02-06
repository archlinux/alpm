# Architecture Guide

The SourceInfo parser consists of two to three different steps to get to a well-formed and well-structured representation.

```mermaid
graph TD;
    A[SRCINFO File] -->|Parse| B
    B[SourceInfoContent] -->|Normalize & Validate| SourceInfo
    subgraph SourceInfo
        C1[PackageBase]
        C2[Vec#lt;Package#gt;]
    end
    SourceInfo -->|Merge for Architecture| D[MergedPackage]
```

## Parsing

The first step is low-level parsing of a given `SRCINFO` file. This logic happens in the [`parser`] module.

The entry point for this is the [`SourceInfoContent::parser`], which receives the reference to the `SRCINFO` file's content.
This parser returns a line-based raw, but already typed representation of the `SRCINFO` file.

This representation ensures that only valid keywords are found in the respective sections of the `SRCINFO` file and already transforms them into the respective [`alpm_types`] equivalents.

However, this step does not yet validates the file and does no sanity checking.

## SourceInfo

The next step is the conversion of the raw [`SourceInfoContent`] into a [`SourceInfo`] struct by calling [`SourceInfo::from_raw`].

This process validates the input, performs error checking and linting and converts the raw content into the well-formed [`SourceInfo`] struct.
The [`SourceInfo`] struct is accurate representation of the `SRCINFO` file, however for most use cases it's still not very ergonomic to use.

The [`SourceInfo`] struct contains:

- A [`PackageBase`] struct that contains the defaults for all packages in this `SRCINFO` file.
  - The `PackageBase::architecture_properties` field then contains additional defaults that're architecture specific.
- A list of [`Package`]s that contains package specific information as well as overrides for the [`PackageBase`] defaults.
  - The `Package::architecture_properties` field contains additional overrides that override the respective defaults of `PackageBase::architecture_properties`.

If you're interested in properties of a specific package for a specific architecture, this representation isn't easy to use.
For that, we have the [`MergedPackage`] representation.

## MergedPackage

To get a [`MergedPackage`], the [`SourceInfo::packages_for_architecture`] function is used, which creates an iterator that merges the [`Package`] specific overrides with the defaults from the [`PackageBase`] and applies any architecture specific additions on top of it.

The way the merging works is as follows:

1. Take the [`PackageBase`] non-architecture default values.
2. Apply the [`Package`] non-architecture override values.
3. Take the [`PackageBase`] architecture-specific default values.
4. Apply the [`Package`] architecture-specific override values.
5. Merge the final architecture-specific values into the non-architecture specific values.

[`MergedPackage`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/merged/struct.MergedPackage.html
[`PackageBase`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/source_info/package_base/struct.PackageBase.html
[`Package`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/source_info/package/struct.Package.html
[`SourceInfo::from_raw`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/source_info/struct.SourceInfo.html#method.from_raw
[`SourceInfo::packages_for_architecture`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/source_info/struct.SourceInfo.html#method.packages_for_architecture
[`SourceInfoContent::parser`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/parser/struct.SourceInfoContent.html#method.parser
[`SourceInfoContent`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/parser/struct.SourceInfoContent.html
[`SourceInfo`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/source_info/struct.SourceInfo.html
[`alpm-types`]: https://docs.rs/alpm-types/latest/alpm_types/index.html
[`parser`]: https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/parser/index.html
