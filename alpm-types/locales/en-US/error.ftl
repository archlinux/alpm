error-invalid-integer = Invalid integer (caused by { $kind })
 
error-invalid-variant = Invalid variant ({ $error })

error-invalid-email = Invalid e-mail ({ $error })

error-invalid-url = Invalid URL ({ $error })

error-invalid-license = Invalid license ({ $error })

error-invalid-semver = Invalid semver ({ $kind })

error-invalid-chars = Value contains invalid characters: { $invalid_char }

error-incorrect-length = Incorrect length, got { $length } expected { $expected }

error-delimiter-not-found = Value is missing the required delimiter: { $delimiter }

error-restrictions-not-met = Does not match the restrictions ({ $restrictions })

error-regex-mismatch = Value '{ $value }' does not match the '{ $regex_type }' regex: { $regex }

error-parse = Parser failed with the following error:
  { $error }

error-missing-component = Missing component: { $component }

error-path-not-absolute = The path is not absolute: { $path }

error-path-not-relative = The path is not relative: { $path }

error-path-not-file = The path is not a file: { $path }

error-filename-invalid-chars = File name ({ $path }) contains invalid characters: { $invalid_char }

error-filename-empty = File name is empty

error-deprecated-license = Deprecated license: { $license }

error-invalid-component = Invalid component { $component } encountered while { $context }

error-context-convert-full-to-minimal = converting a full alpm-package-version to a minimal alpm-package-version

error-invalid-pgp-fingerprint = Invalid OpenPGP v4 fingerprint, only 40 uppercase hexadecimal characters and optional whitespace separators are allowed

error-invalid-pgp-keyid = The string is not a valid OpenPGP key ID: { $keyid }, must be 16 hexadecimal characters

error-invalid-base64-encoding = Invalid base64 encoding encountered while decoding { $expected_item }

error-invalid-soname-v1 = Invalid shared object name (v1): { $name }

error-package = Package error: { $error }

error-unknown-compression = Unknown compression algorithm file extension: { $value }

error-unknown-filetype = Unknown file type identifier: { $value }

error-invalid-architectures = The architecture combination is invalid: { $architectures } ({ $context })

error-invalid-base64-encoding-pgp-signature = base64 encoded OpenPGP detached signature
