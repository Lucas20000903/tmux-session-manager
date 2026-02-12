#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="tsm"
INSTALL_DIR="${HOME}/.local/bin"

echo "==> Building tsm (release)..."
cargo build --release

echo "==> Installing to ${INSTALL_DIR}/${BINARY_NAME}"
mkdir -p "${INSTALL_DIR}"
cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

# Check if INSTALL_DIR is in PATH
if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
    echo ""
    echo "⚠  ${INSTALL_DIR} is not in your PATH."
    echo "   Add this to your shell config:"
    echo ""
    echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

# Add tmux keybinding
TMUX_CONF="${HOME}/.tmux.conf"
KEYBINDING='bind s run-shell '\''H=$(( #{pane_height} * 80 / 100 )); [ $H -gt 50 ] && H=50; tmux display-popup -E -w 80% -h $H -b double "tsm"'\'''

if [ -f "${TMUX_CONF}" ] && grep -qF 'display-popup' "${TMUX_CONF}" && grep -qF 'tsm' "${TMUX_CONF}"; then
    echo "==> tmux keybinding already configured"
else
    echo "" >> "${TMUX_CONF}"
    echo "# tsm (tmux session manager)" >> "${TMUX_CONF}"
    echo "${KEYBINDING}" >> "${TMUX_CONF}"
    echo "==> Added tmux keybinding: prefix + s"
fi

echo ""
echo "✓ Installed! Run 'tsm' or press prefix+s in tmux."
