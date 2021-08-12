#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$MESON_BUILD_ROOT"/cargo-home
export OUTPUT_APP="$3"
export BUILDTYPE="$4"
export APP_BIN="$5"
export DAEMON_BIN="$6"
export OUTPUT_DAEMON="$7"


if [ $BUILDTYPE = "release" ]
then
    echo "RELEASE MODE"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/"$APP_BIN" "$OUTPUT_APP" && \
        cp "$CARGO_TARGET_DIR"/release/"$DAEMON_BIN" "$OUTPUT_DAEMON"
else
    echo "DEBUG MODE"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --verbose && \
        cp "$CARGO_TARGET_DIR"/debug/"$APP_BIN" "$OUTPUT_APP" && \
        cp "$CARGO_TARGET_DIR"/debug/"$DAEMON_BIN" "$OUTPUT_DAEMON"
fi

