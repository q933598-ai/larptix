# Larptrix 0.4

Self-hostable messenger for a small group of friends.
This slice adds **accounts**, **1:1 chats**, **history**, **profiles**, and **attachments**.

Every new chat message requires end-to-end encryption. On first sign-in, the client
creates an E2E device and asks you to save a separate recovery key. Enabling E2E for
an existing account preserves its existing DM history; older messages remain
unencrypted and are marked as legacy in the client. Without the recovery key,
encrypted history cannot be unlocked on another device.

## What changed from 0.1

0.1 was one shared room and a display name in `localStorage`.
0.2 adds cookie sessions and private threads. New accounts use a generated access key
instead of an email address or password.

Photos were not in the original 0.2 list; they are included here because we wanted them.

## Architecture

```
browser --HTTP-->  /api/register, /api/login, /api/upload, static files
browser --WS /ws-->  Axum (cookie required) --SQLite--> users, sessions, DMs
                         |                    --files--> data/uploads
                    Hub (who is online)
```

WebSocket upgrades require a matching `Origin` host and scheme. A TLS-terminating
reverse proxy must overwrite `X-Forwarded-Proto` with the external scheme.

A message is no longer “post to the room”. It is “deliver to this user”.
The Hub sends only to the two participants. History is loaded with `open`.
Message bodies use Olm E2E encryption. File bytes, including GIFs, are encrypted in
the browser with AES-GCM; their decryption key and original metadata travel inside
the encrypted message. The server stores opaque attachment ciphertext. Groups use
an individual Olm ciphertext for each member. Usernames and profile descriptions
are public. Chat backgrounds are compressed and stored locally in the browser,
separately for each chat. Accounts can be created with either an access key or an
email/password pair.

New account keys have 64 hexadecimal characters and are shown only once. Only a
SHA-256 hash is stored on the server. Keep the key in a password manager or another
secure place: losing it means losing access to that account. The key is a login
credential, not proof that an email address belongs to you. Existing email/password
accounts remain available through the legacy login option. After signing in, an old
account can create a key from its profile; its existing password remains available
as a fallback. Sessions are random tokens in an HttpOnly cookie.
Attachments are stored on disk, not in SQLite; the DB keeps metadata. Images and
supported audio formats are detected from their bytes; other files are served as
downloads. Uploads are limited to 25 MiB.

## Run

Run this from the repository root:

```bash
cargo run -p larptrix-server
```

Open http://127.0.0.1:8080 — two browser profiles, two accounts.

## Host on a VPS

The VPS runs the server for everyone; users open the same HTTPS hostname from
their browsers or desktop clients. Do not use the temporary `/tmp` demo database
for a real deployment.

### Automated Debian Install

The project is published at
https://github.com/q933598-ai/larptix. The repository must remain public so the
VPS can download the installer and source without GitHub credentials.

Point a DNS `A` record such as `chat.example.com` to the VPS public IPv4 address.
Allow inbound TCP ports 80 and 443 in the VPS and provider firewalls. Then run this
single command from your computer, replacing `VPS_IP` and `chat.example.com` with
yours:

```bash
ssh admin@VPS_IP 'sudo bash -s -- chat.example.com https://github.com/q933598-ai/larptix.git' \
     < <(curl -fsSL https://raw.githubusercontent.com/q933598-ai/larptix/main/deploy/install-debian.sh)
```

Or log into the VPS first and run:

```bash
curl -fsSL https://raw.githubusercontent.com/q933598-ai/larptix/main/deploy/install-debian.sh \
     | sudo bash -s -- chat.example.com https://github.com/q933598-ai/larptix.git
```

The script installs Docker Engine and Compose from Docker's Debian repository,
clones the project, copies it to `/opt/larptrix`, builds the server image,
configures Caddy for automatic HTTPS, and installs/enables the systemd unit
`larptrix.service`. Visit
`https://chat.example.com` after Caddy obtains its certificate and create user
accounts. Every user should save their E2E recovery key during first sign-in.

Manage the running service with:

```bash
sudo systemctl status larptrix
sudo systemctl restart larptrix
sudo journalctl -u larptrix -f
cd /opt/larptrix && sudo docker compose logs -f
```

To deploy a newer source version, copy it to the VPS again and run
`sudo systemctl restart larptrix`; the unit rebuilds the image before starting.
The named Docker volume `larptrix_larptrix-data` keeps SQLite and uploaded files
across restarts/rebuilds. Back it up regularly and store backups off the VPS.

GIF search uses `LARPTRIX_KLIPY_API_KEY` on the server. The API key is never exposed to browsers. Selected GIF metadata is included in the existing E2E-encrypted message payload, while KLIPY media is loaded directly from KLIPY's static media URL.

For calls between users behind different NATs, configure reachable STUN/TURN in
`LARPTRIX_ICE_SERVERS`. Without TURN, messaging works through HTTPS/WebSocket, but
some peer-to-peer calls may not connect. TURN credentials should be short-lived;
do not put a permanent shared secret in this browser-readable setting.

## Desktop clients

`desktop/` contains a Linux desktop client using Electron's Chromium runtime, so
WebRTC calls work inside the app window. It connects to an existing Larptrix server;
the server and desktop window are separate processes. Start the server with
`cargo run -p larptrix-server` from the repository root, then launch the desktop
client. The desktop window loads its UI and WebAssembly crypto bundle from that
running server, so after updating the client or crypto code, rebuild and restart
the server before reopening the desktop app:

```bash
cd desktop
npm install
npm run dev
```

To build the Linux AppImage, run `npm run build` from `desktop/`.
Electron packages Chromium with the app. To build a portable Windows x64 package,
run `npm run build:windows`; extract `dist/Larptrix-0.4.0-win-x64.zip` and launch
`Larptrix.exe`. The Windows ZIP can be built from Linux without Wine. Tauri remains
available as the optional `npm run dev:tauri` / `npm run build:tauri` WebKitGTK
variant. On Arch-based systems install the media libraries needed for desktop
capture and playback:

To build the portable Windows x64 ZIP from Linux or Windows, run `npm run build:windows`
from `desktop/`. Extract the ZIP and run `larptrix-linux.exe`.

```bash
sudo pacman -S --needed webkit2gtk-4.1 gst-plugins-base gst-plugins-good gst-plugins-bad libnice base-devel curl file openssl librsvg
```

`gst-plugins-good` provides the `autoaudiosink` element reported by WebKit. On Wayland
the optional Tauri client disables WebKit's DMA-BUF renderer to avoid a compositor
protocol error; GTK still uses the native Wayland backend. Set
`LARPTRIX_SERVER_URL` to connect to another server; it defaults to
`http://127.0.0.1:8080`.

### Environment

| Variable | Default | Meaning |
| --- | --- | --- |
| `LARPTRIX_BIND` | `127.0.0.1:8080` | Listen address |
| `LARPTRIX_DB` | `data/larptrix.db` | SQLite path |
| `LARPTRIX_CLIENT` | `client` | Static files |
| `LARPTRIX_UPLOADS` | `data/uploads` | Stored attachments |
| `LARPTRIX_KLIPY_API_KEY` | unset | Server-side KLIPY GIF search key; required for GIF search |

## Protocol

HTTP: register, login, logout, me/profile, upload, avatar, attachments  
WS client: `open`, `send`  
WS server: `welcome`, `directory`, `chat`, `message`, `call_signal`, `error`

### Calls

1:1 audio/video calls and screen sharing use browser WebRTC with ephemeral signaling
relayed over the authenticated WebSocket. Signaling payloads are not stored. The
server uses Google's public STUN server by default. For restrictive NATs or networks
that block direct UDP, set `LARPTRIX_ICE_SERVERS` to a JSON array containing your own
STUN/TURN service, for example `[{"urls":"turn:turn.example.net:3478","username":"...","credential":"..."}]`.
TURN credentials are delivered to authenticated clients, so use short-lived credentials
and never put a permanent shared TURN secret in this setting.

WebRTC provides DTLS-SRTP transport encryption, but calls are not yet protected by
Larptrix device-verified E2E keys. A server that controls signaling could potentially
mediate a connection. Authenticated media transforms remain future work. Screen
sharing starts adaptively (up to 1080p/30); the high preset requests up
to 4K/144 fps, but the browser, device, and network may negotiate less.

