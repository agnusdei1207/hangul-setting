# INTENT-0000 · 프로젝트: hangul-setting

- 작성: 2026-10-31

## 왜

Kali Linux에서 한글을 쓰려면 ibus나 fcitx를 깔고, 데몬·환경 변수·im-config·엔진 설정을 맞추고, 폰트를 따로 찾아 깔아야 한다. 오래된 오픈 소스 엔진은 최신 배포판에서 자주 깨진다. 한 줄 설치로 끝나는 한글 입력기가 필요하다.

## 제약

- 설치는 `apt install hangul-setting` 한 줄이다. 설치 뒤 재로그인만 하면 한글이 입력되고 보인다. 추가 설정 파일 편집은 없다.
- 입력기는 Rust로 만든 단일 바이너리다. 런타임 라이브러리 의존(libxcb, glib, ibus, fcitx)이 없다.
- 1차 대상은 Kali 기본 데스크톱인 Xfce on X11(amd64)이다.
- 툴체인: Rust 1.99.0(2026-10-31 확인, stable). `rust-toolchain.toml`과 `Cargo.lock`으로 고정한다.

## 결정

- 입력 프로토콜은 XIM이다. X11의 표준 입력기 프로토콜이라 데몬·플러그인 없이 GTK3, Firefox, Chromium, xterm 등이 붙는다. ibus/fcitx 프레임워크를 다시 쓰지 않는다.
- 자판은 두벌식 표준 하나다. 한/영 전환은 `Hangul` 키, 오른쪽 `Alt`, `Shift+Space`다.
- 폰트는 직접 그리지 않는다. 11,172자 한글 음절을 쓸 만한 품질로 새로 만드는 일은 입력기 문제와 별개의 대형 작업이고, 배포판에 이미 유지보수되는 OFL 폰트(Noto Sans CJK, 나눔)가 있다. 패키지가 `fonts-noto-cjk | fonts-nanum`에 의존해 함께 설치되게 한다.
- 배포는 GitHub Pages에 올린 서명된 flat APT 저장소다. 태그 `v*` 푸시가 빌드와 게시를 일으킨다.

## 상시 승인 범위

- 게이트를 통과한 증분의 `main` 커밋·푸시.
- `v*` 태그 푸시로 APT 저장소 게시(사용자가 2026-10-31 패키지 게시를 지시함).
