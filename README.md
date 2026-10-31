# hangul-setting

Kali Linux용 한글 입력기. **ibus·fcitx 없이 apt 한 줄**로 두벌식 한글 입력과 무료 한글 글꼴을 설치한다. Rust로 만들었다.

- 패키지 이름: `hangul-setting`
- 대상: Kali Linux 기본 데스크톱 Xfce(X11), amd64. Debian 계열 X11 데스크톱도 같다.
- 소개 페이지: https://agnusdei1207.github.io/hangul-setting/

## 설치

Kali 공식 저장소가 아니라 이 프로젝트의 APT 저장소에 있다. 저장소를 한 번 등록하면 그다음부터는 평소처럼 `apt`로 설치·업데이트한다.

### 한 줄 설치 (저장소 등록 + 설치)

```sh
curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh | sudo sh
```

### 직접 등록하고 apt로 설치

```sh
# 1. 저장소 서명 키 받기
sudo curl -fsSL https://agnusdei1207.github.io/hangul-setting/hangul-setting.gpg -o /usr/share/keyrings/hangul-setting.gpg

# 2. 저장소 등록
echo "deb [signed-by=/usr/share/keyrings/hangul-setting.gpg] https://agnusdei1207.github.io/hangul-setting ./" | sudo tee /etc/apt/sources.list.d/hangul-setting.list

# 3. 설치
sudo apt update
sudo apt install hangul-setting
```

설치가 끝나면 **로그아웃했다가 다시 로그인**한다.

### 업데이트와 삭제

```sh
sudo apt update && sudo apt upgrade                  # 새 버전 받기
sudo apt purge hangul-setting                        # 삭제
sudo rm /etc/apt/sources.list.d/hangul-setting.list /usr/share/keyrings/hangul-setting.gpg   # 저장소 등록도 지우기
```

## 사용

| 할 일 | 키 |
|---|---|
| 한글 ↔ 영문 | `한/영` 키, 오른쪽 `Alt`, `Shift+Space` (셋 중 아무거나) |
| 자판 | 두벌식 표준 |
| 조합 중 글자 지우기 | `Backspace` (자모 하나씩) |

한/영 상태는 모든 앱이 함께 쓴다(GTK4 앱만 앱마다 따로).

### 잘 되는지 확인

1. 다시 로그인한 뒤 터미널(qterminal)이나 Firefox를 연다.
2. `Shift+Space`로 한글을 켠다.
3. `gksdlf`을 치면 `한일`, `dkssud`을 치면 `안녕`이 나온다.

## 한/영 전환 키 바꾸기

`~/.config/hangul-setting/config` 파일을 만들고 한 줄을 쓴 뒤 다시 로그인한다. 쉼표로 여러 개를 쓸 수 있고, 적은 키만 쓰인다(기본값을 대신한다).

```text
toggle = Ctrl+space, Hangul
```

| 쓸 수 있는 키 이름 | 앞에 붙일 수 있는 수식 키 |
|---|---|
| `Hangul`, `Hangul_Hanja`, `Alt_R`, `Alt_L`, `Control_R`, `Control_L`, `Super_R`, `Super_L`, `Shift_R`, `Shift_L`, `Caps_Lock`, `Menu`, `space`, `Tab`, `Escape`, `Pause`, `F1`~`F12`, 글자 한 개(`a`, `1`) | `Shift+`, `Ctrl+`, `Alt+`, `Super+` |

이름을 잘못 쓰면 몇 번째 줄이 틀렸는지 로그(`journalctl --user` 또는 `~/.xsession-errors`)에 남기고 기본 키로 계속 동작한다.

## 되는 앱

| 앱 | 연결 방식 |
|---|---|
| 터미널(qterminal, xfce4-terminal, xterm), Firefox, Chromium, GTK3 앱 | XIM. 조합 중 글자를 스스로 그리지 않는 앱(xterm 등)에는 커서 자리에 조합 창을 띄운다 |
| Qt6 앱(qterminal, Wireshark 등) | Qt에 내장된 입력기 플러그인이 쓰는 D-Bus 규격을 hangul-setting이 직접 제공한다. ibus는 필요 없고, 깔려 있어도 함께 동작한다 |
| GTK4 앱 | Rust로 만든 GTK4 입력 모듈 |

지원하지 않는 것: Wayland 세션의 네이티브 앱, 세벌식, 한자 변환.

## 설치되는 것

| 파일 | 역할 |
|---|---|
| `/usr/bin/hangul-setting` | 입력기 서버(로그인할 때 자동 시작) |
| `/usr/lib/x86_64-linux-gnu/gtk-4.0/4.0.0/immodules/libim-hangul-setting.so` | GTK4 입력 모듈 |
| `/etc/X11/Xsession.d/90hangul-setting` | 세션 환경 변수: `XMODIFIERS=@im=hangul`, `GTK_IM_MODULE=hangul-setting:xim`, `QT_IM_MODULE=ibus`, `IBUS_USE_PORTAL=1` |
| `/etc/xdg/autostart/hangul-setting.desktop` | 로그인할 때 입력기 시작 |
| `fonts-nanum` 패키지 | 무료 나눔 글꼴(SIL OFL 1.1). 터미널용 D2Coding 포함 |

## 문제 해결

| 증상 | 확인 |
|---|---|
| 한글이 전혀 안 켜진다 | 설치 후 다시 로그인했는지. `pgrep -a hangul-setting`으로 입력기가 떠 있는지. 안 떠 있으면 `hangul-setting &`로 직접 띄우고 앱을 새로 연다 |
| 환경 변수가 안 잡혔다 | `echo $XMODIFIERS $GTK_IM_MODULE $QT_IM_MODULE`가 `@im=hangul hangul-setting:xim ibus`인지. 다른 입력기 설정(`~/.xinputrc`, `~/.profile`의 `GTK_IM_MODULE` 등)이 덮어쓰고 있지 않은지 |
| 한글이 네모로 보인다 | `fc-list :lang=ko`에 Nanum이 나오는지. 없으면 `sudo apt install --reinstall fonts-nanum` |
| WSL(WSLg)에서 안 된다 | WSLg는 Wayland라 GTK·Qt 앱이 Wayland로 뜬다. `hangul-setting &` 후 `export XMODIFIERS=@im=hangul GTK_IM_MODULE=hangul-setting:xim QT_IM_MODULE=ibus IBUS_USE_PORTAL=1 GDK_BACKEND=x11 QT_QPA_PLATFORM=xcb`로 X11로 띄운다 |
| 로그를 보고 싶다 | `pkill hangul-setting; RUST_LOG=debug hangul-setting` |

## 개발

| 문서 | 내용 |
|---|---|
| [AGENTS.md](AGENTS.md) | 게이트(포맷·린트·테스트), 실제 화면 E2E, 릴리스 방법 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 모듈 구조와 입력 흐름 |
| [docs/intents/](docs/intents/) | 목표·결정·검증 기록 |
| [docs/GLOSSARY.md](docs/GLOSSARY.md) | 용어 |

CI/CD: `main` 푸시마다 테스트와 Kali 컨테이너 E2E(xterm·GTK3·Qt6·GTK4에 실제 타이핑)가 돈다. `v*` 태그를 붙이면 deb을 빌드·검증해 APT 저장소(GitHub Pages)에 게시하고, `site/`를 바꾸면 소개 페이지가 다시 게시된다.

라이선스: MIT. 글꼴은 나눔 글꼴(SIL OFL 1.1, `fonts-nanum` 패키지).
