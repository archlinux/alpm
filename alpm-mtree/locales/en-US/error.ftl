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

error-path-validation-errors =
  Errors occurred while comparing ALPM-MTREE data to paths in "{ $base_dir }":
  { $errors }

error-alpm-common-path = ALPM common error:
  { $source }

error-create-hash-digest = Unable to create hash digest for path "{ $path }":
  { $source }

error-path-digest-mismatch =
  The hash digest of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_digest }, but that of "{ $path }" is { $path_digest }

error-path-gid-mismatch =
  The GID of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_gid }, but that of path "{ $path }" is { $path_gid }.

error-path-metadata =
  The metadata for path "{ $path }" can not be retrieved:
  { $source }

error-path-mismatch =
  The path "{ $mtree_path }" in the ALPM-MTREE data does not match the path "{ $path }".

error-path-missing =
  The path "{ $path }" does not exist, but the path "{ $mtree_path }" in the ALPM-MTREE data requires it to.

error-path-mode-mismatch =
  The mode of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_mode }, but that of path "{ $path }" is { $path_mode }

error-path-not-a-dir =
  The path "{ $path }" is not a directory, but the ALPM-MTREE data for "{ $mtree_path }" requires it to be.

error-path-not-a-file =
  The path "{ $path }" is not a file, but the ALPM-MTREE data for "{ $mtree_path }" requires it to be.

error-path-size-mismatch =
  The size of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_size }, but that of path "{ $path }" is { $path_size }

error-path-symlink-mismatch =
  The symlink "{ $mtree_path }" in the ALPM-MTREE data points at "{ $mtree_link_path }", while "{ $path }" points at "{ $link_path }"

error-path-time-mismatch =
  The time of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_time }, but that of path "{ $path }" is { $path_time }

error-path-uid-mismatch =
  The UID of "{ $mtree_path }" in the ALPM-MTREE data is { $mtree_uid }, but that of path "{ $path }" is { $path_uid }.

error-read-link =
  The path "{ $path }" does not exist or is not a symlink, but the path "{ $mtree_path }" in the ALPM-MTREE data requires it to be:
  { $source }.

error-unmatched-fs-paths =
  There are no matching ALPM-MTREE paths for the following file system paths:
  { $paths }

error-unmatched-mtree-paths =
  There are no matching file system paths for the following ALPM-MTREE paths:
  { $paths }
