# alpm-parsers

A library for providing various custom parsers/deserializers for the specifications used in Arch Linux Package Management (ALPM).

## Documentation

- <https://alpm.archlinux.page/alpm_parsers/> for development version of the crate
- <https://docs.rs/alpm-parsers> for released versions of the crate

## Examples

### Custom INI parser

```rust
use serde::Deserialize;

const DATA: &str = "
    num=42
    text=foo
    list=bar
    list=baz
    list=qux
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

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt