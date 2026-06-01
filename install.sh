#!/usr/bin/env bash
# install.sh — one-line installer for rustifi (unifi MCP server binary)
#
# Usage:
#   bash <(curl -fsSL https://raw.githubusercontent.com/jmagar/rustifi/main/install.sh)
#
# Or with a custom version:
#   RUSTIFI_VERSION=v0.2.0 bash install.sh
#
set -euo pipefail

BINARY_NAME="runifi"
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"
REPO="jmagar/rustifi"
VERSION="${RUSTIFI_VERSION:-latest}"

# Detect OS/arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "${ARCH}" in
  x86_64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "ERROR: unsupported architecture: ${ARCH}" >&2
    exit 1
    ;;
esac

case "${OS}" in
  linux)  PLATFORM="${ARCH}-unknown-linux-musl" ;;
  darwin) PLATFORM="${ARCH}-apple-darwin" ;;
  *)
    echo "ERROR: unsupported OS: ${OS}" >&2
    exit 1
    ;;
esac

# Resolve latest version tag if needed
if [[ "${VERSION}" == "latest" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
fi

# Build download URL  — adjust asset name pattern to match your release workflow
BINARY_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${PLATFORM}"

echo "Installing rustifi ${VERSION} (${PLATFORM}) → ${INSTALL_DIR}/${BINARY_NAME}"
echo "  Source: ${BINARY_URL}"

mkdir -p "${INSTALL_DIR}"

if ! curl -fSL --progress-bar "${BINARY_URL}" -o "${INSTALL_DIR}/${BINARY_NAME}"; then
  echo ""
  echo "ERROR: download failed. Check:" >&2
  echo "  - Release ${VERSION} exists at https://github.com/${REPO}/releases" >&2
  echo "  - Asset name matches: ${BINARY_NAME}-${PLATFORM}" >&2
  echo "  - Network connectivity" >&2
  exit 1
fi

chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "Installed: ${INSTALL_DIR}/${BINARY_NAME}"
echo "  Version: $(${INSTALL_DIR}/${BINARY_NAME} --version 2>/dev/null || echo 'unknown')"
echo ""

# Write a starter .env if one doesn't exist
ENV_FILE=".env"
if [[ ! -f "${ENV_FILE}" ]]; then
  cat > "${ENV_FILE}" << 'EOF'
# UniFi controller URL (required)
UNIFI_URL=https://unifi.local

# API key for X-API-KEY authentication (required)
# Generate in: UniFi OS → Settings → Admins & Users → API Keys
UNIFI_API_KEY=replace-me

# MCP bearer token (generate with: openssl rand -hex 32)
UNIFI_MCP_TOKEN=replace-me

# Skip TLS verification — required for self-signed controller certs (default: true)
UNIFI_SKIP_TLS_VERIFY=true
EOF
  chmod 600 "${ENV_FILE}"
  echo "Created starter .env — fill in UNIFI_URL, UNIFI_API_KEY, and UNIFI_MCP_TOKEN"
else
  echo ".env already exists — skipping"
fi

# Ensure INSTALL_DIR is in PATH
if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  echo "WARNING: ${INSTALL_DIR} is not in your PATH."
  echo "Add it by running:"
  echo "  export PATH=\"${INSTALL_DIR}:\${PATH}\""
  echo "Or add that line to your ~/.bashrc / ~/.zshrc."
fi

echo ""
echo "Next steps:"
echo "  1. Edit .env — set UNIFI_URL, UNIFI_API_KEY, and UNIFI_MCP_TOKEN"
echo "  2. Start the MCP server: ${BINARY_NAME} serve mcp"
echo "  3. Connect Claude Code: add ${BINARY_NAME} as an MCP server at http://localhost:7474/mcp"
echo ""
echo "See README.md for full setup instructions."
