#!/usr/bin/env python3
"""Build a local, dependency-free web reader for an exported EPUB.

Developer/publishing QA tooling inspired by Papermorph's useful delivery idea:
make the book directly explorable in a browser instead of inspecting only archive
internals. Narration, animation, quizzes, PDF conversion and hosting are outside
this tool's scope. No book content is uploaded.
"""

from __future__ import annotations

import argparse
import html
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tempfile
import xml.etree.ElementTree as ET
import zipfile

MAX_FILES = 5000
MAX_UNCOMPRESSED_BYTES = 512 * 1024 * 1024
CONTAINER_NS = {"c": "urn:oasis:names:tc:opendocument:xmlns:container"}


class PreviewError(RuntimeError):
    pass


def _safe_member(name: str) -> PurePosixPath:
    normalized = name.replace("\\", "/")
    path = PurePosixPath(normalized)
    if not normalized or normalized.startswith("/") or ".." in path.parts:
        raise PreviewError(f"unsafe EPUB member path: {name!r}")
    return path


def _package_path(zf: zipfile.ZipFile) -> str:
    try:
        root = ET.fromstring(zf.read("META-INF/container.xml"))
    except (KeyError, ET.ParseError) as exc:
        raise PreviewError("invalid EPUB container.xml") from exc
    node = root.find(".//c:rootfile", CONTAINER_NS)
    if node is None:
        raise PreviewError("EPUB container has no rootfile")
    full_path = str(node.attrib.get("full-path", "")).strip()
    if not full_path:
        raise PreviewError("EPUB rootfile path is empty")
    _safe_member(full_path)
    return full_path


def _chapter_title(zf: zipfile.ZipFile, member: str) -> str:
    try:
        raw = zf.read(member)
    except KeyError:
        return ""
    if len(raw) > 2 * 1024 * 1024:
        return ""
    try:
        root = ET.fromstring(raw)
    except ET.ParseError:
        return ""
    for node in root.iter():
        if node.tag.rsplit("}", 1)[-1].lower() == "title":
            value = "".join(node.itertext()).strip()
            if value:
                return value[:200]
    return ""


def _package_metadata(zf: zipfile.ZipFile, opf_path: str) -> tuple[str, list[dict[str, str]]]:
    try:
        root = ET.fromstring(zf.read(opf_path))
    except (KeyError, ET.ParseError) as exc:
        raise PreviewError("invalid EPUB package document") from exc

    ns_uri = root.tag[1:].split("}", 1)[0] if root.tag.startswith("{") else "http://www.idpf.org/2007/opf"
    ns = {"opf": ns_uri}
    dc = {"dc": "http://purl.org/dc/elements/1.1/"}
    title_node = root.find(".//dc:title", dc)
    book_title = (title_node.text or "").strip() if title_node is not None else "EPUB Preview"

    manifest: dict[str, dict[str, str]] = {}
    for item in root.findall(".//opf:manifest/opf:item", ns):
        item_id = str(item.attrib.get("id", "")).strip()
        href = str(item.attrib.get("href", "")).strip()
        media_type = str(item.attrib.get("media-type", "")).strip()
        if item_id and href:
            manifest[item_id] = {"href": href, "media_type": media_type}

    opf_dir = PurePosixPath(opf_path).parent
    chapters: list[dict[str, str]] = []
    for index, itemref in enumerate(root.findall(".//opf:spine/opf:itemref", ns), start=1):
        if str(itemref.attrib.get("linear", "yes")).lower() == "no":
            continue
        idref = str(itemref.attrib.get("idref", "")).strip()
        item = manifest.get(idref)
        if not item:
            continue
        member = (opf_dir / PurePosixPath(item["href"])).as_posix()
        _safe_member(member)
        chapters.append(
            {
                "id": idref,
                "label": _chapter_title(zf, member) or f"Chapter {index}",
                "member": member,
                "media_type": item["media_type"],
            }
        )
    if not chapters:
        raise PreviewError("EPUB spine contains no readable linear chapters")
    return book_title, chapters


def _extract_safely(zf: zipfile.ZipFile, root: Path) -> None:
    infos = zf.infolist()
    if len(infos) > MAX_FILES:
        raise PreviewError(f"EPUB contains too many files: {len(infos)} > {MAX_FILES}")
    total = sum(max(0, int(info.file_size)) for info in infos)
    if total > MAX_UNCOMPRESSED_BYTES:
        raise PreviewError(f"EPUB expands above the {MAX_UNCOMPRESSED_BYTES}-byte preview limit")
    root.mkdir(parents=True, exist_ok=True)
    for info in infos:
        member = _safe_member(info.filename)
        target = root.joinpath(*member.parts)
        if info.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        with zf.open(info, "r") as src, target.open("wb") as dst:
            shutil.copyfileobj(src, dst)


def _reader_html(title: str, chapters: list[dict[str, str]]) -> str:
    payload = [{"label": item["label"], "src": "book/" + item["member"]} for item in chapters]
    data = json.dumps(payload, ensure_ascii=False).replace("</", "<\\/")
    safe_title = html.escape(title, quote=True)
    return f"""<!doctype html>
<html lang="fa" dir="rtl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'self' data: blob:; img-src 'self' data: blob:; font-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; frame-src 'self'; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'">
<title>{safe_title} — local EPUB preview</title>
<style>
:root {{ color-scheme: light dark; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; min-height: 100vh; background: Canvas; color: CanvasText; }}
header {{ height: 54px; display: flex; align-items: center; gap: 10px; padding: 8px 14px; border-bottom: 1px solid color-mix(in srgb, CanvasText 18%, transparent); }}
header strong {{ overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
button {{ font: inherit; padding: 7px 11px; border-radius: 8px; border: 1px solid color-mix(in srgb, CanvasText 24%, transparent); background: ButtonFace; color: ButtonText; cursor: pointer; }}
button:disabled {{ opacity: .45; cursor: default; }}
main {{ height: calc(100vh - 54px); display: grid; grid-template-columns: minmax(220px, 300px) 1fr; direction: ltr; }}
nav {{ overflow: auto; padding: 10px; border-right: 1px solid color-mix(in srgb, CanvasText 18%, transparent); direction: rtl; }}
nav button {{ width: 100%; display: block; text-align: right; margin-bottom: 6px; }}
nav button[aria-current="page"] {{ font-weight: 700; outline: 2px solid Highlight; }}
.reader {{ min-width: 0; min-height: 0; background: white; }}
iframe {{ width: 100%; height: 100%; border: 0; background: white; }}
.counter {{ margin-inline-start: auto; direction: ltr; font-variant-numeric: tabular-nums; }}
@media (max-width: 760px) {{
  main {{ grid-template-columns: 1fr; grid-template-rows: auto 1fr; }}
  nav {{ display: flex; gap: 6px; overflow-x: auto; border-right: 0; border-bottom: 1px solid color-mix(in srgb, CanvasText 18%, transparent); direction: rtl; }}
  nav button {{ width: auto; min-width: max-content; margin: 0; }}
}}
</style>
</head>
<body>
<header>
  <button id="prev" type="button" aria-label="فصل قبلی">قبلی</button>
  <button id="next" type="button" aria-label="فصل بعدی">بعدی</button>
  <strong>{safe_title}</strong>
  <span id="counter" class="counter"></span>
</header>
<main>
  <nav id="chapters" aria-label="فصل‌ها"></nav>
  <div class="reader"><iframe id="reader" title="EPUB chapter preview" sandbox=""></iframe></div>
</main>
<script>
const chapters = {data};
const nav = document.getElementById("chapters");
const frame = document.getElementById("reader");
const counter = document.getElementById("counter");
const prev = document.getElementById("prev");
const next = document.getElementById("next");
let current = 0;
function openChapter(index) {{
  if (!Number.isInteger(index) || index < 0 || index >= chapters.length) return;
  current = index;
  frame.src = chapters[index].src;
  [...nav.children].forEach((button, i) => {{
    if (i === index) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }});
  counter.textContent = (index + 1) + " / " + chapters.length;
  prev.disabled = index === 0;
  next.disabled = index === chapters.length - 1;
}}
chapters.forEach((chapter, index) => {{
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = chapter.label;
  button.addEventListener("click", () => openChapter(index));
  nav.appendChild(button);
}});
prev.addEventListener("click", () => openChapter(current - 1));
next.addEventListener("click", () => openChapter(current + 1));
document.addEventListener("keydown", (event) => {{
  if (event.key === "ArrowLeft") openChapter(current + 1);
  if (event.key === "ArrowRight") openChapter(current - 1);
}});
openChapter(0);
</script>
</body>
</html>
"""


def build_preview(epub_path: Path, output_dir: Path, *, overwrite: bool = False) -> dict[str, object]:
    if not epub_path.is_file():
        raise FileNotFoundError(epub_path)
    if output_dir.exists() and any(output_dir.iterdir()) and not overwrite:
        raise PreviewError(f"output directory is not empty: {output_dir}")

    with zipfile.ZipFile(epub_path, "r") as zf:
        bad = zf.testzip()
        if bad:
            raise PreviewError(f"EPUB zip integrity failed at member: {bad}")
        opf_path = _package_path(zf)
        title, chapters = _package_metadata(zf, opf_path)
        output_dir.parent.mkdir(parents=True, exist_ok=True)
        staged = Path(tempfile.mkdtemp(prefix=f".{output_dir.name}.preview-", dir=output_dir.parent))
        try:
            _extract_safely(zf, staged / "book")
            (staged / "index.html").write_text(_reader_html(title, chapters), encoding="utf-8")
            (staged / "preview-manifest.json").write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "source_epub": epub_path.name,
                        "title": title,
                        "chapter_count": len(chapters),
                        "chapters": chapters,
                        "sandboxed_chapter_scripts": True,
                        "network_upload": False,
                    },
                    ensure_ascii=False,
                    indent=2,
                ) + "\n",
                encoding="utf-8",
            )
            if output_dir.exists():
                shutil.rmtree(output_dir)
            os.replace(staged, output_dir)
        except Exception:
            shutil.rmtree(staged, ignore_errors=True)
            raise
    return {"title": title, "chapter_count": len(chapters), "output": str(output_dir)}


def main() -> int:
    parser = argparse.ArgumentParser(description="Build a local interactive browser preview from an EPUB.")
    parser.add_argument("epub", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()
    result = build_preview(args.epub, args.output, overwrite=args.overwrite)
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    print(f"serve locally: python3 -m http.server 8765 -d {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
