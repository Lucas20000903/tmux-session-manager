#!/bin/zsh

# tmux session manager - fzf-based tmux session management
# https://github.com/Lucas20000903/tmux-session-manager

ts() {
    local mode_file=$(mktemp)
    local result_file=$(mktemp)
    echo "attach" >"$mode_file"

    trap 'rm -f "$mode_file" "$result_file"; clear' EXIT

    # fzf color theme
    local fzf_colors="--color=fg:-1,bg:-1,hl:cyan:bold"
    fzf_colors+=",fg+:white:bold,bg+:black,hl+:cyan:bold"
    fzf_colors+=",info:yellow,prompt:cyan:bold,pointer:magenta:bold"
    fzf_colors+=",marker:green:bold,spinner:cyan,header:blue"
    fzf_colors+=",border:bright-black,label:cyan"

    while true; do
        echo "attach" >"$mode_file"

        local initial_list
        initial_list=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
        [ -z "$initial_list" ] && initial_list="[No sessions]"

        local selection
        selection=$(
            echo "$initial_list" | fzf \
                --ansi \
                --reverse \
                --border=rounded \
                --border-label-pos=bottom:2 \
                --cycle \
                --no-clear \
                --header-first \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="› Attach " \
                --pointer="▸" \
                --marker="●" \
                --border-label=" ⇥ Mode │ ↵ Select │ esc Cancel " \
                $fzf_colors \
                --preview 'zsh ~/.ts_preview.sh '"$mode_file"' {}' \
                --bind 'tab:transform:
                mode=$(cat '"$mode_file"')
                case "$mode" in
                    attach)
                        echo "new" > '"$mode_file"'
                        echo "reload(printf \"[New Session]\n[New Claude Session]\")+change-prompt(+ New )+refresh-preview+change-border-label( ⇥ Mode │ ↵ Select │ esc Cancel )"
                        ;;
                    new)
                        echo "manage" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"[Delete All]\n$sessions\")+change-prompt(× Manage )+refresh-preview+change-border-label( ⇥ Mode │ ↵ Delete │ esc Cancel )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(× Manage )+refresh-preview+change-border-label( ⇥ Mode │ esc Cancel )"
                        fi
                        ;;
                    manage)
                        echo "attach" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"$sessions\")+change-prompt(› Attach )+refresh-preview+change-border-label( ⇥ Mode │ ↵ Select │ esc Cancel )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(› Attach )+refresh-preview+change-border-label( ⇥ Mode │ esc Cancel )"
                        fi
                        ;;
                esac
            '
        )

        [ -z "$selection" ] && return 1
        [ "$selection" = "[No sessions]" ] && return 1

        local final_mode=$(cat "$mode_file")

        case "$final_mode" in
        attach)
            if [ -n "$TMUX" ]; then
                tmux switch-client -t "$selection"
            else
                tmux attach -t "$selection"
            fi
            return 0
            ;;
        new)
            local session_name
            session_name=$(echo "" | fzf \
                --ansi \
                --reverse \
                --border=rounded \
                --border-label-pos=bottom:2 \
                --no-clear \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="+ Session name " \
                --pointer="▸" \
                --border-label=" ↵ Create │ esc Cancel " \
                $fzf_colors \
                --print-query --disabled \
                --preview 'printf "\033[36m┌─ New Session ─┐\033[0m\n\n  Name: \033[32m{q}\033[0m\n\n\033[2mPress Enter to create\033[0m"' |
                head -1)

            [ -z "$session_name" ] && continue

            if tmux has-session -t "$session_name" 2>/dev/null; then
                echo "Session '$session_name' already exists."
                continue
            fi

            if [ "$selection" = "[New Session]" ]; then
                tmux new-session -s "$session_name"
            else
                tmux new-session -s "$session_name" -d
                tmux send-keys -t "$session_name" "claude" C-m
                if [ -n "$TMUX" ]; then
                    tmux switch-client -t "$session_name"
                else
                    tmux attach -t "$session_name"
                fi
            fi
            return 0
            ;;
        manage)
            if [ "$selection" = "[Delete All]" ]; then
                local confirm
                confirm=$(printf "No\nYes" | fzf \
                    --ansi \
                    --reverse \
                    --border=rounded \
                    --border-label-pos=bottom:2 \
                    --no-clear \
                    --preview-window=right:60%:border-left \
                    --preview-label=" Preview " \
                    --prompt="! Delete ALL? " \
                    --pointer="▸" \
                    --border-label=" ↵ Confirm │ esc Cancel " \
                    --color=fg:-1,bg:-1,hl:red:bold \
                    --color=fg+:white:bold,bg+:black,hl+:red:bold \
                    --color=prompt:red:bold,pointer:red:bold \
                    --color=border:bright-black,label:red \
                    --preview 'printf "\033[31m┌─ WARNING ─┐\033[0m\n\n  Delete ALL sessions!\n\n\033[33m  This cannot be undone.\033[0m"')
                [ "$confirm" = "Yes" ] && tmux kill-server
                continue
            fi

            [ "$selection" = "[No sessions]" ] && continue

            local confirm
            confirm=$(printf "No\nYes" | fzf \
                --ansi \
                --reverse \
                --border=rounded \
                --border-label-pos=bottom:2 \
                --no-clear \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="! Delete session? " \
                --pointer="▸" \
                --border-label=" ↵ Confirm │ esc Cancel " \
                --color=fg:-1,bg:-1,hl:yellow:bold \
                --color=fg+:white:bold,bg+:black,hl+:yellow:bold \
                --color=prompt:yellow:bold,pointer:yellow:bold \
                --color=border:bright-black,label:yellow \
                --preview 'printf "\033[33m┌─ Confirm Delete ─┐\033[0m\n\n  Session: \033[36m'"$selection"'\033[0m\n\n\033[2m  Select Yes to delete\033[0m"')
            [ "$confirm" = "Yes" ] && tmux kill-session -t "$selection"
            continue
            ;;
        esac
    done
}

# Alias for quick detach
alias td='tmux detach'
