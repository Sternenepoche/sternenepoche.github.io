"""Baut den eigenständigen Spielguide aus Vorlage und aktuellem Handbuch."""

from __future__ import annotations

import base64
import html
import re
from pathlib import Path
from urllib.parse import urlsplit

import markdown
from site_policy import decorate
from markdown.extensions.toc import slugify


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "startseite.html"
DESTINATION = ROOT / "Sternenepoche-Start.html"
ASSETS = ROOT / "web-client" / "assets"
HANDBUCH = ROOT / "docs" / "HANDBUCH.md"

SCREENS = [
    ("reich", "Übersicht", 10), ("gebaeude", "Gebäude", 11),
    ("kolonie", "Versorgung", 10), ("forschen", "Forschung", 11),
    ("werft", "Schiffswerft", 11), ("verteidigung", "Verteidigung", 15),
    ("flotten", "Flotten & Saven", 14), ("karte", "Galaxie", 12),
    ("kolonien", "Kolonisation", 16), ("kampf", "Kampf & Verbände", 15),
    ("aktionen", "Markt & Diplomatie", 17), ("berichte", "Nachrichten & Berichte", 12),
    ("imperium", "Imperiumsvergleich", 19), ("freischaltungen", "Technologiebaum", 11),
    ("agent", "Agentensteuerung", 6), ("profil", "Spielerprofil", 5),
    ("regeln", "Spielanleitung", 21),
]


def render(source: str) -> str:
    body = markdown.markdown(
        source, extensions=["tables", "toc", "fenced_code", "sane_lists"],
        extension_configs={"toc": {"slugify": lambda value, sep: "guide-" + slugify(value, sep)}},
    )
    body = re.sub(r'<h([23])(\b[^>]*)>(.*?)</h\1>',
                  lambda m: f'<h{int(m[1])+1}{m[2]}>{m[3]}</h{int(m[1])+1}>', body, flags=re.S)
    body = re.sub(r'<p>(?=<strong>(?:Beispiel|Zeitbeispiel|Frachtbeispiel|Rechenbeispiel))',
                  '<p class="guide-example">', body)
    body = body.replace('<table>', '<div class="guide-table" tabindex="0" role="region" aria-label="Tabelle, bei Bedarf seitlich scrollen"><table>').replace('</table>', '</table></div>')

    def link(match: re.Match[str]) -> str:
        href = html.unescape(match[1])
        if urlsplit(href).scheme or href.startswith(("#", "/")):
            return match[0]
        href = href.removeprefix("../") if href.startswith("../") else "docs/" + href
        return 'href="' + html.escape(href, quote=True) + '"'

    return re.sub(r'href="([^"]+)"', link, body)


def guide_parts() -> tuple[dict[str, str], int]:
    source = HANDBUCH.read_text(encoding="utf-8")
    chapters = {int(m[1]): (m[2], m[3]) for m in re.finditer(
        r'^## (\d+) ([^\n]+)\n(.*?)(?=^## |\Z)', source, re.M | re.S)}
    anchors = {number: "guide-" + slugify(f"{number} {title}", "-")
               for number, (title, _) in chapters.items()}
    first = chapters[7][1]
    intro = first.split("### ", 1)[0]
    steps = list(re.finditer(r'^### (\d+) ([^\n]+)\n(.*?)(?=^### |\Z)', first, re.M | re.S))
    first_html = render(intro) + '<div class="session-steps">' + ''.join(
        f'<article class="session-step" id="einstieg-{m[1]}"><span class="step-index" aria-hidden="true">{m[1]}</span>'
        f'<div><h3>{html.escape(m[2])}</h3>{render(m[3])}</div></article>' for m in steps) + '</div>'
    practical = {m[1]: m[2] for m in re.finditer(
        r'^### ([^\n]+)\n(.*?)(?=^### |\Z)', chapters[8][1], re.M | re.S)}
    screen_links, options, panels = [], [], []
    for key, name, chapter in SCREENS:
        if name not in practical:
            raise ValueError(f"Bedienweg fehlt im Handbuch: {name}")
        screen_links.append(f'<a href="#bedienweg-{key}" data-screen="{key}">{html.escape(name)}</a>')
        options.append(f'<option value="{key}">{html.escape(name)}</option>')
        panels.append(f'<article class="screen-panel" id="bedienweg-{key}" aria-labelledby="screen-{key}">'
                      f'<span class="eyebrow">Links im Spielmenü · {html.escape(name)}</span>'
                      f'<h3 id="screen-{key}" tabindex="-1">{html.escape(name)}</h3>{render(practical[name])}'
                      f'<a class="text-link" href="#{anchors[chapter]}">Die Spielregeln dazu lesen <span aria-hidden="true">↓</span></a></article>')
    screens_html = '<div class="screen-finder" id="screen-finder">' + (
        '<div class="screen-top"><strong>So orientierst du dich im Spiel</strong>'
        '<p>Oben: Vorräte des gewählten Planeten und reichsweite Credits. Links: Aufgabenbereiche. '
        'Rechts: Deine Planeten. Warnungen: zuerst lesen.</p></div>'
        '<label class="screen-select" hidden>Spielbereich auswählen<select id="screen-select">'
        + ''.join(options) + '</select></label><div class="screen-layout">'
        '<nav class="screen-nav" aria-label="Die 17 Bereiche der Spieloberfläche">'
        + ''.join(screen_links) + '</nav><div class="screen-content">' + ''.join(panels) + '</div></div>'
        '<p id="screen-status" class="sr-only" role="status"></p></div>')
    toc, articles = [], []
    for number in range(1, 23):
        title, content = chapters[number]
        target = anchors[number]
        heading = f'## {number} {title}\n\n'
        if number == 7:
            content = 'Die ausführliche erste Spielsitzung mit acht praktischen Schritten steht [weiter oben auf dieser Seite](#erste-sitzung). Du kannst sie direkt im Spiel abarbeiten.'
        elif number == 8:
            content = 'Der [Oberflächen-Wegweiser](#oberflaeche) erklärt alle 17 Bereiche: wo du etwas findest, welche Knöpfe du benutzt, was danach passiert und welche Fehler du prüfen solltest.\n\n' + chapters[8][1].split('### ', 1)[0]
        label = f'{number:02d} · {title}' + (' · Betreiber' if number == 20 else '')
        toc.append(f'<li><a href="#{target}">{html.escape(label)}</a></li>')
        articles.append(f'<section class="guide-chapter" aria-labelledby="{target}" data-search-title="{html.escape(title, quote=True)}">{render(heading + content)}<a class="chapter-back" href="#spielguide">↑ Zur Kapitelauswahl</a></section>')
    book = '<div class="guide-layout"><aside class="guide-index"><details open><summary>22 Kapitel zum Nachschlagen</summary><nav aria-label="Spielguide-Kapitel"><ol>' + ''.join(toc) + '</ol></nav></details><p>Kapitel 20 erklärt die private Serververwaltung. Alle anderen Kapitel helfen beim Spielen.</p></aside><div class="guide-reading">' + ''.join(articles) + '</div></div>'
    return {"ERSTE_SITZUNG": first_html, "OBERFLAECHE": screens_html, "SPIELGUIDE": book}, len(source.split())


def build() -> None:
    document = SOURCE.read_text(encoding="utf-8")
    parts, words = guide_parts()
    for name, body in parts.items():
        marker = f"<!-- {name} -->"
        if document.count(marker) != 1:
            raise ValueError(f"Genau ein Inhaltsplatzhalter erforderlich: {name}")
        document = document.replace(marker, body)
    embedded: dict[str, str] = {}

    def embed(match: re.Match[str]) -> str:
        name = match.group(1)
        if name not in embedded:
            path = ASSETS / name
            if path.suffix != ".webp" or not path.is_file():
                raise ValueError(f"Spielmotiv fehlt oder ist ungültig: {name}")
            embedded[name] = "data:image/webp;base64," + base64.b64encode(
                path.read_bytes()
            ).decode("ascii")
        return embedded[name]

    document = re.sub(r"asset://([a-zA-Z0-9_.-]+)", embed, document)
    if "asset://" in document:
        raise ValueError("Nicht aufgelöster Motivverweis")
    document = decorate(document, "KI-Hinweis.html")
    DESTINATION.write_text(document, encoding="utf-8")
    print(f"Erstellt: {DESTINATION}")
    print(f"{len(embedded)} Spielmotive eingebettet, {DESTINATION.stat().st_size:,} Bytes")
    print(f"17 Bedienwege, 8 Einstiegsschritte, 22 Kapitel; Handbuchquelle: {words:,} Wörter")


if __name__ == "__main__":
    build()
