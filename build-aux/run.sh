#!/bin/sh

ninja -C "${MESON_BUILD_ROOT}"
meson devenv -C "${MESON_BUILD_ROOT}" "${MESON_BUILD_ROOT}"/target/debug/health
