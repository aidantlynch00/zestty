#!/bin/sh
# This script builds the zestty zellij plugin and loads it in zellij for
# development. This script is meant to be run from the zestty project root.

if [ -n "$1" ] && [ "$1" = "release" ]; then
    cargo build --release && \
    zellij action start-or-reload-plugin file:target/wasm32-wasip1/release/zestty.wasm
else
    cargo build && \
    zellij action start-or-reload-plugin file:target/wasm32-wasip1/debug/zestty.wasm
fi

