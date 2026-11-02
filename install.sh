#!/usr/bin/env bash
# ==============================================================================
# Reflex: Low-Latency Desktop Automation & Resident Engine Daemon
# Fast Binary Installer
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/itshimelz/reflex/main/install.sh | bash
#
# Options (via environment variables or flags):
#   REFLEX_VERSION=v0.1.0   Install a specific release version (default: latest)
#   INSTALL_DIR=~/.local/bin Custom destination directory for binaries
#   SKIP_SERVICE=1          Skip systemd user service setup
#   SKIP_CONFIG=1           Skip default config.toml creation
# ==============================================================================

set -euo pipefail

# Visual styling
BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
YELLOW="\033[33m"
RED="\033[31m"
CYAN="\033[36m"
DIM="\033[2m"
RESET="\033[0m"

# Disable color if stdout is not a TTY or NO_COLOR is set
if [ ! -t 1 ] || [ -n "${NO_COLOR:-}" ]; then
    BOLD=""
    GREEN=""
    BLUE=""
    YELLOW=""
    RED=""
    CYAN=""
    DIM=""
    RESET=""
fi

log_info() {
    printf "${BLUE}ℹ${RESET} %b\n" "$*"
}

log_success() {
    printf "${GREEN}✔${RESET} %b\n" "$*"
}

log_warn() {
    printf "${YELLOW}⚠${RESET} %b\n" "$*"
}

log_error() {
    printf "${RED}✖${RESET} %b\n" "$*" >&2
}

log_step() {
    printf "\n${BOLD}${CYAN}==>${RESET} ${BOLD}%b${RESET}\n" "$*"
}

# 1. Platform & Architecture Verification
log_step "Checking system environment..."

OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    log_error "Reflex is designed for Linux desktop environments (Wayland/Hyprland). Detected OS: ${OS}"
    exit 1
fi

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    *)
        log_error "Unsupported CPU architecture: ${ARCH}. Currently official pre-built binaries are provided for x86_64."
        log_info "You can build Reflex from source on this architecture using: cargo build --release"
        exit 1
        ;;
esac

# 2. Dependency Checking
for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        log_error "Required utility '$cmd' is not installed."
        exit 1
    fi
done

# 3. Determine Version
REPO="itshimelz/reflex"
VERSION="${1:-${REFLEX_VERSION:-}}"

if [ -z "$VERSION" ]; then
    log_info "Querying latest release from GitHub (${REPO})..."
    # Extract latest tag via GitHub redirect (avoids GitHub API rate limits)
    LATEST_REDIRECT="$(curl -sIL "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -i "^location:" | tr -d '\r\n' || true)"
    if [ -n "$LATEST_REDIRECT" ]; then
        VERSION="$(echo "$LATEST_REDIRECT" | sed -E 's/.*\/tag\/([^\r\n]+).*/\1/')"
    fi

    # Fallback to GitHub REST API if redirect extraction fails
    if [ -z "$VERSION" ]; then
        VERSION="$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"
    fi

    # Hard fallback to current stable tag
    if [ -z "$VERSION" ]; then
        VERSION="v0.1.0"
        log_warn "Could not resolve latest release tag automatically. Falling back to default: ${VERSION}"
    fi
fi

# Ensure version has 'v' prefix
case "$VERSION" in
    v*) ;;
    *) VERSION="v${VERSION}" ;;
esac

ASSET_NAME="reflex-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

log_info "Selected release: ${BOLD}${VERSION}${RESET} (${TARGET})"

# 4. Determine Installation Paths
if [ -n "${INSTALL_DIR:-}" ]; then
    BIN_DIR="${INSTALL_DIR}"
elif [ "$(id -u)" -eq 0 ]; then
    BIN_DIR="/usr/local/bin"
else
    BIN_DIR="${HOME}/.local/bin"
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/reflex"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/systemd/user"

# 5. Download and Extract
log_step "Downloading binary package..."
TMP_DIR="$(mktemp -d -t reflex-install-XXXXXX)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

log_info "Downloading ${DOWNLOAD_URL}..."
if ! curl --proto '=https' --tlsv1.2 -sSLf -o "${TMP_DIR}/${ASSET_NAME}" "$DOWNLOAD_URL"; then
    log_error "Failed to download ${DOWNLOAD_URL}"
    log_error "Please ensure release ${VERSION} exists at https://github.com/${REPO}/releases"
    exit 1
fi

# Verify SHA256 checksum if sha256sum is available
if command -v sha256sum >/dev/null 2>&1; then
    log_info "Verifying cryptographic checksum..."
    if curl --proto '=https' --tlsv1.2 -sSLf -o "${TMP_DIR}/${ASSET_NAME}.sha256" "$CHECKSUM_URL" 2>/dev/null; then
        (
            cd "$TMP_DIR"
            if sha256sum -c "${ASSET_NAME}.sha256" --status 2>/dev/null; then
                log_success "SHA-256 checksum verified."
            else
                log_error "SHA-256 checksum verification failed!"
                exit 1
            fi
        )
    else
        log_warn "Checksum file not found on release server, skipping verification."
    fi
fi

log_info "Extracting release bundle..."
tar -xzf "${TMP_DIR}/${ASSET_NAME}" -C "$TMP_DIR"

# Find extracted directory
EXTRACTED_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d -name "reflex-*" | head -n 1)"
if [ -z "$EXTRACTED_DIR" ] || [ ! -d "$EXTRACTED_DIR" ]; then
    EXTRACTED_DIR="$TMP_DIR"
fi

# 6. Install Executables
log_step "Installing binaries to ${BIN_DIR}..."
mkdir -p "$BIN_DIR"

install -m 755 "${EXTRACTED_DIR}/reflex" "${BIN_DIR}/reflex"
install -m 755 "${EXTRACTED_DIR}/reflexd" "${BIN_DIR}/reflexd"
log_success "Installed ${BOLD}reflex${RESET} -> ${BIN_DIR}/reflex"
log_success "Installed ${BOLD}reflexd${RESET} -> ${BIN_DIR}/reflexd"

if [ -f "${EXTRACTED_DIR}/reflex-agent-fallback" ]; then
    install -m 755 "${EXTRACTED_DIR}/reflex-agent-fallback" "${BIN_DIR}/reflex-agent-fallback"
    log_success "Installed ${BOLD}reflex-agent-fallback${RESET} -> ${BIN_DIR}/reflex-agent-fallback"
fi

# If ~/.cargo/bin exists and is in PATH, maintain symlinks for seamless compatibility
if [ -d "${HOME}/.cargo/bin" ] && [ "${BIN_DIR}" != "${HOME}/.cargo/bin" ]; then
    case ":${PATH}:" in
        *":${HOME}/.cargo/bin:"*)
            ln -sf "${BIN_DIR}/reflex" "${HOME}/.cargo/bin/reflex" 2>/dev/null || true
            ln -sf "${BIN_DIR}/reflexd" "${HOME}/.cargo/bin/reflexd" 2>/dev/null || true
            ;;
    esac
fi

# 7. Install Default Configuration
if [ "${SKIP_CONFIG:-0}" -ne 1 ]; then
    log_step "Configuring Reflex (${CONFIG_DIR})..."
    mkdir -p "$CONFIG_DIR"
    if [ -f "${CONFIG_DIR}/config.toml" ]; then
        log_info "Configuration file already exists at ${CONFIG_DIR}/config.toml (keeping existing)."
    elif [ -f "${EXTRACTED_DIR}/config/default_config.toml" ]; then
        cp "${EXTRACTED_DIR}/config/default_config.toml" "${CONFIG_DIR}/config.toml"
        log_success "Created initial configuration at ${CONFIG_DIR}/config.toml"
    elif [ -f "${EXTRACTED_DIR}/default_config.toml" ]; then
        cp "${EXTRACTED_DIR}/default_config.toml" "${CONFIG_DIR}/config.toml"
        log_success "Created initial configuration at ${CONFIG_DIR}/config.toml"
    fi
fi

# 8. Setup Systemd User Service
if [ "${SKIP_SERVICE:-0}" -ne 1 ] && command -v systemctl >/dev/null 2>&1; then
    log_step "Setting up systemd user service..."
    SERVICE_SRC="${EXTRACTED_DIR}/systemd/reflexd.service"
    if [ ! -f "$SERVICE_SRC" ] && [ -f "${EXTRACTED_DIR}/reflexd.service" ]; then
        SERVICE_SRC="${EXTRACTED_DIR}/reflexd.service"
    fi

    if [ -f "$SERVICE_SRC" ]; then
        mkdir -p "$SYSTEMD_USER_DIR"
        # Substitute ExecStart with the actual installed binary path
        sed "s|ExecStart=.*|ExecStart=${BIN_DIR}/reflexd|" "$SERVICE_SRC" > "${SYSTEMD_USER_DIR}/reflexd.service"
        log_success "Installed service file -> ${SYSTEMD_USER_DIR}/reflexd.service"

        # Reload systemd user daemon
        systemctl --user daemon-reload 2>/dev/null || true

        # If systemd user manager is active, enable and start the service
        if systemctl --user is-system-running >/dev/null 2>&1 || [ -n "${XDG_RUNTIME_DIR:-}" ]; then
            systemctl --user enable --now reflexd.service 2>/dev/null || true
            if systemctl --user is-active --quiet reflexd.service 2>/dev/null; then
                log_success "Daemon service reflexd.service is active and running!"
            else
                log_info "Service registered. Start it anytime with: systemctl --user start reflexd.service"
            fi
        else
            log_info "Systemd user session not active. Enable anytime with: systemctl --user enable --now reflexd.service"
        fi
    fi
fi

# 9. Verify PATH
case ":${PATH}:" in
    *":${BIN_DIR}:"*) ;;
    *)
        printf "\n"
        log_warn "${BIN_DIR} is not currently in your PATH environment variable."
        log_info "Add it to your shell configuration:"
        printf "    ${DIM}# Bash / Zsh (~/.bashrc or ~/.zshrc)${RESET}\n"
        printf "    ${CYAN}export PATH=\"%s:\$PATH\"${RESET}\n" "$BIN_DIR"
        printf "    ${DIM}# Fish (~/.config/fish/config.fish)${RESET}\n"
        printf "    ${CYAN}fish_add_path %s${RESET}\n" "$BIN_DIR"
        ;;
esac

# 10. Check Recommended Runtime Utilities (Arch / Hyprland)
MISSING_PKGS=()
for pkg_cmd in hyprctl:hyprland wpctl:pipewire notify-send:libnotify grim:grim slurp:slurp wl-copy:wl-clipboard; do
    CMD_NAME="${pkg_cmd%%:*}"
    PKG_NAME="${pkg_cmd##*:}"
    if ! command -v "$CMD_NAME" >/dev/null 2>&1; then
        MISSING_PKGS+=("$PKG_NAME")
    fi
done

if [ ${#MISSING_PKGS[@]} -gt 0 ]; then
    printf "\n"
    log_info "Optional recommended desktop packages missing: ${YELLOW}${MISSING_PKGS[*]}${RESET}"
    log_info "On Arch Linux, you can install them with:"
    printf "    ${CYAN}sudo pacman -S --needed %s${RESET}\n" "${MISSING_PKGS[*]}"
fi

# 11. Completion Banner
printf "\n"
printf "${GREEN}${BOLD}⚡ Reflex ${VERSION} installed successfully!${RESET}\n\n"
printf "  ${BOLD}CLI Client:${RESET}    %s/reflex\n" "$BIN_DIR"
printf "  ${BOLD}Engine Daemon:${RESET} %s/reflexd\n" "$BIN_DIR"
printf "  ${BOLD}Configuration:${RESET} %s/config.toml\n" "$CONFIG_DIR"
printf "  ${BOLD}Service:${RESET}       systemctl --user status reflexd.service\n\n"
printf "To test dispatch immediately:\n"
printf "  ${CYAN}reflex \"volume up\"${RESET}\n"
printf "  ${CYAN}reflex \"play\"${RESET}\n"
printf "  ${CYAN}reflex \"screenshot display\"${RESET}\n\n"
printf "${DIM}For documentation and Hyprland keybinds, visit: https://github.com/${REPO}${RESET}\n"
