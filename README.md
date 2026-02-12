# tsm — tmux session manager

tmux 세션을 TUI로 관리하는 도구. Claude Code 상태 감지를 지원한다.

![demo](demo.gif)

## Features

- **세션 목록** — 디렉토리별 그룹핑, 실시간 갱신
- **Claude Code 감지** — 각 세션의 Claude Code 상태를 자동 감지 (Working / Waiting Input / Idle)
- **프리뷰** — 선택한 세션의 pane 내용을 ANSI 컬러 그대로 미리보기
- **액션 메뉴** — Switch / Rename / Kill 등 인라인 액션
- **세션 생성** — Claude Code 자동 실행 옵션, 경로 자동완성
- **필터링** — 세션 이름/경로로 즉시 검색
- **tmux 설정 적용** — `S` 키 한 번으로 권장 설정 일괄 적용

## Install

```bash
# cargo로 직접 설치
cargo install --path .

# 또는 install script (빌드 + ~/.local/bin 설치 + tmux 키바인딩 설정)
./install.sh
```

설치 후 tmux 안에서 `prefix + s`로 팝업 실행하거나, 터미널에서 `tsm` 직접 실행.

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` | 위/아래 이동 |
| `l` / `→` | 액션 메뉴 열기 |
| `Enter` | 세션 전환 (tsm 종료) |
| `Space` | 세션 전환 (tsm 유지) |
| `n` | 새 세션 생성 |
| `K` | 세션 삭제 |
| `r` | 세션 이름 변경 |
| `/` | 필터 |
| `R` | 새로고침 |
| `S` | tmux 설정 적용 |
| `p` | 프리뷰 토글 |
| `?` | 도움말 |
| `q` / `Esc` | 종료 |

## tmux 설정 (`S` 키)

`S`를 누르면 다음 설정이 적용된다:

```
set-option -g mouse on
set -g set-titles on
set -g set-titles-string "#{pane_title}"
set -g allow-rename on
```

## Build

```bash
cargo build              # dev
cargo build --release    # release
cargo test               # 테스트
```

## Record Demo

```bash
brew install vhs         # VHS 설치
./record-demo.sh         # demo.gif 생성
```

## Requirements

- Rust 1.70+
- tmux
- [Nerd Font](https://www.nerdfonts.com/) (아이콘 표시용)
