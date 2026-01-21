#!/bin/bash

# Demo recording script for tmux-session-manager

echo "=== tmux-session-manager Demo Recording ==="
echo ""
echo "This script will help you record a demo gif."
echo ""

# Setup test sessions
echo "Setting up test sessions..."
tmux kill-server 2>/dev/null
sleep 0.5
tmux new-session -d -s "project-api"
tmux new-session -d -s "project-web"
tmux new-session -d -s "dotfiles"
echo "Created 3 test sessions: project-api, project-web, dotfiles"
echo ""

# Recording instructions
echo "=== Recording Instructions ==="
echo ""
echo "1. Start recording:"
echo "   asciinema rec demo.cast"
echo ""
echo "2. In the recording, demonstrate:"
echo "   - Type: ts"
echo "   - Navigate with arrow keys"
echo "   - Press Tab to switch modes (Attach → New → Manage)"
echo "   - Press Tab again to return to Attach"
echo "   - Select a session with Enter"
echo "   - Type: exit (to end)"
echo ""
echo "3. Stop recording with: exit or Ctrl+D"
echo ""
echo "4. Convert to gif:"
echo "   agg demo.cast demo.gif --font-size 14 --cols 80 --rows 25"
echo ""
echo "=== Ready to record? ==="
read -p "Press Enter to start recording..."

# Start recording
asciinema rec demo.cast

echo ""
echo "Recording saved to demo.cast"
echo ""
echo "Converting to gif..."
agg demo.cast demo.gif --font-size 14 --cols 80 --rows 25

echo ""
echo "Done! demo.gif has been created."
echo ""

# Cleanup
echo "Cleaning up test sessions..."
tmux kill-server 2>/dev/null
