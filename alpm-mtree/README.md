# alpm-mtree

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_mtree/> for development version of the crate
- <https://docs.rs/alpm-mtree/latest/alpm_mtree/> for released versions of the crate

## Examples

### Library

```rust
use alpm_mtree::mtree::v2::parse_mtree_v2;

let data = r#"#mtree
/set mode=644 uid=0 gid=0 type=file
./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
./some_link type=link link=some_file time=1700000000.0
./some_dir type=dir time=1700000000.0
"#.to_string();

assert!(parse_mtree_v2(data).is_ok());
```

### Commandline

Validate an `.MTREE` file.

```shell
alpm-mtree validate path/to/file
```

Parse an `.MTREE` file and output its contents as structured data.

```shell
alpm-mtree format ~/.cache/alpm/testing/packages/core/argon2-20190702-6-x86_64/.MTREE --output-format json --pretty
```

## Features

- `cli` adds the commandline handling needed for the `alpm-mtree` binary.
- `creation` adds library support for the creation of [ALPM-MTREE] files (enabled by default).
- `_winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
