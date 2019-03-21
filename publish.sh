#!/usr/bin/env bash

# Usage:
#
#     ./publish.sh
#
# Run `cargo publish` for each `fart` crate!

set -eux
cd "$(dirname $0)"

CRATES=$(echo "partial-min-max utils aabb 2d-geom" | tr ' ' '\n')

for x in $CRATES; do
    cd "./crates/$x"
    cargo publish --no-verify
    cd -
done

cargo publish --no-verify
