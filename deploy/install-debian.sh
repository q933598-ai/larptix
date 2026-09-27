#!/usr/bin/env bash
set -Eeuo pipefail

APP_DIR=/opt/larptrix
UNIT_NAME=larptrix.service

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

if [[ ${EUID} -ne 0 ]]; then
  fail "run with sudo: sudo ./deploy/install-debian.sh chat.example.com [git-repository-url]"
fi

DOMAIN=${1:-}
SOURCE_REPO=${2:-${LARPTRIX_REPO:-}}
[[ -n "$DOMAIN" ]] || fail "usage: $0 <domain> [git-repository-url]"
[[ "$DOMAIN" =~ ^[A-Za-z0-9]([A-Za-z0-9.-]*[A-Za-z0-9])?$ ]] \
  || fail "domain must contain only hostname characters"
[[ "$DOMAIN" != *..* ]] || fail "domain cannot contain consecutive dots"

SCRIPT_PATH=${BASH_SOURCE[0]:-}
if [[ -n "$SCRIPT_PATH" && -f "$SCRIPT_PATH" ]]; then
  SCRIPT_DIR=$(cd -- "$(dirname -- "$SCRIPT_PATH")" && pwd)
  PROJECT_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)
else
  PROJECT_ROOT=$PWD
fi

if [[ ! -r /etc/os-release ]]; then
  fail "cannot identify Linux distribution"
fi
# shellcheck disable=SC1091
source /etc/os-release
[[ ${ID:-} == debian || ${ID_LIKE:-} == *debian* ]] \
  || fail "this installer supports Debian and Debian-derived distributions"

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends ca-certificates curl git gnupg rsync

if [[ -n "$SOURCE_REPO" ]]; then
  SOURCE_DIR="/tmp/larptrix-source-$$"
  git clone --depth 1 "$SOURCE_REPO" "$SOURCE_DIR"
  PROJECT_ROOT=$SOURCE_DIR
fi

for required in Dockerfile compose.yaml deploy/Caddyfile Cargo.lock; do
  [[ -f "$PROJECT_ROOT/$required" ]] || fail "project file not found: $required; pass the public Git repository URL as the second argument"
done

install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
architecture=$(dpkg --print-architecture)
codename=${VERSION_CODENAME:-}
[[ -n "$codename" ]] || fail "VERSION_CODENAME is missing from /etc/os-release"
printf 'deb [arch=%s signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian %s stable\n' \
  "$architecture" "$codename" > /etc/apt/sources.list.d/docker.list

apt-get update
apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
systemctl enable --now docker.service

install -d -m 0755 "$APP_DIR"
rsync -a \
  --exclude='.git/' \
  --exclude='.env' \
  --exclude='target/' \
  --exclude='desktop/node_modules/' \
  --exclude='desktop/src-tauri/target/' \
  --exclude='data/' \
  "$PROJECT_ROOT/" "$APP_DIR/"

if [[ ! -f "$APP_DIR/.env" ]]; then
  printf 'DOMAIN=%s\n' "$DOMAIN" > "$APP_DIR/.env"
elif grep -q '^DOMAIN=' "$APP_DIR/.env"; then
  sed -i "s/^DOMAIN=.*/DOMAIN=$DOMAIN/" "$APP_DIR/.env"
else
  printf '\nDOMAIN=%s\n' "$DOMAIN" >> "$APP_DIR/.env"
fi
chmod 0600 "$APP_DIR/.env"

cat > "/etc/systemd/system/$UNIT_NAME" <<'UNIT'
[Unit]
Description=Larptrix messenger stack
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/larptrix
ExecStartPre=/usr/bin/docker compose config --quiet
ExecStart=/usr/bin/docker compose up --build --detach --remove-orphans
ExecStop=/usr/bin/docker compose down
TimeoutStartSec=0
TimeoutStopSec=120

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable --now "$UNIT_NAME"

printf '\nLarptrix is starting for https://%s\n' "$DOMAIN"
printf 'Check status:  systemctl status %s\n' "$UNIT_NAME"
printf 'View logs:     cd %s && docker compose logs -f\n' "$APP_DIR"
printf 'Data volume:   docker volume ls | grep larptrix-data\n'
printf '\nBefore opening the site, point DNS to this VPS and allow TCP 80/443.\n'