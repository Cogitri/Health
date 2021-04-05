#!/bin/bash -e

if ! [ "$MESON_BUILD_ROOT" ]; then
    echo "This can only be run via meson, exiting!"
    exit 1
fi

PKGVER=$1-$2
DIST="$MESON_BUILD_ROOT/meson-dist/$PKGVER"

mkdir -p "${DIST}"/.cargo
cargo vendor vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > "${DIST}"/.cargo/config
mv vendor "${DIST}"
