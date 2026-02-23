error-create-zstd-encoder =
  Błąd podczas tworzenia enkodera Zstandard w trakcie { $context } z { $compression_settings }:
  { $source }

error-create-zstd-decoder =
  Błąd podczas tworzenia dekodera Zstandard:
  { $source }

error-create-zstd-encoder-init = inicjalizacji

error-create-zstd-encoder-set-checksum = ustawiania sum kontrolnych do dodania

error-create-zstd-encoder-set-threads = ustawiania wielowątkowości

error-finish-encoder =
  Błąd podczas finalizowania enkodera { $compression_type }:
  { $source }

error-get-parallelism =
  Błąd podczas próby uzyskania dostępnej równoległości:
  { $source }

error-integer-conversion =
  Błąd podczas próby konwersji liczby całkowitej:
  { $source }

error-io-read =
  Błąd odczytu podczas { $context }:
  { $source }

error-io-write =
  Błąd zapisu podczas { $context }:
  { $source }

error-io-write-archive = zapisywania archiwum

error-io-read-archive-entries = odczytu zawartości archiwum

error-io-open-archive = otwierania archiwum do odczytu

error-io-read-archive-entry-content = odczytu zawartości pozycji archiwum

error-io-read-archive-entry-mode = odczytu uprawnień pozycji archiwum

error-io-read-archive-entry = odczytu pozycji archiwum

error-io-read-archive-entry-path = odczytu ścieżki pozycji archiwum

error-invalid-compression-level =
  Nieprawidłowy poziom kompresji { $level } (musi być w zakresie { $min } - { $max }).

error-unknown-compression-extension =
  Nieznane rozszerzenie pliku algorytmu kompresji:
  { $source }

error-unsupported-compression =
  Nieobsługiwany algorytm kompresji: { $value }
