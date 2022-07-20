#!/bin/bash -e

if ! [ "$MESON_BUILD_ROOT" ]; then
    echo "This can only be run via meson, exiting!"
    exit 1
fi

PKGVER=$1-$2
DIST="$MESON_BUILD_ROOT/meson-dist/$PKGVER"

mkdir -p "${DIST}"/.cargo
cargo vendor vendor
cat << EOF > "${DIST}"/.cargo/config
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
mv vendor "${DIST}"
