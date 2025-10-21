error-alpm-type = ALPM type parse error: { $error }

error-io = I/O error while { $context }:
  { $error }

error-io-path = I/O error at path { $path } while { $context }:
  { $error }

error-io-path-opening-file = opening file

error-io-path-reading-file = reading file

error-io-read-srcinfo-data = reading SRCINFO data

error-io-deriving-schema-from-srcinfo = deriving schema version from SRCINFO file

error-invalid-utf8 = UTF-8 parse error: { $error }

error-missing-keyword = The SRCINFO data misses the required keyword '{ $keyword }'

error-no-input-file = No input file given.

error-parse = File parsing error:
  { $error }

error-json = JSON error: { $error }

error-unsupported-schema-version = Unsupported schema version: { $version }

error-bridge = alpm-pkgbuild bridge error: { $error }

error-bridge-conversion = Bridge conversion error: { $error }
