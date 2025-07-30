project('alpm-soname-test', 'c', meson_version: '>=1.1.0')
libname = get_option('libname')
lib = shared_library(libname, 'lib.c', version: '1.2.3', soversion: '1')
executable('sotest', 'main.c', link_with: lib)
