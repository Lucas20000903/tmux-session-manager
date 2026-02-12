# VHS Demo Tape for tsm (tmux session manager)
# Run: vhs demo.tape
# Install VHS: brew install vhs

Output demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Padding 20
Set Theme "Catppuccin Mocha"

# Ensure we have some demo sessions
Hide
Type "tmux new-session -d -s demo-project -c ~/workspace"
Enter
Type "tmux new-session -d -s api-server -c ~/workspace"
Enter
Type "tmux new-session -d -s frontend -c ~/workspace"
Enter
Show

# Launch tsm
Type "tsm"
Enter
Sleep 1s

# Navigate session list
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "k"
Sleep 500ms

# Switch session with Space (peek)
Type " "
Sleep 1s

# Open action menu
Type "l"
Sleep 800ms

# Go back
Type "h"
Sleep 500ms

# New session dialog
Type "n"
Sleep 1s
Escape
Sleep 500ms

# Filter sessions
Type "/"
Sleep 300ms
Type "demo"
Sleep 500ms
Enter
Sleep 800ms

# Clear filter with Ctrl+C
Ctrl+C
Sleep 500ms

# Apply tmux settings
Type "S"
Sleep 1s

# Show help
Type "?"
Sleep 1.5s
Type "q"
Sleep 500ms

# Quit
Type "q"
Sleep 500ms

# Cleanup demo sessions
Hide
Type "tmux kill-session -t demo-project 2>/dev/null; tmux kill-session -t api-server 2>/dev/null; tmux kill-session -t frontend 2>/dev/null"
Enter
Show
