# VHS Demo for tmux-session-manager
Output demo.gif

Set Shell "zsh"
Set FontSize 16
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20

# Setup
Hide
Type "source ~/.zshrc"
Enter
Sleep 500ms
Type "tmux kill-server 2>/dev/null; sleep 0.5"
Enter
Sleep 500ms
Type "tmux new-session -d -s project-api"
Enter
Sleep 300ms
Type "tmux new-session -d -s project-web"
Enter
Sleep 300ms
Type "tmux new-session -d -s dotfiles"
Enter
Sleep 300ms
Show

# Start demo
Sleep 1s
Type "# tmux-session-manager demo"
Enter
Sleep 1s

Type "ts"
Enter
Sleep 2s

# Navigate sessions
Down
Sleep 800ms
Down
Sleep 800ms
Up
Sleep 800ms

# Switch to New mode
Tab
Sleep 1.5s

# Show options
Down
Sleep 800ms
Up
Sleep 800ms

# Switch to Manage mode
Tab
Sleep 1.5s

# Show delete options
Down
Sleep 800ms
Down
Sleep 800ms

# Back to Attach mode
Tab
Sleep 1.5s

# Select a session
Down
Sleep 500ms
Enter
Sleep 2s

# End
Type "# Done!"
Enter
Sleep 2s
