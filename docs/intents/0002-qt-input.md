# INTENT-0002 · Qt 앱에서 한글 입력

- 상태: 완료
- 작성 / 갱신: 2026-10-31 / 2026-10-31

## 왜

Kali 기본 터미널 qterminal과 Wireshark는 Qt6 앱이다. Kali의 Qt6에는 입력기 플러그인이 `compose`와 `ibus` 둘뿐이라 XIM으로는 한글이 들어가지 않는다(2026-10-31 `libqt6gui6 6.11.2` 실측).

## 무엇

`hangul-setting`만 설치된 Kali에서 qterminal 등 Qt6 앱에 한/영 키로 두벌식 한글을 입력한다. ibus 패키지·데몬은 설치하지 않는다.

## 수용 기준

- [x] Kali 컨테이너의 Xvfb 위 Qt6 앱에 xdotool로 `Shift+Space g k s d l f Return`을 치면 `한일`이 입력된다.
- [x] 같은 세션에서 xterm·GTK3 E2E가 계속 통과한다.

## 제약

- Qt6에 이미 들어 있는 `ibus` 플러그인이 쓰는 IBus D-Bus 인터페이스를 `hangul-setting`이 직접 제공한다. ibus 라이브러리·데몬에 의존하지 않는다.
- 조합 로직은 `engine`·`hangul`을 그대로 쓴다.

## 비범위

GTK4, Wayland, ibus 설정 도구 호환.

## 결정

- 연결 방식은 IBus 포털(`IBUS_USE_PORTAL=1`)이다. Qt 플러그인의 일반 모드는 PATH에 `ibus-daemon` 실행 파일과 ibus 전용 버스 주소 파일을 요구한다. 포털 모드는 세션 버스 이름 하나만 요구하고, 입력기가 앱보다 늦게 떠도 Qt가 이름 등록을 감지해 다시 붙는다.

---

## AI 판정

| 수용 기준 | 증거 | 판정 |
|---|---|---|
| Qt6 앱 입력 | `tests/e2e/run.sh`(dbus-run-session, Xvfb): `통과 qt6: 한일 간` — 조합, 스페이스·엔터 앞 확정, 자모 단위 백스페이스, XIM과 공유한 한/영 상태. 일회성으로 실제 qterminal에서 `한글 인` 입력 확인 | 통과 |
| XIM 회귀 없음 | 같은 실행에서 `통과 xterm: 한일 abc`, `통과 gtk3: 안녕 간` | 통과 |

남은 것: 없음.

남는 위험: ibus가 설치돼 `ibus-portal`이 먼저 포털 이름을 차지하면 Qt 입력이 꺼진다(로그에 경고). 조합 중 글자 위치에 후보 창을 그리지 않는다(한글 입력에는 불필요).

