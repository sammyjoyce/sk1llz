#!/usr/bin/env bash
set -euo pipefail

REPO="sammyjoyce/sk1llz"
BINARY="sk1llz"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
info() { printf '\033[1;34m=>\033[0m %s\n' "$*" >&2; }

need() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not found"
}

# ---------------------------------------------------------------------------
# Detect platform
# ---------------------------------------------------------------------------

detect_target() {
  local os arch target

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os_part="unknown-linux" ;;
    Darwin) os_part="apple-darwin"  ;;
    *)      die "Unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64"  ;;
    aarch64|arm64) arch_part="aarch64" ;;
    *)             die "Unsupported architecture: $arch" ;;
  esac

  if [ "$os" = "Linux" ]; then
    # Prefer musl on x86_64 for maximum portability; fall back to gnu
    if [ "$arch_part" = "x86_64" ]; then
      if ldd --version 2>&1 | grep -qi musl; then
        target="${arch_part}-unknown-linux-musl"
      else
        target="${arch_part}-unknown-linux-gnu"
      fi
    else
      target="${arch_part}-unknown-linux-gnu"
    fi
  else
    target="${arch_part}-apple-darwin"
  fi

  printf '%s' "$target"
}

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------

resolve_version() {
  local version="${1:-latest}"

  if [ "$version" = "latest" ]; then
    need curl
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
      | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    [ -n "$version" ] || die "Could not determine latest release"
  fi

  printf '%s' "$version"
}

# ---------------------------------------------------------------------------
# Download & install
# ---------------------------------------------------------------------------

install() {
  local version target archive_name url tmpdir

  need curl
  need tar

  version="$(resolve_version "${VERSION:-latest}")"
  target="$(detect_target)"
  archive_name="${BINARY}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"

  info "Installing ${BINARY} ${version} (${target})"
  info "From: ${url}"

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL --retry 3 "$url" -o "${tmpdir}/${archive_name}" \
    || die "Download failed — check that release ${version} exists for ${target}"

  tar xzf "${tmpdir}/${archive_name}" -C "$tmpdir" \
    || die "Failed to extract archive"

  # Install the binary, matching chmod privilege level to the mv so the
  # privileged path (root-owned destination) doesn't fail under `set -e`.
  if [ -w "$INSTALL_DIR" ]; then
    mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
  else
    info "Elevated permissions required to install to ${INSTALL_DIR}"
    sudo mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"
  fi

  info "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
  "${INSTALL_DIR}/${BINARY}" --version 2>/dev/null || true
}

install
