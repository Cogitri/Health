#!/bin/sh

SCRIPTDIR="$(dirname "$(readlink -f "$0")")"

for file in "$SCRIPTDIR"/../src/**/*.rs; do
    if ! "$SCRIPTDIR/"clone-warn.sh "$file"; then
        failed=true
    fi
done

if [ "$failed" = "true" ]; then
    exit 1
fi
