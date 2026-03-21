#!/bin/bash

export MESON_SOURCE_ROOT="$1"
export MESON_BUILD_ROOT="$2"
RUST_TARGET="$3"
BINARY_NAME="$4"

export CARGO_TARGET_DIR="$MESON_BUILD_ROOT/target"

cd "$MESON_SOURCE_ROOT" || exit 1

if [ "$RUST_TARGET" = "release" ]; then
    cargo build --release --manifest-path "$MESON_SOURCE_ROOT/Cargo.toml"
    cp "$CARGO_TARGET_DIR/release/$BINARY_NAME" "$MESON_BUILD_ROOT/$BINARY_NAME"
else
    cargo build --manifest-path "$MESON_SOURCE_ROOT/Cargo.toml"
    cp "$CARGO_TARGET_DIR/debug/$BINARY_NAME" "$MESON_BUILD_ROOT/$BINARY_NAME"
fi
