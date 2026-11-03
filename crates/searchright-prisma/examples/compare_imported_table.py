"""Compare original synthetic HTML table cells with locally imported DOCX cells.

Read-only diagnostic; does not invoke a renderer or access the network.
Usage: python3 compare_imported_table.py fixture.html fixture.docx
"""

from html.parser import HTMLParser
from pathlib import Path
import sys
import xml.etree.ElementTree as ET
import zipfile


class Cells(HTMLParser):
    def __init__(self):
        super().__init__()
        self.cells = []
        self.current = None

    def handle_starttag(self, tag, attrs):
        if tag in ("th", "td"):
            self.current = ""

    def handle_data(self, data):
        if self.current is not None:
            self.current += data

    def handle_endtag(self, tag):
        if tag in ("th", "td"):
            self.cells.append(self.current)
            self.current = None


def main():
    if len(sys.argv) != 3:
        raise SystemExit("expected fixture.html fixture.docx")
    html = Cells()
    html.feed(Path(sys.argv[1]).read_text(encoding="utf-8"))
    with zipfile.ZipFile(sys.argv[2]) as archive:
        document = ET.fromstring(archive.read("word/document.xml"))
    ns = {"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
    cells = [
        "".join(text.text or "" for text in cell.findall(".//w:t", ns))
        for cell in document.findall(".//w:tbl/w:tr/w:tc", ns)
    ]
    if not html.cells or cells != html.cells:
        raise SystemExit("ordered table text differs or HTML table is empty")
    print(f"{len(cells)} ordered cells match exactly without normalization")


if __name__ == "__main__":
    main()
