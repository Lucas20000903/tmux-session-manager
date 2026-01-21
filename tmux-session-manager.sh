#!/bin/zsh

# tmux session manager - fzf-based tmux session management
# https://github.com/Lucas20000903/tmux-session-manager

ts() {
    local mode_file=$(mktemp)
    local result_file=$(mktemp)
    echo "attach" >"$mode_file"

    trap 'rm -f "$mode_file" "$result_file"; clear' EXIT

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
                --cycle \
                --no-clear \
                --style full \
                --border --padding 0,1 \
                --border-label ' tmux session manager ' \
                --border-label-pos 2 \
                --input-label ' › Attach ' \
                --list-label ' Sessions ' \
                --preview-window=right:60% \
                --preview-label ' Preview ' \
                --prompt="› " \
                --pointer="▸" \
                --marker="●" \
                --color 'border:#5c6370,label:#61afef' \
                --color 'preview-border:#5c6370,preview-label:#c678dd' \
                --color 'list-border:#5c6370,list-label:#98c379' \
                --color 'input-border:#5c6370,input-label:#61afef' \
                --color 'fg:-1,bg:-1,hl:#61afef:bold' \
                --color 'fg+:#ffffff:bold,bg+:#3e4452,hl+:#61afef:bold' \
                --color 'info:#e5c07b,prompt:#61afef:bold,pointer:#c678dd:bold' \
                --color 'marker:#98c379:bold,spinner:#61afef' \
                --preview 'zsh ~/.ts_preview.sh '"$mode_file"' {}' \
                --bind 'tab:transform:
                mode=$(cat '"$mode_file"')
                case "$mode" in
                    attach)
                        echo "new" > '"$mode_file"'
                        echo "reload(printf \"[New Session]\n[New Claude Session]\")+change-prompt(+ )+refresh-preview+change-input-label( + New )+change-list-label( Options )"
                        ;;
                    new)
                        echo "manage" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"[Delete All]\n$sessions\")+change-prompt(× )+refresh-preview+change-input-label( × Manage )+change-list-label( Sessions )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(× )+refresh-preview+change-input-label( × Manage )+change-list-label( Sessions )"
                        fi
                        ;;
                    manage)
                        echo "attach" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"$sessions\")+change-prompt(› )+refresh-preview+change-input-label( › Attach )+change-list-label( Sessions )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(› )+refresh-preview+change-input-label( › Attach )+change-list-label( Sessions )"
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
                --no-clear \
                --style full \
                --border --padding 0,1 \
                --border-label ' Create Session ' \
                --border-label-pos 2 \
                --input-label ' + Session Name ' \
                --preview-window=right:60% \
                --preview-label ' Preview ' \
                --prompt="+ " \
                --pointer="▸" \
                --color 'border:#5c6370,label:#98c379' \
                --color 'preview-border:#5c6370,preview-label:#c678dd' \
                --color 'input-border:#5c6370,input-label:#98c379' \
                --color 'fg:-1,bg:-1,hl:#98c379:bold' \
                --color 'fg+:#ffffff:bold,bg+:#3e4452,hl+:#98c379:bold' \
                --color 'prompt:#98c379:bold,pointer:#98c379:bold' \
                --print-query --disabled \
                --preview 'printf "  Name: \033[32m{q}\033[0m\n\n  \033[2mPress Enter to create\033[0m"' |
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
                    --no-clear \
                    --style full \
                    --border --padding 0,1 \
                    --border-label ' ⚠ WARNING ' \
                    --border-label-pos 2 \
                    --input-label ' ! Confirm ' \
                    --list-label ' Choice ' \
                    --preview-window=right:60% \
                    --preview-label ' Warning ' \
                    --prompt="! " \
                    --pointer="▸" \
                    --color 'border:#e06c75,label:#e06c75' \
                    --color 'preview-border:#e06c75,preview-label:#e06c75' \
                    --color 'list-border:#e06c75,list-label:#e06c75' \
                    --color 'input-border:#e06c75,input-label:#e06c75' \
                    --color 'fg:-1,bg:-1,hl:#e06c75:bold' \
                    --color 'fg+:#ffffff:bold,bg+:#3e4452,hl+:#e06c75:bold' \
                    --color 'prompt:#e06c75:bold,pointer:#e06c75:bold' \
                    --preview 'printf "  \033[31mDelete ALL sessions!\033[0m\n\n  \033[33mThis cannot be undone.\033[0m"')
                [ "$confirm" = "Yes" ] && tmux kill-server
                continue
            fi

            [ "$selection" = "[No sessions]" ] && continue

            local confirm
            confirm=$(printf "No\nYes" | fzf \
                --ansi \
                --reverse \
                --no-clear \
                --style full \
                --border --padding 0,1 \
                --border-label ' Delete Session ' \
                --border-label-pos 2 \
                --input-label ' ! Confirm ' \
                --list-label ' Choice ' \
                --preview-window=right:60% \
                --preview-label ' Session Info ' \
                --prompt="! " \
                --pointer="▸" \
                --color 'border:#e5c07b,label:#e5c07b' \
                --color 'preview-border:#e5c07b,preview-label:#e5c07b' \
                --color 'list-border:#e5c07b,list-label:#e5c07b' \
                --color 'input-border:#e5c07b,input-label:#e5c07b' \
                --color 'fg:-1,bg:-1,hl:#e5c07b:bold' \
                --color 'fg+:#ffffff:bold,bg+:#3e4452,hl+:#e5c07b:bold' \
                --color 'prompt:#e5c07b:bold,pointer:#e5c07b:bold' \
                --preview 'printf "  Session: \033[36m'"$selection"'\033[0m\n\n  \033[2mSelect Yes to delete\033[0m"')
            [ "$confirm" = "Yes" ] && tmux kill-session -t "$selection"
            continue
            ;;
        esac
    done
}

# Alias for quick detach
alias td='tmux detach'
