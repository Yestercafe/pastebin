#!/usr/bin/env bash
# 在 Ubuntu 20.04 容器内构建发布包，得到的二进制可在 Ubuntu 20.04 上直接运行。
# 需已安装 Docker。首次运行会安装系统依赖与 Rust，耗时较长。
# 若拉取报错（short read / content size of zero），国内用户可参考 docs/DOCKER-MIRRORS-CN.md
set -e
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

sudo docker run --rm \
  -v "$ROOT:/app" -w /app \
  ubuntu:20.04 \
  bash -c '
    set -e
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
      curl build-essential pkg-config libsqlite3-dev
    curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    . "$HOME/.cargo/env"
    cargo build --release
  '

rm -rf dist
mkdir -p dist
cp target/release/pastebin dist/
cp -r templates static dist/
cp pastebin.toml dist/
tar -czvf pastebin-dist-ubuntu2004.tar.gz -C dist .
echo "Done: pastebin-dist-ubuntu2004.tar.gz (built for Ubuntu 20.04)"
echo "Unpack and run on Ubuntu 20.04; optionally set CONFIG to path of pastebin.toml."