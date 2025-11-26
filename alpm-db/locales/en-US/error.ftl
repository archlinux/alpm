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

error-no-input-file = No input file given.

error-json = JSON error while { $context }:
  { $source }

error-unsupported-schema-version = Unsupported schema version: { $version }

error-invalid-format = Failed to parse v1 or v2 format.

error-io-path-open-file = opening the file for reading

error-io-read-db-desc = reading DB desc data

error-io-context-reading-alpm-db-files-data = reading alpm-db-files data

error-io-context-deriving-a-schema-version-from-alpm-db-files-data = deriving a schema version from alpm-db-files data

error-io-path-context-opening-the-file-for-reading = opening the file for reading

error-io-path-context-deriving-schema-version-from-alpm-db-files-file = deriving schema version from alpm-db-files file
error-io-path-schema-file =
  deriving schema version from DB desc file

error-io-read-schema-data =
  deriving schema version from DB desc data

error-io-path-output-dir = creating output directory

error-io-path-output-file = creating output file

error-io-path-write-file = writing to output file

error-json-serialize-pretty = serializing to pretty JSON

error-json-serialize = serializing to JSON

error-invalid-files-paths = Invalid paths for alpm-db-files data:
  { $message }

error-invalid-backup-entries = Invalid backup entries for alpm-db-files data:
  { $message }

filesv1-path-errors-absolute-paths = Absolute paths

filesv1-path-errors-paths-without-a-parent = Paths without a parent

filesv1-path-errors-duplicate-paths = Duplicate paths

backupv1-errors-absolute-paths = Absolute backup paths

backupv1-errors-not-in-files-section = Backup paths not listed in %FILES% section

backupv1-errors-duplicate-paths = Duplicate backup paths

error-schema-version-is-unknown = The schema version of the alpm-db-files data is unknown
