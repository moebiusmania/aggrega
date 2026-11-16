#!/bin/sh
# Builds a portable x86_64 AppImage from target/release/aggrega.
#
# Build on an old distribution (the release workflow uses Ubuntu 22.04): the
# AppImage runs on systems with that glibc or newer. linuxdeploy copies the
# libraries the binary links against, except the ones every desktop has and
# that must match the host's drivers (glibc, libGL, fontconfig, freetype, X11,
# ...; see linuxdeploy's excludelist). Wayland, libxkbcommon and libGL are
# loaded at run time from the host, like any desktop app.
#
# Usage (from the repository root, after `cargo build --release`):
#   packaging/appimage/build.sh
# Output: dist/aggrega-<version>-x86_64.AppImage
set -eu

# linuxdeploy has no stable releases; pin a dated build.
linuxdeploy_tag=1-alpha-20251107-1
plugin_tag=1-alpha-20250213-1

cd "$(dirname "$0")/../.."
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
tools=dist/appimage-tools
appdir=dist/AppDir
out="dist/aggrega-$version-x86_64.AppImage"

mkdir -p "$tools"
fetch() {
  [ -x "$tools/$2" ] || { curl -fsSL "$1" -o "$tools/$2" && chmod +x "$tools/$2"; }
}
fetch "https://github.com/linuxdeploy/linuxdeploy/releases/download/$linuxdeploy_tag/linuxdeploy-x86_64.AppImage" linuxdeploy-x86_64.AppImage
fetch "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/$plugin_tag/linuxdeploy-plugin-appimage-x86_64.AppImage" linuxdeploy-plugin-appimage-x86_64.AppImage

rm -rf "$appdir" "$out"
install -Dm644 LICENSE "$appdir/usr/share/licenses/aggrega/LICENSE"
install -Dm644 assets/fonts/OFL.txt "$appdir/usr/share/licenses/aggrega/OFL.txt"

# No FUSE on CI runners (or in containers): the tools extract themselves.
export APPIMAGE_EXTRACT_AND_RUN=1
export PATH="$PWD/$tools:$PATH"
export LDAI_OUTPUT="$PWD/$out" OUTPUT="$PWD/$out"
export LINUXDEPLOY_OUTPUT_VERSION="$version"
linuxdeploy-x86_64.AppImage --appdir "$appdir" \
  --executable target/release/aggrega \
  --desktop-file packaging/aggrega.desktop \
  --icon-file assets/aggrega.svg \
  --output appimage
echo "$out"
