# Contributing

For general best practices refer to the [contributing guidelines](../CONTRIBUTING.md) of the ALPM project.

## Tech stack

The [zola] static site generator is used in combination with the [linkita] theme template.

## Development

- `just serve` to locally serve the template for development.
- `just build-tailwind` This **must** be called every time you update any tailwind styling. Otherwise the CSS won't work as expected.
   Be aware that `just serve` doesn't rebuild tailwind automatically. \
   `watchexec --exts html --exts scss 'just build-tailwind'` to automatically compile tailwind on HTML changes.
- `git submodule update --remote themes/linkita` Update the linkita theme. After doing so, make sure that this didn't break anything!

### How to test the ALPM-book integration

To test the integration in the alpm mdbook you need to do the following:

- Change the `base_url` in `./config.toml` to `http://127.0.0.1:8080/lints/`
- Run `just serve-book` in the root project.
- Visit <http://127.0.0.1:8080/lints/index.html>

## How to build

1. `git submodule init` to setup the linkita theme.
1. `just update-lint-definition` to generate the newest lint rule definitions from [alpm-lint]. They're placed into `static/rules.json`.
1. `just build-tailwind` to compile the tailwind CSS.
1. `just build` to build the site.

## Tooling

[biome] is used to lint and format most project files.

[biome]: https://biomejs.dev/
[alpm-lint]: ../alpm-lint/README.md
[linkita]: https://salif.github.io/linkita
[zola]: https://github.com/getzola/zola
