#!/bin/zsh
# ts preview script for tmux-session-manager

mode_file="$1"
item="$2"

mode=$(cat "$mode_file" 2>/dev/null)
[[ -z "$mode" ]] && mode="attach"

case "$mode" in
    attach)
        printf "\033[36mSession Preview\033[0m\n\n"
        if [[ -n "$item" && "$item" != "[No sessions]" ]]; then
            tmux capture-pane -ep -t "$item" 2>/dev/null || printf "\033[2mNo preview\033[0m"
        else
            printf "\033[2mSelect a session\033[0m"
        fi
        ;;
    new)
        printf "\033[36mCreate Session\033[0m\n\n"
        printf "\033[32m•\033[0m New Session - Regular tmux\n"
        printf "\033[33m•\033[0m Claude Session - Auto-launch Claude\n"
        ;;
    manage)
        printf "\033[36mSession Windows\033[0m\n\n"
        if [[ -n "$item" && "$item" != "[Delete All]" && "$item" != "[No sessions]" ]]; then
            tmux list-windows -t "$item" -F "[#{window_index}] #{window_name}" 2>/dev/null || printf "\033[2mNo info\033[0m"
        else
            printf "\033[2mSelect a session\033[0m"
        fi
        ;;
    *)
        printf "\033[31mUnknown mode: $mode\033[0m"
        ;;
esac
