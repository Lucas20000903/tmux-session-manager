# tmux-session-manager

fzf 기반의 tmux 세션 관리 도구입니다. 깔끔한 UI로 세션을 빠르게 전환, 생성, 삭제할 수 있습니다.

![demo](https://github.com/Lucas20000903/tmux-session-manager/raw/main/demo.gif)

## 주요 기능

- **세션 전환** - 기존 tmux 세션에 빠르게 연결
- **세션 생성** - 새 세션 또는 Claude가 자동 실행되는 세션 생성
- **세션 삭제** - 개별 또는 전체 세션 삭제
- **실시간 미리보기** - 세션 내용을 컬러로 미리보기
- **모드 전환** - Tab 키로 Attach/New/Manage 모드 간 빠른 전환

## 요구사항

- [tmux](https://github.com/tmux/tmux)
- [fzf](https://github.com/junegunn/fzf) (0.30.0+)
- zsh

## 설치

### 원라인 설치

```bash
curl -fsSL https://raw.githubusercontent.com/Lucas20000903/tmux-session-manager/main/install.sh | bash
```

### 수동 설치

1. 저장소 클론:
```bash
git clone https://github.com/Lucas20000903/tmux-session-manager.git ~/.tmux-session-manager
```

2. 프리뷰 스크립트 복사:
```bash
cp ~/.tmux-session-manager/ts_preview.sh ~/.ts_preview.sh
chmod +x ~/.ts_preview.sh
```

3. `.zshrc`에 추가:
```bash
echo 'source ~/.tmux-session-manager/tmux-session-manager.sh' >> ~/.zshrc
```

4. 쉘 재시작:
```bash
source ~/.zshrc
```

## 사용법

### 명령어

| 명령어 | 설명 |
|--------|------|
| `ts` | 세션 관리자 열기 |
| `td` | 현재 세션에서 detach |

### 키 바인딩

| 키 | 동작 |
|----|------|
| `Tab` | 모드 전환 (Attach → New → Manage → Attach) |
| `Enter` | 선택 실행 |
| `Esc` | 취소/종료 |
| `↑/↓` | 항목 이동 |

### 모드

#### › Attach 모드
기존 tmux 세션 목록을 보여주고 선택한 세션에 연결합니다.

#### + New 모드
새 세션을 생성합니다:
- **New Session** - 일반 tmux 세션
- **New Claude Session** - Claude가 자동 실행되는 세션

#### × Manage 모드
세션을 삭제합니다:
- 개별 세션 삭제
- 전체 세션 삭제 (Delete All)

## 제거

```bash
rm -rf ~/.tmux-session-manager
rm ~/.ts_preview.sh
# .zshrc에서 source 라인 제거
```

## 라이선스

MIT License
