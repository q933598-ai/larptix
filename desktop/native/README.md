# Larptrix Native Desktop

This is the standalone native desktop client for Larptrix. It is a separate Cargo workspace, so GPUI's GPU and window-system dependencies are not pulled into the VPS server or Docker build.

## Stack

- Rust
- GPUI + gpui_platform pinned to Zed commit a0e666ae930a
- larptrix-client-core (shared application state and protocol state)
- reqwest + tokio-tungstenite (HTTP session and authenticated WebSocket)

The desktop UI is rendered natively by GPUI. It does not load the web client's HTML and does not embed Electron, Chromium, Tauri, or an OS WebView.

## Run

From the repository root:

```bash
cargo run --manifest-path desktop/native/Cargo.toml
```

Enter the server URL and the 64-character access key in the native sign-in screen. The key is used to obtain the existing HttpOnly Larptrix session cookie; it is not written to disk. The client then connects to `/ws` with the session cookie and matching `Origin` header. WebSocket reconnects use exponential backoff.

You can prefill the server URL with `LARPTRIX_SERVER_URL`:

```bash
LARPTRIX_SERVER_URL=https://chat.example.com cargo run --manifest-path desktop/native/Cargo.toml
```

GPUI 0.2.2 is pinned exactly while the native shell is being established. GPUI is pre-1.0, so changes to it should be intentional and tested.

On Arch/CachyOS, install the normal Rust/C/C++ toolchain and GPUI's Linux development libraries before building. A starting package set is:

```bash
sudo pacman -S --needed base-devel cmake clang fontconfig freetype2 glib2 \
  wayland libx11 libxcb libxkbcommon libxkbcommon-x11 vulkan-icd-loader \
  libva openssl zstd pkgconf
```

On Linux, test both Wayland and X11 sessions; the intended primary path is Wayland/niri. GPUI's upstream Linux build requirements are documented in the [Zed repository](https://github.com/zed-industries/zed/blob/main/script/linux).

On Windows 10/11 x64, install the stable Rust toolchain with the MSVC target and
the Visual Studio 2022 C++ Build Tools (including the Windows 10/11 SDK). Then run
the same native client command from PowerShell:

```powershell
cargo run --manifest-path desktop/native/Cargo.toml
```

For an optimized local build:

```powershell
cargo build --release --manifest-path desktop/native/Cargo.toml
```

The executable is written to
`desktop/native/target/release/larptrix-native.exe`. Windows is a first-class
build target for this native client; CI checks and builds the Windows executable
alongside the Linux build. A signed installer and automatic update channel are
release work and are not implied by the current CI executable.

## Current scope

- Native GPUI window and three-panel messenger layout.
- Native server URL and access-key sign-in form with clipboard paste.
- Cookie-based HTTP login against the existing Larptrix API.
- Authenticated WebSocket connection and reconnect loop.
- Real server directory, groups, presence and DM history loading.
- Shared Rust client core updates from incoming protocol events.
- Matrix SDK 0.19.1 E2E device initialization, key upload/query/claim and to-device processing.
- Persistent Matrix SQLite crypto store protected by the E2E recovery key.
- Restore and refresh of the same encrypted Megolm room-key backup used by the web client.
- Encryption of outgoing 1:1 text messages before sending; decryption of incoming Matrix-encrypted 1:1 messages.
- Requests for missing room keys and retry of pending message decryption after key delivery.

The native E2E implementation is experimental and has not yet completed live browser/native interoperability testing. Enter the E2E recovery key created by the web client when signing in; it is not the account access key. The local Matrix store is encrypted with this recovery key, and the server only receives Matrix public key material, encrypted room-key backup and opaque message ciphertext.

## Not implemented yet

- Device verification / cross-signing UX and visible trust/fingerprint confirmation.
- Encrypted attachments, reactions and read receipts.
- Group message composition, group E2E and full conversation UI.
- Native audio/video calls, screen sharing and notifications.
- Signed Windows installer and automatic desktop updates.

The native composer only sends a message after Matrix encryption succeeds. There is no plaintext fallback.

## Notes

The web client remains separate and keeps its HTML/CSS UI. GPUI is pre-1.0 and its API can change; the desktop depends on a pinned upstream revision so the renderer and platform backend stay compatible.
