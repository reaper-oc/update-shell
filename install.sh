#!/usr/bin/env bash
set -e

BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/updsh"
DATA_DIR="${HOME}/.local/share/updsh"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "==> Building updSH..."
cargo build --release --manifest-path "${PROJECT_DIR}/Cargo.toml"

echo "==> Installing to ${BIN_DIR}/updsh..."
mkdir -p "${BIN_DIR}"
cp "${PROJECT_DIR}/target/release/updsh" "${BIN_DIR}/updsh"

echo "==> Creating config directory ${CONFIG_DIR}..."
mkdir -p "${CONFIG_DIR}"
if [ ! -f "${CONFIG_DIR}/env" ]; then
    cat > "${CONFIG_DIR}/env" << 'CONFIG'
UPD_PROMPT_STYLE=multiline
UPD_COLOR_USER=green
UPD_COLOR_HOST=blue
UPD_COLOR_PATH=yellow
UPD_COLOR_GIT=red
UPD_COLOR_EXIT=red
UPD_SHOW_GIT=yes
UPD_SHOW_EXIT_CODE=yes
CONFIG
    echo "  Created default config."
else
    echo "  Config already exists, skipping."
fi

echo "==> Creating data directories..."
mkdir -p "${DATA_DIR}"
mkdir -p "${CONFIG_DIR}/packages/installed"
mkdir -p "${CONFIG_DIR}/packages/enabled"
mkdir -p "${DATA_DIR}/bin"

echo "==> Checking PATH..."
case ":${PATH}:" in
    *:${BIN_DIR}:*) ;;
    *) echo "  WARNING: ${BIN_DIR} is not in your PATH."
       echo "  Add this to your ~/.bashrc (or ~/.zshrc):"
       echo "    export PATH=\"\${HOME}/.local/bin:\${PATH}\"" ;;
esac

if [ "$SHELL" != "${BIN_DIR}/updsh" ] && [ -x "${BIN_DIR}/updsh" ]; then
    echo ""
    echo "==> To set updSH as your default shell:"
    echo "  1. Add updSH to /etc/shells (needs sudo):"
    echo "     echo \"${BIN_DIR}/updsh\" | sudo tee -a /etc/shells"
    echo "  2. Change your default shell:"
    echo "     chsh -s \"${BIN_DIR}/updsh\""
    echo ""
    echo "  Or just run it directly: ${BIN_DIR}/updsh"
fi

echo ""
echo "==> Done! Run updSH with: ${BIN_DIR}/updsh"
