# NAME

package relation - package relationships for ALPM based packages.

# DESCRIPTION

**Package relations** describe relationships between ALPM based packages for various scenarios.
They are used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **PKGINFO** or **SRCINFO**) to describe relationships of packages to other packages.
Software such as package managers or package build software rely on **package relations** to resolve dependency graphs, e.g. when installing or uninstalling packages.

## Packages and virtual components

Every **package relation** contains an **alpm-package-name**, which may be used to refer to an existing package or a *virtual component*.
*Virtual components* do not represent the names of existing packages, but instead a component that is implicitly defined by package metadata.
With the help of **package relations**, *virtual components* are defined and used similarly to names of existing packages (see **EXAMPLES** for further information).

## Types of package relations

The definition of a **package relation** is bound to a set of types.
The keyword definition for each type depends on the context it is used in.

### Run-time dependency

A run-time dependency of a package.
This **package relation** specifies a hard requirement (another package, optionally in a specific version), that must be present when using a given package.

The value for a run-time dependency is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more run-time dependencies of a package are specified using the **depends** array.
- In **PKGINFO** files the **depend** keyword is used to specify a run-time dependency.
- In **SRCINFO** files the **depends** keyword is used to specify a run-time dependency.

### Build dependency

A build-time dependency of a package.
This **package relation** specifies a build requirement (another package, optionally in a specific version), that must be present when building a given package.

The value for a build dependency is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more build-time dependencies of a package are specified using the **makedepends** array.
- In **PKGINFO** files the **makedepend** keyword is used to specify a build-time dependency.
- In **SRCINFO** files the **makedepends** keyword is used to specify a build-time dependency.

### Test dependency

A package dependency, that is only required when running the tests of the package.
This **package relation** specifies a test requirement (another package, optionally in a specific version), that must be present when running the tests of a given package.

The value for a test dependency is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more test dependencies of a package are specified using the **checkdepends** array.
- In **PKGINFO** files the **checkdepend** keyword is used to specify a test dependency.
- In **SRCINFO** files the **checkdepends** keyword is used to specify a test dependency.

### Optional dependency

A package dependency, that provides optional functionality for a package but is otherwise not required during run-time.
This **package relation** specifies a requirement (another package and an optional description), that is only needed for optional functionality of a given package.

The value for an optional dependency is either an **alpm-package-name** or an **alpm-package-name** directly followed by a ':' sign, a whitespace and a UTF-8-formatted description string that specifies a reason for the optional dependency for the given package (e.g. `example` or `example: for feature X`).

- In **PKGBUILD** files zero or more optional dependencies of a package are specified using the **optdepends** array.
- In **PKGINFO** files the **optdepend** keyword is used to specify an optional dependency.
- In **SRCINFO** files the **optdepends** keyword is used to specify an optional dependency.

### Provision

This **package relation** specifies a component name (an **alpm-package-name** or a *virtual component*), that is provided by a given package.
The use of a provision allows for scenarios in which e.g. several packages provide the same component, allowing package managers to provide a choice.

The value for a **provision** is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more provisions are specified using the **provides** array.
- In **PKGINFO** files the **provides** keyword is used to specify a provision.
- In **SRCINFO** files the **provides** keyword is used to specify a provision.

### Conflict

This **package relation** specifies a component name (which may also be a package name), that a given package conflicts with.
A conflict is usually used to ensure that package managers are not able to install two packages, that provide the same files.

The value for a conflict is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more conflicts are specified using the **conflicts** array.
- In **PKGINFO** files the **conflict** keyword is used to specify a conflict.
- In **SRCINFO** files the **conflicts** keyword is used to specify a conflict.

### Replacement

A **package relation** that specifies which other component or package a given package replaces upon installation.
The feature is used e.g. by package managers to replace existing packages or virtual components if they are e.g. renamed or superseded by another project offering the same functionality.

The value for a replacement is either an **alpm-package-name** or an **alpm-comparison** (e.g. `example` or `example>=1.0.0`).

- In **PKGBUILD** files zero or more replacements are specified using the **replaces** array.
- In **PKGINFO** files the **replaces** keyword is used to specify a conflict.
- In **SRCINFO** files the **replaces** keyword is used to specify a conflict.

# EXAMPLES

## Provisions as virtual components

Mail servers working with the SMTP protocol can usually be used in several scenarios (e.g. as SMTP forwarder or server).
It is commonplace to have packages that only require one of these scenarios.
Given the mail server package `my-mailserver`, which represents a full mail server solution, it is therefore useful to define **provisions** for it (e.g. introducing the *virtual components* `smtp-forwarder` and `smtp-server`).
Another mail server package - `minimal-mailserver` - can only be used as SMTP forwarder, so defining only one **provision** (i.e. introducing the *virtual component* `smtp-forwarder`) is possible.

Other packages may now depend on these *virtual components*, instead of one specific mail server:
Given the monitoring package `my-monitoring`, which allows sending out monitoring mails using a local SMTP forwarder, a **run-time dependency** can be defined for it to depend on the *virtual component* `smtp-forwarder`.

This scenario enables a package manager to provide the user with the choice to rely on one of the providers of `smtp-forwarder` (i.e. `my-mailserver` or `minimal-mailserver`).

# SEE ALSO

BUILDINFO(5), PKGBUILD(5), PKGINFO(5), alpm-comparison(7), alpm-epoch(7), alpm-pkgrel(7), alpm-pkgver(7), vercmp(8)
