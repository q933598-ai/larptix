# Larptrix Rust Rewrite

## Goal

Rebuild Larptrix as two clients over one Rust application core:
- a native desktop application with a Rust-rendered UI (GPUI is the leading candidate);
- a separate web client that keeps HTML/CSS and loads the shared Rust core as WebAssembly.

The desktop target must not be an HTML page in Tauri or Electron. HTML is retained for the web version only. The backend is already Rust and should be optimized rather than rewritten for the sake of changing languages.

The aim is lower startup time, lower memory use, smoother interaction, and a smaller attack surface. Rust does not automatically make every workload faster; improvements must be measured.

## Current architecture (main)

- Server: Rust, Axum, Tokio, rusqlite, SQLite.
- Shared wire protocol: crates/protocol.
- E2E implementation: Rust/WASM based on vodozemac, plus the Matrix crypto WASM package used by the client.
- Web UI: HTML/CSS plus a large JavaScript application (client/app.js is approximately 9,000 lines) and support scripts.
- Desktop: Electron is the default package; Tauri is an optional WebKit-based variant. Both are transitional, not the target.
- Browser-owned capabilities: DOM, WebRTC, camera/microphone, screen capture, browser notifications, file picker and browser media playback.

## Target architecture

    ┌─────────────────────────────────────────────────────┐
    │                 Shared Rust Core                    │
    │ state · domain logic · protocol · crypto · sync      │
    │ message workflows · validation · tests              │
    └─────────────────────┬───────────────────────────────┘
                          │
             ┌────────────┴────────────┐
             │                         │
    ┌────────▼────────────┐  ┌─────────▼──────────────────┐
    │ Native Desktop      │  │ Web Client                 │
    │ Rust + GPUI         │  │ HTML/CSS                    │
    │ own GPU UI renderer │  │ Rust Core compiled to WASM  │
    │ native OS adapters  │  │ thin browser JS adapter    │
    └────────┬────────────┘  └─────────┬──────────────────┘
             │                         │
             └───────────┬─────────────┘
                         │ HTTPS / WebSocket
                 ┌───────▼─────────┐
                 │ Rust Backend   │
                 │ Axum + Tokio   │
                 │ SQLite + files │
                 └─────────────────┘

### Shared Rust Core

The core is an ordinary Rust library, independent of UI rendering. It owns application state, message and group workflows, typed HTTP/WebSocket protocol handling, synchronization, local domain rules, and shared tests. It is compiled natively for desktop and to WebAssembly for the web client.

The initial implementation lives in crates/client-core. Keep platform-specific dependencies out of this crate unless they are behind narrow traits or target-specific modules.

### Native Desktop

The desktop app is a standalone native Rust program. GPUI is the preferred UI framework because it draws the interface itself rather than embedding Chromium or an operating-system WebView. It is the closest architectural match to Discord's announced Phoenix approach.

- Rebuild the complete Larptrix interface as GPUI views: workspace/server navigation, friends and conversations, chat timeline, message composer, profile/settings, media viewer and call windows.
- Do not load index.html, use Tauri's WebView, or bundle Electron in the final desktop application.
- Connect views to the shared Rust core through typed state/actions. Keep business logic out of view components.
- Add native adapters for notifications, tray/menu, file picker, audio devices, camera, screen capture, clipboard, deep links and OS integration.
- Target Linux (Wayland and X11), Windows and macOS. Test Wayland/niri explicitly.
- Pin GPUI to a known revision/version and isolate it in the desktop UI crate: it is evolving and may introduce breaking changes.
- Keep Electron/Tauri as temporary fallback packages only until the native app reaches feature parity, then remove them.

GPUI is not the same as Tauri: Tauri normally uses a WebView for the UI; GPUI has its own native rendering and windowing path.

### Web Client

Keep the existing HTML/CSS version as a separate product surface. The web client loads the shared Rust core compiled to WASM. JavaScript should shrink to a narrow adapter for browser-specific APIs such as DOM access, WebRTC, media capture, notifications, and browser storage where Rust bindings are impractical. Chat business logic and application state should not be duplicated in JavaScript.

### Backend

The server is already Rust. Preserve the external HTTP and WebSocket protocol while optimizing its internals. Native desktop and web clients must continue to connect to the same self-hosted server.

## Migration rules

1. Do not replace the existing client with one big-bang rewrite.
2. Keep the current web client and desktop packages usable as fallbacks until parity tests pass.
3. Keep the JSON HTTP and WebSocket wire contract stable during the first migration.
4. Reuse larptrix-protocol instead of defining parallel message types.
5. Preserve existing SQLite data and migration history. Never recreate or reset production databases as part of the rewrite.
6. Preserve access keys, sessions, Matrix device state, Olm sessions, recovery keys, legacy encrypted messages and attachment ciphertext.
7. Do not change cryptographic algorithms or serialization formats as part of a language migration. Crypto changes require a separate compatibility and security review.
8. Do not make Electron or Tauri the target desktop architecture; they are transitional fallbacks only.
9. Every migrated feature needs tests for success, errors, reconnect behavior and legacy data where relevant.
10. Measure before and after. Do not claim a performance improvement without repeatable measurements.

## Phases

### 0. Baseline and compatibility contract

- Record current commit, server build, client bundle sizes, desktop startup time, API latency, WebSocket reconnect behavior, memory use and SQLite throughput.
- Add protocol fixtures for every HTTP and WebSocket message used by the current client.
- Add smoke tests for login, 1:1 messages, history, groups, channels, files, reactions, presence, device sync, E2E and calls.
- Capture the existing DB schema and create test fixtures from a sanitized DB copy.

Exit criteria: baseline measurements and repeatable compatibility tests exist before changing runtime behavior.

### 1. Server runtime and I/O

The server remains Rust. Optimize its current architecture rather than rewriting it for language parity.

- Replace synchronous SQLite work on async request paths with a well-defined async database boundary. Evaluate SQLx SQLite pooling or an actor-based SQLite connection; choose based on transaction semantics and migration compatibility.
- Preserve WAL, foreign keys, busy timeout and all schema migrations.
- Replace unbounded per-connection WebSocket queues with bounded queues and an explicit slow-consumer policy.
- Avoid holding global Hub/presence locks while doing unrelated work.
- Move filesystem uploads/reads off Tokio executor threads.
- Add backpressure, limits, timeouts, graceful shutdown and metrics for queue depth and DB wait time.
- Retain the current API and WebSocket JSON format.

Exit criteria: existing tests pass; migration tests run against a copy of the current schema; load tests show stable memory with slow WebSocket clients; no compatibility regression.

### 2. Rust client foundation

- Reuse the current protocol crate and types.
- Add typed HTTP and WebSocket client modules with reconnect handling.
- Define explicit application state, actions, reducers and errors.
- Add deterministic tests for message ordering, duplicate delivery, history merging, presence, group state and reconnect.
- Keep rendering and platform APIs behind narrow interfaces.

Exit criteria: core builds for native and wasm32-unknown-unknown; native tests pass; a browser test loads the WASM module and connects to the current server.

### 3. Port client features incrementally

Recommended order:

1. Authentication, session bootstrap, profile and settings.
2. Directory, friends, groups, channels and presence.
3. WebSocket lifecycle, chat history, send/delete/react/view and unread state.
4. Attachments, uploads, previews, downloads and local saved items.
5. E2E device lifecycle, key backup/recovery, verification and resync.
6. Voice/video calls, group-call signaling, screen sharing, device selection and media controls.
7. Notifications, sounds, themes, music/activity integrations and remaining workflows.

For each feature, compare the Rust implementation against the existing implementation using identical protocol fixtures before switching the default.

Exit criteria: web and desktop expose the complete current feature set; JavaScript no longer owns chat business logic.

### 4. Native GPUI desktop UI

- Create a dedicated native desktop crate, separate from crates/client-core.
- Build the native window, navigation, chat list, message timeline, composer, dialogs, settings, media and call surfaces as GPUI components.
- Add native adapters for operating-system integration and media capture.
- Validate Linux Wayland/X11, Windows and macOS builds, including GPU fallbacks and text rendering.
- Compare startup time, idle/active RAM, scrolling, message rendering and input latency against Electron and Tauri builds.
- Keep fallback clients available until feature parity and release checks pass.

Exit criteria: desktop operates without an embedded browser UI and passes the full functional test matrix.

### 5. Remove legacy desktop and web logic

- Remove Electron/Tauri desktop packaging only after the GPUI desktop has feature parity.
- Remove old JavaScript application-logic modules only after web parity tests pass.
- Keep the HTML/CSS web interface; do not remove HTML from the web product.
- Keep only minimal browser glue required by Web APIs.
- Document rollback and database backup/restore procedures.

## Initial performance risks already visible in main

- crates/server/src/db.rs wraps one synchronous rusqlite::Connection in std::sync::Mutex. Calls serialize on one connection, and synchronous DB work can block async executor threads.
- crates/server/src/hub.rs and the WebSocket handler use unbounded Tokio channels. A slow consumer can accumulate queued messages.
- Upload helpers use synchronous filesystem calls. These should not block async request workers.
- client/app.js is large and difficult to profile, but line count alone does not prove it is the main performance bottleneck.

These are audit findings, not proof that every deployment is slow. Baseline tests and measurements determine priorities.

## Definition of done

- All current user-visible features are preserved across desktop and web.
- Existing accounts, sessions, messages, files and encrypted device state remain readable.
- The native desktop client runs without Chromium, Electron, Tauri or an embedded WebView for its main UI.
- The separate web client continues to work with HTML/CSS and the shared WASM core.
- The Rust core and server have automated unit, integration, protocol and load tests.
- No unbounded per-connection message queues remain.
- Server DB work does not block Tokio runtime workers.
- Benchmarks document p50/p95 latency, throughput, memory, startup time and resource use before/after.
