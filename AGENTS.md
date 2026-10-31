# 운영 계약

## 게이트

```sh
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

사용자 흐름이 닿는 변경(XIM, 패키징)은 E2E도 통과해야 한다. 리눅스 바이너리와 Kali 컨테이너가 필요하다.

```sh
docker build -t hangul-e2e tests/e2e
docker run --rm -v "$PWD:/src" hangul-e2e /src/tests/e2e/run.sh /src/<빌드한 .deb>
```

## 규칙

- 커밋: Conventional Commits(`feat:`, `fix:`, `docs:`, `ci:` …). 본문에 이유와 검증, 관련 인텐트 번호.
- 인텐트 `docs/intents/`, 용어집 `docs/GLOSSARY.md`, 구조 지도 `docs/ARCHITECTURE.md`. 코드 구조가 바뀌면 같은 커밋에서 갱신한다.
- `hangul`·`engine`·`keymap`은 X 연결에 의존하지 않는다. X 관련 코드는 `xim_handler`·`main`에만 둔다.
- 테스트는 각 모듈 아래 `#[cfg(test)]`, 실제 X 검증은 `tests/e2e/`.
- 줄 끝은 LF(`.gitattributes`).
- 릴리스: `Cargo.toml` 버전과 같은 `v<버전>` 태그를 푸시하면 CI가 `.deb`을 만들고 APT 저장소(GitHub Pages)에 게시한다.
