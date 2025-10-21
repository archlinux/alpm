error-bridge-no-name = No 'pkgname' has been specified. At least one must be given.

error-bridge-invalid-package-name = The package name '{ $name }' is not valid:
  { $error }

error-bridge-undeclared-package = The split package '{ $name }' is not declared in pkgname, but a package function is present for it.

error-bridge-unused-package-function = An unused package function exists for undeclared split package: '{ $name }'

error-bridge-missing-required-keyword = Missing keyword: '{ $keyword }'

error-bridge-parse-error = Failed to parse input for keyword '{ $keyword }':
  { $error }

error-bridge-wrong-variable-type = Got wrong variable type for keyword '{ $keyword }'. Expected a { $expected }, got a { $actual }.

error-bridge-unexpected-arch = Found unexpected architecture suffix '{ $suffix }' for keyword '{ $keyword }'

error-bridge-unclearable-value = Tried to clear value for keyword '{ $keyword }', which is not allowed.

error-bridge-unexpected-array = Found array of values for keyword '{ $keyword }' that expects a single value:
  { $values }
