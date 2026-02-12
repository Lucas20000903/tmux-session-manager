# VHS Demo Tape for tsm (tmux session manager)
# Usage:
#   ./record-demo.sh    # setup + record + cleanup
#   vhs demo.tape       # record only (requires tmux sessions)

Output demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 80ms

# Launch tsm
Type "tsm"
Enter
Sleep 1.5s

# Navigate session list
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "k"
Sleep 600ms

# Open action menu
Type "l"
Sleep 1s

# Go back
Type "h"
Sleep 600ms

# New session dialog
Type "n"
Sleep 1.2s
Escape
Sleep 600ms

# Filter sessions
Type "/"
Sleep 300ms
Type "web"
Sleep 800ms
Enter
Sleep 800ms

# Clear filter
Ctrl+C
Sleep 600ms

# Apply tmux settings
Type "S"
Sleep 1.2s

# Show help
Type "?"
Sleep 2s
Type "q"
Sleep 600ms

# Quit
Type "q"
Sleep 500ms
