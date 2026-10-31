"""조합 창이 화면 맨 위에 실제로 보이는지 본다. 사용: xwd -root | ink.py <x> <y> <폭> <높이>

화면 전체(XWD, 32비트 픽셀)에서 조합 창 영역을 잘라, 조합 창만 긋는 가로 밑줄(한 줄이 거의 다 검정)과
글자 픽셀이 있으면 종료 코드 0. 창이 다른 창 아래에 깔리면 밑줄이 보이지 않는다."""

import struct
import sys

x0, y0, w, h = map(int, sys.argv[1:5])
data = sys.stdin.buffer.read()
fields = struct.unpack(">25I", data[:100])
header_size, bits_per_pixel, bytes_per_line, ncolors = fields[0], fields[11], fields[12], fields[19]
if bits_per_pixel != 32:
    sys.exit(f"32비트 픽셀이 아님: {bits_per_pixel}")
pixels = data[header_size + ncolors * 12:]


def dark(x, y):
    i = y * bytes_per_line + x * 4
    return max(pixels[i:i + 3]) < 100


rows = [sum(dark(x0 + x, y0 + y) for x in range(w)) for y in range(h)]
underline = any(n >= 0.8 * w for n in rows)
ink = sum(rows)
print(f"밑줄 {underline}, 어두운 픽셀 {ink}")
sys.exit(0 if underline and ink > 2 * w else 1)
