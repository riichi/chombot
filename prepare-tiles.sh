#!/bin/sh

if [ $# -eq 0 ]; then
    echo "Usage: $0 tile_repo_root"
    exit 1
fi

ROOT=$1
if [ ! -d "$ROOT" ]; then
    echo "Directory $ROOT not found."
    exit 2
fi

set -e

TMP=$(mktemp -d)
trap "rm -rf '$TMP'" EXIT

pushd .
cd "$ROOT"
cp -r Black Regular "$TMP/"
cd "$TMP"
cp -r Regular Yellow
mv Regular Red

patch Yellow/Back.svg << EOF
258c258
<        style="opacity:1;fill:#ff3737;fill-opacity:1;fill-rule:nonzero;stroke:none;stroke-width:10;stroke-linecap:butt;stroke-linejoin:round;stroke-miterlimit:4;stroke-dasharray:none;stroke-dashoffset:0;stroke-opacity:1"
---
>        style="opacity:1;fill:#fcdb03;fill-opacity:1;fill-rule:nonzero;stroke:none;stroke-width:10;stroke-linecap:butt;stroke-linejoin:round;stroke-miterlimit:4;stroke-dasharray:none;stroke-dashoffset:0;stroke-opacity:1"
EOF

find -type f -name '*.svg' -print0 | xargs -0 sh -c $'
while [ -n "$1" ]; do
    SRC=$1
    DST="out/${SRC%.svg}.png"
    DIR=$(dirname "$SRC")
    mkdir -p "out/$DIR"
    inkscape -z -e "$DST" -w 600 "$SRC" >/dev/null
    shift
done'

popd
mkdir -p src/main/resources
mv "$TMP/out" src/main/resources/tiles
