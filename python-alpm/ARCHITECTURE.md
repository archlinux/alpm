# Architecture Guide

## Technology Stack

- [PyO3](https://pyo3.rs/): The core technology enabling Rust/Python interop.
- [Maturin](https://www.maturin.rs/): Build tool for creating Python packages from Rust code.
- [uv](https://docs.astral.sh/uv/): Python package installer and project manager.
- [ruff](https://docs.astral.sh/ruff/): Python linter and formatter.
- [mypy](https://mypy.readthedocs.io/): Static type checker for Python.
- [pytest](https://docs.pytest.org/): Testing framework.
- [pdoc](https://pdoc.dev/): API documentation generator.
- [Zig](https://ziglang.org/): Used as a C linker for [manylinux](https://github.com/pypa/manylinux) wheel building without Docker.

For detailed information on how PyO3 maps Rust types to Python, 
see the [PyO3 documentation on type conversions](https://pyo3.rs/latest/conversions.html).

## Design Philosophy

- `python-alpm` is a single Python library that provides bindings for multiple ALPM Rust crates, rather than separate Python packages per crate.
- Expose Rust bindings as `alpm._native` with a thin Python layer on top.
- Use Python for type stubs, docstrings, and type aliases.
- Mirror Rust crate structure but diverge when it makes things more Pythonic.
- Currently supports `alpm-types` and `alpm-srcinfo`, with plans to extend to other ALPM crates as needed.

## Writing Bindings

These are the patterns we've established for mapping Rust types to Python.
Follow these conventions when adding bindings for new crates.

### Structs

Structs can usually be mapped using newtypes.
Since we convert between the newtype and inner type frequently, we always implement two-way `From` using the `impl_from!` macro.

### Enums

**Simple enums (no data):** Create a second enum with more Pythonic variant names and map to Rust enum variants (e.g. `ElfArchitectureFormat`).

**Enums with data:** Can be mapped several ways depending on what we're mapping:

- Single class + boolean getter (e.g. `License`)
- Class with internal simple enum + data field(s) (e.g. `SonameV1` with `SonameV1Type`) 
- Python unions with type alias and factory function (e.g. `MakepkgOption` and `makepkg_option_from_str`)

### Generics

**Simple approach:** Create separate classes for each concrete type we want to expose.
Use macros to reduce boilerplate (e.g. checksum types).

**Complex approach:** Map Rust generics to Python generics.
This is more complex because Rust generics are monomorphized at compile time while Python generics are just type hints.
We must track all possible types and restrict on the Python side using `TypeVar` with `bound=`.
Currently only implemented for `Override<T>` - conversion details are documented in the code.

### Vecs and Maps

We convert `Vec`s to Python `list`s and `BTreeMap`s to Python `dict`s.
PyO3 handles this automatically when inner types are convertible to Python types. 

Use the `vec_convert!` and `btree_convert!` macros to easily convert between newtypes and inner types of `Vec`s and `BTreeMap`s content.

### Dotted Import Paths

PyO3 doesn't automatically add dotted import paths for nested modules ([see discussion](https://github.com/PyO3/pyo3/discussions/5397)), so we add those manually.
Easy to miss - add all new possible import paths to tests.

### Type Aliases

We expose type aliases for common unions in the `alpm.type_aliases` module.
This avoids boilerplate that would be needed to reexport native modules with added type aliases.
