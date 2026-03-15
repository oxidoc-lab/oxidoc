#!/bin/sh
# Oxidoc installer — downloads the latest (or specified) release binary.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh -s -- v0.1.0

set -eu

REPO="oxidoc-lab/oxidoc"
BINARY="oxidoc"
INSTALL_DIR="${OXIDOC_INSTALL_DIR:-/usr/local/bin}"

main() {
    version="${1:-latest}"
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="$(detect_target "$os" "$arch")"

    if [ "$version" = "latest" ]; then
        version="$(fetch_latest_version)"
    fi

    echo "Installing oxidoc ${version} (${target})..."

    url="https://github.com/${REPO}/releases/download/${version}/oxidoc-${version#v}-${target}.tar.gz"
    if [ "$os" = "windows" ]; then
        url="https://github.com/${REPO}/releases/download/${version}/oxidoc-${version#v}-${target}.zip"
    fi

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "Downloading ${url}..."
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "$tmpdir/archive"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO "$tmpdir/archive" "$url"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi

    echo "Extracting..."
    if [ "$os" = "windows" ]; then
        unzip -oq "$tmpdir/archive" -d "$tmpdir"
    else
        tar xzf "$tmpdir/archive" -C "$tmpdir"
    fi

    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
    else
        echo "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
    fi
    chmod +x "$INSTALL_DIR/$BINARY"

    echo "Installed oxidoc to ${INSTALL_DIR}/${BINARY}"
    "${INSTALL_DIR}/${BINARY}" --version
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) echo "Error: unsupported OS: $(uname -s)" >&2; exit 1 ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) echo "Error: unsupported architecture: $(uname -m)" >&2; exit 1 ;;
    esac
}

detect_target() {
    os="$1"
    arch="$2"
    case "${os}-${arch}" in
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
        macos-x86_64)   echo "x86_64-apple-darwin" ;;
        macos-aarch64)  echo "aarch64-apple-darwin" ;;
        windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
        *) echo "Error: unsupported platform: ${os}-${arch}" >&2; exit 1 ;;
    esac
}

fetch_latest_version() {
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4
    elif command -v wget > /dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4
    fi
}

main "$@"
