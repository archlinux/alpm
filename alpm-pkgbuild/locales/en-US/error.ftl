error-io = I/O error while { $context }:
  { $source }

error-io-path = I/O error at path { $path } while { $context }:
  { $source }

error-invalid-file = Encountered invalid file for path { $path }:
  { $context }

error-script-not-found = Could not find '{ $script_name }' script in $PATH:
  { $source }

error-script-failed-start = Failed to { $context } process to extract PKGBUILD:
  Command: alpm-pkgbuild-bridge { $parameters }
  { $source }

error-script-execution = Error during pkgbuild bridge execution:
  Command: alpm-pkgbuild-bridge { $parameters }
  
  stdout:
  { $stdout }
  
  stderr:
  { $stderr }

error-bridge-parse = An unexpected error occurred in the output parser for the 'alpm-pkgbuild-bridge' script:
  { $error }
  
  Please report this as a bug at:
  https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues

error-json = JSON error:
  { $source }
