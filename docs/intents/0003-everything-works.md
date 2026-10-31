# INTENT-0003 · 남은 빈틈 메우기: 조합 창, GTK4, ibus 공존, 한/영 키 설정, 글꼴 하나, 소개 페이지

- 상태: 완료
- 작성 / 갱신: 2026-10-31 / 2026-10-31

## 왜

0.2.0에는 빈틈이 있었다. xterm에서 조합 중 글자가 안 보이고, GTK4 앱은 입력이 안 되고, ibus가 깔려 있으면 Qt 입력이 꺼질 수 있었다. 사용자는 "싹 다 되게, 미니멀하게, ibus 재사용 말고 우리 것으로, 한/영 키만 설정 가능하게, 무료 글꼴 하나로, 설치 안내 HTML과 CI/CD"를 요청했다.

## 수용 기준

- [x] xterm에서 조합 중 글자가 커서 자리 맨 위 창에 보인다.
- [x] GTK4 앱에 두벌식이 입력된다(조합, 스페이스·엔터 앞 확정, 자모 단위 백스페이스). ibus 패키지 없이.
- [x] ibus가 깔려 있고 ibus-portal이 먼저 떠 있어도 Qt6 입력이 된다.
- [x] `~/.config/hangul-setting/config`의 `toggle = …`로 한/영 키를 바꾼다. 잘못 쓰면 줄 번호와 함께 알리고 기본값으로 계속 동작한다.
- [x] 패키지는 `fonts-nanum` 하나에 의존하고 GTK4를 강제로 설치하지 않는다.
- [x] 소개 페이지(설치 명령, 사용법, 검색 엔진용 메타·구조화 데이터·sitemap·robots)가 공개 URL에 있고, `main`의 `site/` 변경이 자동 게시된다.

## 비범위

Wayland 네이티브 앱, 세벌식·한자 변환, 설정 GUI, GTK4 앱 간 한/영 상태 공유.

---

## AI 판정

| 수용 기준 | 증거 | 판정 |
|---|---|---|
| xterm 조합 창 | E2E `통과 xterm 조합 창`: 루트 화면 캡처에서 창 영역에 조합 창만 긋는 밑줄과 글자 픽셀 확인(다른 창에 깔리면 실패). 스크린샷으로 `한글 입` 뒤 `려` 표시 확인 | 통과 |
| GTK4 입력 | E2E `통과 gtk4: 안녕 간`. 처리하지 않는 키는 GTK 기본 입력기로 넘겨 영문·스페이스가 들어간다 | 통과 |
| ibus 공존 | 일회성: ibus 설치, ibus-daemon·ibus-portal 먼저 실행 → hangul-setting이 이름을 넘겨받고 ibus-portal 종료 → Qt6 `한글` 입력 | 통과 |
| 한/영 키 설정 | `cargo test`: `config::tests` 4개, `engine::tests::configured_toggle_keys_replace_defaults`, `toggle_needs_exact_modifiers`, `hotkey_names_parse`, `bad_hotkey_names_are_errors` | 통과 |
| 의존성 | 0.3.0 deb `Depends: fonts-nanum, libc6 (>= 2.34)`. deb 설치 E2E에서 `통과 폰트: NanumSquare…`와 다섯 검사 통과 | 통과 |
| 소개 페이지 | 로컬: HTML 태그 짝·JSON-LD 검사, 헤드리스 Chromium 데스크톱·모바일 캡처, `publish.sh` 세 모드 검증. 공개: `site.yml` 게시 성공, 페이지 제목·캐노니컬·JSON-LD 응답, `robots.txt`·`sitemap.xml`·`install.sh`·공개키 200 | 통과 |
| 공개 배포 | `v0.3.0` 릴리스 워크플로(deb 설치 E2E 포함) 통과. 깨끗한 Kali 컨테이너에서 공개 `install.sh`로 0.3.0 설치 후 다섯 검사 통과 | 통과 |

남은 것: 없음. 검색 노출을 원하면 Google Search Console에 사이트를 등록하고 `sitemap.xml`을 제출한다(사용자 계정 작업).

남는 위험: gtk4-rs 0.11의 `get_preedit_string` 연결 함수가 NULL 출력 인자를 확인하지 않아(앱이 죽음) 그 vfunc를 우리 함수로 바꿔 끼웠다. gtk4-rs를 올릴 때 다시 확인한다. 조합 창은 화면이 32비트 픽셀일 때만 그린다(일반 데스크톱은 해당).
