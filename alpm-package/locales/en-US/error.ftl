error-compression = Compression error:
  { $source }

error-install-scriptlet = The alpm-install-scriptlet at { $path } is invalid because { $context }

error-package-input = Package input error:
  { $source }

error-input-dir-is-output-dir = The package input directory is also used as the output directory: { $path }

error-input-dir-in-output-dir = The package output directory ({ $output_path }) is located inside of the package input directory ({ $input_path })

error-io-path = I/O error at path { $path } while { $context }:
  { $source }

error-io-read = I/O read error while { $context }:
  { $source }

error-io-create-output-dir = creating output directory

error-io-create-abs-dir = creating absolute directory

error-io-create-package-file = creating a package file

error-io-get-metadata = retrieving metadata

error-io-read-file = reading the file

error-io-read-mtree = reading the ALPM-MTREE file

error-io-read-install-scriptlet = reading install scriptlet

error-io-open-scriptlet = opening an alpm-install-scriptlet file for reading

error-io-read-to-string = reading the contents to string

error-invalid-utf8 = Invalid UTF-8 while { $context }:
  { $source }

error-metadata-not-found = Metadata file { $name } not found in package.

error-end-of-entries = Reached the end of known entries while reading a package.

error-output-dir-in-input-dir = The package input directory ({ $input_path }) is located inside of the output directory ({ $output_path })

error-package = Package error:
  { $source }

error-path-not-exist = The path { $path } does not exist.

error-path-no-parent = The path { $path } has no parent.

error-path-not-file = The path { $path } is not a file.

error-path-read-only = The path { $path } is read-only.
