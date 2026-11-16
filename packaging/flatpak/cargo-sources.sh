#!/bin/sh
# Writes packaging/flatpak/cargo-sources.json (the vendored crates the Flatpak
# build uses offline) from Cargo.lock, with flatpak-cargo-generator pinned to a
# known commit of flatpak-builder-tools. Needs python3, pip and network access.
set -eu

# Bump together with a check that the generated file still builds.
rev=9d86d336a39b9af38b86f114b6ce5820fe260f91
url="https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/$rev/cargo/flatpak-cargo-generator.py"

cd "$(dirname "$0")/../.."
tools="$(mktemp -d)"
trap 'rm -rf "$tools"' EXIT

curl -fsSL "$url" -o "$tools/flatpak-cargo-generator.py"
python3 -m venv "$tools/venv"
"$tools/venv/bin/pip" install --quiet 'aiohttp>=3.9.5,<4' 'PyYAML>=6.0.2,<7' 'tomlkit>=0.13.3,<1'
"$tools/venv/bin/python" -I "$tools/flatpak-cargo-generator.py" Cargo.lock \
  -o packaging/flatpak/cargo-sources.json
