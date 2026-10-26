#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../../../.." && pwd)
cd "$repo_root"

case $(uname -s) in
    Darwin) library_name=libboa_ffi.dylib ;;
    Linux) library_name=libboa_ffi.so ;;
    *) echo "The C test runner supports native macOS and Linux builds." >&2; exit 1 ;;
esac

if [ "$#" -gt 1 ]; then
    echo "Usage: $0 [--ci]" >&2
    exit 2
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$repo_root/target"}"
case "${1-}" in
    "")
        # Select the host explicitly so Cargo's configured target cannot cause us to
        # accidentally link a stale native library after a cross-compilation build.
        host=$(rustc -vV | sed -n 's/^host: //p')
        cargo build --locked -p boa_ffi --target "$host"
        library_dir=$(CDPATH= cd -- "$CARGO_TARGET_DIR/$host/debug" && pwd)
        ;;
    --ci)
        cargo build --locked --profile ci -p boa_ffi
        library_dir=$(CDPATH= cd -- "$CARGO_TARGET_DIR/ci" && pwd)
        ;;
    *)
        echo "Usage: $0 [--ci]" >&2
        exit 2
        ;;
esac

test_dir=$(mktemp -d "${TMPDIR:-/tmp}/boa-ffi-c.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT HUP INT TERM

for test_name in context value error; do
    "${CC:-cc}" -std=c11 -Wall -Wextra -Werror -pedantic \
        -I "$script_dir/../generated" "$script_dir/$test_name.c" \
        "$library_dir/$library_name" "-Wl,-rpath,$library_dir" -lm \
        -o "$test_dir/$test_name"
    "$test_dir/$test_name"
done
