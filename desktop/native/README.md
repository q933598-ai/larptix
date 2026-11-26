# Larptrix Native Desktop

This is the standalone native desktop client for Larptrix. It is a separate Cargo workspace, so GPUI's GPU and window-system dependencies are not pulled into the VPS server or Docker build.

## Stack

- Rust
- GPUI + gpui_platform pinned to Zed commit a0e666ae930a
- larptrix-client-core (shared application state and protocol state)
- reqwest + tokio-tungstenite (HTTP session and authenticated WebSocket)
- AES-256-GCM file encryption and native XDG/Windows/macOS file dialogs

The desktop UI is rendered natively by GPUI. It does not load the web client's HTML and does not embed Electron, Chromium, Tauri, or an OS WebView.

## Run

From the repository root:

```bash
cargo run --manifest-path desktop/native/Cargo.toml
```

Enter the server URL, your 64-character account access key, and the E2E recovery key saved when enabling encryption in the web client. They are separate credentials: the access key creates the HTTP session, while the recovery key unlocks the encrypted local Matrix store and the shared room-key backup. Neither key is written to disk as plaintext. The client then connects to `/ws` with the HttpOnly session cookie and matching `Origin` header. WebSocket reconnects use exponential backoff.

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

On Linux, test both Wayland and X11 sessions; the intended primary path is Wayland/niri. File pickers use the XDG Desktop Portal backend. Install a supported portal backend for your desktop session; Zenity is used as a fallback if the portal is unavailable. GPUI's upstream Linux build requirements are documented in the [Zed repository](https://github.com/zed-industries/zed/blob/main/script/linux).

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
- Real server directory, groups, channels, presence, and direct/group conversation history.
- Clickable group and channel entries that open the existing server-side conversation history.
- Shared Rust client core updates from incoming protocol events, including routing group messages by group ID.
- Matrix SDK 0.19.0 E2E device initialization, key upload/query/claim and to-device processing.
- Persistent Matrix SQLite crypto store protected by the E2E recovery key.
- Restore and refresh of the same encrypted Megolm room-key backup used by the web client.
- Encryption of outgoing 1:1 and group text messages before sending; decryption of incoming Matrix-encrypted messages.
- Group room-key sharing with the current group membership before sending a new group message.
- Message reactions with live counts and per-user toggle state in direct and group conversations.
- Multiple file attachments in direct and group messages using AES-256-GCM before upload.
- Native multi-file selection and decrypted Save As downloads for images, video, audio and documents.
- Attachment keys, nonces, MIME types and original filenames carried inside the Matrix-encrypted message payload.
- Download validation, ciphertext authentication, optional gzip decompression and plaintext size verification.
- Requests for missing room keys and retry of pending message decryption after key delivery.
- Shared browser/native golden tests for direct/group room IDs, message envelopes, AES-GCM ciphertext and recovery-key format.

The golden tests validate the wire contract but do not replace live Olm/Megolm testing against a running server. The native E2E implementation is experimental and has not yet completed live browser/native interoperability testing.

### Manual browser/native acceptance test

1. Sign in to the web client and unlock E2E with the account's existing recovery key.
2. Sign in to the native client with the same account and recovery key, then open a direct conversation with a second account.
3. Send one message from the browser and verify that the native client decrypts it. Send a reply from native and verify that the browser decrypts it.
4. Open a group where every member has E2E enabled, send a message from the browser, and verify that native displays it in the group conversation.
5. Send a group message from native and verify that the browser and another group member can decrypt it.
6. Send an image and a document from the native client; verify the web client can open them.
7. Send an encrypted attachment from the web client; verify the native client can decrypt and save it.
8. Restart the native client and verify that the existing room-key backup restores readable direct and group history.

Use a test account and a non-sensitive test conversation. Do not share the recovery key with anyone or paste it into chat logs. Enter the E2E recovery key created by the web client when signing in; it is not the account access key. The local Matrix store is encrypted with this recovery key. The server receives Matrix public key material, encrypted room-key backup, message ciphertext and opaque attachment bytes; attachment keys, original filenames, MIME types and sizes stay inside the Matrix-encrypted message payload.

## Not implemented yet

- Device verification / cross-signing UX and visible trust/fingerprint confirmation.
- Read receipts and richer attachment previews (native attachments currently use a decrypted Save As flow).
- Group/channel creation, membership management and richer channel controls.
- Native audio/video calls, screen sharing and notifications.
- Signed Windows installer and automatic desktop updates.

The native composer only sends a message after Matrix encryption succeeds. There is no plaintext fallback.

## Notes

The web client remains separate and keeps its HTML/CSS UI. GPUI is pre-1.0 and its API can change; the desktop depends on a pinned upstream revision so the renderer and platform backend stay compatible.
