# E2E 검증용 Kali rolling: 가상 X 서버, 세션 D-Bus, XIM 앱(xterm, GTK3), GTK4·Qt6 앱, 키 입력·화면 캡처 도구.
FROM kalilinux/kali-rolling
# 기본 미러가 HTTPS로 넘기는데 베이스 이미지에는 CA 인증서가 없어 공식 CDN을 HTTP로 쓴다(apt가 서명을 검증한다).
# 기본 저장소 설정은 sources.list 또는 sources.list.d/kali.sources에 있으므로 둘 다 바꾼다.
RUN rm -f /etc/apt/sources.list.d/* \
 && echo "deb http://kali.download/kali kali-rolling main contrib non-free non-free-firmware" > /etc/apt/sources.list \
 && apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      ca-certificates xvfb xterm xdotool x11-utils x11-apps locales dbus-daemon fonts-nanum \
      python3-gi gir1.2-gtk-3.0 gir1.2-gtk-4.0 python3-pyqt6 qt6-qpa-plugins \
 && sed -i 's/^# *en_US.UTF-8/en_US.UTF-8/' /etc/locale.gen && locale-gen \
 && rm -rf /var/lib/apt/lists/*
