# 구조 지도

입력 흐름: 앱 키 입력 → X 서버 → XIM 메시지 → `xim_handler` → `engine` → `hangul` → 확정·조합 중 글자 → XIM 메시지 → 앱.

| 모듈 | 책임 | 공개 인터페이스 요지 | 근거 |
|---|---|---|---|
| `src/hangul.rs` | 자모를 음절로 조합하는 상태 기계 | `Composer::{push, backspace, preedit, flush}`, `Jamo::from_compat` | 0001 |
| `src/engine.rs` | 키 하나를 확정·조합·통과로 판정, 한/영 모드, 두벌식 자판 | `Engine::handle_key(&mut Composer, Key) -> KeyOutcome` | 0001 |
| `src/keymap.rs` | X 키코드를 기본 키심으로 변환 | `Keymap::{from_mapping, base_keysym}` | 0001 |
| `src/xim_handler.rs` | XIM 요청을 엔진에 연결. 입력 컨텍스트마다 조합기, 확정 뒤 넘길 키의 순서 보장 | `Handler` (`xim::ServerHandler` 구현), `handle_sync_reply` | 0001 |
| `src/main.rs` | X 연결, `@im=hangul` 등록, 이벤트 루프 | 바이너리 `hangul-setting` | 0001 |
| `packaging/` | deb에 들어가는 세션 환경 변수·자동 시작 항목·설치 후 안내, APT 저장소 게시·설치 스크립트와 공개키 | `Cargo.toml`의 `[package.metadata.deb]`, `apt/publish.sh`, `apt/install.sh` | 0001 |
| `tests/e2e/` | Kali 컨테이너의 Xvfb 위 xterm·GTK3에 실제로 타이핑하는 검증 | `run.sh <.deb 또는 바이너리>` | 0001 |

배포: 태그 `v*` → `release.yml`이 테스트·deb 빌드·deb 설치 E2E → `publish.sh`가 gh-pages에 서명된 flat APT 저장소 갱신 → GitHub Pages가 제공.

의존 방향: `main` → `xim_handler` → `engine` → `hangul`. `hangul`·`engine`·`keymap`은 X 연결 없이 테스트한다.

순서 보장: 조합 중 글자를 확정하면서 그 키도 앱에 넘겨야 할 때(스페이스, 엔터 등) 확정을 동기로 보내고 앱의 SyncReply가 온 뒤 키를 넘긴다. libX11이 IM을 거친 키 표시를 한 비트로만 관리해, 둘을 한꺼번에 보내면 GTK3에서 순서가 뒤집히기 때문이다.
