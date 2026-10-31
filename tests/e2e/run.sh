#!/bin/sh
# 컨테이너 안에서 hangul-setting을 설치하고 실제 입력을 검증한다. 사용: run.sh <.deb 또는 바이너리>
set -eu
target=$1
here=$(dirname "$0")
case "$target" in
    *.deb)
        apt-get update -qq
        DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$target" >/dev/null
        # 패키지가 깐 세션 스크립트·자동 시작 항목·폰트를 그대로 쓴다.
        . /etc/X11/Xsession.d/90hangul-setting
        HANGUL_SERVER_CMD=$(sed -n 's/^Exec=//p' /etc/xdg/autostart/hangul-setting.desktop)
        export HANGUL_SERVER_CMD
        fonts=$(fc-list :lang=ko family | head -n1)
        [ -n "$fonts" ] || { echo "실패 폰트: 한글 폰트 없음" >&2; exit 1; }
        echo "통과 폰트: $fonts"
        ;;
    *) install -m 755 "$target" /usr/local/bin/hangul-setting ;;
esac
hangul-setting --version
exec sh "$here/xim-input.sh"
