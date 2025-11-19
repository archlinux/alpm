error-alpm-types =
  ALPM type error:
  { $source }

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

error-invalid-file = Invalid file at "{ $path }":
  { $context }

error-invalid-file-name = Invalid file name at "{ $path }":
  { $context }

error-no-input-file = No input file given.

error-json = JSON error while { $context }:
  { $source }

error-unsupported-schema-version = Unsupported schema version: { $version }

error-invalid-format = Failed to parse v1 or v2 format.

error-invalid-file-context-entry-name = extracting entry name from path

error-invalid-file-name-context-to-string = converting entry name to string

error-io-path-db-base-create = creating database base directory

error-io-path-db-base-metadata = reading metadata for database base directory

error-io-path-db-entries-read = reading database entries

error-io-path-db-entries-iterate = iterating database entries

error-io-path-open-file = opening the file for reading

error-io-read-db-desc = reading DB desc data

error-io-path-schema-file =
  deriving schema version from DB desc file

error-io-read-schema-data =
  deriving schema version from DB desc data

error-io-path-output-dir = creating output directory

error-io-path-output-file = creating output file

error-io-path-write-file = writing to output file

error-io-path-mtree-file-read = reading mtree file

error-io-path-entry-dir-create = creating database entry directory
error-io-path-db-entry-remove = removing database entry directory

error-io-path-write-desc = writing desc component

error-io-path-write-files = writing files component

error-io-path-write-mtree = writing mtree component

error-io-path-write-db-version = writing ALPM_DB_VERSION file

error-io-path-read-db-version = reading ALPM_DB_VERSION file

error-io-path-open-db-version = opening ALPM_DB_VERSION

error-io-read-db-version = reading ALPM_DB_VERSION

error-json-serialize-pretty = serializing to pretty JSON

error-json-serialize = serializing to JSON
