error-alpm-type =
  ALPM type parse error:
  { $source }

error-io-path =
  I/O error at path "{ $path }" while { $context }:
  { $source }

error-io-read =
  Read error while { $context }:
  { $source }

error-io-open-file = opening the file for reading

error-io-read-buildinfo = reading BuildInfo data

error-io-derive-schema-file = deriving schema version from BUILDINFO file

error-io-derive-schema-data = deriving schema version from BUILDINFO data

error-io-create-output-dir = creating output directory

error-io-create-output-file = creating output file

error-io-write-output-file = writing to output file

error-deserialize-buildinfo =
  Failed to deserialize BUILDINFO file:
  { $source }

error-no-input-file =
  No input file given.

error-unsupported-schema =
  Unsupported schema version: { $version }

error-wrong-schema-version =
  Wrong schema version used to create a BUILDINFO: { $version }

error-missing-format-field =
  Missing format field.

error-json =
  JSON error:
  { $source }
