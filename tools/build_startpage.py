"""Baut den eigenständigen Spielguide aus Vorlage und aktuellem Handbuch."""

from __future__ import annotations

import html
import re
from pathlib import Path
from urllib.parse import urlsplit

import markdown
from site_policy import decorate
from markdown.extensions.toc import slugify
from website_content import finish, screenshot, chapter_visual


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "startseite.html"
DESTINATION = ROOT / "Sternenepoche-Start.html"
HANDBUCH = ROOT / "docs" / "HANDBUCH.md"

SCREENS = [
    ("reich", "Übersicht", 10), ("gebaeude", "Gebäude", 11),
    ("kolonie", "Versorgung", 10), ("forschen", "Forschung", 11),
    ("werft", "Schiffswerft", 11), ("verteidigung", "Verteidigung", 15),
    ("flotten", "Flotten & Saven", 14), ("karte", "Galaxie", 12),
    ("kolonien", "Kolonisation", 16), ("kampf", "Kampf & Verbände", 15),
    ("briefkasten", "Briefkasten", 17), ("allianzbereich", "Allianz", 17), ("allianzpost", "Allianzpost · Führung", 17),
    ("aktionen", "Markt", 17), ("berichte", "Berichte", 12),
    ("imperium", "Imperiumsvergleich", 19), ("freischaltungen", "Technologiebaum", 11),
    ("agent", "Modelle & Team", 20), ("pinwaende", "Pinwände", 20), ("profil", "Spielerprofil", 5),
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
    step_screens = {"01":"profil", "02":"kolonie", "03":"gebaeude", "04":"forschen", "05":"werft", "06":"flotten", "07":"karte", "08":"berichte"}
    first_html = render(intro) + '<div class="session-steps">' + ''.join(
        f'<article class="session-step" id="einstieg-{m[1]}"><span class="step-index" aria-hidden="true">{m[1]}</span>'
        f'<div><h3>{html.escape(m[2])}</h3>{screenshot(step_screens[m[1]])}{render(m[3])}</div></article>' for m in steps) + '</div>'
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
                      f'<h3 id="screen-{key}" tabindex="-1">{html.escape(name)}</h3>{screenshot(key)}{render(practical[name])}'
                      f'<a class="text-link" href="#{anchors[chapter]}">Die Spielregeln dazu lesen <span aria-hidden="true">↓</span></a></article>')
    screens_html = '<div class="screen-finder" id="screen-finder">' + (
        '<div class="screen-top"><strong>So orientierst du dich im Spiel</strong>'
        '<p>Oben: Vorräte des gewählten Planeten und reichsweite Credits. Links: Aufgabenbereiche. '
        'Rechts: Deine Planeten. Warnungen: zuerst lesen.</p></div>'
        '<label class="screen-select" hidden>Spielbereich auswählen<select id="screen-select">'
        + ''.join(options) + '</select></label><div class="screen-layout">'
        '<nav class="screen-nav" aria-label="Die 21 Bereiche der Spieloberfläche">'
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
            content = 'Der [Oberflächen-Wegweiser](#oberflaeche) erklärt alle 21 Bereiche: wo du etwas findest, welche Knöpfe du benutzt, was danach passiert und welche Fehler du prüfen solltest.\n\n' + chapters[8][1].split('### ', 1)[0]
        label = f'{number:02d} · {title}'
        toc.append(f'<li><a href="#{target}">{html.escape(label)}</a></li>')
        chapter = render(heading + content)
        chapter = re.sub(r'(</h3>)', lambda m: m[1] + chapter_visual(number), chapter, count=1)
        articles.append(f'<section class="guide-chapter" aria-labelledby="{target}" data-search-title="{html.escape(title, quote=True)}">{chapter}<a class="chapter-back" href="#spielguide">↑ Zur Kapitelauswahl</a></section>')
    book = '<div class="guide-layout"><aside class="guide-index"><details open><summary>22 Kapitel zum Nachschlagen</summary><nav aria-label="Spielguide-Kapitel"><ol>' + ''.join(toc) + '</ol></nav></details><p>Kapitel 20 hilft dir, eigene OpenRouter- oder Ollama-Modelle für dein Reich einzurichten.</p></aside><div class="guide-reading">' + ''.join(articles) + '</div></div>'
    return {"ERSTE_SITZUNG": first_html, "OBERFLAECHE": screens_html, "SPIELGUIDE": book}, len(source.split())


def build() -> None:
    document = SOURCE.read_text(encoding="utf-8")
    parts, words = guide_parts()
    for name, body in parts.items():
        marker = f"<!-- {name} -->"
        if document.count(marker) != 1:
            raise ValueError(f"Genau ein Inhaltsplatzhalter erforderlich: {name}")
        document = document.replace(marker, body)
    document = finish(document)
    document = decorate(document, "KI-Hinweis.html")
    DESTINATION.write_text(document, encoding="utf-8")
    print(f"Erstellt: {DESTINATION}")
    print(f"Aktuelle Screenshots und Spielmotive eingebettet, {DESTINATION.stat().st_size:,} Bytes")
    print(f"20 Bedienwege, 8 Einstiegsschritte, 22 Kapitel; Handbuchquelle: {words:,} Wörter")


if __name__ == "__main__":
    build()
