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
[[ ${#DOMAIN} -le 253 ]] || fail "domain is too long (maximum 253 characters)"
[[ "$DOMAIN" == *.* ]] || fail "use a public DNS hostname such as chat.example.com, not localhost"
[[ "$DOMAIN" =~ ^[A-Za-z0-9.-]+$ ]] || fail "domain must contain only hostname characters"
domain_labels=()
IFS='.' read -r -a domain_labels <<< "$DOMAIN"
for label in "${domain_labels[@]}"; do
  [[ -n "$label" && ${#label} -le 63 ]] || fail "domain labels must be 1–63 characters long"
  [[ "$label" =~ ^[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?$ ]] || fail "each domain label must start and end with a letter or number"
done

SOURCE_DIR=
cleanup() {
  if [[ -n "${SOURCE_DIR:-}" && -d "$SOURCE_DIR" ]]; then
    rm -rf -- "$SOURCE_DIR" || true
  fi
}
trap cleanup EXIT

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
case "${ID:-}" in
  debian) docker_repo_os=debian ;;
  ubuntu) docker_repo_os=ubuntu ;;
  *) fail "supported systems are Debian and Ubuntu; detected ${ID:-unknown}" ;;
esac

command -v apt-get >/dev/null 2>&1 || fail "apt-get is required"
command -v dpkg >/dev/null 2>&1 || fail "dpkg is required"
command -v systemctl >/dev/null 2>&1 || fail "systemd is required"

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends ca-certificates curl git gnupg rsync

if [[ -n "$SOURCE_REPO" ]]; then
  SOURCE_DIR=$(mktemp -d /tmp/larptrix-source.XXXXXX)
  git clone --depth 1 "$SOURCE_REPO" "$SOURCE_DIR"
  PROJECT_ROOT=$SOURCE_DIR
fi

for required in Dockerfile compose.yaml deploy/Caddyfile Cargo.lock; do
  [[ -f "$PROJECT_ROOT/$required" ]] || fail "project file not found: $required; pass the public Git repository URL as the second argument"
done

install -m 0755 -d /etc/apt/keyrings
curl -fsSL "https://download.docker.com/linux/$docker_repo_os/gpg" -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
architecture=$(dpkg --print-architecture)
codename=${VERSION_CODENAME:-}
[[ -n "$codename" ]] || fail "VERSION_CODENAME is missing from /etc/os-release"
printf 'deb [arch=%s signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/%s %s stable\n' "$architecture" "$docker_repo_os" "$codename" > /etc/apt/sources.list.d/docker.list

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
systemctl enable "$UNIT_NAME"
if systemctl is-active --quiet "$UNIT_NAME"; then
  systemctl restart "$UNIT_NAME"
else
  systemctl start "$UNIT_NAME"
fi

printf '\nLarptrix is starting for https://%s\n' "$DOMAIN"
printf 'Check status:  systemctl status %s\n' "$UNIT_NAME"
printf 'View logs:     cd %s && docker compose logs -f\n' "$APP_DIR"
printf 'Data volume:   docker volume ls | grep larptrix-data\n'
printf '\nBefore opening the site, point DNS to this VPS and allow TCP 80/443.\n'