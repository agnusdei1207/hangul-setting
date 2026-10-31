# 운영 계약

## 한 번에 검증 (Docker Compose, 권장)

```sh
docker compose up --build --abort-on-container-exit --exit-code-from e2e   # 게이트 → 빌드 → deb → Kali에 설치해 실제 입력 검사
docker compose run --rm public                                             # 공개 APT 저장소에서 설치해 실제 입력 검사
```

`build`(rust:1.99-trixie + GTK4 개발 라이브러리)가 `tests/build.sh`로 포맷·린트·테스트·빌드·deb을 하고, `e2e`(Kali)가 그 deb을 설치해 xterm 조합 창·xterm·GTK3·Qt6·GTK4에 실제로 타이핑한다. 로컬 검증은 WSL이 아니라 이 컨테이너로 한다.

## 게이트

윈도우·맥 등 GTK4 개발 라이브러리가 없는 곳(기본 멤버만):

```sh
cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

리눅스·CI(워크스페이스 전체, `libgtk-4-dev` 필요):

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked
```

GTK4 개발 라이브러리가 없으면 Docker로 빌드한다(`rust:1.99-trixie`에 `libgtk-4-dev`).

사용자 흐름이 닿는 변경(XIM, IBus 규격, GTK4 모듈, 조합 창, 패키징)은 E2E도 통과해야 한다.

```sh
docker build -t hangul-e2e tests/e2e
docker run --rm -v "$PWD:/src" hangul-e2e sh /src/tests/e2e/run.sh /src/target/release/hangul-setting /src/target/release/libhangul_gtk4_im.so
docker run --rm -v "$PWD:/src" hangul-e2e sh /src/tests/e2e/run.sh /src/<빌드한 .deb>
```

## 규칙

- 커밋: Conventional Commits(`feat:`, `fix:`, `docs:`, `ci:` …). 본문에 이유와 검증, 관련 인텐트 번호.
- 인텐트 `docs/intents/`, 용어집 `docs/GLOSSARY.md`, 구조 지도 `docs/ARCHITECTURE.md`. 코드 구조가 바뀌면 같은 커밋에서 갱신한다.
- `hangul`·`engine`·`config`·`keymap`은 X·D-Bus·GTK에 의존하지 않는다. 연결 코드는 `xim_handler`·`ibus`·`preedit_window`·`main`·`gtk4-im`에만 둔다.
- 테스트는 각 모듈 아래 `#[cfg(test)]`, 실제 화면 검증은 `tests/e2e/`.
- 줄 끝은 LF(`.gitattributes`).
- 릴리스: `Cargo.toml`·`gtk4-im/Cargo.toml` 버전을 올리고 같은 `v<버전>` 태그를 푸시하면 CI가 deb을 만들어 APT 저장소(GitHub Pages)에 게시한다. 소개 페이지는 `site/`를 `main`에 푸시하면 게시된다.
