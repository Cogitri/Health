#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$MESON_BUILD_ROOT"/cargo-home
export BUILDTYPE="$3"

if ! [ -z $FLATPAK_ID ]; then
    export PATH="$PATH:/usr/lib/sdk/rust-stable/bin"
fi

if [ $BUILDTYPE = "release" ]
then
    echo "RELEASE MODE"
    cargo test --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release -- --test-threads=1
else
    echo "DEBUG MODE"
    cargo test --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --verbose -- --test-threads=1
fi

