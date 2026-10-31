# INTENT-0000 · 프로젝트: hangul-setting

- 작성 / 갱신: 2026-10-31 / 2026-10-31

## 왜

Kali Linux에서 한글을 쓰려면 ibus나 fcitx를 깔고, 데몬·환경 변수·im-config·엔진 설정을 맞추고, 폰트를 따로 찾아 깔아야 한다. 오래된 오픈 소스 엔진은 최신 배포판에서 자주 깨진다. 한 줄 설치로 끝나는 한글 입력기가 필요하다.

## 제약

- 설치는 `apt install hangul-setting` 한 줄이다. 설치 뒤 재로그인만 하면 한글이 입력되고 보인다. 설정은 한/영 전환 키 하나만 둔다(선택).
- 입력기는 Rust로 만든다. ibus·fcitx 패키지·데몬·라이브러리에 의존하지 않는다. 서버 바이너리는 libc만 쓴다.
- 1차 대상은 Kali 기본 데스크톱인 Xfce on X11(amd64)이다.
- 최소 구성: 기능을 더하기 전에 뺄 것을 찾는다.
- 툴체인: Rust 1.99.0(2026-10-31 확인, stable). `rust-toolchain.toml`과 `Cargo.lock`으로 고정한다.

## 결정

- 앱 종류마다 그 앱이 이미 가진 입력 경로에 붙는다. 새 프레임워크를 앱에 깔게 하지 않는다.
  - GTK3·Firefox·Chromium·xterm 등: XIM(X11 표준). 조합 중 글자를 스스로 그리지 않는 앱에는 커서 자리에 조합 창을 띄운다.
  - Qt6: Qt에 내장된 `ibus` 플러그인이 쓰는 D-Bus 규격(포털 모드)을 우리 서버가 직접 말한다. Qt 플러그인은 C++ ABI라 Rust로 따로 만들지 않는다. ibus 소프트웨어는 쓰지도 깔지도 않는다.
  - GTK4: XIM이 없어 Rust로 만든 GTK4 입력 모듈을 싣는다.
- 자판은 두벌식 표준 하나다. 한/영 전환 기본값은 `Hangul`, 오른쪽 `Alt`, `Shift+Space`이고 `~/.config/hangul-setting/config`의 `toggle =` 한 줄로 바꾼다.
- 글꼴은 직접 그리지 않는다. 무료(SIL OFL 1.1) 나눔 글꼴 `fonts-nanum` 하나에 의존한다. 한글 전용이라 Noto CJK보다 작고(41MB) 터미널용 D2Coding을 포함한다. 조합 창도 나눔고딕으로 그린다.
- 배포는 GitHub Pages에 올린 서명된 flat APT 저장소와 소개 페이지다. 태그 `v*`가 패키지를, `main`의 `site/` 변경이 페이지를 게시한다.

## 상시 승인 범위

- 게이트를 통과한 증분의 `main` 커밋·푸시.
- `v*` 태그 푸시로 APT 저장소 게시, `main` 푸시로 소개 페이지 게시(사용자가 2026-10-31 지시함).
