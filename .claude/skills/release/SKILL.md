---
name: release
description: Cut a new Aggrega release by tagging vX.Y.Z and watching the Release build workflow build every platform (Arch tarball, AppImage, Flatpak, macOS dmg, Windows zip) and publish the GitHub Release. Use when asked to release, tag, ship or publish a new version.
---

# Release Aggrega

Pushing a tag matching `v[0-9]+.[0-9]+.[0-9]+*` triggers `.github/workflows/release.yml`. Each build job first stamps the tag's version into `Cargo.toml` and `Cargo.lock` with `packaging/set-version.sh` (that's what the sidebar shows) and checks `aggrega --version` against it. The build jobs run in parallel:

- `build-arch` builds in an `archlinux` container: `aggrega-<version>-x86_64.tar.gz`.
- `build-appimage` builds on `ubuntu-22.04` with `packaging/appimage/build.sh`: `aggrega-<version>-x86_64.AppImage`.
- `flatpak-sources` generates `cargo-sources.json` from `Cargo.lock`, then `build-flatpak` builds the manifest in Flathub's `freedesktop-25.08` container: `aggrega-<version>-x86_64.flatpak`.
- `build-macos` runs `packaging/macos/bundle.sh` on `macos-latest`: `aggrega-<version>-macos-arm64.dmg` (an Apple silicon only, ad-hoc signed `Aggrega.app` in a disk image).
- `build-windows` builds on `windows-latest` (MSVC): `aggrega-<version>-windows-x86_64.zip` (an unsigned `aggrega.exe` with the README and licenses).

Once all of them succeeded, `publish` creates the GitHub Release for the tag with generated notes and the five files attached (a pre-release when the tag has a suffix like `-rc1`). If any build fails, no release is created.

## 1. Pick the version

- List existing tags with `git tag --sort=-v:refname | head`. The new version must be higher than the latest one.
- If the user didn't give a version, propose one from the commits since the last tag (`git log <last-tag>..main --oneline`): patch for fixes, minor for features. Wait for them to confirm.

## 2. Check `main`

- Be on `main`, up to date with `origin/main`, with a clean working tree. The tag goes on this commit.
- Run the `precheck` skill. Don't tag if it fails.
- Check that the latest CI run on `main` passed: `gh run list --workflow CI --branch main --limit 1`.

## 3. Don't bump the version in the repo

The Git tag is the only source of truth for release versions. `Cargo.toml` and `Cargo.lock` keep the placeholder `0.0.0-dev`, and the PKGBUILD derives `pkgver` from `git describe`. Leave them alone: the workflow stamps the real version inside CI. No release commit is needed.

Tag a commit on `main` (`git describe` on `main` must reach the new tag), never a PR or backup branch: the PKGBUILD's version only sees tags in `main`'s history.

To check the pipeline before tagging (for example after changing `release.yml` or `packaging/`), run it by hand with `gh workflow run "Release build" --ref main`. It builds everything as `0.0.0-ci` and publishes nothing.

## 4. Tag and push (confirm first)

Pushing a tag is outward-facing and triggers a public build. Ask the user before running:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

## 5. Watch the build and check the release

```bash
gh run list --workflow "Release build" --limit 1          # get the run id
gh run watch <run-id> --exit-status
gh release view vX.Y.Z --json url,assets -q '.url, .assets[].name'
```

Report the result. On success, give the release URL and the five file names. The build jobs are independent, so one platform can fail while the others succeed, but then `publish` is skipped and there's no release. On failure, show the failing step's log (`gh run view <run-id> --log-failed`). A failed job can be re-run (`gh run rerun <run-id> --failed`), and `publish` then runs once everything is green. Don't delete or move the tag without asking the user.
