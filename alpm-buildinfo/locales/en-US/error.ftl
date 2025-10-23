error-alpm-type =
  ALPM type parse error:
  { $source }

error-io-path =
  I/O error at path "{ $path }" while { $context }:
  { $source }

error-io-read =
  Read error while { $context }:
  { $source }

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
