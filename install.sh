#!/bin/bash

set -e

INSTALL_DIR="$HOME/.tmux-session-manager"
REPO_URL="https://raw.githubusercontent.com/Lucas20000903/tmux-session-manager/main"

echo "Installing tmux-session-manager..."

# 설치 디렉토리 생성
mkdir -p "$INSTALL_DIR"

# 스크립트 다운로드
download_file() {
    local url="$1"
    local dest="$2"

    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$dest"
    else
        echo "Error: curl or wget is required"
        exit 1
    fi
}

echo "Downloading scripts..."
download_file "$REPO_URL/tmux-session-manager.sh" "$INSTALL_DIR/tmux-session-manager.sh"
download_file "$REPO_URL/ts_preview.sh" "$HOME/.ts_preview.sh"

# preview 스크립트 실행 권한 부여
chmod +x "$HOME/.ts_preview.sh"

# 쉘 자동 감지 및 RC 파일 결정
detect_shell_rc() {
    if [ -n "$ZSH_VERSION" ]; then
        echo "$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            echo "$HOME/.bashrc"
        else
            echo "$HOME/.bash_profile"
        fi
    else
        case "$SHELL" in
            */zsh)
                echo "$HOME/.zshrc"
                ;;
            */bash)
                if [ -f "$HOME/.bashrc" ]; then
                    echo "$HOME/.bashrc"
                else
                    echo "$HOME/.bash_profile"
                fi
                ;;
            *)
                echo "$HOME/.profile"
                ;;
        esac
    fi
}

SHELL_RC=$(detect_shell_rc)

# RC 파일에 source 추가
SOURCE_LINE="source $INSTALL_DIR/tmux-session-manager.sh"

if [ -f "$SHELL_RC" ]; then
    if ! grep -q "tmux-session-manager.sh" "$SHELL_RC"; then
        echo "" >> "$SHELL_RC"
        echo "# tmux-session-manager" >> "$SHELL_RC"
        echo "$SOURCE_LINE" >> "$SHELL_RC"
        echo "Added to $SHELL_RC"
    else
        echo "Already exists in $SHELL_RC"
    fi
else
    echo "$SOURCE_LINE" > "$SHELL_RC"
    echo "Created $SHELL_RC"
fi

echo ""
echo "Installation complete!"
echo ""
echo "To start using it:"
echo "  1. Reload your shell: source $SHELL_RC"
echo "  2. Or restart your terminal"
echo ""
echo "Usage:"
echo "  ts  - Open tmux session manager"
echo "  td  - Detach from current session"
