---
name: release
description: Cut a new Aggrega release by bumping the version, tagging vX.Y.Z, watching the Release build workflow and fetching the Arch Linux tarball. Use when asked to release, tag, ship or publish a new version.
---

# Release Aggrega

Pushing a tag matching `v[0-9]+.[0-9]+.[0-9]+*` triggers `.github/workflows/release.yml`. It builds in an `archlinux` container, rewrites the `[package]` version in `Cargo.toml` from the tag (that's what the sidebar shows), and uploads `aggrega-<version>-x86_64.tar.gz` as a workflow **artifact**. It does not create a GitHub Release.

## 1. Pick the version

- List existing tags with `git tag --sort=-v:refname | head`. The new version must be higher than the latest one.
- If the user didn't give a version, propose one from the commits since the last tag (`git log <last-tag>..main --oneline`): patch for fixes, minor for features. Wait for them to confirm.

## 2. Check `main`

- Be on `main`, up to date with `origin/main`, with a clean working tree.
- Run the `precheck` skill. Don't tag if it fails.
- Check that the latest CI run on `main` passed: `gh run list --workflow CI --branch main --limit 1`.

## 3. Bump the version in the repo

The workflow patches the version only inside CI, so local builds and the Arch package keep showing whatever the repo says. Keep them in sync:

- `Cargo.toml`: the `version =` line under `[package]`.
- `Cargo.lock`: run `cargo check` (or `cargo update -p aggrega`) so aggrega's own entry picks up the new version.
- `packaging/arch/PKGBUILD`: set `pkgver` to the new version and reset `pkgrel=1`.

Commit these as `Release vX.Y.Z` (following the repo's commit conventions) and show the user the diff.

## 4. Tag and push (confirm first)

Pushing to `main` and pushing a tag are both outward-facing, and a pushed tag triggers a public build. Ask the user before running:

```bash
git push origin main
git tag vX.Y.Z
git push origin vX.Y.Z
```

## 5. Watch the build and fetch the artifact

```bash
gh run list --workflow "Release build" --limit 1          # get the run id
gh run watch <run-id> --exit-status
gh run download <run-id> --dir <scratch dir>
```

Report the result. On success, give the run URL and the artifact name. On failure, show the failing step's log (`gh run view <run-id> --log-failed`). Don't delete or move the tag without asking the user.
