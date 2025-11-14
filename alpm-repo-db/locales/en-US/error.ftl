error-io = I/O error while { $context }:
  { $source }

error-io-path = I/O error at path "{ $path }" while { $context }:
  { $source }

error-io-read = Read error while { $context }:
  { $source }

error-parse =
  Parser failed with the following error:
  { $error }

error-missing-section = Missing section: %{ $section }%

error-duplicate-section = Duplicate section: %{ $section }%

error-invalid-section-for-version = Section %{ $section }% is invalid for the schema version { $version }.

error-empty-section = Unexpected empty section: %{ $section }%

error-no-input-file = No input file given.

error-unsupported-schema-version = Unsupported schema version: { $version }

error-invalid-format = Failed to parse v1 or v2 format.

error-io-path-open-file = opening the file for reading

error-io-path-schema-file =
  deriving schema version from package repository desc file

error-io-read-schema-data =
  deriving schema version from package repository desc data

error-io-read-repo-desc = reading package repository desc data

