"""Qt6 입력 칸 하나. 엔터를 누르면 입력된 글자를 파일에 쓰고 끝난다. 사용: qt-entry.py <창제목> <결과파일>"""

import sys

from PyQt6.QtWidgets import QApplication, QLineEdit

title, out = sys.argv[1], sys.argv[2]
app = QApplication(sys.argv[:1])
entry = QLineEdit()
entry.setWindowTitle(title)


def finish():
    with open(out, "w", encoding="utf-8") as f:
        f.write(entry.text())
    app.quit()


entry.returnPressed.connect(finish)
entry.show()
app.exec()
