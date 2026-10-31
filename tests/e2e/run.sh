#!/bin/sh
# 컨테이너 안에서 hangul-setting을 설치하고 실제 입력을 검증한다. 사용: run.sh <.deb 또는 바이너리>
set -eu
target=$1
case "$target" in
    *.deb)
        apt-get update -qq
        DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$target" >/dev/null
        ;;
    *) install -m 755 "$target" /usr/local/bin/hangul-setting ;;
esac
hangul-setting --version
exec sh "$(dirname "$0")/xim-input.sh"
