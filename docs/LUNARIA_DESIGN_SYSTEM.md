# Lunaria Atelier Design System

Status: product UI theme for the Tauri desktop application.

## Theme thesis

Lunaria Atelier is a **restrained pastel-fantasy editorial system**, not a generic kawaii skin.

The visual metaphor combines:
- literary bookbinding and paper layers;
- moonlight / botanical stationery;
- soft pastel atmosphere;
- professional editorial hierarchy;
- low-distraction reading surfaces.

The application may feel magical around the manuscript, but the manuscript itself remains calm.

## Palette architecture

Foundation colors are named by hue and step. Components must consume semantic tokens instead of hard-coding decorative colors.

Core families:
- Lilac — identity / focus / selection.
- Dusty rose — warmth / secondary accent.
- Cloud blue — context / source-side coolness.
- Mint — safe / continuity / secondary depth.
- Parchment gold — restrained editorial highlight.
- Ink — all functional text and high-contrast anchors.

Semantic tokens:
- `--bg-*`
- `--surface-*`
- `--text-*`
- `--accent*`
- `--border-*`
- `--success / --warning / --danger`

## Typography

- UI: system sans.
- Literary display: Iowan Old Style / New York / Palatino / Georgia fallback chain.
- Persian reading: Geeza Pro / Tahoma / Noto Naskh Arabic fallback chain.
- Metadata: SF Mono / Menlo / Consolas.

Display serif is for product identity and manuscript hierarchy only. Controls stay sans-serif.

## Surface rules

1. Glass is reserved for framing surfaces such as navigation.
2. Working cards remain more opaque for legibility.
3. The translation editor is the calmest surface in the product.
4. Source and Persian target use distinct, low-chroma backgrounds.
5. Decorative effects cannot carry status or workflow meaning.

## Shape language

- small controls: 8–12px radii;
- cards: 18–26px;
- hero frame: 36px;
- pills only for short actions/statuses, never for every container.

The asymmetric book-mark radius is the signature shape used sparingly.

## Motion grammar

- Fast (150ms): direct control feedback.
- Base (280ms): hover and component state.
- Panel (380ms): view change.
- Ambient (12s+): decorative field.

Ambient motion must be slow and low-amplitude. No bouncing interface chrome. `prefers-reduced-motion` collapses all animation and removes ambient ornaments.

## Ornament language

Allowed:
- four-point geometric star;
- paper stack;
- orbit line;
- abstract cloud;
- quiet circular/botanical geometry.

Avoid:
- emoji decoration;
- random hearts/stickers;
- neon glows;
- glitter noise;
- novelty animations that compete with text.

## Editor rules

The editor is a professional reading desk:
- source left / Persian right on wide screens;
- stacked source / Persian on narrow screens;
- Persian direction always RTL;
- long-form line-height prioritized over density;
- save actions appear quietly and strengthen on focus/hover;
- no ambient animation inside text areas.

## Privacy / implementation

The theme stays entirely inside local bundled HTML/CSS/JS.

No:
- remote fonts;
- CDN assets;
- remote scripts;
- UI framework;
- motion library;
- analytics/session replay SDK.

The Rust/Tauri application layer remains authoritative for workflow and state.
