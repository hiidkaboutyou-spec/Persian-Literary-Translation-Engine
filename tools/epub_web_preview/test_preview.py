from __future__ import annotations

import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from tools.epub_web_preview.build_preview import PreviewError, build_preview, read_epub


CONTAINER = """<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>
"""

OPF = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>کتاب آزمایشی</dc:title></metadata>
  <manifest>
    <item id="c1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine page-progression-direction="rtl"><itemref idref="c1"/><itemref idref="c2"/></spine>
</package>
"""

CH1 = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" lang="fa" dir="rtl">
<head><title>فصل یک</title><script>window.evil=true</script></head>
<body>
<h1 onclick="evil()">فصل یک</h1>
<p>سلام <strong>دنیا</strong>.</p>
<iframe src="https://example.com/"></iframe>
</body></html>
"""

CH2 = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" lang="fa" dir="rtl">
<head><title>فصل دو</title></head>
<body><h1>فصل دو</h1><p>متن دوم با <code>English()</code>.</p></body></html>
"""


def make_epub(path: Path) -> None:
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr("mimetype", "application/epub+zip")
        archive.writestr("META-INF/container.xml", CONTAINER)
        archive.writestr("OEBPS/content.opf", OPF)
        archive.writestr("OEBPS/ch1.xhtml", CH1)
        archive.writestr("OEBPS/ch2.xhtml", CH2)


class EpubWebPreviewTests(unittest.TestCase):
    def test_reads_spine_order_and_sanitizes_active_content(self):
        with tempfile.TemporaryDirectory() as temp:
            epub = Path(temp) / "book.epub"
            make_epub(epub)
            title, chapters = read_epub(epub)
            self.assertEqual(title, "کتاب آزمایشی")
            self.assertEqual([c.title for c in chapters], ["فصل یک", "فصل دو"])
            self.assertIn("<strong>دنیا</strong>", chapters[0].body_html)
            self.assertNotIn("<script", chapters[0].body_html)
            self.assertNotIn("<iframe", chapters[0].body_html)
            self.assertNotIn("onclick", chapters[0].body_html)
            self.assertNotIn("https://example.com", chapters[0].body_html)

    def test_build_is_self_contained_rtl_and_deterministic(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            epub = root / "book.epub"
            out = root / "preview"
            make_epub(epub)

            first = build_preview(epub, out)
            html_first = (out / "index.html").read_text(encoding="utf-8")
            manifest_first = (out / "manifest.json").read_text(encoding="utf-8")
            second = build_preview(epub, out)
            html_second = (out / "index.html").read_text(encoding="utf-8")
            manifest_second = (out / "manifest.json").read_text(encoding="utf-8")

            self.assertEqual(first, second)
            self.assertEqual(html_first, html_second)
            self.assertEqual(manifest_first, manifest_second)
            self.assertIn('<html lang="fa" dir="rtl">', html_first)
            self.assertIn("chapter-select", html_first)
            self.assertNotIn("http://", html_first)
            self.assertNotIn("https://", html_first)

            manifest = json.loads(manifest_first)
            self.assertEqual(manifest["chapter_count"], 2)
            self.assertFalse(manifest["network_dependencies"])
            self.assertTrue(manifest["contains_private_book_text"])
            self.assertFalse(manifest["canonical_edit_surface"])

    def test_missing_container_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            epub = Path(temp) / "broken.epub"
            with zipfile.ZipFile(epub, "w") as archive:
                archive.writestr("mimetype", "application/epub+zip")
            with self.assertRaises(PreviewError):
                read_epub(epub)


if __name__ == "__main__":
    unittest.main()
