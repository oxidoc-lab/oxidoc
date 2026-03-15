#!/bin/sh
# Oxidoc CLI installer
#
# Install:
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh
#
# Install specific version:
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh -s -- --version v0.1.0
#
# Upgrade (same command as install):
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh
#
# Uninstall:
#   curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh -s -- --uninstall
#
# Environment variables:
#   OXIDOC_INSTALL_DIR  Install directory (default: ~/.local/bin)

set -eu

REPO="oxidoc-lab/oxidoc"
BINARY="oxidoc"
INSTALL_DIR="${OXIDOC_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    action="install"
    version="latest"

    while [ $# -gt 0 ]; do
        case "$1" in
            --help|-h)   usage; exit 0 ;;
            --uninstall) action="uninstall"; shift ;;
            --version)   version="$2"; shift 2 ;;
            v*)          version="$1"; shift ;;
            *)           echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
        esac
    done

    case "$action" in
        install)   do_install "$version" ;;
        uninstall) do_uninstall ;;
    esac
}

usage() {
    cat <<EOF
Oxidoc CLI installer

Usage:
  install.sh [OPTIONS]

Options:
  --version VERSION  Install a specific version (e.g., v0.1.0)
  --uninstall        Remove oxidoc from the install directory
  --help, -h         Show this help message

Environment:
  OXIDOC_INSTALL_DIR  Override install directory (default: ~/.local/bin)
EOF
}

do_install() {
    version="$1"

    # Check prerequisites
    check_prereqs

    os="$(detect_os)"
    arch="$(detect_arch)"
    target="$(detect_target "$os" "$arch")"

    if [ "$version" = "latest" ]; then
        version="$(fetch_latest_version)"
        if [ -z "$version" ]; then
            echo "Error: could not determine latest version." >&2
            echo "Check https://github.com/${REPO}/releases" >&2
            exit 1
        fi
    fi

    # Check if already installed and same version
    if command -v "$BINARY" > /dev/null 2>&1; then
        current="$("$BINARY" --version 2>/dev/null | awk '{print $2}' || echo "")"
        if [ "$current" = "${version#v}" ]; then
            echo "oxidoc ${version#v} is already installed."
            exit 0
        fi
        if [ -n "$current" ]; then
            echo "Upgrading oxidoc ${current} -> ${version#v} (${target})..."
        else
            echo "Installing oxidoc ${version#v} (${target})..."
        fi
    else
        echo "Installing oxidoc ${version#v} (${target})..."
    fi

    ext="tar.gz"
    bin_name="$BINARY"
    if [ "$os" = "windows" ]; then
        ext="zip"
        bin_name="${BINARY}.exe"
    fi

    url="https://github.com/${REPO}/releases/download/${version}/oxidoc-${version#v}-${target}.${ext}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "Downloading from GitHub Releases..."
    download "$url" "$tmpdir/archive.${ext}"

    if [ "$os" = "windows" ]; then
        unzip -oq "$tmpdir/archive.zip" -d "$tmpdir"
    else
        tar xzf "$tmpdir/archive.tar.gz" -C "$tmpdir"
    fi

    if [ ! -f "$tmpdir/$bin_name" ]; then
        echo "Error: binary not found in archive" >&2
        exit 1
    fi

    mkdir -p "$INSTALL_DIR" 2>/dev/null || true
    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmpdir/$bin_name" "$INSTALL_DIR/$bin_name"
        chmod +x "$INSTALL_DIR/$bin_name" 2>/dev/null || true
    else
        echo "Installing to $(display_path "$INSTALL_DIR") (requires sudo)..."
        sudo mkdir -p "$INSTALL_DIR"
        sudo mv "$tmpdir/$bin_name" "$INSTALL_DIR/$bin_name"
        sudo chmod +x "$INSTALL_DIR/$bin_name" 2>/dev/null || true
    fi

    echo ""
    echo "Installed oxidoc to $(display_path "$INSTALL_DIR/$bin_name")"
    "${INSTALL_DIR}/${bin_name}" --version 2>/dev/null || true

    # Check if install dir is in PATH and set up if needed
    setup_path

    echo ""
    echo "Get started:"
    echo "  oxidoc init my-docs"
    echo "  cd my-docs"
    echo "  oxidoc dev"
}

do_uninstall() {
    os="$(detect_os)"
    bin_name="$BINARY"
    if [ "$os" = "windows" ]; then
        bin_name="${BINARY}.exe"
    fi

    target="${INSTALL_DIR}/${bin_name}"
    if [ ! -f "$target" ]; then
        echo "oxidoc is not installed at $(display_path "$target")"
        exit 0
    fi

    if [ -w "$target" ]; then
        rm "$target"
    else
        sudo rm "$target"
    fi

    echo "Uninstalled oxidoc from $(display_path "$target")"
}

check_prereqs() {
    missing=""
    command -v curl > /dev/null 2>&1 || command -v wget > /dev/null 2>&1 || missing="curl or wget"

    os="$(detect_os)"
    if [ "$os" = "windows" ]; then
        command -v unzip > /dev/null 2>&1 || missing="${missing:+$missing, }unzip"
    else
        command -v tar > /dev/null 2>&1 || missing="${missing:+$missing, }tar"
    fi

    if [ -n "$missing" ]; then
        echo "Error: missing required tools: ${missing}" >&2
        exit 1
    fi
}

setup_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) return ;;
    esac

    # Install dir is not in PATH — try to add it
    shell_name="$(basename "${SHELL:-/bin/sh}")"
    line="export PATH=\"${INSTALL_DIR}:\$PATH\""

    case "$shell_name" in
        zsh)  rc="$HOME/.zshrc" ;;
        bash)
            # Prefer .bashrc, fall back to .bash_profile for login shells
            if [ -f "$HOME/.bashrc" ]; then
                rc="$HOME/.bashrc"
            else
                rc="$HOME/.bash_profile"
            fi
            ;;
        fish)
            line="fish_add_path ${INSTALL_DIR}"
            rc="$HOME/.config/fish/config.fish"
            ;;
        *)    rc="" ;;
    esac

    if [ -n "$rc" ]; then
        # Don't add if already present
        if [ -f "$rc" ] && grep -qF "$INSTALL_DIR" "$rc" 2>/dev/null; then
            return
        fi
        echo "$line" >> "$rc"
        echo ""
        echo "Added $(display_path "$INSTALL_DIR") to PATH in $(display_path "$rc")"
        echo "Restart your shell or run: $line"
    else
        echo ""
        echo "Add $(display_path "$INSTALL_DIR") to your PATH:"
        echo "  $line"
    fi
}

download() {
    url="$1"
    dest="$2"
    if command -v curl > /dev/null 2>&1; then
        if ! curl -fsSL "$url" -o "$dest"; then
            echo "Error: download failed. Check the version and try again." >&2
            echo "  ${url}" >&2
            exit 1
        fi
    elif command -v wget > /dev/null 2>&1; then
        if ! wget -qO "$dest" "$url"; then
            echo "Error: download failed. Check the version and try again." >&2
            echo "  ${url}" >&2
            exit 1
        fi
    fi
}

display_path() {
    echo "$1" | sed "s|^$HOME|~|"
}

detect_os() {
    case "$(uname -s)" in
        Linux*)              echo "linux" ;;
        Darwin*)             echo "macos" ;;
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
        linux-x86_64)    echo "linux-x64" ;;
        linux-aarch64)   echo "linux-arm64" ;;
        macos-x86_64)    echo "macos-x64" ;;
        macos-aarch64)   echo "macos-arm64" ;;
        windows-x86_64)  echo "windows-x64" ;;
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
