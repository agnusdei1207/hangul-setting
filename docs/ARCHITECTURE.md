# 구조 지도

입력은 앱 종류마다 그 앱이 이미 가진 경로로 들어와 같은 엔진으로 모인다.

- XIM: GTK3·Firefox·Chromium·xterm → X 서버 → `xim_handler` → `engine` → `hangul`
- IBus 포털 규격: Qt6 → 세션 D-Bus `org.freedesktop.portal.IBus` → `ibus` → `engine` → `hangul`
- GTK4 모듈: GTK4 앱 프로세스 안의 `gtk4-im` → `engine` → `hangul`

| 모듈 | 책임 | 공개 인터페이스 요지 | 근거 |
|---|---|---|---|
| `src/hangul.rs` | 자모를 음절로 조합하는 상태 기계 | `Composer::{push, backspace, preedit, flush}` | 0001 |
| `src/engine.rs` | 키 하나를 확정·조합·통과로 판정, 한/영 모드, 두벌식 자판, 한/영 키 이름 해석 | `Engine::new(Vec<Hotkey>)`, `handle_key`, `Hotkey::parse`, `Modifiers::from_x_state` | 0001, 0003 |
| `src/config.rs` | 사용자 설정 파일(`toggle =`) 읽기 | `config::load`, `load_from`, `parse` | 0003 |
| `src/keymap.rs` | X 키코드를 기본 키심으로 변환 | `Keymap::{from_mapping, base_keysym}` | 0001 |
| `src/xim_handler.rs` | XIM 요청을 엔진에 연결. 입력 컨텍스트마다 조합기, 확정 뒤 넘길 키의 순서 보장, 앱이 그리지 않는 조합 중 글자는 조합 창으로 | `Handler` (`xim::ServerHandler`) | 0001, 0003 |
| `src/preedit_window.rs` | 커서 자리에 뜨는 조합 창. 나눔고딕을 fontdue로 그려 X 창에 올린다 | `PreeditWindow::{new, show, redraw}`, `render` | 0003 |
| `src/ibus.rs` | Qt 내장 ibus 플러그인이 붙는 D-Bus 규격 서비스. 입력 컨텍스트 생성·회수, 이름을 먼저 가진 ibus-portal에게서 넘겨받기 | `serve(Arc<Mutex<Engine>>)` | 0002, 0003 |
| `src/main.rs` | 설정을 읽어 엔진 하나를 만들고 XIM(메인 스레드)과 IBus 규격(별도 스레드)에 공유 | 바이너리 `hangul-setting` | 0001–0003 |
| `gtk4-im/` | GTK4 입력 모듈(cdylib). 앱 안에서 엔진을 돌리고 처리하지 않는 키는 GTK 기본 입력기로 넘긴다 | `libim-hangul-setting.so`, `GTK_IM_MODULE=hangul-setting` | 0003 |
| `packaging/` | deb의 세션 환경 변수·자동 시작·설치 후 안내, APT 저장소·사이트 게시 스크립트와 공개키 | `[package.metadata.deb]`, `apt/publish.sh`, `apt/install.sh` | 0001, 0003 |
| `site/` | 소개 페이지와 검색 엔진용 파일 | `index.html`, `robots.txt`, `sitemap.xml` | 0003 |
| `tests/e2e/` | Kali 컨테이너의 Xvfb·세션 D-Bus 위 xterm(조합 창 포함)·GTK3·Qt6·GTK4에 실제로 타이핑하는 검증 | `run.sh <.deb>` 또는 `run.sh <바이너리> <GTK4 모듈>` | 0001–0003 |

의존 방향: `main` → `xim_handler`·`ibus`·`preedit_window` → `engine`·`config` → `hangul`. `gtk4-im` → `engine`·`config` → `hangul`. `hangul`·`engine`·`config`·`keymap`은 X·D-Bus·GTK 없이 테스트한다.

배포: `main` 푸시 → `ci.yml`(게이트·E2E), `site/` 변경이면 `site.yml`이 페이지 게시. 태그 `v*` → `release.yml`이 테스트·빌드·deb·deb 설치 E2E → `publish.sh`가 gh-pages에 서명된 flat APT 저장소와 페이지 갱신 → GitHub Pages가 제공. 두 게시는 `gh-pages` 동시성 그룹으로 차례로 돈다.

순서 보장: 조합 중 글자를 확정하면서 그 키도 앱에 넘겨야 할 때(스페이스, 엔터 등) XIM은 확정을 동기로 보내고 앱의 SyncReply가 온 뒤 키를 넘긴다(libX11이 IM을 거친 키 표시를 한 비트로만 관리해 GTK3에서 순서가 뒤집히기 때문). IBus 규격은 확정 시그널을 메서드 응답보다 먼저 보내고, GTK4 모듈은 확정 시그널을 낸 뒤 키 처리 결과를 돌려준다.
