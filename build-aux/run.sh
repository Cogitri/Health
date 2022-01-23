#!/bin/sh

set -e

export RUST_BACKTRACE=1

ninja -C "${MESON_BUILD_ROOT}"
meson devenv -C "${MESON_BUILD_ROOT}" "${MESON_BUILD_ROOT}"/src/dev.Cogitri.Health
