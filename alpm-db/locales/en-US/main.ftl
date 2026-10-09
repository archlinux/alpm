error-alpm-types =
  ALPM type error:
  { $source }

error-alpm-mtree =
  ALPM-MTREE error:
  { $source }

error-io = I/O error while { $context }:
  { $source }

error-io-database-read-dir = reading the entries in an alpm-db root directory

error-io-database-read-dir-entry = reading an entry in the alpm-db base directory

error-io-read-db-version = reading alpm-db version information

error-io-path = I/O error at path "{ $path }" while { $context }:
  { $source }

error-io-path-db-base-canonicalize = canonicalizing the alpm-db root directory

error-io-path-db-base-create = creating the alpm-db root directory

error-io-path-db-create-component = creating a file component for an alpm-db entry

error-io-path-db-entry-remove = removing an alpm-db entry directory

error-io-path-db-lock-create = creating a lock file for an alpm-db

error-io-path-db-set-permissions-on-dir = setting permissions on a directory

error-io-path-db-write-component = writing a file component for an alpm-db entry

error-io-path-check-existence = checking if the file exists

error-io-path-chown = changing ownership for the path

error-io-path-entry-dir-create = creating an alpm-db entry directory

error-io-path-metadata = retrieving metadata for the path

error-io-path-open-read-only = opening the file for reading

error-io-path-read-file-to-string = reading the file to string

error-io-path-rewind = rewinding the file handle for later use

error-io-path-write-db-version = writing an alpm-db version file

error-parse =
  Parser failed with the following error:
  { $error }

error-missing-section = Missing section: %{ $section }%

error-duplicate-section = Duplicate section: %{ $section }%

error-no-input-file = No input file given.

error-json = JSON error while { $context }:
  { $source }

error-unsupported-schema-version = Unsupported schema version: { $version }

error-database = Database error: { $source }

error-database-creation-misses-lock = No write lock provided when attempting to create a new alpm-db: "{ $path }"

error-database-entry-already-exists = The entry { $name } already exists in the database

error-database-entry-component-is-not-a-file = The component file of an alpm-db entry is not a regular file: "{ $path }"

error-database-entry-component-name-mismatch = The name of the entry and that of its alpm-db-desc component do not match: { $entry_name } vs. { $component_entry_name }

error-database-entry-component-wrong-permissions = The permissions for database entry component "{ $path }" differ from the defaults ("{ $defaults }"): "{ $permissions }"

error-database-entry-dir-wrong-permissions = The permissions for database entry directory "{ $path }" differ from the defaults ("{ $defaults }"): "{ $permissions }"

error-database-entry-is-not-a-directory = The alpm-db entry is not a directory: "{ $path }"

error-database-entry-does-not-exist = The entry { $name } does not exist in the database

error-database-entry-name-mismatch = Entry { $entry_name } does not match desc name { $desc_name }-{ $desc_version }{ $has_path ->
      [true]  at "{ $path }"
    *[false] ""
  }

error-database-exclusive-lock-file-failed = Failed to establish exclusive access to the alpm-db lock file "{ $path }": { $source }

error-database-is-locked = Unable to access alpm-db at "{ $path }" as it is currently locked. The lock file is located at: "{ $lock_file }"

error-database-root-is-not-a-directory = The root of the alpm-db is not a directory: "{ $path }"

error-database-transaction-misses-lock = No write lock provided for alpm db "{ $path }" when attempting to use transaction: "{ $uuid }"

error-database-transaction-no-action = The transaction has no actions: { $uuid }

error-database-wrong-permissions = The permissions for database directory "{ $path }" differ from the defaults ("{ $defaults }"): "{ $permissions }"

error-io-path-open-file = opening the file for reading

error-io-context-reading-alpm-db-files-data = reading alpm-db-files data

error-io-context-deriving-a-schema-version-from-alpm-db-files-data = deriving a schema version from alpm-db-files data

error-io-path-context-opening-the-file-for-reading = opening the file for reading

error-io-path-context-deriving-schema-version-from-alpm-db-files-file = deriving schema version from alpm-db-files file
error-io-path-schema-file =
  deriving schema version from DB desc file

error-io-read-db-desc-schema-data = deriving alpm-db-desc schema version from data

error-io-read-db-desc-data = reading alpm-db-desc data

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

cli-about = Command line interface for interacting with alpm-db-files data.

cli-long-about = Command line interface for interacting with alpm-db-files files.
  
  This CLI interacts with the alpm-db-files file format: <https://alpm.archlinux.page/specifications/alpm-db-files.5.html>

cli-create-about = Create alpm-db-files data from a directory.

cli-create-long-about = Create alpm-db-files data from a directory.
  
  Outputs to stdout by default.

cli-format-format-help = Set the output format.

cli-create-input-dir-help = The directory to read from.

cli-format-about = Read and validate alpm-db-files data and return it in another file format.

cli-format-long-about = Read and validate alpm-db-files data and return it in another file format.
  
  If the data can be validated, the program exits with the data returned in another file
  format on stdout and an exit code of zero. If the file can not be validated, an error is
  emitted on stderr and the program exits with a non-zero exit code.

cli-input-file-help = An input file to read from.

cli-input-file-long-help = An input file to read from.
  
  If no file is provided, stdin is used instead.

cli-format-pretty-help = Determines whether the output will be displayed in a pretty non-minimized fashion.

cli-format-pretty-long-help = Determines whether the output will be displayed in a pretty non-minimized fashion.
  
  Only applies to formats that support pretty output, otherwise it is ignored.

cli-output-help = A file path to write to.

cli-output-long-help = A file path to write to.
  
  If no file is provided, stdout is used instead.

cli-validate-about = Validate an alpm-db-files file.

cli-validate-long-about = Validate an alpm-db-files file.

  If the file can be validated, the program exits with no output and an exit code of zero.
  If the file cannot be validated, an error is emitted on stderr and the program exits with
  a non-zero exit code.

cli-output-format-json-help = The JSON output format.

cli-output-format-v1-help = The alpm-db-files output format.

cli-error-json = JSON error while { $context }:
  { $source }

cli-error-stdin-is-terminal = Stdin is a terminal and cannot be piped to

cli-error-io-path-opening-output-file-for-writing = opening output file for writing

cli-error-io-writing-to-output-file = writing to output file

cli-error-io-writing-to-stdout = writing to stdout

cli-error-io-path-reading-file-to-string = reading file to string

cli-error-io-reading-stdin-to-string = reading stdin to string

cli-error-json-serializing-alpm-db-files-data-as-pretty-printed-json-string = serializing alpm-db-files data as pretty printed JSON string

cli-error-json-serializing-alpm-db-files-data-as-json-string = serializing alpm-db-files data as JSON string

error-db-files = Alpm-db-files error: { $source }
