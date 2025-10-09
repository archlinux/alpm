#!/bin/bash
# This script creates SRCINFO files used for testing based on PKGBUILD files.
# It expects *.pkgbuild files in subdirectories and creates respective *.srcinfo
# files that're located next to the pkgbuild file.
#
# Take a look at the docs in `correct.rs` to get a better understanding on how
# this is used in practice.
set -euo pipefail

tmpdir="$(mktemp --dry-run --directory)"
readonly tmpdir="$tmpdir"
mkdir -p "$tmpdir"

# Remove temporary dir on exit
cleanup() {
    if [[ -n "${tmpdir:-}" ]]; then
        rm -rf "$tmpdir"
    fi
}

trap cleanup EXIT

for file in "$@"; do
    [[ "$file" != *.pkgbuild ]] && continue
    echo "Generating srcinfo file for $file"
    output="${file%.pkgbuild}.srcinfo"
    cp "$file" "$tmpdir/PKGBUILD"

    # Create changelog and INSTALL stub files that `makepkg` is looking
    # file while creating the SRCINFO files. The file names are based
    echo "Stub file" >"$tmpdir/changelog.stub"
    echo "Stub file" >"$tmpdir/install.sh.stub"
    echo "Stub file" >"$tmpdir/overridden.stub"

    makepkg -D "$tmpdir" --printsrcinfo >"$output"
    rm "$tmpdir/PKGBUILD"
done
