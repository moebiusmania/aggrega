#!/bin/sh
# Stamps a release version into Cargo.toml and Cargo.lock.
#
# The Git tag is the only source of truth for release versions: the repository
# keeps the placeholder version 0.0.0-dev, and the release workflow runs this
# script with the tag's version before building, so `CARGO_PKG_VERSION` (the
# sidebar, About tab, `--version`, the macOS Info.plist and the Flatpak
# metainfo) matches the tag. Updating Cargo.lock too keeps `--locked` builds
# working.
#
# Usage: packaging/set-version.sh <version>      e.g. 1.2.3 or 1.2.3-rc1
# Portable across GNU and BSD userlands (Linux, macOS, Git Bash on Windows),
# and tolerant of CRLF checkouts.
set -eu

version="${1:?usage: $0 <version>}"
if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$'; then
  echo "not a release version (X.Y.Z or X.Y.Z-suffix): $version" >&2
  exit 1
fi

cd "$(dirname "$0")/.."

# Cargo.toml: only the [package] version (the first `version = ` line).
awk -v v="$version" '
  !done && /^version = "/ { print "version = \"" v "\""; done = 1; next }
  { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# Cargo.lock: the version line right after `name = "aggrega"`.
awk -v v="$version" '
  pending && /^version = "/ { print "version = \"" v "\""; pending = 0; next }
  { pending = ($0 ~ /^name = "aggrega"\r?$/); print }
' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock

# Fail loudly rather than leave a half-stamped tree for `--locked` to trip on.
want="version = \"$version\""
grep -m1 '^version = ' Cargo.toml | tr -d '\r' | grep -Fqx "$want" \
  || { echo "Cargo.toml was not updated" >&2; exit 1; }
grep -A1 '^name = "aggrega"' Cargo.lock | tail -n1 | tr -d '\r' | grep -Fqx "$want" \
  || { echo "Cargo.lock was not updated" >&2; exit 1; }
echo "$want"
