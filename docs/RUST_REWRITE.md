# Larptrix Rust Rewrite

## Goal

Move Larptrix application logic to Rust without losing the current web, desktop, messaging, E2E, group, attachment, and calling features. The migration is incremental: the existing client remains runnable until the Rust implementation reaches feature parity.

The target is not "Rust at any cost". Every change must improve measured latency, memory use, reliability, maintainability, or security. A Rust/WASM rewrite is not automatically faster than JavaScript for DOM work, browser media, or network-bound operations.

## Current architecture (main)

- Server: Rust, Axum, Tokio, rusqlite, SQLite.
- Shared wire protocol: crates/protocol.
- E2E implementation: Rust/WASM based on vodozemac, plus the Matrix crypto WASM package used by the client.
- Web client: HTML/CSS and a large JavaScript application (client/app.js is approximately 9,000 lines), plus small supporting scripts.
- Desktop: Electron is the default package; Tauri is an optional WebKit-based variant.
- Browser-owned capabilities: DOM, WebSocket, camera/microphone, WebRTC, screen capture, notifications, file picker and media playback.

The server is already Rust. The largest language migration is the client, not the server.

## Target architecture

    ┌────────────────────────────────────────────┐
    │                 Rust core                  │
    │ app state · reducers · domain logic         │
    │ API · WebSocket · validation · workflows    │
    │ shared protocol · tests                     │
    └──────────────────────┬─────────────────────┘
                           │ wasm-bindgen
    ┌──────────────────────▼─────────────────────┐
    │          Browser platform adapter          │
    │ DOM events · WebRTC · media · notifications│
    │ storage · file picker · downloads           │
    └──────────────────────┬─────────────────────┘
                           │
                     Existing HTML + CSS UI

The Rust client core should compile as both a native Rust library (for tests and possible desktop reuse) and WebAssembly (for the browser). Browser-owned APIs can be called through web-sys where practical. A minimal JavaScript bridge is acceptable for APIs that are awkward or incomplete in the Rust bindings, especially WebRTC and Matrix SDK interop. It must not contain chat business logic.

The current HTML/CSS should be kept initially. Rebuilding the design system at the same time as moving the application logic would multiply risk and make visual regressions hard to isolate.

## Migration rules

1. Do not replace the entire client in one big-bang commit.
2. Keep the current JavaScript client as the fallback until each Rust module passes parity tests.
3. Keep the JSON HTTP and WebSocket wire contract stable during the first migration.
4. Reuse larptrix-protocol instead of defining parallel message types.
5. Preserve existing SQLite data and migration history. Do not recreate or reset production databases.
6. Preserve account keys, sessions, Matrix device state, Olm sessions, recovery keys, old encrypted messages, and attachment ciphertext.
7. Do not change cryptographic algorithms or serialization formats as part of a language migration. Crypto changes require their own compatibility and security review.
8. Do not remove Electron until Tauri has passed camera, microphone, WebRTC, screen sharing, notifications, packaging, and update tests on supported platforms.
9. Every migrated feature needs tests for success, errors, reconnection, and legacy data where relevant.
10. Measure before and after. Do not claim a performance improvement without repeatable measurements.

## Phases

### 0. Baseline and compatibility contract

- Record current commit, server build, client bundle sizes, startup time, API latency, WebSocket reconnect behavior, memory use, and SQLite throughput.
- Add protocol fixtures for every HTTP and WebSocket message currently used by the client.
- Add smoke tests for login, 1:1 messages, history, groups, files, reactions, presence, device sync, E2E, and calls.
- Capture current DB schema and test fixtures using a copy of a real database with secrets removed.

Exit criteria: baseline measurements and repeatable compatibility tests exist before changing runtime behavior.

### 1. Server runtime and I/O

The server remains Rust. Optimize its current architecture rather than rewriting it for language parity.

- Replace synchronous SQLite work on async request paths with a well-defined async database boundary. Evaluate SQLx SQLite pooling or an actor-based SQLite connection; choose based on transaction semantics and migration compatibility.
- Preserve WAL, foreign keys, busy timeout, and all existing schema migrations.
- Replace unbounded per-connection WebSocket queues with bounded queues and an explicit slow-consumer policy.
- Avoid holding global Hub/presence locks while doing work that can be performed on snapshots.
- Move filesystem uploads/reads off Tokio executor threads.
- Add limits, backpressure, timeouts, graceful shutdown, and metrics for queue depth and DB wait time.
- Retain the current API and WebSocket JSON format until client migration is complete.

Exit criteria: all existing server tests pass; DB migration tests run against a copy of the current schema; load tests show stable memory under slow WebSocket clients; no compatibility regression.

### 2. Rust client foundation

Create a workspace crate for the client core with native and WASM targets.

- Reuse the protocol crate and its types.
- Define explicit application state, actions, reducers, and error types.
- Implement typed HTTP API calls and WebSocket connection/reconnection handling.
- Add deterministic unit tests for message ordering, duplicate delivery, history merging, presence, group state, and disconnect/reconnect.
- Keep rendering and browser capability access behind narrow interfaces.

Exit criteria: the Rust core builds for native and wasm32-unknown-unknown; tests pass in native CI; a browser test can initialize the WASM module and connect to the existing server.

### 3. Port client features incrementally

Recommended order:

1. Authentication, session bootstrap, profile and settings.
2. Directory, friends, groups, channels and presence.
3. WebSocket lifecycle, chat history, send/edit/delete/react/view and unread state.
4. Attachments, uploads, previews, downloads and local saved items.
5. E2E device lifecycle, key backup/recovery, verification and resync.
6. Voice/video calls, group-call signaling, screen sharing, device selection and media controls.
7. Notifications, sounds, themes, music/activity integrations and remaining UI workflows.

For each feature, the Rust implementation is enabled behind a feature flag or route. Compare it with the existing implementation using the same protocol fixtures before switching the default.

Exit criteria: the complete existing feature set works with the Rust core; the JavaScript implementation is no longer needed for application logic.

### 4. Desktop consolidation

- Keep Electron as the compatibility baseline during the web-client migration.
- Test Tauri on Linux, Windows and Android separately. In particular validate WebRTC, capture, codecs, notifications, device selection, tray behavior and upgrades.
- Move desktop-only application logic to Rust commands/plugins.
- Only then make Tauri the default and remove Electron if the feature and platform matrix is satisfied.

### 5. Remove legacy implementation

- Remove old JavaScript business-logic modules only after parity, regression, and release tests pass.
- Keep only the smallest necessary browser bridge, if one is still required.
- Document rollback and database backup/restore procedures.

## Initial performance risks already visible in main

- crates/server/src/db.rs wraps one synchronous rusqlite::Connection in std::sync::Mutex. Calls serialize on one connection and synchronous DB work can block async executor threads.
- crates/server/src/hub.rs and the WebSocket handler use unbounded Tokio channels. A slow or disconnected consumer can accumulate queued messages until it is cleaned up.
- Upload helpers use synchronous filesystem calls. These should not block async request workers.
- The large client entry point is difficult to profile and maintain, but its line count alone does not prove it is the main performance bottleneck.

These are audit findings, not proof that every current deployment is slow. Baseline tests and measurements determine the priority and the size of each optimization.

## Definition of done

- All existing user-visible features are preserved.
- Existing accounts, sessions, messages, files, and encrypted device state remain readable.
- Browser and desktop builds work on the documented platforms.
- The Rust client core and server have automated unit, integration, protocol, and load tests.
- No unbounded per-connection message queues remain.
- Server DB work does not block Tokio runtime workers.
- Benchmarks document p50/p95 latency, throughput, memory, startup time, and resource use before/after.
- The legacy JavaScript application logic can be removed without breaking the browser or desktop client.
