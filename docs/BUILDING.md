# Building and running Aggrega

Arch Linux is the first supported target. Every release also ships a portable AppImage and a Flatpak for other Linux distributions (see [section 5](#5-install-on-linux)), a macOS app ([section 6](#6-macos)) and a Windows zip ([section 7](#7-windows)), all attached to the [GitHub Release](https://github.com/moebiusmania/aggrega/releases) for that version.

## 1. Prerequisites (Arch Linux)

```bash
# Toolchain + native libraries used by the windowing/rendering stack
sudo pacman -S --needed base-devel rustup pkgconf \
    fontconfig freetype2 libxkbcommon wayland libglvnd \
    libx11 libxcursor libxi libxrandr

# Install the stable Rust toolchain (Rust 1.88 or newer is required)
rustup default stable
```

> If you'd rather not use `pacman` for Rust, the official installer works too:
> `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
> Then run `source ~/.cargo/env` or open a new shell. In fish, use `fish_add_path ~/.cargo/bin`.

What each native library is for:

| Package | Used for |
|---|---|
| `fontconfig`, `freetype2` | Finding and rasterising system fonts |
| `libxkbcommon` | Keyboard handling (Wayland and X11) |
| `wayland` | Native Wayland windows |
| `libx11`, `libxcursor`, `libxi`, `libxrandr` | X11 / XWayland fallback |
| `libglvnd` (plus your GPU's GL driver) | OpenGL for the GPU renderer |

SQLite is compiled into the binary (the `rusqlite` `bundled` feature), and TLS uses `rustls`. You don't need system `sqlite` or `openssl` packages.

## 2. Run in development

```bash
cargo run
```

The first build downloads and compiles all dependencies, which takes a few minutes. After that, rebuilds take seconds. The dev profile optimises dependencies (`opt-level = 2`), so animations stay smooth while you iterate, but it doesn't optimise Aggrega's own code, which keeps compiles quick.

`build.rs` compiles the `.slint` files at build time, so a Slint error shows up as a normal `cargo build` error with file and line.

### Faster UI iteration (optional)

- **VS Code / VSCodium:** install the *Slint* extension to get live preview, completion and diagnostics for `ui/*.slint`.
- **Standalone viewer:** `cargo install slint-viewer`, then run `slint-viewer --auto-reload ui/app.slint`. The UI renders with default data and doesn't call into the Rust backend.

## 3. Release build

```bash
cargo build --release
./target/release/aggrega
```

The release profile uses fat LTO, a single codegen unit, `panic = "abort"`, and strips symbols. It produces one self-contained binary of about 22 MB. Measured on the first test machine: the window appears in about 250 ms, and Aggrega's own heap is roughly 15 MB. Total RSS is higher because the GPU driver's shared libraries are mapped in.

A `Makefile` wraps the common commands:

```bash
make dev        # cargo run
make release    # optimised build
make run        # build + run the release binary
make test       # unit tests
make lint       # rustfmt check + clippy (warnings are errors)
make install    # installs binary, .desktop and icon into ~/.local (PREFIX=... to change)
make uninstall
make package    # builds an Arch package with makepkg
make macos-app  # builds Aggrega.app and a .dmg (macOS only, see section 6)
make appimage   # builds the AppImage (Linux, see section 5)
make flatpak    # builds the Flatpak bundle (Linux, see section 5)
```

### Release builds in CI

Pushing a version tag triggers the `Release build` GitHub Actions workflow (`.github/workflows/release.yml`):

```bash
git tag v0.3.0
git push origin v0.3.0
```

The tag is the only source of truth for the version. The repository keeps the placeholder `0.0.0-dev` in `Cargo.toml`, which is what local builds show, and every job first runs `packaging/set-version.sh` to stamp the tag's version into `Cargo.toml` and `Cargo.lock`: `v0.3.0` becomes `0.3.0`, and the sidebar (under the logo), the About tab in Settings, `aggrega --version`, the macOS `Info.plist` and the Flatpak metainfo show it. Every job fails if `aggrega --version` doesn't print the tag's version.

The workflow builds the Arch tarball, the AppImage, the Flatpak, the macOS disk image and the Windows zip in parallel. Once all five succeeded, a last `publish` job creates the GitHub Release for the tag, with generated release notes and the five files attached. If any platform fails, no release is created; fix it and push a new tag. Tags with a suffix (`v1.0.0-rc1`) become pre-releases.

## 4. Tests and linting

```bash
cargo test                  # parser, storage, HTML/text helpers
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The tests don't touch the network or your real data. The storage tests use a temporary directory.

The UI tests in `src/app.rs` create the real window with Slint's headless testing backend (`i-slint-backend-testing`, a dev-dependency pinned to the exact `slint` version), so they need no display. They check the responsive layout at several window sizes and drive the reader view: opening an article, swapping a summary for the full page, pictures, read state, and closing it with `Esc` or the back button. They also open the settings modal and switch its tabs, and check OPML export and import (the parser's tests read real exports from Feedly, Inoreader, NetNewsWire and Miniflux in `tests/fixtures/opml/`). When you upgrade `slint`, bump `i-slint-backend-testing` to the same version.

## 5. Install on Linux

### Option A: as a pacman package (recommended)

```bash
cd packaging/arch
makepkg -si
```

This builds from the current checkout. It runs `cargo fetch --locked`, a release build and the tests, then installs:

- `/usr/bin/aggrega`
- `/usr/share/applications/aggrega.desktop`, so Aggrega appears in your app launcher
- `/usr/share/icons/hicolor/scalable/apps/aggrega.svg`

The package version comes from the nearest tag (`git describe`): `0.9.0` on a tag, `0.9.0.r3.gabc1234` three commits after it, so pulling new commits and running `makepkg -si` again upgrades it. The binary itself reports the placeholder `0.0.0-dev`, as every local build does. To remove it, run `sudo pacman -R aggrega`.

### Option B: user-local install without pacman

```bash
make install                  # to ~/.local/bin, ~/.local/share/...
make uninstall
```

Make sure `~/.local/bin` is on your `PATH`.

### Option C: prebuilt binary from a release

Each [GitHub Release](https://github.com/moebiusmania/aggrega/releases) has `aggrega-<version>-x86_64.tar.gz`, built inside an `archlinux` container. It contains the binary, the `.desktop` file, the icon, the README and the licenses (`LICENSE`, plus `OFL.txt` for the bundled fonts). It links against Arch's current glibc, so it won't start on older distributions (Ubuntu 22.04, Debian stable): use the AppImage or the Flatpak there.

### Option D: AppImage (any distribution)

`aggrega-<version>-x86_64.AppImage` runs on most x86_64 distributions with glibc 2.35 or newer (Ubuntu 22.04, Debian 12, Fedora 36 and later), on Wayland and X11. Make it executable and run it:

```bash
chmod +x aggrega-*-x86_64.AppImage
./aggrega-*-x86_64.AppImage
```

If it complains about FUSE, install `libfuse2` (`fuse2` on Arch) or run it with `--appimage-extract-and-run`. Your data stays in the usual XDG paths ([section 8](#8-data-and-reset)), so it shares the database with the other Linux builds.

To build it yourself, on an old enough distribution (the release workflow uses Ubuntu 22.04, and the AppImage only runs on systems with that glibc or newer):

```bash
make appimage        # cargo build --release, then packaging/appimage/build.sh
```

`packaging/appimage/build.sh` downloads pinned builds of `linuxdeploy` and its AppImage plugin into `dist/appimage-tools`, builds the AppDir from `target/release/aggrega`, `packaging/aggrega.desktop` and `assets/aggrega.svg`, and writes `dist/aggrega-<version>-x86_64.AppImage`. linuxdeploy bundles the libraries the binary links against, except the ones every desktop provides and that must match the host's drivers (glibc, OpenGL, fontconfig, freetype, X11). Wayland, libxkbcommon and OpenGL are loaded from the host at run time.

### Option E: Flatpak (any distribution)

`aggrega-<version>-x86_64.flatpak` is a single-file bundle with the app ID `io.github.moebiusmania.Aggrega`, on the `org.freedesktop.Platform` 25.08 runtime, which Flatpak fetches from Flathub:

```bash
flatpak install --user aggrega-*-x86_64.flatpak
flatpak run io.github.moebiusmania.Aggrega
```

The sandbox only gets network access (feeds and LAN sync), Wayland with an X11 fallback, and the GPU. It has no access to your files: OPML import and export go through the file chooser portal, links open in your browser through the OpenURI portal, and the light/dark preference comes from the settings portal. Data lives inside the sandbox, under `~/.var/app/io.github.moebiusmania.Aggrega/` (`data/aggrega/aggrega.db` and `cache/aggrega/thumbs/`), so it doesn't share a database with the other Linux builds; sync or export/import to move between them.

To build it yourself you need `flatpak`, `flatpak-builder`, the Flathub remote (`flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo`) and `python3`:

```bash
make flatpak         # = packaging/flatpak/build.sh
```

The manifest (`packaging/flatpak/io.github.moebiusmania.Aggrega.yml`) builds offline. `packaging/flatpak/cargo-sources.sh` first generates `cargo-sources.json`, the list of crates to vendor, from `Cargo.lock` with a pinned `flatpak-cargo-generator`. The file is generated on every build and not committed, so it can't drift from `Cargo.lock`. The script then runs `flatpak-builder` and exports `dist/aggrega-<version>-x86_64.flatpak`. The AppStream metadata is in `packaging/flatpak/io.github.moebiusmania.Aggrega.metainfo.xml`. Flathub isn't set up yet; submitting there needs the manifest and a committed `cargo-sources.json` in a `flathub/io.github.moebiusmania.Aggrega` repository.

## 6. macOS

Every release also has `aggrega-<version>-macos-arm64.dmg`. It holds `Aggrega.app`, built for Apple silicon (M series) Macs running macOS 11 or newer; Intel Macs aren't supported. Open the disk image and drag `Aggrega.app` onto the `Applications` link.

### First launch (Gatekeeper)

The app isn't signed with an Apple Developer ID or notarized; it only carries an ad-hoc signature. The first time you open it, macOS says it can't verify the developer. Either:

- open **System Settings → Privacy & Security**, scroll to the message about Aggrega and click **Open Anyway**, or
- remove the quarantine flag in a terminal: `xattr -dr com.apple.quarantine /Applications/Aggrega.app`

After that it opens normally.

### Building it yourself

```bash
xcode-select --install                                        # Apple's compilers and linker
rustup target add aarch64-apple-darwin
brew install librsvg                                          # rsvg-convert, for the icon
make macos-app                                                # = packaging/macos/bundle.sh
open dist/Aggrega.app
```

`packaging/macos/bundle.sh` builds the `aarch64-apple-darwin` target, writes `Info.plist` from `packaging/macos/Info.plist`, renders `aggrega.icns` from `assets/aggrega.svg`, ad-hoc signs the bundle and packs it, with an `/Applications` link, into a compressed disk image with `hdiutil`. For day-to-day development, plain `cargo run` works on macOS too; no extra native libraries are needed.

## 7. Windows

Every release also has `aggrega-<version>-windows-x86_64.zip`, for 64-bit Windows 10 and 11. It holds `aggrega.exe`, the README and the licenses. There's no installer: unzip it wherever you like (for example `%LOCALAPPDATA%\Programs\Aggrega`) and run `aggrega.exe`, or pin it to Start or the taskbar. Your data lives elsewhere (see [section 8](#8-data-and-reset)), so replacing the folder with a newer version keeps it.

### First launch (SmartScreen)

The executable isn't code-signed, so Microsoft Defender SmartScreen may say it "prevented an unrecognized app from starting". Click **More info → Run anyway**. If Windows marked the zip as downloaded from the internet, you can also clear that before extracting: right-click the zip → **Properties** → tick **Unblock**.

### Building it yourself

Install [Rust](https://rustup.rs) with the default MSVC toolchain (`x86_64-pc-windows-msvc`); `rustup-init` offers to install the Visual Studio Build Tools ("Desktop development with C++"), which provide the linker and the Windows SDK. Then, from a terminal in the repository:

```powershell
cargo run                # dev build, with a console window for log output
cargo build --release    # target\release\aggrega.exe, no console window
```

No other native libraries are needed: SQLite is compiled in, and the window uses OpenGL through the GPU driver. `build.rs` embeds `assets/aggrega.ico` (and the version from `Cargo.toml`, `0.0.0-dev` outside a release build) into the executable via `winresource`, so Explorer and the taskbar show the app icon. Release builds use the GUI subsystem (`#![windows_subsystem = "windows"]` in `src/main.rs`), so no console window opens next to the app; debug builds keep it. The release workflow's `build-windows` job runs the same `cargo build --release` on `windows-latest` and zips the result.

## 8. Data and reset

| What | Linux | macOS | Windows |
|---|---|---|---|
| Database | `${XDG_DATA_HOME:-~/.local/share}/aggrega/aggrega.db` | `~/Library/Application Support/aggrega/aggrega.db` | `%APPDATA%\aggrega\data\aggrega.db` |
| Thumbnails | `${XDG_CACHE_HOME:-~/.cache}/aggrega/thumbs/` | `~/Library/Caches/aggrega/thumbs/` | `%LOCALAPPDATA%\aggrega\cache\thumbs\` |

On Linux, both paths honour `XDG_DATA_HOME` / `XDG_CACHE_HOME`. The Flatpak keeps its own copy inside the sandbox, under `~/.var/app/io.github.moebiusmania.Aggrega/data/aggrega/` and `.../cache/aggrega/thumbs/`.

To try Aggrega with a throwaway profile without touching your real data, point `AGGREGA_HOME` at a scratch directory. It works on every platform and takes precedence over the paths above. The database and thumbnails then live together as `$AGGREGA_HOME/aggrega.db` and `$AGGREGA_HOME/thumbs/`, and deleting the directory resets the profile:

```bash
AGGREGA_HOME=/tmp/agg cargo run
```

## 9. Troubleshooting

**The window opens under XWayland instead of native Wayland (or the reverse).**
Aggrega uses Wayland whenever `WAYLAND_DISPLAY` is set. To force X11, run:

```bash
env -u WAYLAND_DISPLAY aggrega
```

**Black window, or crashes at startup mentioning GL/EGL.**
Make sure your GPU's OpenGL driver is installed: `mesa` for AMD/Intel, or `nvidia-utils` for NVIDIA. Check with `glxinfo -B` from `mesa-utils`.

**Text looks wrong or boxes appear instead of glyphs.**
Install a basic font set, for example `sudo pacman -S noto-fonts noto-fonts-emoji`.

**A source shows a red ⓘ icon in the sidebar.**
Its last refresh failed. The site might be down, or the feed might have moved. The error is stored in the `last_error` column of the `feeds` table. Aggrega retries automatically on the next refresh.

**Clicking an article does nothing.**
Aggrega opens links with the system's default browser: `xdg-open` on Linux (check that one is set with `xdg-settings get default-web-browser`), `open` on macOS, and the shell's default handler on Windows (*Settings → Apps → Default apps*).

**Import… / Export… does nothing on Linux.**
The file dialogs go through the XDG desktop portal over D-Bus. Install `xdg-desktop-portal` plus the backend for your desktop (`xdg-desktop-portal-gtk`, `-kde`, `-gnome`…), or `zenity`, which Aggrega falls back to when no portal answers.

**Settings → Sync doesn't list the other computer.**
Both need the Sync tab open, on the same network. Discovery uses UDP broadcast on port 47811, which guest and hotel Wi-Fi often block between devices; type the address shown on the other computer's Sync tab instead. Pulling uses TCP port 47812, so a firewall must let it in on the computer you pull *from*, e.g. `sudo ufw allow 47811/udp && sudo ufw allow 47812/tcp`. On Windows, allow Aggrega through Windows Defender Firewall when it asks (on private networks). On macOS, allow Aggrega when it asks to find devices on your local network (or later under *System Settings → Privacy & Security → Local Network*) and to accept incoming connections.

**`error: rustc 1.xx is not supported`.**
Update the toolchain with `rustup update stable`. Aggrega uses Rust 2024 edition features and needs Rust 1.88 or newer.
