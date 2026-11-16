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
# Portable across GNU and BSD userlands (Linux, macOS, Git Bash on Windows).
set -eu

version="${1:?usage: $0 <version>}"
case "$version" in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) echo "not a semver version: $version" >&2; exit 1 ;;
esac

cd "$(dirname "$0")/.."

# Cargo.toml: only the [package] version (the first `version = ` line).
awk -v v="$version" '
  !done && /^version = "/ { print "version = \"" v "\""; done = 1; next }
  { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# Cargo.lock: the version line right after `name = "aggrega"`.
awk -v v="$version" '
  pending && /^version = "/ { print "version = \"" v "\""; pending = 0; next }
  { pending = ($0 == "name = \"aggrega\""); print }
' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock

grep -m1 '^version' Cargo.toml
grep -A1 '^name = "aggrega"$' Cargo.lock | tail -n1
