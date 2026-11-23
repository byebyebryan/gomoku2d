#!/usr/bin/env bash
# Install the same upstream Linux x86_64 build on the CI and Pages runners.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <bin-directory>" >&2
  exit 1
fi

version=v0.15.1
asset="wasm-pack-${version}-x86_64-unknown-linux-musl"
sha256=410101009739b9d01c7d5affaaad6e9c9a24b4cba6791a4805db8e89b9bf7af2
download_dir="$(mktemp -d)"
trap 'rm -rf -- "$download_dir"' EXIT

(
  cd "$download_dir"
  curl --fail --silent --show-error --location --retry 3 \
    "https://github.com/wasm-bindgen/wasm-pack/releases/download/${version}/${asset}.tar.gz" \
    --output "${asset}.tar.gz"
  printf '%s  %s\n' "$sha256" "${asset}.tar.gz" | sha256sum --check
  tar -xzf "${asset}.tar.gz" --strip-components=1 "${asset}/wasm-pack"
)

install -d -- "$1"
install -m 0755 -- "$download_dir/wasm-pack" "$1/wasm-pack"
"$1/wasm-pack" --version
