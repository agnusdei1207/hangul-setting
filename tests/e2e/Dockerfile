# E2E 검증용 Kali rolling: 가상 X 서버, XIM 앱(xterm, GTK3), 키 입력 도구.
FROM kalilinux/kali-rolling
# 기본 미러가 HTTPS로 넘기는데 베이스 이미지에는 CA 인증서가 없어 공식 CDN을 HTTP로 쓴다(apt가 서명을 검증한다).
RUN echo "deb http://kali.download/kali kali-rolling main contrib non-free non-free-firmware" > /etc/apt/sources.list \
 && apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      ca-certificates xvfb xterm xdotool x11-utils locales python3-gi gir1.2-gtk-3.0 \
 && sed -i 's/^# *en_US.UTF-8/en_US.UTF-8/' /etc/locale.gen && locale-gen \
 && rm -rf /var/lib/apt/lists/*
