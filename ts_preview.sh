#!/bin/zsh
# ts preview script - professional styling

mode_file="$1"
item="$2"

mode=$(cat "$mode_file" 2>/dev/null)
[[ -z "$mode" ]] && mode="attach"

# Colors
C_CYAN=$'\033[36m'
C_GREEN=$'\033[32m'
C_YELLOW=$'\033[33m'
C_RED=$'\033[31m'
C_DIM=$'\033[2m'
C_BOLD=$'\033[1m'
C_RESET=$'\033[0m'

case "$mode" in
    attach)
        printf "${C_CYAN}${C_BOLD}┌─ Session Preview ─┐${C_RESET}\n\n"
        if [[ -n "$item" && "$item" != "[No sessions]" ]]; then
            tmux capture-pane -ep -t "$item" 2>/dev/null || printf "${C_DIM}  No preview available${C_RESET}"
        else
            printf "${C_DIM}  Select a session to preview${C_RESET}"
        fi
        ;;
    new)
        printf "${C_GREEN}${C_BOLD}┌─ Create Session ─┐${C_RESET}\n\n"
        printf "  ${C_CYAN}▸${C_RESET} New Session\n"
        printf "    ${C_DIM}Regular tmux session${C_RESET}\n\n"
        printf "  ${C_YELLOW}▸${C_RESET} Claude Session\n"
        printf "    ${C_DIM}Auto-launch Claude CLI${C_RESET}\n"
        ;;
    manage)
        printf "${C_YELLOW}${C_BOLD}┌─ Session Info ─┐${C_RESET}\n\n"
        if [[ -n "$item" && "$item" != "[Delete All]" && "$item" != "[No sessions]" ]]; then
            printf "  ${C_DIM}Windows:${C_RESET}\n"
            tmux list-windows -t "$item" -F "    ${C_CYAN}▸${C_RESET} [#{window_index}] #{window_name}" 2>/dev/null || printf "    ${C_DIM}No info${C_RESET}"
        elif [[ "$item" == "[Delete All]" ]]; then
            printf "  ${C_RED}! Warning${C_RESET}\n\n"
            printf "  ${C_DIM}This will delete ALL sessions${C_RESET}\n"
        else
            printf "  ${C_DIM}Select a session${C_RESET}"
        fi
        ;;
    *)
        printf "${C_RED}Unknown mode: $mode${C_RESET}"
        ;;
esac
