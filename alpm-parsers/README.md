# alpm-parsers

A library for providing various custom parsers/deserializers for the specifications used in Arch Linux Package Management (ALPM).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_parsers/> for development version of the crate
- <https://docs.rs/alpm-parsers> for released versions of the crate

## Examples

### Custom INI parser

```rust
use serde::Deserialize;

const DATA: &str = "
num = 42
text = foo
list = bar
list = baz
list = qux
";

#[derive(Debug, Deserialize)]
struct Data {
    num: u64,
    text: String,
    list: Vec<String>,
}

fn main() {
    let data: Data = alpm_parsers::custom_ini::from_str(DATA).unwrap();
}
```

The main difference between the regular INI parser and this one is that it allows duplicate keys in a section and collects them into a `Vec`.

Furthermore, the delimiter must be a `=`, which is much more rigid than classic `ini`, as that allows to not use surrounding whitespaces or even other characters as delimiters.

**Note:** Serde's [`flatten`](https://serde.rs/attr-flatten.html) attribute is currently not supported. See [this issue](https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/78) for more details.

## Features

- `_winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
