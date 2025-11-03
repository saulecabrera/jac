#! /usr/bin/env bash

set -ex

cargo check -p=jacrt --target=wasm32-wasip2
cargo check --workspace --exclude=jacrt

cargo test --workspace --exclude=jacrt
