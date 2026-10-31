#!/bin/sh
# 컨테이너(rust:1.99-trixie) 안에서 게이트를 돌리고 바이너리·GTK4 모듈·deb을 만든다. compose.yaml의 build가 쓴다.
set -eu
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends libgtk-4-dev fonts-nanum >/dev/null
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
tools="$CARGO_TARGET_DIR/tools"
[ -x "$tools/bin/cargo-deb" ] || cargo install --locked --root "$tools" cargo-deb@3.8.0
rm -f "$CARGO_TARGET_DIR"/debian/hangul-setting_*.deb
"$tools/bin/cargo-deb" --no-build --locked
