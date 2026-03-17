#!/usr/bin/env bash
#
# FlowSight installer (macOS / Linux)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/TbusOS/flowsight/main/install.sh | bash
#
# Options (via environment variables):
#   FLOWSIGHT_VERSION=0.2.0  Install a specific version
#   FLOWSIGHT_DIR=~/.local    Change install prefix (default: ~/.cargo/bin or /usr/local/bin)
#   FLOWSIGHT_NO_MODIFY_PATH=1  Skip PATH modification
#
set -euo pipefail

readonly REPO="TbusOS/flowsight"
readonly BINARY_NAME="flowsight"
readonly VERSION="${FLOWSIGHT_VERSION:-}"
readonly NO_MODIFY_PATH="${FLOWSIGHT_NO_MODIFY_PATH:-0}"

# ── Colours (low-saturation) ─────────────────────────────────────────

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()    { printf "${CYAN}[info]${NC}  %s\n" "$*"; }
success() { printf "${GREEN}[ok]${NC}    %s\n" "$*"; }
warn()    { printf "${YELLOW}[warn]${NC}  %s\n" "$*"; }
fail()    { printf "${RED}[error]${NC} %s\n" "$*" >&2; exit 1; }

# ── Detect platform ──────────────────────────────────────────────────

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux"  ;;
        Darwin*) os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) fail "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64"  ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7l)        arch="armv7"   ;;
        *) fail "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# ── Resolve install directory ────────────────────────────────────────

resolve_install_dir() {
    if [ -n "${FLOWSIGHT_DIR:-}" ]; then
        echo "${FLOWSIGHT_DIR}/bin"
        return
    fi

    # Prefer ~/.cargo/bin if it exists (Rust users)
    if [ -d "${HOME}/.cargo/bin" ]; then
        echo "${HOME}/.cargo/bin"
        return
    fi

    # Fall back to /usr/local/bin
    echo "/usr/local/bin"
}

# ── Check prerequisites ─────────────────────────────────────────────

check_prerequisites() {
    local missing=()

    if ! command -v cargo &>/dev/null; then
        missing+=("cargo (install via https://rustup.rs)")
    fi

    if ! command -v git &>/dev/null; then
        missing+=("git")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        fail "Missing prerequisites: ${missing[*]}"
    fi
}

# ── Build from source ────────────────────────────────────────────────

build_from_source() {
    local install_dir="$1"
    local tmpdir

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Cloning repository..."
    if [ -n "$VERSION" ]; then
        git clone --depth 1 --branch "v${VERSION}" \
            "https://github.com/${REPO}.git" "$tmpdir/flowsight"
    else
        git clone --depth 1 \
            "https://github.com/${REPO}.git" "$tmpdir/flowsight"
    fi

    info "Building FlowSight CLI (release)..."
    cd "$tmpdir/flowsight"
    cargo build --package flowsight-cli --release

    info "Installing to ${install_dir}..."
    mkdir -p "$install_dir"
    cp "target/release/${BINARY_NAME}" "$install_dir/${BINARY_NAME}"
    chmod +x "$install_dir/${BINARY_NAME}"

    success "Binary installed: ${install_dir}/${BINARY_NAME}"
}

# ── Install shell completions ────────────────────────────────────────

install_completions() {
    local install_dir="$1"
    local bin="${install_dir}/${BINARY_NAME}"

    if [ ! -x "$bin" ]; then
        return
    fi

    # Bash completions
    local bash_dir="${HOME}/.local/share/bash-completion/completions"
    if [ -d "$(dirname "$bash_dir")" ] || command -v bash &>/dev/null; then
        mkdir -p "$bash_dir"
        "$bin" completions bash > "$bash_dir/flowsight" 2>/dev/null && \
            info "Bash completions installed to ${bash_dir}/flowsight"
    fi

    # Zsh completions
    local zsh_dir="${HOME}/.zsh/completions"
    if command -v zsh &>/dev/null; then
        mkdir -p "$zsh_dir"
        "$bin" completions zsh > "$zsh_dir/_flowsight" 2>/dev/null && \
            info "Zsh completions installed to ${zsh_dir}/_flowsight"
    fi

    # Fish completions
    local fish_dir="${HOME}/.config/fish/completions"
    if command -v fish &>/dev/null; then
        mkdir -p "$fish_dir"
        "$bin" completions fish > "$fish_dir/flowsight.fish" 2>/dev/null && \
            info "Fish completions installed to ${fish_dir}/flowsight.fish"
    fi
}

# ── Update PATH in shell profiles ────────────────────────────────────

update_path() {
    local install_dir="$1"

    if [ "$NO_MODIFY_PATH" = "1" ]; then
        return
    fi

    # Check if already in PATH
    if echo "$PATH" | tr ':' '\n' | grep -qx "$install_dir"; then
        return
    fi

    local line="export PATH=\"${install_dir}:\$PATH\""

    for rc in "${HOME}/.bashrc" "${HOME}/.zshrc" "${HOME}/.profile"; do
        if [ -f "$rc" ]; then
            if ! grep -qF "$install_dir" "$rc" 2>/dev/null; then
                printf '\n# FlowSight\n%s\n' "$line" >> "$rc"
                info "Added to ${rc}"
            fi
        fi
    done
}

# ── Main ─────────────────────────────────────────────────────────────

main() {
    local platform install_dir

    printf '\n'
    info "FlowSight installer"
    info "==================="
    printf '\n'

    platform="$(detect_platform)"
    info "Platform: ${platform}"

    check_prerequisites

    install_dir="$(resolve_install_dir)"
    info "Install directory: ${install_dir}"

    printf '\n'
    build_from_source "$install_dir"
    install_completions "$install_dir"
    update_path "$install_dir"

    printf '\n'
    success "Installation complete!"
    printf '\n'
    echo "  Get started:"
    echo "    flowsight --help            Show CLI options"
    echo "    flowsight analyze <file>    Analyze a C source file"
    echo "    flowsight flow <file> <fn>  Show execution flow"
    echo "    flowsight                   Launch interactive REPL"
    printf '\n'

    # Hint about reloading shell if PATH was modified
    if [ "$NO_MODIFY_PATH" != "1" ] && ! command -v "$BINARY_NAME" &>/dev/null; then
        warn "Restart your shell or run:  source ~/.bashrc  (or ~/.zshrc)"
    fi
}

main "$@"
