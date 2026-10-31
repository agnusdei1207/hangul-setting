# hangul-setting

Kali Linux(Xfce, X11)용 한글 입력기. ibus·fcitx 없이 Rust 바이너리 하나가 동작한다. 두벌식 표준 자판.

## 설치

```sh
curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh | sudo sh
```

설치 후 로그아웃했다가 다시 로그인한다. 한글 폰트(`fonts-noto-cjk`)도 함께 설치된다.

스크립트 없이 직접 등록하려면:

```sh
sudo curl -fsSL https://agnusdei1207.github.io/hangul-setting/hangul-setting.gpg -o /usr/share/keyrings/hangul-setting.gpg
echo "deb [signed-by=/usr/share/keyrings/hangul-setting.gpg] https://agnusdei1207.github.io/hangul-setting ./" | sudo tee /etc/apt/sources.list.d/hangul-setting.list
sudo apt update && sudo apt install hangul-setting
```

설치되는 것: `/usr/bin/hangul-setting`, 세션 환경 변수 `/etc/X11/Xsession.d/90hangul-setting`(`XMODIFIERS=@im=hangul`, `GTK_IM_MODULE=xim`, `QT_IM_MODULE=ibus`, `IBUS_USE_PORTAL=1`), 자동 시작 `/etc/xdg/autostart/hangul-setting.desktop`.

## 사용

- 한/영 전환: `한/영`(Hangul) 키, 오른쪽 `Alt`, `Shift+Space`
- 지원: GTK3 앱, Firefox, Chromium, xterm(XIM), qterminal·Wireshark 등 Qt6 앱(Qt에 내장된 ibus 플러그인이 붙는 IBus 포털 인터페이스를 직접 제공한다. ibus 패키지는 필요 없다)
- 한/영 상태는 모든 앱이 함께 쓴다
- 미지원: Wayland 네이티브 입력, GTK4 앱

## 개발

게이트와 E2E 실행법은 [AGENTS.md](AGENTS.md), 구조는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), 목표와 결정은 [docs/intents/](docs/intents/).
