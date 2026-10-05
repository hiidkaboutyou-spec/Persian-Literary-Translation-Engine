from __future__ import annotations

import argparse
import hashlib
import html
import json
import os
import posixpath
import re
import zipfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
import xml.etree.ElementTree as ET

MAX_CHAPTERS = 500
MAX_MEMBER_BYTES = 5_000_000
DROP_TAGS = {"script", "style", "iframe", "object", "embed", "form", "input", "button", "video", "audio"}
ALLOWED_TAGS = {
    "p", "div", "span", "section", "article", "blockquote", "pre", "code",
    "h1", "h2", "h3", "h4", "h5", "h6", "em", "strong", "b", "i", "u",
    "ul", "ol", "li", "hr", "br", "sup", "sub", "small",
}
ALLOWED_ATTRS = {"dir", "lang", "xml:lang", "class"}


class PreviewError(RuntimeError):
    pass


@dataclass(frozen=True, slots=True)
class Chapter:
    href: str
    title: str
    body_html: str


def _local(tag: str) -> str:
    return str(tag).rsplit("}", 1)[-1].lower()


def _member_bytes(archive: zipfile.ZipFile, name: str) -> bytes:
    try:
        info = archive.getinfo(name)
    except KeyError as exc:
        raise PreviewError(f"EPUB member missing: {name}") from exc
    if info.file_size > MAX_MEMBER_BYTES:
        raise PreviewError(f"EPUB member too large for preview: {name}")
    return archive.read(name)


def _resolve_member(base_file: str, href: str) -> str:
    href = href.split("#", 1)[0].split("?", 1)[0]
    value = posixpath.normpath(posixpath.join(posixpath.dirname(base_file), href))
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts:
        raise PreviewError(f"unsafe EPUB path: {href}")
    return path.as_posix()


def _safe_text(value: str | None) -> str:
    return html.escape(value or "", quote=False)


def _safe_attrs(element: ET.Element) -> str:
    attrs: list[str] = []
    for raw_key, raw_value in element.attrib.items():
        key = _local(raw_key)
        if key not in ALLOWED_ATTRS:
            continue
        value = str(raw_value or "").strip()
        if not value:
            continue
        attrs.append(f' {key}="{html.escape(value, quote=True)}"')
    return "".join(attrs)


def _sanitize_element(element: ET.Element) -> str:
    tag = _local(element.tag)
    if tag in DROP_TAGS:
        return _safe_text(element.tail)

    inner = _safe_text(element.text)
    for child in list(element):
        inner += _sanitize_element(child)

    tail = _safe_text(element.tail)
    if tag not in ALLOWED_TAGS:
        return inner + tail
    if tag in {"br", "hr"}:
        return f"<{tag}{_safe_attrs(element)}>" + tail
    return f"<{tag}{_safe_attrs(element)}>{inner}</{tag}>" + tail


def _chapter_title(root: ET.Element, fallback: str) -> str:
    for wanted in ("h1", "h2", "h3", "title"):
        for node in root.iter():
            if _local(node.tag) == wanted:
                text = " ".join("".join(node.itertext()).split())
                if text:
                    return text[:200]
    return fallback[:200]


def _chapter_body(root: ET.Element) -> str:
    body = next((node for node in root.iter() if _local(node.tag) == "body"), None)
    if body is None:
        raise PreviewError("XHTML chapter has no body")
    rendered = _safe_text(body.text)
    for child in list(body):
        rendered += _sanitize_element(child)
    if not re.sub(r"<[^>]+>", "", rendered).strip():
        raise PreviewError("XHTML chapter body is empty")
    return rendered


def read_epub(epub_path: Path) -> tuple[str, list[Chapter]]:
    if not epub_path.is_file():
        raise PreviewError(f"EPUB not found: {epub_path}")

    with zipfile.ZipFile(epub_path) as archive:
        container = ET.fromstring(_member_bytes(archive, "META-INF/container.xml"))
        rootfile = next((node for node in container.iter() if _local(node.tag) == "rootfile"), None)
        if rootfile is None:
            raise PreviewError("EPUB container has no rootfile")
        opf_path = str(rootfile.attrib.get("full-path", "")).strip()
        if not opf_path:
            raise PreviewError("EPUB rootfile path is empty")

        package = ET.fromstring(_member_bytes(archive, opf_path))
        title = epub_path.stem
        for node in package.iter():
            if _local(node.tag) == "title":
                candidate = " ".join("".join(node.itertext()).split())
                if candidate:
                    title = candidate[:200]
                    break

        manifest: dict[str, tuple[str, str]] = {}
        for node in package.iter():
            if _local(node.tag) != "item":
                continue
            item_id = str(node.attrib.get("id", "")).strip()
            href = str(node.attrib.get("href", "")).strip()
            media_type = str(node.attrib.get("media-type", "")).strip().lower()
            if item_id and href:
                manifest[item_id] = (href, media_type)

        spine_ids = [
            str(node.attrib.get("idref", "")).strip()
            for node in package.iter()
            if _local(node.tag) == "itemref" and str(node.attrib.get("idref", "")).strip()
        ]
        if len(spine_ids) > MAX_CHAPTERS:
            raise PreviewError(f"EPUB has too many spine items: {len(spine_ids)}")

        chapters: list[Chapter] = []
        for index, item_id in enumerate(spine_ids, start=1):
            item = manifest.get(item_id)
            if item is None:
                raise PreviewError(f"spine idref missing from manifest: {item_id}")
            href, media_type = item
            if media_type not in {"application/xhtml+xml", "text/html"}:
                continue
            member = _resolve_member(opf_path, href)
            try:
                root = ET.fromstring(_member_bytes(archive, member))
            except ET.ParseError as exc:
                raise PreviewError(f"invalid XHTML in spine item: {member}") from exc
            chapters.append(
                Chapter(
                    href=member,
                    title=_chapter_title(root, f"Chapter {index}"),
                    body_html=_chapter_body(root),
                )
            )

    if not chapters:
        raise PreviewError("EPUB spine contains no readable XHTML chapters")
    return title, chapters


def _render_html(book_title: str, chapters: list[Chapter], source_sha256: str, language: str) -> str:
    options = "\n".join(
        f'<option value="{index}">{html.escape(chapter.title)}</option>'
        for index, chapter in enumerate(chapters)
    )
    sections = "\n".join(
        (
            f'<section class="chapter" data-index="{index}" '
            f'aria-label="{html.escape(chapter.title, quote=True)}"{" hidden" if index else ""}>'
            f'<div class="chapter-body">{chapter.body_html}</div></section>'
        )
        for index, chapter in enumerate(chapters)
    )
    safe_title = html.escape(book_title)
    safe_lang = html.escape(language, quote=True)
    safe_sha = html.escape(source_sha256, quote=True)
    return f"""<!doctype html>
<html lang="{safe_lang}" dir="rtl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="referrer" content="no-referrer">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:; font-src data:">
<title>{safe_title} — local preview</title>
<style>
:root {{ --reader-size: 1.08rem; --measure: 46rem; }}
* {{ box-sizing: border-box; }}
body {{ margin:0; background:#f4f1eb; color:#211f1b; font-family: Tahoma, "Noto Naskh Arabic", sans-serif; }}
header {{ position:sticky; top:0; z-index:10; display:flex; gap:.6rem; align-items:center; padding:.7rem 1rem; background:#fffdf8; border-bottom:1px solid #d8d2c8; }}
header strong {{ margin-inline-end:auto; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }}
button, select {{ font:inherit; padding:.45rem .65rem; border:1px solid #bdb5aa; border-radius:.55rem; background:white; }}
main {{ min-height:calc(100vh - 4rem); padding:clamp(1rem, 4vw, 3rem); }}
.reader {{ max-width:var(--measure); margin:0 auto; background:#fff; padding:clamp(1.4rem, 5vw, 4rem); box-shadow:0 .35rem 2rem #0001; border-radius:.8rem; }}
.chapter-body {{ font-size:var(--reader-size); line-height:2; overflow-wrap:anywhere; }}
.chapter-body p {{ margin:0 0 1.15em; text-align:start; }}
.chapter-body h1,.chapter-body h2,.chapter-body h3 {{ line-height:1.5; margin:1.5em 0 .7em; }}
.chapter-body pre,.chapter-body code {{ direction:ltr; unicode-bidi:isolate; font-family:ui-monospace, monospace; }}
.chapter-body [dir="ltr"] {{ unicode-bidi:isolate; }}
footer {{ max-width:var(--measure); margin:1rem auto 3rem; display:flex; justify-content:space-between; gap:1rem; color:#6e675e; font-size:.88rem; }}
@media (max-width:700px) {{
  header {{ flex-wrap:wrap; }}
  header strong {{ width:100%; }}
  select {{ flex:1; min-width:10rem; }}
  main {{ padding:.65rem; }}
  .reader {{ border-radius:.35rem; padding:1.15rem; }}
}}
</style>
</head>
<body>
<header>
  <strong>{safe_title}</strong>
  <button id="prev" type="button" aria-label="فصل قبل">→</button>
  <select id="chapter-select" aria-label="انتخاب فصل">{options}</select>
  <button id="next" type="button" aria-label="فصل بعد">←</button>
  <button id="smaller" type="button" aria-label="کوچک‌تر کردن متن">A−</button>
  <button id="larger" type="button" aria-label="بزرگ‌تر کردن متن">A+</button>
</header>
<main><div class="reader">{sections}</div></main>
<footer><span>Local diagnostic preview · no network dependencies</span><span>source sha256: {safe_sha}</span></footer>
<script>
(() => {{
  const chapters = [...document.querySelectorAll('.chapter')];
  const select = document.getElementById('chapter-select');
  const prev = document.getElementById('prev');
  const next = document.getElementById('next');
  let index = 0;
  const show = (nextIndex) => {{
    index = Math.max(0, Math.min(chapters.length - 1, nextIndex));
    chapters.forEach((chapter, i) => chapter.hidden = i !== index);
    select.value = String(index);
    prev.disabled = index === 0;
    next.disabled = index === chapters.length - 1;
    history.replaceState(null, '', '#chapter-' + (index + 1));
    window.scrollTo({{top: 0, behavior: 'instant'}});
  }};
  const hashIndex = Number((location.hash.match(/chapter-(\d+)/) || [])[1] || 1) - 1;
  show(Number.isFinite(hashIndex) ? hashIndex : 0);
  select.addEventListener('change', () => show(Number(select.value)));
  prev.addEventListener('click', () => show(index - 1));
  next.addEventListener('click', () => show(index + 1));
  document.getElementById('smaller').addEventListener('click', () => {{
    const current = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--reader-size')) || 1.08;
    document.documentElement.style.setProperty('--reader-size', Math.max(.8, current - .08) + 'rem');
  }});
  document.getElementById('larger').addEventListener('click', () => {{
    const current = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--reader-size')) || 1.08;
    document.documentElement.style.setProperty('--reader-size', Math.min(1.8, current + .08) + 'rem');
  }});
  document.addEventListener('keydown', (event) => {{
    if (event.key === 'ArrowLeft') show(index + 1);
    if (event.key === 'ArrowRight') show(index - 1);
  }});
}})();
</script>
</body>
</html>
"""


def build_preview(epub_path: Path, output_dir: Path, *, language: str = "fa") -> dict[str, object]:
    raw = epub_path.read_bytes()
    source_sha256 = hashlib.sha256(raw).hexdigest()
    title, chapters = read_epub(epub_path)
    output_dir.mkdir(parents=True, exist_ok=True)

    html_text = _render_html(title, chapters, source_sha256, language)
    manifest = {
        "schema_version": 1,
        "source_sha256": source_sha256,
        "book_title": title,
        "language": language,
        "direction": "rtl",
        "chapter_count": len(chapters),
        "chapters": [{"index": i + 1, "title": ch.title, "source_href": ch.href} for i, ch in enumerate(chapters)],
        "network_dependencies": False,
        "contains_private_book_text": True,
        "canonical_edit_surface": False,
    }

    _atomic_text(output_dir / "index.html", html_text)
    _atomic_text(
        output_dir / "manifest.json",
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
    )
    return manifest


def _atomic_text(path: Path, content: str) -> None:
    staged = path.with_name(path.name + ".tmp")
    with staged.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write(content)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(staged, path)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build a self-contained, local RTL web reading preview from a generated EPUB."
    )
    parser.add_argument("epub", type=Path)
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Output folder. Defaults to output/interactive-preview/<epub-stem>.",
    )
    parser.add_argument("--lang", default="fa")
    args = parser.parse_args()

    output = args.out or Path("output/interactive-preview") / args.epub.stem
    manifest = build_preview(args.epub, output, language=args.lang)
    print(
        json.dumps(
            {
                "output": str(output),
                "chapters": manifest["chapter_count"],
                "source_sha256": manifest["source_sha256"],
                "network_dependencies": False,
            },
            sort_keys=True,
        )
    )
    print("Preview contains book text. Keep output local and out of Git.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
