# fluent-i18n 🗣️🔥

![docs.rs](https://img.shields.io/docsrs/fluent-i18n)
![Crates.io MSRV](https://img.shields.io/crates/msrv/fluent-i18n)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache-blue.svg)](https://opensource.org/license/apache-2-0)

**A declarative and ergonomic internationalization for Rust using [Fluent]**.

_Built on top of [`fluent-templates`] and inspired by the simplicity of [`rust-i18n`]_.

## Features

- Static file loading via [`i18n!()`] macro
- Zero-boilerplate [`t!()`] macro for inline translations
- Extensible argument system via [`ToFluentValue`] trait
- Thread-local handling via [`set_locale()`] and [`get_locale()`]
- Clean fallback locale management

## Example

```rust,ignore
use fluent_i18n::{i18n, t};

i18n!("locales", fallback = "en-US");

println!("{}", t!("greeting"));
println!("{}", t!("welcome", { "name" => "Orhun" }));
```

Place your localization files in `locales/en-US/main.ftl`:

```fluent
greeting = Hello, world!
welcome = Welcome, { $name }!
```

See the [Fluent syntax] for more details about FTL files.

## Usage

### Installation

Add [`fluent-i18n`] to your `Cargo.toml`:

```bash
cargo add fluent-i18n
```

### Initialization

At the entry point of your application (e.g., `main.rs` or `lib.rs`), invoke the [`i18n!()`] macro to initialize the localization system:

```rust,ignore
i18n!("locales");
```

Or with a fallback locale:

```rust,ignore
i18n!("locales", fallback = "en-US");
```

This will expose a static loader named `LOCALES` that will be used by the [`t!()`] macro for translations throughout your application.

You can also dynamically change the locale at runtime using the [`set_locale()`] function:

```rust,ignore
use fluent_i18n::{set_locale, get_locale};

set_locale(Some("tr"))?;

let current_locale = get_locale()?;
```

Running `set_locale(None)` will detect the system locale automatically.

### Lookup

To look up a translation for a given key, use the [`t!()`] macro:

```rust,ignore
t!("greeting");
```

With parameters:

```rust,ignore
t!("count-items", { "count" => 3 })
t!("key", { "arg1" => value1, "arg2" => value2 })
```

The given parameters should be one of the [supported types](#supported-types).

### Supported Types

The [`t!()`] macro interpolates values into the message using the [`ToFluentValue`] trait.

The following types implement the [`ToFluentValue`] trait:

- `String`, `&'static str`, `Cow<'static, str>`
- Integer and float primitives (`usize`, `u32`, `i64`, etc.)
- `Path`, `PathBuf`
- `Option<T>` where `T` implements [`ToFluentValue`]

You can extend support for your own types by implementing this trait:

```rust,ignore
use fluent_i18n::{FluentValue, ToFluentValue};

impl ToFluentValue for MyType {
    fn to_fluent_value(&self) -> FluentValue<'static> {
        FluentValue::from(self.to_string())
    }
}
```

### Locale Layout

You can organize `.ftl` files per locale, and shared files like `core.ftl` will be included in all locales.

```text
locales/
├── core.ftl
├── en-US/
│   └── main.ftl
├── fr/
│   └── main.ftl
├── zh-CN/
│   └── main.ftl
└── zh-TW/
    └── main.ftl
```

The directory names should adhere to the [Unicode Language Identifier].
It also respects any `.gitignore` or `.ignore` files present.

See the [Fluent syntax] for more details about FTL files.

### Testing

In tests, you can access the translations as usual without reinitialization:

```rust,ignore
use fluent_i18n::t;

#[test]
fn test_translation() {
    assert_eq!(t!("greeting"), "Hello, world!");
}
```

### Debugging

When raw mode is enabled, translations will return the key itself instead of looking up the translation.
This is useful for debugging purposes to see which keys are being requested.

```rust,ignore
use fluent_i18n::{t, set_raw_mode};

set_raw_mode(true);

let raw_message = t!("some-translation-key"); // "some-translation-key"
```

You could also follow this workflow for translating missing strings:

- Run the program and navigate to the area you want to test.
- Enable raw mode with `set_raw_mode(true)`.
- Look for untranslated output (the raw key will be shown instead of the translation).
- Copy that key.
- Search for it in the project.
- Add the missing translation.

### Security

Unicode directional isolate characters (U+2068, U+2069) are disabled as default to prevent potential security issues like bidirectional text attacks. This also gives clean output without unexpected characters in translations.

Also, please note that this is only applicable for mixed-script languages such as Arabic, Hebrew, and Persian.

## License & Contributions

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

🦀 ノ( º \_ º ノ) - _respect crables!_

Feel free to open issues or PRs for improvements, bug fixes, or ideas!

## Acknowledgements

- [Project Fluent][Fluent]
- [`fluent-templates`]
- [`rust-i18n`]

This library was originally developed as part of the [ALPM project] and later extracted for general-purpose use.

[`t!()`]: https://docs.rs/fluent-i18n/latest/fluent_i18n/macro.t.html
[`i18n!()`]: https://docs.rs/fluent-i18n/latest/fluent_i18n/macro.i18n.html
[`ToFluentValue`]: https://docs.rs/fluent-i18n/latest/fluent_i18n/trait.ToFluentValue.html
[`set_locale()`]: https://docs.rs/fluent-i18n/latest/fluent_i18n/locale/fn.set_locale.html
[`get_locale()`]: https://docs.rs/fluent-i18n/latest/fluent_i18n/locale/fn.get_locale.html
[Fluent]: https://projectfluent.org
[Fluent syntax]: https://projectfluent.org/fluent/guide/
[`fluent-templates`]: https://docs.rs/fluent-templates
[`rust-i18n`]: https://github.com/longbridge/rust-i18n
[`fluent-i18n`]: https://crates.io/crates/fluent-i18n
[ALPM project]: https://alpm.archlinux.page
[Unicode Language Identifier]: https://docs.rs/unic-langid
[Apache-2.0]: https://opensource.org/license/apache-2-0
[MIT]: https://opensource.org/licenses/MIT
