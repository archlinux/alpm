# File Hierarchy for the Verification of OS Artifacts (VOA)

VOA is mechanism for the storage and retrieval of cryptography technology-agnostic signature verifiers.

For specification draft see: <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/>.

## Purpose

The VOA hierarchy acts as structured storage for files that contain "signature verifiers" (such as [OpenPGP certificates], aka "public keys").

### Load Paths

A set of "load paths" may exist on a system, each containing different sets of verifier files.
This library provides an abstract, unified view of sets of signature verifiers in these load paths.

Earlier load paths have precedence over later entries (in some technologies).

VOA operates either in "system mode" or "user mode", depending on the UID of the running process.
The set of load paths differs between the two modes.

In system mode, the set of load paths (in order of descending precedence) is:

- `/etc/voa/`
- `/run/voa/`
- `/usr/local/share/voa/`
- `/usr/share/voa/`

In user mode, the analogous set of load paths (in order of descending precedence) is:

- `$XDG_CONFIG_HOME/voa/`
- the `./voa/` directory in each directory defined in `$XDG_CONFIG_DIRS`
- `$XDG_RUNTIME_DIR/voa/`
- `$XDG_DATA_HOME/voa/`
- the `./voa/` directory in each directory defined in `$XDG_DATA_DIRS`

### Support for different cryptographic technologies

Libraries dealing with a specific cryptographic technology can rely on this library to collect the paths of all verifier files relevant to them.

NOTE: Depending on the technology, multiple versions of the same verifier will either "shadow" one another, or get merged into one coherent view that represents the totality of available information about the verifier.

Shadowing/merging is specific to each signing technology and must be handled in the technology-specific library.
For more details see e.g. the `voa-openpgp` implementation and the VOA specification.

VOA expects that filenames are a strong identifier that signals whether two verifier files contain variants of "the same" logical verifier.
Verifiers from different load paths can be identified as related via their filenames.

For example, [OpenPGP certificates] must be stored using filenames based on their fingerprint.
When the filename doesn't match the fingerprint of an OpenPGP verifier, the file is considered invalid and ignored.

## Example VOA directory structure

In this example, three OpenPGP certificates are stored as verifiers.
They are stored in two system-mode load paths: `/etc/voa` and `/usr/local/share/voa/`:

```text
/etc/voa/arch/packages/default/openpgp/0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33.pgp
/etc/voa/arch/packages/default/openpgp/62cdb7020ff920e5aa642c3d4066950dd1f01f4d.pgp
/usr/local/share/voa/arch/packages/default/openpgp/bbe960a25ea311d21d40669e93df2003ba9b90a2.pgp
```

The four VOA subdirectory layers are the same for each of the verifiers: `arch/packages/default/openpgp`.

These layers signify, respectively:

- the _os_ identifier is `arch`,
- the verifier _purpose_ is `package`,
- the verifier _context_ is `default` and
- the verifier _technology_ is `openpgp`.

In a fully populated VOA structure, verifier files may be stored for different use cases, and stored in a mix of different _os_, _purpose_, _context_ and _technology_ paths.

## Testing

Integration tests in this crate can be run using `just containerized-integration-tests`.

These tests run in a `podman` container environment. They set up VOA file hierarchies and consume/evaluate them with `voa-core`.

Use the parameter `--nocapture` to see output for tests. Pass a set of test names to only run a select number of tests.

## Usage

This crate is not usually used directly in applications. Its main purpose is as a foundational building block for technology-specific VOA libraries, such as `voa-openpgp`.

Technology-specific VOA implementations are, in turn, used in applications (e.g. an application that validates signatures over distribution packages before installation).

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[OpenPGP certificates]: https://openpgp.dev/book/certificates.html
