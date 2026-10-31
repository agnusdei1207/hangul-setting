#!/bin/sh
# hangul-setting APT 저장소를 등록하고 설치한다.
# 사용: curl -fsSL https://agnusdei1207.github.io/hangul-setting/install.sh | sudo sh
set -eu
base=https://agnusdei1207.github.io/hangul-setting
keyring=/usr/share/keyrings/hangul-setting.gpg

[ "$(id -u)" -eq 0 ] || { echo "root 권한이 필요합니다: curl -fsSL $base/install.sh | sudo sh" >&2; exit 1; }
curl -fsSL "$base/hangul-setting.gpg" -o "$keyring"
echo "deb [signed-by=$keyring] $base ./" > /etc/apt/sources.list.d/hangul-setting.list
apt-get update
apt-get install -y hangul-setting
