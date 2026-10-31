# INTENT-0001 · Kali에서 apt 한 줄로 한글 입력·표시

- 상태: 완료
- 작성 / 갱신: 2026-10-31 / 2026-10-31

## 왜

ibus 설치·설정·폰트 설치가 번거롭고 자주 깨진다.

## 무엇

Kali(Xfce, X11)에서 APT 저장소를 등록하고 `sudo apt install hangul-setting` 후 재로그인하면, GTK3 앱·브라우저·xterm에서 한/영 키로 두벌식 한글을 입력하고 한글이 네모 없이 보인다.

## 수용 기준

- [x] 두벌식 조합: `gksdlf` → `한일`, `dkssud` → `안녕`, `rkqt` → `값`, `ekfr` 뒤 `k` → `달가`, `hk` → `와`, 백스페이스는 자모 한 단위씩 지운다.
- [x] 한글 모드에서 조합 중 비한글 키(스페이스·엔터·숫자·Ctrl 조합)를 누르면 조합 중인 글자를 먼저 확정하고 그 키를 앱에 넘긴다.
- [x] `Hangul`·오른쪽 `Alt`·`Shift+Space`로 한/영이 전환되고, 영문 모드에서는 모든 키가 그대로 앱에 간다.
- [x] 실제 X 서버 위의 XIM 클라이언트(xterm, GTK3 앱)에 xdotool로 `gksdlf`을 치면 `한일`이 입력된다.
- [x] `.deb`를 Kali rolling 컨테이너에 설치하면 바이너리, 세션 환경 변수, 자동 시작 항목, 한글 폰트가 깔린다.
- [x] 공개 URL의 APT 저장소에서 `apt install hangul-setting`이 서명 검증을 통과해 설치된다.

## 제약

프로젝트 인텐트의 제약을 따른다.

## 비범위

- Wayland 네이티브 입력(input-method-v2, text-input-v3), GNOME on Wayland.
- GTK4 앱(GTK4는 XIM을 지원하지 않는다).
- 한자 변환, 세벌식 등 다른 자판, 설정 GUI.
- 새 폰트 제작.

## 열린 질문 → 결정

- Q: 폰트를 직접 만들 것인가. 결정: 만들지 않고 기존 OFL 폰트 패키지에 의존한다(프로젝트 인텐트 결정 참고). 사용자에게 보고한다.

---

## AI 판정

| 수용 기준 | 증거 | 판정 |
|---|---|---|
| 두벌식 조합 | `cargo test`: `hangul::tests` 11개, `engine::tests::dubeolsik_composes_words` 등 | 통과 |
| 비한글 키 확정 | `engine::tests::non_hangul_key_commits_then_passes_through`, `command_shortcut_…`; E2E gtk3 `안녕 간`(스페이스·엔터 앞 확정) | 통과 |
| 한/영 전환 | `engine::tests::toggle_keys_switch_mode_and_commit`(세 키 모두); E2E는 `Shift+Space`만 실측 | 통과 |
| 실제 XIM 클라이언트 입력 | `tests/e2e/run.sh`, Kali rolling + Xvfb: `통과 xterm: 한일 abc`, `통과 gtk3: 안녕 간` | 통과 |
| deb 설치 내용 | `run.sh <.deb>`: Kali 컨테이너에 apt로 설치, 패키지의 `90hangul-setting`을 source하고 자동 시작 `Exec`으로 입력기 실행 → `통과 폰트: Noto Serif CJK JP`, `통과 xterm: 한일 abc`, `통과 gtk3: 안녕 간` | 통과 |
| 공개 APT 저장소 설치 | `v0.1.0` 릴리스 워크플로 통과(deb 설치 E2E 포함). 깨끗한 Kali 컨테이너에서 `curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh \| sh` → `InRelease` 서명 검증, `fonts-noto-cjk hangul-setting` 설치, 이어서 xterm·GTK3 E2E 통과 | 통과 |

남은 것: 없음. Qt 앱은 INTENT-0002.

남는 위험: xterm이 XIM 오류(`BadSomething`)를 두 번 보낸다. 입력에는 영향이 없다. Qt 앱(Kali 기본 터미널 qterminal 포함)의 XIM 지원은 아직 확인하지 않았다.
