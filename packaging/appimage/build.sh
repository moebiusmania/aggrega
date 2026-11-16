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

# linuxdeploy has no stable releases; pin dated builds, and the AppImage
# runtime (the loader prepended to the AppImage, which appimagetool would
# otherwise fetch from a moving "continuous" release), all by checksum.
linuxdeploy_url=https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage
linuxdeploy_sha=c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
plugin_url=https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-appimage-x86_64.AppImage
plugin_sha=992d502a248e14ab185448ddf6f6e7d25558cb84d4623c354c3af350c25fccb3
runtime_url=https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64
runtime_sha=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

cd "$(dirname "$0")/../.."
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
tools=dist/appimage-tools
appdir=dist/AppDir
out="dist/aggrega-$version-x86_64.AppImage"

mkdir -p "$tools"
# fetch <url> <sha256>: downloads into $tools once, and checks it every time.
fetch() {
  file="$tools/${1##*/}"
  [ -f "$file" ] || curl -fsSL "$1" -o "$file"
  echo "$2  $file" | sha256sum -c --quiet - || { rm -f "$file"; exit 1; }
  chmod +x "$file"
}
fetch "$linuxdeploy_url" "$linuxdeploy_sha"
fetch "$plugin_url" "$plugin_sha"
fetch "$runtime_url" "$runtime_sha"

rm -rf "$appdir" "$out"
install -Dm644 LICENSE "$appdir/usr/share/licenses/aggrega/LICENSE"
install -Dm644 assets/fonts/OFL.txt "$appdir/usr/share/licenses/aggrega/OFL.txt"

# No FUSE on CI runners (or in containers): the tools extract themselves.
export APPIMAGE_EXTRACT_AND_RUN=1
export PATH="$PWD/$tools:$PATH"
export LDAI_OUTPUT="$PWD/$out" OUTPUT="$PWD/$out"
export LDAI_RUNTIME_FILE="$PWD/$tools/runtime-x86_64"
# Cargo already strips the release binary, and linuxdeploy's bundled strip
# fails the whole build on binaries it doesn't understand.
export NO_STRIP=1
export LINUXDEPLOY_OUTPUT_VERSION="$version"
linuxdeploy-x86_64.AppImage --appdir "$appdir" \
  --executable target/release/aggrega \
  --desktop-file packaging/aggrega.desktop \
  --icon-file assets/aggrega.svg \
  --output appimage
echo "$out"
