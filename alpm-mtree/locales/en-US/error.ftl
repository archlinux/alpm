error-alpm-common = ALPM common error:
  { $source }

error-duplicate-paths = The following file system paths are duplicates:
  { $paths }

error-file-creation = File creation error:
  { $source }

error-io = I/O error while { $context }:
  { $source }

error-io-path = I/O error at path { $path } while { $context }:
  { $source }

error-io-open-file-read = opening the file for reading

error-io-read-mtree-data = reading ALPM-MTREE data

error-io-derive-schema = deriving schema version from ALPM-MTREE file

error-io-create-file = creating the file

error-io-write-gzip = writing data to gzip compressed file

error-io-finish-gzip = finishing gzip compressed file

error-invalid-utf8 = Invalid UTF-8 data:
  { $source }

error-no-input-file = No input file given.

error-invalid-gzip = Error while unpacking gzip file:
  { $source }

error-path-validation = One or more errors occurred during path validation.

error-parse = File parsing error:
  { $source }

error-interpreter =
  Error while interpreting file in line { $line_number }:
  Affected line:
  { $line }

  Reason:
  { $reason }

error-json = JSON error:
  { $source }

error-unsupported-schema-version = Unsupported schema version: { $version }
