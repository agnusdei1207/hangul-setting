#!/bin/sh
# 실제 X 서버(Xvfb) 위의 앱에 xdotool로 두벌식을 쳐서 한글이 입력되는지 확인한다.
# xterm·GTK3는 XIM, GTK4는 우리 GTK4 모듈, Qt6는 IBus 포털 인터페이스로 붙는다.
# 필요: tests/e2e/Dockerfile의 패키지, 세션 D-Bus(dbus-run-session 안에서 실행), PATH의 hangul-setting
# 입력기 환경 변수(XMODIFIERS, GTK_IM_MODULE, QT_IM_MODULE, IBUS_USE_PORTAL)가 이미 있으면 그대로 쓴다(패키지의 세션 스크립트 검증).
# HANGUL_SERVER_CMD가 있으면 그 명령으로 입력기를 띄운다(자동 시작 항목 검증).
set -eu

here=$(dirname "$0")
out=$(mktemp -d)
export DISPLAY=:99 LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
export XMODIFIERS="${XMODIFIERS:-@im=hangul}" GTK_IM_MODULE="${GTK_IM_MODULE:-hangul-setting:xim}"
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

focus() { # 이름 창제목
    wait_for "$1 창" xdotool search --name "^$2\$"
    sleep 0.5
    xdotool windowfocus --sync "$(xdotool search --name "^$2\$" | head -n1)"
    sleep 0.3
}

# 결과 파일을 기다려 기대값과 비교한다.
expect() { # 이름 결과파일 기대값
    wait_for "$1 결과" test -s "$2"
    actual=$(cat "$2")
    if [ "$actual" = "$3" ]; then
        echo "통과 $1: $actual"
    else
        echo "실패 $1: 기대 '$3', 실제 '$actual'" >&2
        exit 1
    fi
}

# 조합 창이 화면 맨 위에 떠서 글자를 보여 주는지(xterm처럼 조합 중 글자 그리기를 입력기에 맡기는 앱).
preedit_drawn() {
    info=$(xwininfo -name hangul-setting-preedit) || return 1
    echo "$info" | grep -q IsViewable || return 1
    geometry=$(echo "$info" | awk '/Absolute upper-left X/{x=$NF} /Absolute upper-left Y/{y=$NF} /Width/{w=$NF} /Height/{h=$NF} END{print x, y, w, h}')
    # shellcheck disable=SC2086
    xwd -silent -root | python3 "$here/ink.py" $geometry
}

# xterm: 한/영 전환, 조합 창, 조합, 스페이스 확정, 영문 통과
xterm -title e2e-xterm -xrm 'XTerm*inputMethod: hangul' \
    -e sh -c "IFS= read -r line; printf %s \"\$line\" > $out/xterm" &
focus xterm e2e-xterm
xdotool key --delay 60 shift+space g k
wait_for "xterm 조합 창" preedit_drawn
echo "통과 xterm 조합 창: 조합 중 글자가 그려짐"
xdotool key --delay 60 s d l f shift+space space a b c Return
expect xterm "$out/xterm" "한일 abc"

# GTK3: 백스페이스는 자모 단위, 엔터는 조합 중 글자를 먼저 확정한다. 엔터에 창이 닫혀 한글 모드로 남는다.
python3 "$here/gtk-entry.py" e2e-gtk "$out/gtk" &
focus gtk3 e2e-gtk
xdotool key --delay 60 shift+space d k s s u d space r k BackSpace k s Return
expect gtk3 "$out/gtk" "안녕 간"

# Qt6: IBus 포털. 한/영 상태를 XIM과 함께 쓰므로 한글 모드로 시작한다.
python3 "$here/qt-entry.py" e2e-qt "$out/qt" &
focus qt6 e2e-qt
xdotool key --delay 60 g k s d l f space r k BackSpace k s Return
expect qt6 "$out/qt" "한일 간"

# GTK4: 앱 안의 우리 모듈. 한/영 상태는 앱마다 따로라 영문 모드로 시작한다.
# Xvfb에는 GPU가 없어 GTK4 기본 렌더러(GL)가 BadMatch로 죽는다. 입력과 무관하므로 소프트웨어 렌더러로 띄운다.
GSK_RENDERER=cairo python3 "$here/gtk4-entry.py" e2e-gtk4 "$out/gtk4" &
focus gtk4 e2e-gtk4
xdotool key --delay 60 shift+space d k s s u d space r k BackSpace k s Return
expect gtk4 "$out/gtk4" "안녕 간"
