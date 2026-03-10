#!/usr/bin/env bash
# 打包发布制品：二进制 + templates + static，不含源码与 data
set -e
cd "$(dirname "$0")/.."
cargo build --release
rm -rf dist
mkdir -p dist
cp target/release/pastebin dist/
cp -r templates static dist/
tar -czvf pastebin-dist.tar.gz -C dist .
echo "Done: pastebin-dist.tar.gz"
