"""GTK4 입력 칸 하나. 엔터를 누르면 입력된 글자를 파일에 쓰고 끝난다. 사용: gtk4-entry.py <창제목> <결과파일>"""

import sys

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gtk  # noqa: E402

title, out = sys.argv[1], sys.argv[2]
loop = GLib.MainLoop()
window = Gtk.Window(title=title)
entry = Gtk.Entry()
window.set_child(entry)


def finish(e):
    with open(out, "w", encoding="utf-8") as f:
        f.write(e.get_text())
    loop.quit()


entry.connect("activate", finish)
window.present()
loop.run()
