#!/usr/bin/env bash

# Usage:
#
#     ./bump.sh $old $new
#
# Bump `Cargo.toml` versions from `$old` to `$new`

set -eux
cd "$(dirname $0)"

old_version=$1
new_version=$2

git ls-files \
    | grep Cargo.toml \
    | xargs sed -i '' -e "s|{ version = \"=$old_version\", path = \"|{ version = \"=$new_version\", path = \"|g"

git ls-files \
    | grep Cargo.toml \
    | xargs sed -i '' -e "s|version = \"$old_version\"|version = \"$new_version\"|"
