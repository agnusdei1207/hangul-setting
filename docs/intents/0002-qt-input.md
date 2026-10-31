# INTENT-0002 · Qt 앱에서 한글 입력

- 상태: 초안
- 작성 / 갱신: 2026-10-31 / 2026-10-31

## 왜

Kali 기본 터미널 qterminal과 Wireshark는 Qt6 앱이다. Kali의 Qt6에는 입력기 플러그인이 `compose`와 `ibus` 둘뿐이라 XIM으로는 한글이 들어가지 않는다(2026-10-31 `libqt6gui6 6.11.2` 실측).

## 무엇

`hangul-setting`만 설치된 Kali에서 qterminal 등 Qt6 앱에 한/영 키로 두벌식 한글을 입력한다. ibus 패키지·데몬은 설치하지 않는다.

## 수용 기준

- [ ] Kali 컨테이너의 Xvfb 위 Qt6 앱에 xdotool로 `Shift+Space g k s d l f Return`을 치면 `한일`이 입력된다.
- [ ] 같은 세션에서 xterm·GTK3 E2E가 계속 통과한다.

## 제약

- Qt6에 이미 들어 있는 `ibus` 플러그인이 쓰는 IBus D-Bus 인터페이스를 `hangul-setting`이 직접 제공한다. ibus 라이브러리·데몬에 의존하지 않는다.
- 조합 로직은 `engine`·`hangul`을 그대로 쓴다.

## 비범위

GTK4, Wayland, ibus 설정 도구 호환.
