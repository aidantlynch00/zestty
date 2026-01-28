#!/bin/sh

release_dir=$(realpath ./release)
if ! [ -d "$release_dir" ]; then
    exit 1
fi

# clean any old release artifacts
rm "$release_dir"/*

# build the zestty plugin
cargo build --release

# create a temp directory for archival
tmp_dir=$(mktemp -d)

# copy release files into temp directory
files="README.md zestty target/wasm32-wasip1/release/zestty.wasm"
names=""
for file in $files; do
    name=$(basename "$file")
    names="$names $name"
    cp "$file" "$tmp_dir"
done

pushd "$tmp_dir"

# archive release files
tar czf "zestty.tar.gz" $names
zip "zestty.zip" $names

# copy everything to release directory
cp ./* "$release_dir"

popd
rm -rf "$tmp_dir"
