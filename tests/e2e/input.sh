#!/bin/sh
# 실제 X 서버(Xvfb) 위의 앱(xterm·GTK3는 XIM, Qt6는 IBus 포털)에 xdotool로 두벌식을 쳐서 한글이 입력되는지 확인한다.
# 필요: tests/e2e/Dockerfile의 패키지, 세션 D-Bus(dbus-run-session 안에서 실행), PATH의 hangul-setting
# 입력기 환경 변수(XMODIFIERS, GTK_IM_MODULE, QT_IM_MODULE, IBUS_USE_PORTAL)가 이미 있으면 그대로 쓴다(패키지의 세션 스크립트 검증).
# HANGUL_SERVER_CMD가 있으면 그 명령으로 입력기를 띄운다(자동 시작 항목 검증).
set -eu

here=$(dirname "$0")
out=$(mktemp -d)
export DISPLAY=:99 LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
export XMODIFIERS="${XMODIFIERS:-@im=hangul}" GTK_IM_MODULE="${GTK_IM_MODULE:-xim}"
export QT_IM_MODULE="${QT_IM_MODULE:-ibus}" IBUS_USE_PORTAL="${IBUS_USE_PORTAL:-1}"

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
${HANGUL_SERVER_CMD:-hangul-setting} &
wait_for "XIM 서버 등록" sh -c 'xprop -root XIM_SERVERS | grep -q hangul'
wait_for "IBus 포털 등록" sh -c 'dbus-send --session --print-reply --dest=org.freedesktop.DBus / org.freedesktop.DBus.NameHasOwner string:org.freedesktop.portal.IBus | grep -q true'

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

# 백스페이스는 자모 단위, 엔터는 조합 중 글자를 먼저 확정한다. 엔터에 창이 닫히므로 한글 모드로 끝난다.
python3 "$here/gtk-entry.py" e2e-gtk "$out/gtk" &
check gtk3 e2e-gtk "$out/gtk" "안녕 간" \
    shift+space d k s s u d space r k BackSpace k s Return

# Qt6는 IBus 포털. 한/영 상태는 XIM과 함께 쓰므로 한글 모드로 시작한다.
python3 "$here/qt-entry.py" e2e-qt "$out/qt" &
check qt6 e2e-qt "$out/qt" "한일 간" \
    g k s d l f space r k BackSpace k s Return
