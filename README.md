# hangul-setting

Kali Linux(Xfce, X11)용 한글 입력기. ibus·fcitx 없이 apt 한 줄로 두벌식 한글 입력과 무료 한글 글꼴을 설치한다. Rust로 만들었다.

소개·설치 페이지: https://agnusdei1207.github.io/hangul-setting/

## 설치

```sh
curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh | sudo sh
```

설치 후 다시 로그인한다. 한/영 전환: `한/영` 키, 오른쪽 `Alt`, `Shift+Space`.

## 한/영 전환 키 바꾸기

`~/.config/hangul-setting/config`:

```text
toggle = Ctrl+space, Hangul
```

이름: `Hangul`, `Alt_R`, `Control_R`, `Super_R`, `Caps_Lock`, `Menu`, `space`, `F1`~`F12`, 글자 한 개. 앞에 `Shift+`, `Ctrl+`, `Alt+`, `Super+`. 바꾼 뒤 다시 로그인한다.

## 되는 곳

- GTK3 앱, Firefox, Chromium, xterm 등 XIM 앱. 조합 중 글자를 스스로 그리지 않는 앱에는 커서 자리에 조합 창을 띄운다.
- GTK4 앱: Rust로 만든 GTK4 입력 모듈.
- qterminal·Wireshark 등 Qt6 앱: Qt에 내장된 ibus 플러그인이 쓰는 D-Bus 규격을 hangul-setting이 직접 제공한다. ibus는 필요 없고, 깔려 있어도 함께 동작한다.
- 한/영 상태는 모든 앱이 함께 쓴다(GTK4 앱은 앱마다 따로).
- 미지원: Wayland 네이티브 앱.

설치되는 것: `/usr/bin/hangul-setting`, GTK4 모듈, 세션 환경 변수 `/etc/X11/Xsession.d/90hangul-setting`, 자동 시작 `/etc/xdg/autostart/hangul-setting.desktop`, 나눔 글꼴(`fonts-nanum`, SIL OFL 1.1).

## 개발

게이트·E2E·릴리스는 [AGENTS.md](AGENTS.md), 구조는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), 목표와 결정은 [docs/intents/](docs/intents/).
