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
  local os arch arch_part target

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux|Darwin) ;;
    *) die "Unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64"  ;;
    aarch64|arm64) arch_part="aarch64" ;;
    *)             die "Unsupported architecture: $arch" ;;
  esac

  if [ "$os" = "Linux" ]; then
    # Detect musl libc independently of architecture.
    #
    # Subtlety: on musl, `ldd --version` itself exits non-zero (musl's
    # ldd is a symlink to the dynamic linker and returns 1 for the
    # --version usage banner). Under `set -o pipefail`, the pipeline
    # `ldd --version 2>&1 | grep -qi musl` therefore returns ldd's
    # non-zero exit *even when grep successfully matches "musl"* —
    # which would make musl systems appear to be glibc and silently
    # download a glibc-linked binary.
    #
    # Capture the output separately (tolerating a non-zero ldd exit
    # with `|| true`) and then grep the captured string, so the match
    # decision is independent of ldd's exit code.
    local is_musl=false ldd_output=""
    if command -v ldd >/dev/null 2>&1; then
      ldd_output="$(ldd --version 2>&1 || true)"
      if printf '%s' "$ldd_output" | grep -qi musl; then
        is_musl=true
      fi
    fi
    # Secondary signal: musl systems ship /lib/ld-musl-*.so.1.
    if ! $is_musl && ls /lib/ld-musl-* >/dev/null 2>&1; then
      is_musl=true
    fi

    if $is_musl; then
      if [ "$arch_part" = "x86_64" ]; then
        # The release workflow publishes x86_64-unknown-linux-musl.
        target="${arch_part}-unknown-linux-musl"
      else
        # No musl artifact is published for aarch64 (see
        # .github/workflows/release.yml). Fail fast instead of silently
        # fetching the glibc-linked aarch64-unknown-linux-gnu binary,
        # which will not run on Alpine or other musl-only ARM64 hosts.
        die "No prebuilt musl binary for ${arch_part}. Build from source: see the 'From source' section in README.md."
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
  local api_response

  if [ "$version" = "latest" ]; then
    need curl
    # Capture the API response first so a 403/rate-limit HTML body or a
    # missing tag_name doesn't silently kill the script via `pipefail`
    # (grep exits 1 on no-match → pipeline fails → set -e → silent exit
    # before the `die` guard can run).
    api_response="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")" \
      || die "Could not query GitHub API for latest release (rate limited or no releases yet?)"
    version="$(printf '%s' "$api_response" \
      | awk -F'"' '/"tag_name":/ { print $4; exit }')"
    [ -n "$version" ] || die "Could not determine latest release (unexpected API response)"
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

  [ -f "${tmpdir}/${BINARY}" ] \
    || die "Binary '${BINARY}' not found in archive — unexpected archive layout"

  # Ensure INSTALL_DIR exists before the writability check so a not-yet-
  # created directory (e.g. fresh ~/.local/bin) doesn't misroute us into
  # the sudo branch. If we can create it as the current user, we keep the
  # unprivileged path; otherwise we fall through to sudo.
  mkdir -p "$INSTALL_DIR" 2>/dev/null || true

  # Install the binary, matching chmod privilege level to the mv so the
  # privileged path (root-owned destination) doesn't fail under `set -e`.
  if [ -w "$INSTALL_DIR" ]; then
    mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
  else
    info "Elevated permissions required to install to ${INSTALL_DIR}"
    need sudo
    sudo mkdir -p "$INSTALL_DIR"
    sudo mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"
  fi

  info "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
  "${INSTALL_DIR}/${BINARY}" --version 2>/dev/null || true
}

install
