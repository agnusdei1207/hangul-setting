#!/bin/sh
# 실제 X 서버(Xvfb) 위의 XIM 앱(xterm, GTK3)에 xdotool로 두벌식을 쳐서 한글이 입력되는지 확인한다.
# 필요: Xvfb xterm xdotool x11-utils python3-gi gir1.2-gtk-3.0, en_US.UTF-8 로캘, PATH의 hangul-setting
set -eu

here=$(dirname "$0")
out=$(mktemp -d)
export DISPLAY=:99 LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 XMODIFIERS=@im=hangul GTK_IM_MODULE=xim

wait_for() { # 설명 명령...
    what=$1; shift
    i=0
    until "$@" >/dev/null 2>&1; do
        i=$((i + 1))
        [ "$i" -le 50 ] || { echo "시간 초과: $what" >&2; exit 1; }
        sleep 0.2
    done
}

Xvfb :99 -screen 0 1024x768x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill $xvfb 2>/dev/null' EXIT
wait_for "X 서버" xdpyinfo
hangul-setting &
wait_for "XIM 서버 등록" sh -c 'xprop -root XIM_SERVERS | grep -q hangul'

# 창이 뜨면 포커스를 주고 키를 친 뒤 결과 파일을 기다려 기대값과 비교한다.
check() { # 이름 창제목 결과파일 기대값 키...
    name=$1 title=$2 file=$3 expected=$4; shift 4
    wait_for "$name 창" xdotool search --name "^$title\$"
    sleep 0.5
    xdotool windowfocus --sync "$(xdotool search --name "^$title\$" | head -n1)"
    sleep 0.3
    xdotool key --delay 60 "$@"
    wait_for "$name 결과" test -s "$file"
    actual=$(cat "$file")
    if [ "$actual" = "$expected" ]; then
        echo "통과 $name: $actual"
    else
        echo "실패 $name: 기대 '$expected', 실제 '$actual'" >&2
        exit 1
    fi
}

# 한/영 전환, 조합, 스페이스 확정, 영문 통과
xterm -title e2e-xterm -xrm 'XTerm*inputMethod: hangul' \
    -e sh -c "IFS= read -r line; printf %s \"\$line\" > $out/xterm" &
check xterm e2e-xterm "$out/xterm" "한일 abc" \
    shift+space g k s d l f shift+space space a b c Return

# 백스페이스는 자모 단위, 엔터 앞에서 조합 중 글자 확정
python3 "$here/gtk-entry.py" e2e-gtk "$out/gtk" &
check gtk3 e2e-gtk "$out/gtk" "안녕 간" \
    shift+space d k s s u d space r k BackSpace k s Return shift+space
