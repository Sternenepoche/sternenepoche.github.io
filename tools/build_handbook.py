"""Erzeugt das eigenständig lesbare Spielhandbuch aus docs/HANDBUCH.md."""

from __future__ import annotations

import html
import re
from pathlib import Path
from urllib.parse import urlsplit

import markdown
from site_policy import decorate
from website_content import chapter_visual, finish, DIALOG


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "HANDBUCH.md"
DESTINATION = ROOT / "Sternenepoche-Handbuch.html"

STYLE = """
:root {
  color-scheme: light;
  --ink: #243345;
  --muted: #526377;
  --line: #dce4ed;
  --nav: #101e30;
  --accent: #137a83;
}
* { box-sizing: border-box; }
html { scroll-padding-top: 28px; }
body {
  margin: 0;
  color: var(--ink);
  background: #eef2f6;
  font: 17px/1.72 "Segoe UI", system-ui, sans-serif;
}
a { color: #086e81; text-underline-offset: 3px; overflow-wrap: anywhere; }
a:hover { color: #004d60; }
a:focus-visible, summary:focus-visible {
  outline: 3px solid #d89f39;
  outline-offset: 4px;
  border-radius: 3px;
}
.skip {
  position: fixed; left: 16px; top: -100px; padding: 12px 18px;
  z-index: 10; background: white; color: var(--ink);
}
.skip:focus { top: 16px; }
.layout {
  display: grid; grid-template-columns: 304px minmax(0, 940px);
  max-width: 1244px; margin: 0 auto;
}
.sidebar {
  position: sticky; top: 0; height: 100vh; height: 100dvh; overflow-y: auto;
  background: var(--nav); color: #d1dfed; padding: 30px 22px 26px;
  scrollbar-color: #455c78 var(--nav);
}
.brand {
  display: flex; gap: 12px; align-items: center; color: #f8fbff;
  font-size: 20px; font-weight: 650; letter-spacing: .02em;
}
.brand svg { flex-shrink: 0; color: #8bdbd4; }
.edition {
  margin: 9px 0 26px; color: #9cb2c8; font-size: 13px;
}
.sidebar summary { cursor: pointer; color: #f0f5fb; font-weight: 650; }
.sidebar .toc ul { padding: 0; list-style: none; margin: 14px 0 24px; }
.sidebar .toc li { margin: 3px 0; }
.sidebar .toc a {
  display: block; text-decoration: none; color: #c1d3e6;
  font-size: 14px; line-height: 1.45; padding: 8px 10px;
  border-radius: 6px;
}
.sidebar .toc a:hover { background: #21364e; color: white; }
.sidebar-note { color: #9cb2c8; font-size: 12px; line-height: 1.6; }
main { min-width: 0; background: white; box-shadow: 8px 0 40px #1b32400a; }
.hero {
  padding: 48px 52px 40px;
  color: #f3f8ff;
  background:
    radial-gradient(ellipse at 100% 0, #26657770, transparent 65%),
    radial-gradient(ellipse at 20% 110%, #435c8640, transparent 70%),
    #182b43;
}
.eyebrow { font-size: 12px; font-weight: 650; letter-spacing: .12em; color: #8fdfd6; }
.hero h1 {
  margin: 14px 0 16px; max-width: 660px;
  font-size: clamp(30px, 3.3vw, 43px); line-height: 1.18; letter-spacing: -.025em;
}
.hero p { margin: 0; max-width: 640px; color: #c6d7e9; font-size: 17px; }
.chapter-links { display: flex; gap: 9px; flex-wrap: wrap; margin-top: 25px; }
.chapter-links a {
  padding: 6px 12px; color: #e7f9f7; border: 1px solid #76bbbc70;
  border-radius: 6px; font-size: 13px; text-decoration: none;
}
.chapter-links a:hover { background: #32617280; }
article { padding: 30px 52px 65px; }
article > :first-child { margin-top: 0; }
h2 {
  margin: 64px 0 23px; padding-top: 25px; border-top: 2px solid var(--line);
  font-size: 27px; line-height: 1.3; color: #17354b; letter-spacing: -.015em;
  break-after: avoid;
}
h3 {
  margin: 32px 0 12px; color: #1e5265;
  font-size: 20px; line-height: 1.4; break-after: avoid;
}
p { margin: 0 0 19px; }
ul, ol { padding-left: 26px; margin: 16px 0 23px; }
li { padding-left: 3px; margin: 7px 0; }
strong { color: #152e43; font-weight: 650; }
.example {
  background: #eef7f7; border-left: 3px solid #419da0;
  padding: 15px 19px; border-radius: 0 7px 7px 0;
}
.table-wrap { overflow-x: auto; margin: 23px 0 28px; border: 1px solid var(--line); border-radius: 8px; }
table { width: 100%; border-collapse: collapse; font-size: 15px; line-height: 1.58; }
th { text-align: left; background: #ecf2f7; color: #1c3e56; font-weight: 650; }
td, th { padding: 12px 14px; border-bottom: 1px solid var(--line); vertical-align: top; }
tr:last-child td { border-bottom: 0; }
tbody tr:nth-child(even) { background: #f8fafc; }
td:first-child { min-width: 115px; }
pre {
  overflow-x: auto; padding: 20px; border-radius: 8px;
  background: #14283c; color: #e9f5ff; font-size: 13px; line-height: 1.6;
}
code { font-family: Consolas, "Courier New", monospace; }
.foot {
  padding: 25px 52px; background: #f4f7fa;
  border-top: 1px solid var(--line); color: var(--muted); font-size: 13px;
}
@media (max-width: 960px) {
  .layout { grid-template-columns: 245px minmax(0, 1fr); }
  .sidebar { padding: 25px 15px; }
  .hero { padding: 38px 30px 32px; }
  article { padding: 26px 30px 48px; }
  .foot { padding: 22px 30px; }
}
@media (max-width: 720px) {
  body { font-size: 16px; }
  .layout { display: block; }
  .sidebar { position: static; height: auto; padding: 20px 22px; }
  .edition { margin-bottom: 16px; }
  .sidebar .toc { max-height: 280px; overflow-y: auto; }
  .sidebar-note { margin-bottom: 0; }
  .hero { padding: 30px 23px; }
  article { padding: 25px 23px 45px; }
  h2 { margin-top: 48px; font-size: 24px; }
  table { font-size: 14px; }
  td, th { padding: 10px; }
  .foot { padding: 22px 23px; }
}
@media print {
  @page { size: A4; margin: 17mm; }
  body { background: white; color: black; font-size: 10.5pt; line-height: 1.5; }
  .layout { display: block; max-width: none; }
  .sidebar, .skip, .chapter-links, .eyebrow { display: none; }
  main { box-shadow: none; }
  .hero { color: black; background: white; padding: 0 0 22px; border-bottom: 2px solid #aaa; }
  .hero h1 { font-size: 25pt; }
  .hero p { color: #333; font-size: 11pt; }
  article { padding: 22px 0; }
  h2 { margin-top: 28px; padding-top: 15px; font-size: 17pt; }
  h3 { font-size: 13pt; }
  .table-wrap { overflow: visible; border: 0; border-radius: 0; }
  table { font-size: 9pt; }
  td, th { padding: 6px 8px; }
  thead { display: table-header-group; }
  tr, .example { break-inside: avoid; }
  pre { white-space: pre-wrap; background: #f0f0f0; color: black; }
  a { color: inherit; }
  .foot { padding: 15px 0 0; background: white; }
}
"""


def build() -> None:
    source = SOURCE.read_text(encoding="utf-8")
    title = source.splitlines()[0].removeprefix("# ").strip()
    renderer = markdown.Markdown(
        extensions=["tables", "toc", "fenced_code", "sane_lists"],
        extension_configs={"toc": {"toc_depth": "2-2"}},
    )
    body = renderer.convert(source)
    body = re.sub(r'(<h2\b[^>]*>(\d+)\s.*?</h2>)',
                  lambda m: m[1] + chapter_visual(int(m[2])), body)
    body = re.sub(r"^<h1[^>]*>.*?</h1>\s*", "", body, count=1, flags=re.S)
    body = re.sub(
        r"<p>(<(?:strong)[^>]*>)?(?:Beispiel|Zeitbeispiel|Frachtbeispiel|Rechenbeispiel)",
        lambda match: match.group(0).replace("<p>", '<p class="example">', 1),
        body,
    )
    body = body.replace("<table>", '<div class="table-wrap"><table>').replace(
        "</table>", "</table></div>"
    )

    def local_link(match: re.Match[str]) -> str:
        href = html.unescape(match.group(1))
        if urlsplit(href).scheme or href.startswith(("#", "/")):
            return match.group(0)
        return f'href="{html.escape("docs/" + href, quote=True)}"'

    body = re.sub(r'href="([^"]+)"', local_link, body)
    chapters = [
        item for item in renderer.toc_tokens if item["level"] == 2
    ]
    shortcuts = [(2, "Anmelden"), (6, "Erste Schritte"), (13, "Saven"), (19, "Server verwalten")]
    shortcut_html = "".join(
        f'<a href="#{html.escape(chapters[index]["id"], quote=True)}">{label}</a>'
        for index, label in shortcuts
    )
    safe_title = html.escape(title)
    document = f"""<!doctype html>
<html lang="de">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="Sternenepoche spielen und verwalten: Anmeldung, Völker, Menschen, Bots, Agenten, Wirtschaft, Aufklärung, Kampf und Epochen.">
  <title>{safe_title}</title>
  <style>{STYLE}</style>
</head>
<body>
  <a class="skip" href="#inhalt">Zum Handbuch</a>
  <div class="layout">
    <aside class="sidebar" aria-label="Handbuchnavigation">
      <div class="brand">
        <svg width="30" height="30" viewBox="0 0 30 30" aria-hidden="true">
          <circle cx="15" cy="15" r="4" fill="currentColor"/>
          <ellipse cx="15" cy="15" rx="14" ry="7" transform="rotate(-32 15 15)" fill="none" stroke="currentColor"/>
          <circle cx="25" cy="8" r="2.5" fill="#f4c875"/>
        </svg>
        Sternenepoche
      </div>
      <p class="edition">Das Spielhandbuch · 8. Oktober 2026</p>
      <details open>
        <summary>Alle 23 Kapitel</summary>
        <nav aria-label="Kapitel">{renderer.toc}</nav>
      </details>
      <p class="sidebar-note">Offline lesbar. Über die Druckfunktion des Browsers kannst du das Handbuch ausdrucken oder als PDF speichern.</p>
    </aside>
    <main id="inhalt">
      <header class="hero">
        <div class="eyebrow">SPIELEN UND VERWALTEN</div>
        <h1>{safe_title}</h1>
        <p>Vom ersten Login bis zur nächsten Epoche. Klare Regeln, konkrete Beispiele und Hilfe für Menschen, Bots und Agenten.</p>
        <div class="chapter-links" aria-label="Schnelleinstieg">{shortcut_html}</div>
      </header>
      <article>{body}</article>
      <footer class="foot">Bearbeitbare Fassung: docs/HANDBUCH.md · Stand 8. Oktober 2026 · Keine Verbindung zum Spielserver für das Lesen erforderlich.</footer>
    </main>
  </div>
</body>
</html>
"""
    # Reading layout shares the artwork and screenshot provenance with the guide.
    reading_style = '''<style>
    :root{--ink:#dbe6f3;--muted:#adbdcf;--line:#a1c9e222}
    body{color:var(--ink)}.layout{max-width:1400px;grid-template-columns:290px minmax(0,1110px)}
    main{background:#091421}.sidebar{background:#0c192b}.sidebar .brand{color:#f2f5f8}
    .hero{display:block;min-height:0;padding:50px;isolation:isolate}.hero h1{font-size:52px;max-width:800px}.hero p{margin:0}
    h2{color:#f2f5f8;font-size:31px}h3{color:#8be0dc}strong{color:#eef3f9}a{color:#8be0dc}
    .example{background:#8be0dc0a;border-color:#8be0dc}th{background:#1a2c42;color:#edf5ff}
    tbody tr:nth-child(even){background:#ffffff04}.foot{background:#0c192b}article{line-height:1.85}
    .illustration strong{color:#efc88b}.screenshot{margin-block:28px 35px}
    @media(max-width:960px){.layout{grid-template-columns:235px minmax(0,1fr)}.hero{padding:35px 30px}.hero h1{font-size:40px}}
    @media(max-width:720px){.layout{display:block}.sidebar{position:static}.hero{padding:32px 23px}.hero h1{font-size:36px}h2{font-size:26px}}
    @media print{body,main{background:white;color:#172c40}h2,h3,strong{color:#172c40}.hero h1{font-size:28pt}.hero::before,.hero::after{display:none}.example{background:#eef5f6}.foot{background:white}.sidebar{display:none}}
    </style>'''
    document = document.replace('</head>', '<!-- SITE_THEME -->' + reading_style + '</head>')
    document = document.replace('</body>', DIALOG + '<!-- SITE_MOTION --></body>')
    document = document.replace('<div class="brand">','<a class="brand" href="index.html" style="text-decoration:none">',1)
    document = document.replace('        Sternenepoche\n      </div>', '        Sternenepoche\n      </a>',1)
    document = re.sub(r'(<a class="brand"[^>]*>)\s*<svg.*?</svg>', lambda m: m[1] + '<span class="brand-symbol" aria-hidden="true"><img src="logo://sternenepoche-mark.png" alt="" width="256" height="256"><span class="brand-glint"></span></span>', document, count=1, flags=re.S)
    document = document.replace('</head>','<link rel="icon" type="image/png" href="docs/branding/favicon.png"></head>')
    document = decorate(finish(document), "KI-Hinweis.html")
    DESTINATION.write_text(document, encoding="utf-8")
    print(f"Erstellt: {DESTINATION}")
    print(f"{len(chapters)} Kapitel, {len(source.split())} Wörter, {len(document.encode('utf-8')):,} Bytes")


if __name__ == "__main__":
    build()
