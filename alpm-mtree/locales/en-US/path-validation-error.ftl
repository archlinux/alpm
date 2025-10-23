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
