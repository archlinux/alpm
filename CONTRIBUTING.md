# Contributing

These are the contributing guidelines for the alpm project.

Development takes place at https://gitlab.archlinux.org/archlinux/alpm/alpm.

## Writing code

This project is written in [Rust] and formatted using the nightly [`rustfmt`] version.

All contributions are linted using [`clippy`] and spell checked using [`codespell`].
The dependencies are linted with [`cargo-deny`].
License identifiers and copyright statements are checked using [`reuse`].

Various [`just`] targets are used to run checks and tests.

To aide in development, it is encouraged to install the relevant [git pre-commit] and [git pre-push] hooks:

```shell
just add-hooks
```

## Writing specifications

Specifications for technology of this project are written in markdown documents in the context of a [component], that serves as its reference implementation.
The specifications are located in the [component]'s `resources/specification/` directory and must end on `.5.md` or `.7.md`, so that they can be used as [section 5] or [section 7] man pages, respectively.

### Specification versioning

A new specification version must be created, if fields of an existing specification are altered (e.g. a field is removed, added or otherwise changed semantically).

By default, given an example specification named `topic` and given only one version of `topic` exists, there would only be a document named `topic.7.md`.
If the need for version two of `topic` arises, the document is renamed to `topicv1.7.md`, a new file named `topicv2.7.md` is used for the new version and a symlink from the generic specification name to the most recent version (here `topic.7.md -> topicv2.7.md`) is created.
Versioned specifications additionally must clearly state the specification version number they are addressing in the `NAME` and `DESCRIPTION` section of the document.

New (versions of) specifications must be accompanied by examples and code testing those examples.

The examples and code testing those examples must be kept around for legacy and deprecated specifications to guarantee backwards compatibility.

## Writing commit messages

To ensure compatibility and automatic creation of [semantic versioning] compatible releases the commit message style follows [conventional commits].

## Merging changes

Changes to the project are proposed and reviewed using merge requests and merged by the developers of this project using [fast-forward merges].

## Creating releases

Releases are created by the developers of this project using [`release-plz`] by running (per package in the workspace):

```shell
just prepare-release <package>
```

Changed files are added in a pull request towards the default branch.

Once the changes are merged to the default branch a tag is created and pushed for the respective package:

```shell
just release <package>
```

The crate is afterwards automatically published on https://crates.io using a pipeline job.

## License

All code contributions fall under the terms of the [Apache-2.0] and [MIT].

Configuration file contributions fall under the terms of the [CC0-1.0].

Documentation contributions fall under the terms of the [CC-BY-SA-4.0].

Specific license assignments and attribution are handled using [`REUSE.toml`].
Individual contributors are all summarized as *"ALPM Contributors"*.
For a full list of individual contributors, refer to `git log --format="%an <%aE>" | sort -u`.

[Rust]: https://www.rust-lang.org/
[`rustfmt`]: https://github.com/rust-lang/rustfmt
[`clippy`]: https://github.com/rust-lang/rust-clippy
[`codespell`]: https://github.com/codespell-project/codespell
[`cargo-deny`]: https://github.com/EmbarkStudios/cargo-deny
[`reuse`]: https://git.fsfe.org/reuse/tool
[`just`]: https://github.com/casey/just
[git pre-commit]: https://man.archlinux.org/man/githooks.5#pre-commit
[git pre-push]: https://man.archlinux.org/man/githooks.5#pre-push
[semantic versioning]: https://semver.org/
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0/
[`release-plz`]: https://github.com/MarcoIeni/release-plz
[Apache-2.0]: ./LICENSES/Apache-2.0.txt
[MIT]: ./LICENSES/MIT.txt
[CC0-1.0]: ./LICENSES/CC0-1.0.txt
[CC-BY-SA-4.0]: ./LICENSES/CC-BY-SA-4.0.txt
[`REUSE.toml`]: ./REUSE.toml
[component]: .README.md#components
[fast-forward merges]: https://man.archlinux.org/man/git-merge.1#FAST-FORWARD_MERGE
[section 5]: https://en.wikipedia.org/wiki/Man_page#Manual_sections
[section 7]: https://en.wikipedia.org/wiki/Man_page#Manual_sections
