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

## Writing commit messages

To ensure compatibility and automatic creation of [semantic versioning] compatible releases the commit message style follows [conventional commits].

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
