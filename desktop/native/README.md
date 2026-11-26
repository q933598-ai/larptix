# Larptrix Native Desktop

This is the first native desktop UI shell for Larptrix. It is a standalone Cargo workspace so the native GPU/windowing dependencies are not pulled into the VPS server or Docker build.

## Stack

- Rust
- GPUI 0.2.2 (Zed's GPU-accelerated UI framework)
- larptrix-client-core (shared application state crate)

The desktop UI is rendered natively by GPUI. It does not load the web client's HTML and does not embed Electron, Chromium, Tauri, or an OS WebView.

## Run

From the repository root:

```bash
cargo run --manifest-path desktop/native/Cargo.toml
```

GPUI's published crate enables its standard platform backends. On Linux, build and test both Wayland and X11 sessions; the intended primary path is Wayland.

## Current scope

- Native app window bootstrap.
- Three-panel messenger shell: navigation rail, conversation sidebar and chat view.
- Clickable navigation between Messages, Friends, Groups and Settings.
- Clickable sample conversations.
- Shared Rust client core is linked into the desktop binary.

The visible messages and people are placeholder preview data. The shell is not connected to a Larptrix server yet. The next integration step is to connect the shared core to the existing HTTP and WebSocket protocol, then replace the preview data with real application state.

## Notes

GPUI is pre-1.0 and its API can change. The dependency is pinned to a released 0.2.2 crate version for reproducible dependency selection. Linux prerequisites are documented by GPUI upstream.
