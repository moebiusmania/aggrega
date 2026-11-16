#!/bin/sh
# Builds the Flatpak bundle from this checkout.
#
# Needs flatpak, flatpak-builder (with the flathub remote added, for the
# runtime, SDK and Rust extension), python3 and git. Generates
# cargo-sources.json from Cargo.lock with flatpak-cargo-generator, then builds
# and exports dist/aggrega-<version>-x86_64.flatpak.
#
# Usage (from the repository root):  packaging/flatpak/build.sh
set -eu

cd "$(dirname "$0")/../.."
here=packaging/flatpak
app_id=io.github.moebiusmania.Aggrega
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
arch="$(uname -m)"

"$here/cargo-sources.sh"

flatpak-builder --user --force-clean --install-deps-from=flathub \
  --repo="$here/repo" "$here/build" "$here/$app_id.yml"

mkdir -p dist
flatpak build-bundle --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo \
  "$here/repo" "dist/aggrega-$version-$arch.flatpak" "$app_id"
echo "dist/aggrega-$version-$arch.flatpak"
