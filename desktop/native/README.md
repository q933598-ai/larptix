# Larptrix Native Desktop

This is the standalone native desktop client for Larptrix. It is a separate Cargo workspace, so GPUI's GPU and window-system dependencies are not pulled into the VPS server or Docker build.

## Stack

- Rust
- GPUI 0.2.2 (Zed's GPU-accelerated UI framework)
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

## Current scope

- Native GPUI window and three-panel messenger layout.
- Native server URL and access-key sign-in form with clipboard paste.
- Cookie-based HTTP login against the existing Larptrix API.
- Authenticated WebSocket connection and reconnect loop.
- Real server directory, groups, presence and DM history loading.
- Shared Rust client core updates from incoming protocol events.

## Not implemented yet

- Native Olm/Matrix E2E decryption and encrypted sending.
- Message composition, uploads, attachments, reactions and read receipts.
- Native audio/video calls, screen sharing and notifications.

For that reason encrypted message bodies are intentionally shown as an E2E placeholder, and the composer is disabled. The native client does not send plaintext messages.

## Notes

The web client remains separate and keeps its HTML/CSS UI. GPUI is pre-1.0 and its API can change; the dependency is pinned to a released version for reproducible dependency selection.
