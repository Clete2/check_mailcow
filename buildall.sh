#!/bin/bash

echo "##### Building local (dev) #####"
cargo build

echo "##### Building local (release) #####"
cargo build --release # Build dev for current arch

targets="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"

for target in $targets; do
  echo "##### Building $target (release) #####"
  cross build --release --target $target || exit 1
done
