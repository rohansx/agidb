#!/usr/bin/env sh
# agidb install script (mirrors the get.docker.com / get.rustup shape)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rohansx/agidb/master/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/rohansx/agidb/master/install.sh | sh -s -- --tag v0.1.0-dev.1
#   curl -fsSL ... | sh -s -- --to ~/.local/bin
#
# What it does:
#   1. detect OS + arch
#   2. fetch the matching binary from the GitHub release tagged --tag
#      (default: latest)
#   3. verify its sha256 against the checksums.txt published in the
#      same release
#   4. drop the binary at $INSTALL_DIR/agidb (default /usr/local/bin,
#      or ~/.local/bin if we can't write there)
#
# Exits non-zero on any failure with a clear message. Network is the
# only external dependency.

set -eu

# ─── defaults ─────────────────────────────────────────────────────
REPO="${AGIDB_REPO:-rohansx/agidb}"
TAG="${AGIDB_TAG:-latest}"
INSTALL_DIR=""
BIN_NAME="agidb"
BIN_EXT=""

# ─── tiny arg parser (no getopts, no deps) ────────────────────────
while [ "$#" -gt 0 ]; do
  case "$1" in
    --tag)   TAG="$2"; shift 2 ;;
    --tag=*) TAG="${1#*=}"; shift ;;
    --repo)  REPO="$2"; shift 2 ;;
    --repo=*) REPO="${1#*=}"; shift ;;
    --to)    INSTALL_DIR="$2"; shift 2 ;;
    --to=*)  INSTALL_DIR="${1#*=}"; shift ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "agidb-install: unknown arg: $1" >&2; exit 64 ;;
  esac
done

# ─── logging helpers (no colors if not on a tty) ──────────────────
if [ -t 1 ]; then
  C_RESET=$'\033[0m'; C_DIM=$'\033[2m'; C_BOLD=$'\033[1m'; C_SAFF=$'\033[38;5;72m'; C_RED=$'\033[31m'
else
  C_RESET=""; C_DIM=""; C_BOLD=""; C_SAFF=""; C_RED=""
fi

log()  { printf '%s==>%s %s\n' "$C_SAFF" "$C_RESET" "$*"; }
warn() { printf '%s!! %s%s\n' "$C_RED" "$*" "$C_RESET" >&2; }
die()  { warn "$@"; exit 1; }

# ─── required tools ───────────────────────────────────────────────
need() { command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1"; }
need curl
pick_sha() {
  if   command -v sha256sum >/dev/null 2>&1; then echo sha256sum
  elif command -v shasum     >/dev/null 2>&1; then echo "shasum -a 256"
  else die "neither sha256sum nor shasum found"
  fi
}
SHACMD=$(pick_sha)
need tar

# ─── detect OS / arch ─────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux*)   echo linux ;;
    Darwin*)  echo darwin ;;
    MINGW*|MSYS*|CYGWIN*) echo windows ;;
    *) die "unsupported OS: $(uname -s)" ;;
  esac
}
detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)   echo x86_64 ;;
    aarch64|arm64)  echo aarch64 ;;
    *) die "unsupported arch: $(uname -m)" ;;
  esac
}

OS=$(detect_os)
ARCH=$(detect_arch)
log "detected ${OS}/${ARCH}"

# windows binary extension (handled separately later if we ever ship
# powershell install path; for now POSIX script fails fast above).
[ "$OS" = "windows" ] && BIN_EXT=".exe"

# ─── resolve tag → version string ─────────────────────────────────
if [ "$TAG" = "latest" ]; then
  log "resolving latest release from ${REPO} …"
  LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
  # GitHub returns the tag_name in JSON. No jq dependency — use sed.
  TAG=$(curl -fsSL -H 'Accept: application/vnd.github+json' \
        "$LATEST_URL" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  [ -n "$TAG" ] || die "could not resolve latest release tag from $LATEST_URL"
  log "latest is ${TAG}"
fi

# ─── compose asset name ──────────────────────────────────────────
# Convention (matches release.yml):
#   agidb-<tag>-<os>-<arch>.tar.gz
#   agidb-<tag>-<os>-<arch>.tar.gz.sha256
#   checksums.txt        (all sha256 lines)
ASSET="agidb-${TAG}-${OS}-${ARCH}.tar.gz"
BASE="https://github.com/${REPO}/releases/download/${TAG}"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT INT TERM

log "downloading ${ASSET} …"
curl -fsSL -o "${WORK}/${ASSET}" "${BASE}/${ASSET}" \
  || die "download failed: ${BASE}/${ASSET}"

log "downloading checksums.txt …"
curl -fsSL -o "${WORK}/checksums.txt" "${BASE}/checksums.txt" \
  || die "checksums download failed"

log "verifying sha256 …"
# checksums.txt format is "<sha>   <basename>" (optionally with a
# path prefix from the workflow's `find`). Compare on basename only.
EXPECTED=$(awk -v a="${ASSET}" '
  {
    # field 2 may be "agidb-...", "./agidb-...", or
    # "artifacts/agidb-...". Take the last path component.
    n = split($2, parts, "/")
    base = parts[n]
    if (base == a) print $1
  }
' "${WORK}/checksums.txt")
[ -n "$EXPECTED" ] || die "no checksum found for ${ASSET} in checksums.txt"

cd "${WORK}"
echo "${EXPECTED}  ${ASSET}" > check.sha256
$SHACMD -c check.sha256

# ─── unpack ───────────────────────────────────────────────────────
log "extracting …"
tar -xzf "${WORK}/${ASSET}" -C "${WORK}"

# tarball contains a single file: agidb (or agidb.exe)
BIN_PATH="${WORK}/${BIN_NAME}${BIN_EXT}"
[ -f "$BIN_PATH" ] || die "expected ${BIN_NAME}${BIN_EXT} in archive, not found"

# ─── install location ─────────────────────────────────────────────
pick_install_dir() {
  if [ -n "$INSTALL_DIR" ]; then echo "$INSTALL_DIR"; return; fi
  for d in /usr/local/bin /usr/bin; do
    if [ -d "$d" ] && [ -w "$d" ]; then echo "$d"; return; fi
  done
  echo "${HOME}/.local/bin"
}
INSTALL_DIR=$(pick_install_dir)

# ensure dir exists
if [ ! -d "$INSTALL_DIR" ]; then
  if mkdir -p "$INSTALL_DIR" 2>/dev/null; then :; else
    warn "cannot create $INSTALL_DIR — falling back to ${HOME}/.local/bin"
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

# ─── drop the binary ──────────────────────────────────────────────
TARGET="${INSTALL_DIR}/${BIN_NAME}${BIN_EXT}"
# if we don't own INSTALL_DIR and we're not root, sudo the install
NEED_SUDO=""
if [ ! -w "$INSTALL_DIR" ]; then
  if command -v sudo >/dev/null 2>&1; then
    NEED_SUDO=sudo
  else
    die "no write access to $INSTALL_DIR and sudo not available; pass --to <dir>"
  fi
fi

log "installing → ${TARGET}"
$NEED_SUDO install -m 0755 "$BIN_PATH" "$TARGET"

# ─── verify ───────────────────────────────────────────────────────
log "verifying install …"
if command -v "$BIN_NAME" >/dev/null 2>&1; then
  "$BIN_NAME" --version
else
  printf '%snote:%s %s is installed but not on PATH.\n' "$C_DIM" "$C_RESET" "$TARGET"
  printf '       add it:  export PATH="%s:$PATH"\n' "$INSTALL_DIR"
fi

printf '\n'
log "agidb installed at ${TARGET}"
cat <<EOF

  next steps
  ────────────────────────────────────────────────────
  agidb observe  ./mem.agidb  "Sarah recommended Bawri"
  agidb recall   ./mem.agidb  "what thai place?"
  agidb stats    ./mem.agidb

  full docs:  https://github.com/${REPO}#readme
  demo page:  https://agidb.dev/demo
EOF