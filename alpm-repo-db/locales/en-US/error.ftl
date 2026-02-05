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

error-invalid-section-for-version = Section %{ $section }% is invalid for the schema version { $version }.

error-empty-section = Unexpected empty section: %{ $section }%

error-no-input-file = No input file given.

error-unsupported-schema-version = Unsupported schema version: { $version }

error-invalid-format = Failed to parse v1 or v2 format.

error-json = JSON error while { $context }:
  { $source }

error-io-path-open-file = opening the file for reading

error-io-read-db-desc = reading DB desc data

error-io-read-repo-desc = reading package repository desc data

error-io-context-reading-alpm-repo-files-data = reading alpm-repo-files data

error-io-context-deriving-a-schema-version-from-alpm-repo-files-data = deriving a schema version from alpm-repo-files data

error-io-path-context-opening-the-file-for-reading = opening the file for reading

error-io-path-context-deriving-schema-version-from-alpm-repo-files-file = deriving schema version from alpm-repo-files file

error-io-path-schema-file =
  deriving schema version from package repository desc file

error-io-read-schema-data =
  deriving schema version from package repository desc data

error-io-create-output-dir = creating output directory

error-io-path-output-dir = creating output directory

error-io-create-output-file = creating output file

error-io-path-output-file = creating output file

error-io-write-output-file = writing to output file

error-io-path-write-file = writing to output file

error-json-serialize-pretty = serializing to pretty JSON

error-json-serialize = serializing to JSON

error-invalid-files-paths = Invalid paths for alpm-repo-files data:
  { $message }

filesv1-path-errors-absolute-paths = Absolute paths

filesv1-path-errors-paths-without-a-parent = Paths without a parent

filesv1-path-errors-duplicate-paths = Duplicate paths

error-version-is-unknown = The schema version of the alpm-repo-files data is unknown

cli-about = Command line interface for interacting with alpm-repo-files data.

cli-long-about = Command line interface for interacting with alpm-repo-files files.
  
  This CLI interacts with the alpm-repo-files file format: <https://alpm.archlinux.page/specifications/alpm-repo-files.5.html>

cli-create-about = Create alpm-repo-files data from a directory.

cli-create-long-about = Create alpm-repo-files data from a directory.
  
  Outputs on stdout by default.

cli-format-format-help = Set the output format.

cli-create-input-dir-help = The directory to read from.

cli-format-about = Read and validate alpm-repo-files data and return it in another file format.

cli-format-long-about = Read and validate alpm-repo-files data and return it in another file format.
  
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

cli-validate-about = Validate an alpm-repo-files file.

cli-validate-long-about = Validate an alpm-repo-files file.

  If the file can be validated, the program exits with no output and an exit code of zero.
  If the file cannot be validated, an error is emitted on stderr and the program exits with
  a non-zero exit code.

cli-output-format-json-help = The JSON output format.

cli-output-format-v1-help = The alpm-repo-files output format.

cli-error-json = JSON error while { $context }:
  { $source }

cli-error-stdin-is-terminal = Stdin is a terminal and cannot be piped to

cli-error-io-path-opening-output-file-for-writing = opening output file for writing

cli-error-io-writing-to-output-file = writing to output file

cli-error-io-writing-to-stdout = writing to stdout

cli-error-io-path-reading-file-to-string = reading file to string

cli-error-io-reading-stdin-to-string = reading stdin to string

cli-error-json-serializing-alpm-repo-files-data-as-pretty-printed-json-string = serializing alpm-repo-files data as pretty printed JSON string

cli-error-json-serializing-alpm-repo-files-data-as-json-string = serializing alpm-repo-files data as JSON string
