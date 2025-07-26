# alpm-rootless

A library for the execution of commands as root without being root.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_rootless/> for development version of the crate
- <https://docs.rs/alpm-rootless/latest/alpm_rootless/> for released versions of the crate

## Examples

### Library

```rust
use alpm_rootless::{
    FakerootBackend,
    FakerootOptions,
    RootlessBackend,
    RootlesskitBackend,
    RootlesskitOptions,
};
# use alpm_rootless::detect_virt;

# fn main() -> testresult::TestResult {
// Create a fakeroot backend with default options.
let backend = FakerootBackend::new(FakerootOptions::default());

// Call `whoami` using fakeroot and return its output.
let output = backend.run(&["whoami"])?;

assert_eq!("root\n", String::from_utf8_lossy(&output.stdout));

# let virt = detect_virt()?;
# if !virt.uses_namespaces() {
// Create a rootlesskit backend with default options.
let backend = RootlesskitBackend::new(RootlesskitOptions::default());

// Call `whoami` using rootlesskit and return its output.
let output = backend.run(&["whoami"])?;

assert_eq!("root\n", String::from_utf8_lossy(&output.stdout));
# }
# Ok(())
# }
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
