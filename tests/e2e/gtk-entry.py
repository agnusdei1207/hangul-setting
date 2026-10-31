"""GTK3 입력 칸 하나. 엔터를 누르면 입력된 글자를 파일에 쓰고 끝난다. 사용: gtk-entry.py <창제목> <결과파일>"""

import sys

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

title, out = sys.argv[1], sys.argv[2]
window = Gtk.Window(title=title)
entry = Gtk.Entry()
window.add(entry)


def finish(e):
    with open(out, "w", encoding="utf-8") as f:
        f.write(e.get_text())
    Gtk.main_quit()


entry.connect("activate", finish)
window.connect("destroy", Gtk.main_quit)
window.show_all()
Gtk.main()
