# Contribution Guide for `alpm-linting`

First of, it's very much encouraged to read the [architectural guide](./ARCHITECTURE.md) so that you're up-to-speed with the glossary and rough outline of the project.

If you're familiar with that document, feel free to continue!

## How to add a new lint

1. Figure out to which scope/s the new lint will be applied.
1. Create a new file in the submodules of the respective scope in `src/lint_rules`. E.g. `src/lint_rules/source_info/my_new_lint.rs`.
1. Take a look at how the other lints are structured and read the documentation for the `Lint` trait.
   The `source_info/duplicate_architecture` lint is a good starting point.
1. Once you're finished writing your lint, you must manually add the lint to the `LintStore::register_all_lints` function.
1. Write a new test for your lint.
1. Run the test suite.
