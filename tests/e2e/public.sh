#!/bin/sh
# 공개 APT 저장소에서 README의 한 줄 설치 그대로 설치하고 실제 입력을 검사한다. compose.yaml의 public이 쓴다.
set -eu
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends curl >/dev/null
curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh | DEBIAN_FRONTEND=noninteractive sh >/dev/null
dpkg -l hangul-setting | tail -n1
. /etc/X11/Xsession.d/90hangul-setting
HANGUL_SERVER_CMD=$(sed -n 's/^Exec=//p' /etc/xdg/autostart/hangul-setting.desktop)
export HANGUL_SERVER_CMD
exec dbus-run-session -- sh "$(dirname "$0")/input.sh"
