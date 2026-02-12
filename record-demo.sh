#!/usr/bin/env bash
set -euo pipefail

# Setup demo sessions
echo "==> Creating demo sessions..."
tmux new-session -d -s my-api -c ~/workspace 2>/dev/null || true
tmux new-session -d -s web-frontend -c ~/workspace 2>/dev/null || true
tmux new-session -d -s data-pipeline -c ~/projects 2>/dev/null || true

# Record
echo "==> Recording demo.gif..."
vhs demo.tape

# Cleanup
echo "==> Cleaning up demo sessions..."
tmux kill-session -t my-api 2>/dev/null || true
tmux kill-session -t web-frontend 2>/dev/null || true
tmux kill-session -t data-pipeline 2>/dev/null || true

echo "✓ Done! demo.gif created."
