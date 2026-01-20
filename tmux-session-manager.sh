#!/bin/zsh

# tmux session manager - fzf-based tmux session management
# https://github.com/Lucas20000903/tmux-session-manager

ts() {
    local mode_file=$(mktemp)
    local result_file=$(mktemp)
    trap 'rm -f "$mode_file" "$result_file"; clear' EXIT

    while true; do
        echo "attach" >"$mode_file"

        # 초기 목록 생성
        local initial_list
        initial_list=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
        [ -z "$initial_list" ] && initial_list="[No sessions]"

        # 메인 fzf 실행
        local selection
        selection=$(
            echo "$initial_list" | fzf \
                --ansi \
                --reverse \
                --border=sharp \
                --border-label-pos=bottom:4 \
                --cycle \
                --no-clear \
                --header-first \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="📎 Attach > " \
                --border-label=" Tab: Mode | Enter: Select | Esc: Cancel " \
                --preview 'zsh ~/.ts_preview.sh '"$mode_file"' {}' \
                --bind 'tab:transform:
                mode=$(cat '"$mode_file"')
                case "$mode" in
                    attach)
                        echo "new" > '"$mode_file"'
                        echo "reload(printf \"[New Session]\n[New Claude Session]\")+change-prompt(➕ New > )+refresh-preview+change-border-label( Tab: Mode | Enter: Select | Esc: Cancel )"
                        ;;
                    new)
                        echo "manage" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"[Delete All]\n$sessions\")+change-prompt(🗑 Manage > )+refresh-preview+change-border-label( Tab: Mode | Enter: Delete | Esc: Cancel )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(🗑 Manage > )+refresh-preview+change-border-label( Tab: Mode | Esc: Cancel )"
                        fi
                        ;;
                    manage)
                        echo "attach" > '"$mode_file"'
                        sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                        if [ -n "$sessions" ]; then
                            echo "reload(printf \"$sessions\")+change-prompt(📎 Attach > )+refresh-preview+change-border-label( Tab: Mode | Enter: Select | Esc: Cancel )"
                        else
                            echo "reload(printf \"[No sessions]\")+change-prompt(📎 Attach > )+refresh-preview+change-border-label( Tab: Mode | Esc: Cancel )"
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
            # 세션 이름 입력
            local session_name
            session_name=$(echo "" | fzf \
                --ansi \
                --reverse \
                --border=sharp \
                --border-label-pos=bottom:4 \
                --no-clear \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="Session name > " \
                --border-label=" Enter: Create | Esc: Cancel " \
                --print-query --disabled \
                --preview 'printf "\033[36mNew Session\033[0m\n\nName: \033[32m{q}\033[0m"' |
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
                    --border=sharp \
                    --border-label-pos=bottom:4 \
                    --no-clear \
                    --preview-window=right:60%:border-left \
                    --preview-label=" Preview " \
                    --prompt="⚠️  Delete ALL? > " \
                    --border-label=" Enter: Confirm | Esc: Cancel " \
                    --preview 'printf "\033[31mWARNING\033[0m\n\nDelete ALL sessions!\n\n\033[33mCannot be undone.\033[0m"')
                [ "$confirm" = "Yes" ] && tmux kill-server
                continue
            fi

            [ "$selection" = "[No sessions]" ] && continue

            # 단일 세션 삭제 확인
            local confirm
            confirm=$(printf "No\nYes" | fzf \
                --ansi \
                --reverse \
                --border=sharp \
                --border-label-pos=bottom:4 \
                --no-clear \
                --preview-window=right:60%:border-left \
                --preview-label=" Preview " \
                --prompt="⚠️  Delete '$selection'? > " \
                --border-label=" Enter: Confirm | Esc: Cancel " \
                --preview 'printf "\033[31mConfirm Delete\033[0m\n\nSession: \033[33m'"$selection"'\033[0m"')
            [ "$confirm" = "Yes" ] && tmux kill-session -t "$selection"
            continue
            ;;
        esac
    done
}

# Alias for quick detach
alias td='tmux detach'
