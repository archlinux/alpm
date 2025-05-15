# Contributing

These are the contributing guidelines for the alpm project.

Development takes place at <https://gitlab.archlinux.org/archlinux/alpm/alpm>.

## Writing code

This project is written in [Rust] and formatted using the nightly [`rustfmt`] version.
The linking is performed via [`mold`].

All contributions are linted using [`clippy`] and spell checked using [`codespell`].
The dependencies are linted with [`cargo-deny`] and unused dependencies are detected using [`cargo-machete`].
License identifiers and copyright statements are checked using [`reuse`].
Toml files are formatted via [`taplo-cli`].

Various [`just`] targets are used to run checks and tests.
[`shellcheck`] is used to check the just targets for correctness.
In order to review the snapshot changes in tests, you can use [`cargo-insta`].

Code examples in READMEs is tested via [`tangler`].
Links in markdown files or doc blocks are tested via [`lychee`].

To get all of the necessary tools installed on Arch Linux, run `just install-pacman-dev-packages`.
To setup Rust for this project run `just install-rust-dev-tools`.
Both can also be done in one fell swoop via `just dev-install`.

To aide in development, it is encouraged to configure git to follow this project's guidelines:

```shell
just configure-git
```

This `just` command takes care of a few things:

- Configure `git` to sign commits for this repository using OpenPGP.
- Install the relevant [git pre-commit] and [git pre-push] hooks.
- Install the [git prepare-commit-msg] hook to automatically add a signoff section to the commit message.

## Testing

We run [`nextest`] for fast execution of unit and integration tests.
`just test` calls `cargo nextest` as well as `just test-docs`, which runs the doc tests as `nextest` isn't capable of doing that [yet](https://github.com/nextest-rs/nextest/issues/16).

The `just test-coverage` command creates a cobertura code coverage report and an HTML report.
Both are written to `target/llvm-cov/`, which utilizes the [`llvm-tools-preview`] component.
The `just test-coverage doc` additionally includes doc tests into the coverage report.
However, this is still an [unstable nightly-only feature](https://github.com/rust-lang/rust/issues/85658).

The `just containerized-integration-tests` recipe executes all tests that are made available by a `_containerized-integration-test` feature and are located in an integration test module named `containerized`.
With the help of a [custom target runner], these tests are executed in a containerized environment using [`podman`].

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

The commit messages are checked by `just run-pre-push-hook` via the following tools: [`cocogitto`] & [`committed`].

Follow these rules when writing commit messages:

1. The subject line should be capitalized and should not end with a period.

2. The total length of the line should not exceed **72** characters.

3. The commit body should be in the imperative mood.

4. Avoid using the crate name as the commit scope. (e.g. `feat(alpm-types)`)
   The changelog entries will be generated for the associated crate accordingly using [`release-plz`] and [`git-cliff`].

Here is an example of a good commit message:

```
feat(parser): Enhance error handling in parser

Improve error handling by adding specific error codes and messages
to make debugging easier and more informative. This update enhances
parsing accuracy and provides more context for handling parsing errors.

Signed-off-by: John Doe <john@archlinux.org>
```

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
Individual contributors are all summarized as _"ALPM Contributors"_.
For a full list of individual contributors, refer to `git log --format="%an <%aE>" | sort -u`.

[Rust]: https://www.rust-lang.org/
[`mold`]: https://github.com/rui314/mold
[`rustfmt`]: https://github.com/rust-lang/rustfmt
[`clippy`]: https://github.com/rust-lang/rust-clippy
[`codespell`]: https://github.com/codespell-project/codespell
[`cargo-deny`]: https://github.com/EmbarkStudios/cargo-deny
[`cargo-insta`]: https://github.com/mitsuhiko/insta
[`git-cliff`]: https://git-cliff.org
[`shellcheck`]: https://www.shellcheck.net/
[`cocogitto`]: https://docs.cocogitto.io/
[`committed`]: https://github.com/crate-ci/committed
[`release-plz`]: https://release-plz.ieni.dev
[`reuse`]: https://git.fsfe.org/reuse/tool
[`lychee`]: https://github.com/lycheeverse/lychee
[`nextest`]: https://nexte.st/
[`just`]: https://github.com/casey/just
[`podman`]: https://podman.io/
[`tangler`]: https://github.com/wiktor-k/tangler
[`taplo`]: https://github.com/tamasfe/taplo
[`cargo-machete`]: https://github.com/bnjbvr/cargo-machete
[`llvm-tools-preview`]: https://github.com/rust-lang/rust/issues/85658
[custom target runner]: ./.cargo/runner.sh
[git pre-commit]: https://man.archlinux.org/man/githooks.5#pre-commit
[git pre-push]: https://man.archlinux.org/man/githooks.5#pre-push
[git prepare-commit-msg]: https://man.archlinux.org/man/githooks.5#prepare-commit-msg
[semantic versioning]: https://semver.org/
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0/
[Apache-2.0]: ./LICENSES/Apache-2.0.txt
[MIT]: ./LICENSES/MIT.txt
[CC0-1.0]: ./LICENSES/CC0-1.0.txt
[CC-BY-SA-4.0]: ./LICENSES/CC-BY-SA-4.0.txt
[`REUSE.toml`]: ./REUSE.toml
[component]: ./README.md#components
[fast-forward merges]: https://man.archlinux.org/man/git-merge.1#FAST-FORWARD_MERGE
[section 5]: https://en.wikipedia.org/wiki/Man_page#Manual_sections
[section 7]: https://en.wikipedia.org/wiki/Man_page#Manual_sections
