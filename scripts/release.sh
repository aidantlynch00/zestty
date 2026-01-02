#!/bin/sh
set -e

rm "release/zestty.tar.gz" "release/zestty.zip"
cargo build --release

dir=$(mktemp -d)

files="README.md zestty target/wasm32-wasip1/release/zestty.wasm"
names=""
for file in $files; do
    name=$(basename "$file")
    names="$names $name"
    cp "$file" "$dir"
done

pushd "$dir"
tar czf "zestty.tar.gz" $names
zip "zestty.zip" $names
popd

cp "$dir/zestty.tar.gz" "$dir/zestty.zip" "release/"
rm -rf "$dir"
