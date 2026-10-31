#!/bin/sh
# 컨테이너 안에서 hangul-setting을 설치하고 실제 입력을 검증한다.
# 사용: run.sh <.deb>  또는  run.sh <바이너리> <GTK4 모듈 .so>
set -eu
here=$(dirname "$0")
case "$1" in
    *.deb)
        apt-get update -qq
        DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$1" >/dev/null
        # 패키지가 깐 세션 스크립트·자동 시작 항목·폰트를 그대로 쓴다.
        . /etc/X11/Xsession.d/90hangul-setting
        HANGUL_SERVER_CMD=$(sed -n 's/^Exec=//p' /etc/xdg/autostart/hangul-setting.desktop)
        export HANGUL_SERVER_CMD
        font=$(fc-list :lang=ko family | grep -m1 Nanum) || { echo "실패 폰트: 나눔 폰트 없음" >&2; exit 1; }
        echo "통과 폰트: $font"
        ;;
    *)
        install -m 755 "$1" /usr/local/bin/hangul-setting
        install -D -m 644 "$2" /usr/lib/x86_64-linux-gnu/gtk-4.0/4.0.0/immodules/libim-hangul-setting.so
        ;;
esac
hangul-setting --version
exec dbus-run-session -- sh "$here/input.sh"
