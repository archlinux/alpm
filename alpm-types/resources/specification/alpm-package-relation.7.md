# NAME

package relation - package relationships for ALPM based packages.

# DESCRIPTION

**Package relations** describe relationships between ALPM based packages for various scenarios.
They are used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **PKGINFO** or **SRCINFO**) to describe relationships of packages to other packages.
Software such as package managers or package build software rely on **package relations** to resolve dependency graphs, e.g. when installing or uninstalling packages.

## Packages and virtual components

Any **package relation** may contain an **alpm-package-name**, which may be used to refer to an existing package or a *virtual component*.
*Virtual components* do not represent the names of existing packages, but instead a component that is implicitly defined by package metadata.
With the help of **package relations**, *virtual components* are defined and used similarly to names of existing packages (see **EXAMPLES** for further information).

## Sonames

A **package relation** of type **provision** or **run-time dependency** may contain an **alpm-soname** that represents a hard dependency based on a specific _shared object_ in a package.
They are used to encode compatibility guarantees between **ELF**[1] and _shared object_ files with the help of **soname**[2] data (see **EXAMPLES** for further information).

## Architecture specific use

**Package relations** may be used in contexts that can describe multiple **alpm-architectures** at the same time.
For these situations, architecture-specific identifiers are available and may be used.

## Types of package relations

The definition of a **package relation** is bound to a set of types.
Which keywords and what data structures are used for each type depends on the context they are used in.

### Run-time dependency

A run-time dependency of a package.
This **package relation** specifies a hard requirement (another package, optionally in a specific version), that must be present when using a given package.

The value for a run-time dependency is either an **alpm-package-name**, an **alpm-comparison** or an **alpm-soname** (e.g. `example`, `example>=1.0.0`, or `lib:libexample.so.1`, respectively).

- In **PKGBUILD** files zero or more run-time dependencies of a package are specified using the **depends** array.
  Architecture-specific run-time dependencies may be specified using an array named 'depends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `depends_aarch64`.
- In **PKGINFO** files the **depend** keyword is used to specify a run-time dependency.
- In **SRCINFO** files the **depends** keyword is used to specify a run-time dependency.
  An architecture-specific run-time dependency may be specified using a keyword consisting of 'depends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `depends_aarch64`.

### Build dependency

A build-time dependency of a package.
This **package relation** specifies a build requirement (another package, optionally in a specific version), that must be present when building a given package.

The value for a build dependency is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more build-time dependencies of a package are specified using the **makedepends** array.
  Architecture-specific build dependencies may be specified using an array named 'makedepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `makedepends_aarch64`.
- In **PKGINFO** files the **makedepend** keyword is used to specify a build-time dependency.
- In **SRCINFO** files the **makedepends** keyword is used to specify a build-time dependency.
  An architecture-specific build dependency may be specified using a keyword consisting of 'makedepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `makedepends_aarch64`.

### Test dependency

A package dependency, that is only required when running the tests of the package.
This **package relation** specifies a test requirement (another package, optionally in a specific version), that must be present when running the tests of a given package.

The value for a test dependency is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more test dependencies of a package are specified using the **checkdepends** array.
  Architecture-specific test dependencies may be specified using an array named 'makedepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `checkdepends_aarch64`.
- In **PKGINFO** files the **checkdepend** keyword is used to specify a test dependency.
- In **SRCINFO** files the **checkdepends** keyword is used to specify a test dependency.
  An architecture-specific test dependency may be specified using a keyword consisting of 'checkdepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `checkdepends_aarch64`.

### Optional dependency

A package dependency, that provides optional functionality for a package but is otherwise not required during run-time.
This **package relation** specifies a requirement (another package with optional version constraints and an optional description) that is only needed to enable optional functionality of a given package.

The value for an optional dependency can be one of the following:

1. An **alpm-package-name** (e.g., `example`) or an **alpm-comparison** (e.g., `example>=1.2.3`)
1. An **alpm-package-name** or **alpm-comparison** followed by a `:` sign, a whitespace, and a UTF-8-formatted description string that specifies the reason for the optional dependency (e.g., `example: for feature X` or `example>=1.2.3: for feature X`).
   Note that newline (`\n`) and carriage return (`\r`) control characters are not allowed in the description string.

- In **PKGBUILD** files zero or more optional dependencies of a package are specified using the **optdepends** array.
  Architecture-specific optional dependencies may be specified using an array named 'optdepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `optdepends_aarch64`.
- In **PKGINFO** files the **optdepend** keyword is used to specify an optional dependency.
- In **SRCINFO** files the **optdepends** keyword is used to specify an optional dependency.
  An architecture-specific optional dependency may be specified using a keyword consisting of 'optdepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `optdepends_aarch64`.

### Provision

This **package relation** specifies a component name (an **alpm-package-name**, a *virtual component*, or an **alpm-soname**) that is provided by a given package.
The use of a provision allows for scenarios in which e.g. several packages provide the same component, allowing package managers to provide a choice.

The value for a **provision** is either an **alpm-package-name**, a _composite comparison expression_ (an **alpm-comparison** that must use an _equal to_ comparison), or an **alpm-soname** (e.g. `example`, `example=1.0.0`, `libexample.so=1-64` or `lib:libexample.so.1`).

- In **PKGBUILD** files zero or more provisions are specified using the **provides** array.
  Architecture-specific provisions may be specified using an array named 'provides', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `provides_aarch64`.
- In **PKGINFO** files the **provides** keyword is used to specify a provision.
- In **SRCINFO** files the **provides** keyword is used to specify a provision.
  An architecture-specific provision may be specified using a keyword consisting of 'provides', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `provides_aarch64`.

### Conflict

This **package relation** specifies a component name (which may also be a package name), that a given package conflicts with.
A conflict is usually used to ensure that package managers are not able to install two packages, that provide the same files.

The value for a conflict is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more conflicts are specified using the **conflicts** array.
  Architecture-specific conflicts may be specified using an array named 'conflicts', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `conflicts_aarch64`.
- In **PKGINFO** files the **conflict** keyword is used to specify a conflict.
- In **SRCINFO** files the **conflicts** keyword is used to specify a conflict.
  An architecture-specific conflict may be specified using a keyword consisting of 'conflicts', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `conflicts_aarch64`.

### Replacement

A **package relation** that specifies which other component or package a given package replaces upon installation.
The feature is used e.g. by package managers to replace existing packages or virtual components if they are e.g. renamed or superseded by another project offering the same functionality.

The value for a replacement is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more replacements are specified using the **replaces** array.
  Architecture-specific replacements may be specified using an array named 'replaces', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `replaces_aarch64`.
- In **PKGINFO** files the **replaces** keyword is used to specify a conflict.
- In **SRCINFO** files the **replaces** keyword is used to specify a conflict.
  An architecture-specific replacement may be specified using a keyword consisting of 'replaces', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `replaces_aarch64`.

# EXAMPLES

## Provisions as virtual components

Mail servers working with the SMTP protocol can usually be used in several scenarios (e.g. as SMTP forwarder or server).
It is commonplace to have packages that only require one of these scenarios.
Given the mail server package `my-mailserver`, which represents a full mail server solution, it is therefore useful to define **provisions** for it (e.g. introducing the *virtual components* `smtp-forwarder` and `smtp-server`).
Another mail server package - `minimal-mailserver` - can only be used as SMTP forwarder, so defining only one **provision** (i.e. introducing the *virtual component* `smtp-forwarder`) is possible.

Other packages may now depend on these *virtual components*, instead of one specific mail server:
Given the monitoring package `my-monitoring`, which allows sending out monitoring mails using a local SMTP forwarder, a **run-time dependency** can be defined for it to depend on the *virtual component* `smtp-forwarder`.

This scenario enables a package manager to provide the user with the choice to rely on one of the providers of `smtp-forwarder` (i.e. `my-mailserver` or `minimal-mailserver`).

## Dependencies using sonames

Some packages provide _shared object_ files that **ELF**[1] files in other packages may dynamically link against.

For example, the package `example` may contain the following files:

```bash
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

Here, the shared object file `/usr/lib/libexample.so.1.0.0` encodes the **soname**[2] `libexample.so.1`.

Following the **alpm-soname** specification, `lib:libexample.so.1` can be added as a **provision** to the **PKGINFO** file of the `example` package, if the library _lookup directory_ `/usr/lib` is represented by the _prefix_ `lib`.

Afterwards, another example package called `application` may contain the **ELF**[1] file `/usr/bin/application`, which dynamically links against the _shared object_ `/usr/lib/libexample.so.1` contained in the `example` package.
The **ELF**[1] file encodes this requirement by relying on the **soname**[2] `libexample.so.1`.

Following the **alpm-soname** specification, `lib:libexample.so.1` can be added as a **run-time dependency** to the **PKGINFO** file of the `application` package, if the library _lookup directory_ `/usr/lib` is represented by the _prefix_ `lib` and the _shared object_ encoding the `libexample.so.1` **soname**[2] (here `/usr/lib/libexample.so.1.0.0`) is present in the _lookup directory_.

**Note**: For legacy behavior of **alpm-soname** refer to **alpm-sonamev1**.

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-architecture**(7), **alpm-comparison**(7), **alpm-package-name**(7), **alpm-package-version**(7), **alpm-soname**(7)

# NOTES

1. **ELF**
   
   <https://en.wikipedia.org/wiki/Executable_and_Linkable_Format>
1. **soname**
   
   <https://en.wikipedia.org/wiki/Soname>
