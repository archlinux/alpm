cli-about = Command line interface for interacting with alpm-files data.

cli-long-about = Command line interface for interacting with alpm-files files.
  
  This CLI interacts with the alpm-files file format: <https://alpm.archlinux.page/specifications/alpm-files.5.html>

cli-create-about = Create alpm-files data from a directory.

cli-create-long-about = Create alpm-files data from a directory.
  
  Outputs on stdout by default.

cli-format-format-help = Set the output format.

cli-create-input-dir-help = The directory to read from.

cli-create-style-help = Which style of the alpm-files format to use.

cli-format-about = Read and validate alpm-files data and return it in another file format.

cli-format-long-about = Read and validate alpm-files data and return it in another file format.
  
  If the data can be validated, the program exits with the data returned in another file
  format on stdout and an exit code of zero. If the file can not be validated, an error is
  emitted on stderr and the program exits with a non-zero exit code.

cli-input-file-help = An input file to read from.

cli-input-file-long-help = An input file to read from.
  
  If no file is provided, stdin is used instead.

cli-format-pretty-help = Determines whether the output will be displayed in a pretty non-minimized fashion.

cli-format-pretty-long-help = Determines whether the output will be displayed in a pretty non-minimized fashion.
  
  Only applies to formats that support pretty output, otherwise it is ignored.

cli-style-help = Which style of the alpm-files format to use.

cli-style-long-help = Which style of the alpm-files format to use.
  
  Only applies to formats that support different styles of the alpm-files format.

cli-output-help = A file path to write to.

cli-output-long-help = A file path to write to.
  
  If no file is provided, stdout is used instead.

cli-validate-about = Validate an alpm-files file.

cli-validate-long-about = Validate an alpm-files file.

  If the file can be validated, the program exits with no output and an exit code of zero.
  If the file cannot be validated, an error is emitted on stderr and the program exits with
  a non-zero exit code.

cli-output-format-json-help = The JSON output format.

cli-output-format-v1-help = The alpm-files output format.

cli-style-db-help = The alpm-db-files style.

cli-style-repo-help = The alpm-repo-files style.
