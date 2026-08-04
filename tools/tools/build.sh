#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/../bin"

echo "ROOT: $ROOT"
echo "BIN: $BIN"

mkdir -p "$BIN"

cd "$ROOT"

copy_bin() {
    local src="$1"
    local dst="$2"

    install -m755 "$src" "$dst" 2>/dev/null || cp "$src" "$dst"
}

case "$(uname -s)" in
    Linux*)
        echo "==> Linux"

        cargo build --release --target x86_64-unknown-linux-musl
        copy_bin \
            target/x86_64-unknown-linux-musl/release/cli \
            "$BIN/cli"

        echo "Done"
        ;;

    MINGW*|MSYS*|CYGWIN*)
        echo "==> Windows"

        cargo build --release
        copy_bin \
            target/release/cli.exe \
            "$BIN/cli.exe"

        echo "Done"
        ;;

    *)
        echo "Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac

echo
ls -lh "$BIN"