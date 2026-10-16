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

# Select the host explicitly so Cargo's configured target cannot cause us to
# accidentally link a stale native library after a cross-compilation build.
host=$(rustc -vV | sed -n 's/^host: //p')
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$repo_root/target"}"
cargo build --locked -p boa_ffi --target "$host"
library_dir=$(CDPATH= cd -- "$CARGO_TARGET_DIR/$host/debug" && pwd)

test_dir=$(mktemp -d "${TMPDIR:-/tmp}/boa-ffi-c.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT HUP INT TERM

"${CC:-cc}" -std=c11 -Wall -Wextra -Werror -pedantic \
    -I "$script_dir/../generated" "$script_dir/context.c" \
    "$library_dir/$library_name" "-Wl,-rpath,$library_dir" \
    -o "$test_dir/context"
"$test_dir/context"
