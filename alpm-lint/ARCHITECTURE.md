# Architecture for alpm-linting

The following document should give you a high-level overview on how the `alpm-linting` framework is structured, its components and explain common terminology.

## Glossary

- `scope`: The scope of a specific lint rule defines in which context that rule should be run. For example, one possible scope is `source_info`, which means that this lint rule will be selected when linting a `.SRCINFO` file.
- `group`: Some lints aren't enabled by default, such as pedantic lints that are prone for false-positives, or experimental lints that haven't seen enough usage yet. These are assigned to groups, such as `pedantic` or `testing`, which can then be included by users.
- `rule`: A "lint rule" is a single linting action that is performed. The linting process consists of many linting rules being executed on a wide array of different file formats and project files. Single lint rules are always assigned to a single scope. This implies that some linting logic is duplicated for data such as the `architecture` field. `architecture` may exist in `.SRCINFO`, `.PKGBUILD` and `.BUILDINFO` files and each file format has their own lint rules.
- `rule configuration`: Lint rules may be configured to behave in a certain way. As ALPM metadata is sometimes highly duplicated among several metadata files, rule configuration options may apply to multiple lint rules

## Components

- `Lint`: Is the main trait that is implemented by every lint rule.
        It defines a shared interface to expose documentation, scope, severity level, its name and more.
- Lint rules: Every lint in has one struct in the `lint_rules` module, grouped the respective scope in a submodule with the name of the scope. E.g. `lint_rules/source_info/duplicate_architecture.rs`.
- `LintStore`: The store initializes and stores all available lints. It's the main entry point to get lints. Lints can be queried either by name, or by various characteristics such as scope, group and such.
- `Configuration`: This struct contains all available options for to fine-tune any lint rule, including the default values.
