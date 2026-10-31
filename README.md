# hangul-setting

Kali Linux(Xfce, X11)용 한글 입력기. ibus·fcitx 없이 Rust 바이너리 하나가 XIM 입력기로 동작한다. 두벌식 표준 자판.

## 설치

준비 중(INTENT-0001). APT 저장소가 열리면 여기에 한 줄 설치 명령을 적는다.

## 사용

- 한/영 전환: `한/영`(Hangul) 키, 오른쪽 `Alt`, `Shift+Space`
- 지원: GTK3 앱, Firefox, Chromium, xterm 등 XIM을 쓰는 X11 앱
- 미지원: Wayland 네이티브 입력, GTK4 앱

## 개발

게이트와 E2E 실행법은 [AGENTS.md](AGENTS.md), 구조는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), 목표와 결정은 [docs/intents/](docs/intents/).
