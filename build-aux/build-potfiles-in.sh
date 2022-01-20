#!/usr/bin/env sh

cd "${MESON_SOURCE_ROOT}"

# ensure proper sorting
export LC_ALL=C

printf "# Misc data\n%s\n%s\n\n# UI Files\n%s\n\n# Source files\n%s\n" \
    "data/dev.Cogitri.Health.gschema.xml" \
    "$(find data -name "*.in" | sort -u )" \
    "$(find data -name "*.blp" | sort -u)" \
    "$(find src -name "*.rs" | sort -u | tail -n +2)" \
    > po/POTFILES.in
