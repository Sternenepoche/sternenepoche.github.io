"""Baut die öffentliche Sternenepoche-Website 0.4 komplett aus den aktuellen Quellen – kein Text aus der alten Seite.

Quellen: docs/HANDBUCH.md (Regeln, 32 Kapitel), docs/flottenkodex.json + docs/technologie-regeln.json (Kern-Export),
content/beschreibungen.json (Texte, Archiv), lore/Lore-Gesamtwerk.md (Lore), docs/rat/ (Rat), web-client/art.js
(Motive), web-client/techbaum/ (Technologiebaum), docs/website/fotos/ (frische Bildschirmfotos, tools/website/fotos.py),
docs/website/medien/ (Weltenkrümmer-Filme, tools/website/kopfvideo.py), docs/android-download.json, docs/cockpit-download.json.
Gestaltung: docs/website/stil.css + docs/action-surfaces.css, Verhalten docs/website/seite.js.

Aufruf:
  python -X utf8 tools/website/bauen.py                 Vorschau nach ausgabe/website/ (verweist auf ../../ im Projekt)
  python -X utf8 tools/website/bauen.py --pages         Seiten ins Projektwurzelverzeichnis (für tools/build_pages.py)
"""
from __future__ import annotations

import argparse, html, json, re, shutil, sys
from datetime import date
from pathlib import Path

import markdown

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from site_policy import decorate  # noqa: E402
import build_fleet_codex  # noqa: E402

Q = ROOT / "docs/website"
VERSION = "0.4.6"
# Öffentliche Spieladresse; im Pages-Repo fehlt data/online (Betriebsdaten werden nie veröffentlicht).
_URL = ROOT / "data/online/public-url.txt"
SPIEL_URL = (_URL.read_text(encoding="utf-8").strip() if _URL.exists() else "https://sternenepoche.taila4f584.ts.net").rstrip("/") + "/"
VOELKER = [("aurelianer", "Aurelianer", "#ffcf33"), ("krath", "Krath", "#ff4a2e"),
           ("veyari", "Veyari", "#5dff7a"), ("syntheten", "Syntheten", "#5ab8ff")]
VOLKFARBE = {v: f for v, _, f in VOELKER}
VOLKNAME = {v: n for v, n, _ in VOELKER}
esc = lambda s: html.escape(str(s), quote=True)

NAV = [("index.html", "Start", "Überblick"), ("Trailer.html", "Trailer", "Der Weltenkrümmer im Film"),
       ("Sternenepoche-Start.html", "Spielguide", "Einstieg und alle Spielbereiche"),
       ("Sternenepoche-Handbuch.html", "Handbuch", "Alle Regeln in 32 Kapiteln"),
       ("Sternenepoche-Kodex.html", "Kodex", "Einheiten, Werte, Zeichnungen"),
       ("Technologiebaum.html", "Technologiebaum", "Alle Völker auf einer Karte"),
       ("Weltenkruemmer.html", "Weltenkrümmer", "Die vier Mondbrecher"),
       ("Lore.html", "Lore-Archiv", "Kanon, Archiv, Rat"),
       ("Hoerchronik.html", "Hörchronik", "Die Geschichte der Weite in fünf Folgen"),
       ("Harness-Cockpit.html", "Mit KI spielen", "Modelle, Rat, Harness-API"),
       ("Download.html", "Download", "Android-App und Cockpit")]
# Leise Seiten: im Menü am Handy, im Fuß und auf den passenden Seiten verlinkt, nicht in der Kopfleiste.
NEBEN: list = []  # Karl 22.10.: die Hörchronik gehört sichtbar in die Kopfleiste
ALLE = NAV + NEBEN
HC = ROOT / "docs/website/medien/hoerchronik"
TRAILER = ROOT / "docs/website/medien/trailer"

# Frische Fotos (docs/website/fotos, 18.10.2026) je Spielbereich – Name wie im Spielmenü (web-client/rahmen.js).
BEREICHSFOTO = {"Übersicht": "uebersicht", "Anlagen": "anlagen", "Versorgung": "versorgung", "Forschung": "forschung",
                "Schiffswerft": "werft", "Verteidigung": "verteidigung", "Truppen": "truppen", "Flotte": "flotte",
                "Galaxie": "galaxie", "Kolonisation": "kolonisation", "Kampf & Verbände": "kampfsimulator",
                "Handel": "handel", "Nachrichten": "briefkasten", "Allianz": "allianz", "Großprojekt": "grossprojekt",
                "Berichte": "berichte", "Imperium": "imperium", "Technologiebaum": "techbaum-nah", "Rat": "rat",
                "Pinwände": "pinwaende", "Mond": "mond"}
KAPITELFOTO = {7: "uebersicht", 10: "produktion", 11: "forschung", 12: "galaxie", 14: "flotte", 15: "kampfsimulator",
               16: "kolonisation", 17: "handel", 19: "imperium", 20: "rat", 24: "auftraege", 26: "mond", 29: "versorgung",
               30: "lage", 31: "grossprojekt", 32: "flotte"}


def lade(pfad):
    return json.loads((ROOT / pfad).read_text(encoding="utf-8"))


ART = json.loads((ROOT / "web-client/art.js").read_text(encoding="utf-8").removeprefix("window.STERNEN_ART=").strip().removesuffix(";"))
KODEX = lade("docs/flottenkodex.json")
REGELN = lade("docs/technologie-regeln.json")
TEXTE = lade("content/beschreibungen.json")["eintraege"]
from heute import heute  # noqa: E402
HANDBUCH = heute((ROOT / "docs/HANDBUCH.md").read_text(encoding="utf-8"))
FOTOS = lade("docs/website/fotos/fotos.json")
KOPF = lade("docs/website/medien/weltenkruemmer.json")


def android_daten() -> dict:
    """Download-Daten der App. STERN_ANDROID_JSON: Vorschau eines gebauten, noch nicht veröffentlichten Release
    (JSON aus dem Probelauf von android/veroeffentlichen_*.py) – docs/android-download.json schreibt nur das Release."""
    import os
    return json.loads(Path(os.environ["STERN_ANDROID_JSON"]).read_text(encoding="utf-8")) if os.environ.get("STERN_ANDROID_JSON") else lade("docs/android-download.json")


def slug(text: str) -> str:
    text = re.sub(r"<[^>]+>", "", html.unescape(text)).strip().lower()
    return re.sub(r"\s+", "-", re.sub(r"[^\w\s-]", "", text))


def zahl_de(x) -> str:
    """Zahl mit deutschem Tausenderpunkt und Komma."""
    if isinstance(x, float) and not x.is_integer():
        return str(x).replace(".", ",")
    return f"{int(x):,}".replace(",", ".")


def kapitel_liste():
    return [(int(m[1]), m[2].strip(), m[3]) for m in re.finditer(r"^## (\d+) ([^\n]+)\n(.*?)(?=^## |\Z)", HANDBUCH, re.M | re.S)]


class Bau:
    def __init__(self, ziel: Path, basis: str):
        self.ziel, self.basis = ziel, basis
        self.css_basis = "../" + basis  # CSS liegt in <ziel>/seite/

    # ---------- Pfade ----------
    def u(self, pfad: str) -> str:
        return self.basis + pfad

    def art(self, kat: str, key: str, volk: str = "") -> str:
        im, al = ART["images"], ART.get("aliases", {})
        for k in (f"{kat}|{key}", al.get(f"{kat}|{key}")):
            if not k:
                continue
            for v in (volk, "", "aurelianer"):
                if f"{k}|{v}" in im:
                    return self.u("web-client/" + im[f"{k}|{v}"]["url"])
        return ""

    def foto(self, name: str, titel: str, text: str = "", klasse: str = "glas") -> str:
        meta = FOTOS.get(name + ".webp")
        if not meta:
            raise ValueError("Foto fehlt: " + name)
        pfad = self.u(f"docs/website/fotos/{name}.webp")
        return (f'<figure class="foto {klasse}"><button type="button" data-gross="{pfad}" data-titel="{esc(titel)}" aria-label="Bild vergrößern: {esc(titel)}">'
                f'<img src="{pfad}" alt="Spieloberfläche: {esc(titel)}" width="{meta["breite"]}" height="{meta["hoehe"]}" loading="lazy" decoding="async">'
                f'<span class="lupe" aria-hidden="true">Vergrößern</span></button><figcaption><b>{esc(titel)}</b>{esc(text)}'
                f'{" · " if text else ""}<span class="leise">Echte Oberfläche</span></figcaption></figure>')

    def ausschnitt(self, name: str) -> str:
        """Inhaltsbereich eines Fotos (ohne Menü, Rohstoffleiste, Kolonienliste) für Kacheln – lesbar statt verkleinert."""
        from PIL import Image
        quelle = ROOT / f"docs/website/fotos/{name}.webp"
        ziel = ROOT / f"docs/website/fotos/ausschnitt/{name}.webp"
        if not ziel.exists() or ziel.stat().st_mtime < quelle.stat().st_mtime:
            ziel.parent.mkdir(parents=True, exist_ok=True)
            box = {"galaxie-3d": (219, 315, 1207, 871), "techbaum-karte": (190, 84, 1207, 656)}.get(name) or ((232, 4, 1208, 553) if name.startswith(("technik", "archiv")) else (205, 160, 1222, 732))
            with Image.open(quelle) as im:
                im.convert("RGB").crop(box).save(ziel, "WEBP", quality=86, method=6)
        return self.u(f"docs/website/fotos/ausschnitt/{name}.webp")

    # ---------- Markdown ----------
    def md(self, text: str, verweise: dict | None = None) -> str:
        # Versionsnotizen alter APKs sind keine Regel – sie würden auf der neuen Seite in die Irre führen.
        text = re.sub(r"^\*\*Stand des [^*]+:\*\*[^\n]*Android 0\.3[^\n]*\n", "", text, flags=re.M)
        # Eingebettete Altbilder (docs/bilder/team, kommunikation: 8.–11.10.) zeigen die alte Oberfläche – nur frische Fotos.
        text = re.sub(r"!\[[^\]]*\]\([^)]*\)(\{[^}]*\})?", "", text)
        text = re.sub(r"<figure\b.*?</figure>|<img\b[^>]*>", "", text, flags=re.S)
        h = markdown.markdown(text, extensions=["tables", "sane_lists", "toc"],
                              extension_configs={"toc": {"slugify": lambda v, s: slug(v)}})
        h = h.replace("<table>", '<div class="tabelle" tabindex="0" role="region" aria-label="Tabelle"><table>').replace("</table>", "</table></div>")
        h = re.sub(r"<p>(?=<strong>(?:Beispiel|Zeitbeispiel|Frachtbeispiel|Rechenbeispiel))", '<p class="beispiel">', h)

        def link(m):
            ziel = html.unescape(m[1])
            if ziel.startswith(("http://", "https://", "#", "mailto:")):
                return m[0]
            name = ziel.split("#")[0].split("/")[-1]
            anker = "#" + ziel.split("#", 1)[1] if "#" in ziel else ""
            if name.endswith(".html") and any(name == n for n, _, _ in NAV):
                return f'href="{esc(name + anker)}"'
            if verweise and name in verweise:
                return f'href="{esc(verweise[name])}"'
            return 'data-intern="1"'
        h = re.sub(r'href="([^"]+)"', link, h)
        # Verweise auf interne Arbeitsdateien werden zu reinem Text.
        h = re.sub(r'<a data-intern="1">(.*?)</a>', r"\1", h, flags=re.S)
        return h

    # ---------- Hülle ----------
    def huelle(self, datei: str, titel: str, beschreibung: str, inhalt: str, kopf_extra: str = "", fuss_extra: str = "",
               koerper_klasse: str = "", nur_kopf: bool = False) -> str:
        nav = "".join(f'<a href="{d}"{" aria-current=\"page\"" if d == datei else ""}>{esc(t)}</a>' for d, t, _ in NAV[1:])
        mobil = "".join(f'<a href="{d}"{" aria-current=\"page\"" if d == datei else ""}>{esc(t)}<small>{esc(u)}</small></a>' for d, t, u in ALLE)
        jetzt = date.today().strftime("%d.%m.%Y")
        doc = f"""<!doctype html>
<html lang="de"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<title>{esc(titel)}</title>
<meta name="description" content="{esc(beschreibung)}">
<meta name="theme-color" content="#040913">
<meta property="og:title" content="{esc(titel)}"><meta property="og:description" content="{esc(beschreibung)}"><meta property="og:type" content="website">
<meta property="og:image" content="https://sternenepoche.github.io/docs/website/fotos/uebersicht.webp">
<link rel="icon" type="image/png" href="{self.u('docs/branding/favicon.png')}">
<link rel="preload" href="{self.u('web-client/fonts/oxanium-latin-600-normal.woff2')}" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="seite/kopf.css?v={self.stand}">
{"" if nur_kopf else f'<link rel="stylesheet" href="seite/stil.css?v={self.stand}">'}
{kopf_extra}
</head>
<body class="{koerper_klasse}">
<a class="springen" href="#inhalt">Zum Inhalt</a>
<header class="sk-kopf"><div class="sk-rahmen sk-innen">
<a class="sk-marke" href="index.html" aria-label="Sternenepoche – Startseite"><img src="{self.u('docs/branding/sternenepoche-mark.png')}" alt="" width="30" height="30">STERNENEPOCHE</a>
<nav class="sk-nav" aria-label="Hauptnavigation">{nav}</nav>
<a class="button spielen" href="{SPIEL_URL}">Spielen ↗</a>
<details class="sk-menue"><summary>Menü</summary><nav aria-label="Navigation">{mobil}<a href="{SPIEL_URL}">Spielen ↗<small>Im Browser, sofort</small></a></nav></details>
</div></header>
<main id="inhalt">
{inhalt}
</main>
<footer class="fuss"><div class="rahmen">
<div class="fuss-raster">
<div><a class="sk-marke" href="index.html"><img src="{self.u('docs/branding/sternenepoche-mark.png')}" alt="" width="30" height="30">STERNENEPOCHE</a>
<p style="margin-top:12px">Weltraumstrategie in einer gemeinsamen Galaxie – im Browser, als Android-App und mit deinem eigenen KI-Rat.</p></div>
<div><h4>Spielen</h4><ul><li><a href="{SPIEL_URL}">Im Browser spielen</a></li><li><a href="Download.html">Android-App</a></li><li><a href="Harness-Cockpit.html">Harness-Cockpit</a></li><li><a href="Sternenepoche-Start.html">Erste Schritte</a></li></ul></div>
<div><h4>Nachschlagen</h4><ul><li><a href="Sternenepoche-Handbuch.html">Handbuch</a></li><li><a href="Sternenepoche-Kodex.html">Kodex</a></li><li><a href="Technologiebaum.html">Technologiebaum</a></li><li><a href="Lore.html">Lore-Archiv</a></li><li><a href="Hoerchronik.html">Hörchronik</a></li></ul></div>
<div><h4>Projekt</h4><ul><li><a href="Trailer.html">Trailer</a></li><li><a href="Weltenkruemmer.html">Weltenkrümmer</a></li><li><a href="KI-Hinweis.html">KI-Hinweis</a></li><li><a href="https://github.com/Sternenepoche/sternenepoche.github.io">Quellcode</a></li></ul></div>
</div>
<p class="ki">Bild, Ton und Text der Sternenepoche sind KI-generiert. Stand {jetzt}.</p>
</div></footer>
<script src="seite/seite.js?v={self.stand}" defer></script>
{fuss_extra}
</body></html>"""
        return decorate(doc, "KI-Hinweis.html")

    def seitenkopf(self, kennung: str, titel: str, text: str, bild: str = "", krumen: str = "") -> str:
        # Bild direkt am Element: eine url() in einer CSS-Variablen würde relativ zur Stildatei aufgelöst.
        hinter = f'<div class="seitenkopf-bild" aria-hidden="true" style="background-image:url(\'{bild}\')"></div>' if bild else ""
        return (f'<section class="seitenkopf">{hinter}<div class="rahmen">'
                f'<nav class="brotkrumen" aria-label="Pfad"><a href="index.html">Start</a><span aria-hidden="true">›</span><span>{esc(krumen or titel)}</span></nav>'
                f'<span class="kennung">{esc(kennung)}</span><h1>{titel}</h1><p class="einleitung">{text}</p></div></section>')

    def schreibe(self, name: str, inhalt: str):
        ziel = self.ziel / name
        ziel.parent.mkdir(parents=True, exist_ok=True)
        ziel.write_text(inhalt, encoding="utf-8")
        print(f"  {name:32} {len(inhalt.encode('utf-8')) / 1024:8.0f} KB")

    # ===================================================================== Startseite
    def start(self):
        units = KODEX["units"]
        boden = REGELN["bodentruppen"]["typen"]
        monde = REGELN.get("monde") or {}
        zeichnungen = sum(1 for k in ART["images"] if k.startswith("blueprints|"))
        archiv = sum(1 for e in TEXTE.values() if isinstance(e.get("archiv"), dict))
        forschungen = len(REGELN["forschung"])
        marken = json.dumps(KOPF["voelker"], ensure_ascii=False)
        balken = "".join("<i></i>" for _ in KOPF["voelker"])
        medien = self.u("docs/website/medien/weltenkruemmer")
        buehne = f"""
<section class="buehne" aria-labelledby="titel">
 <div class="buehne-raum" aria-hidden="true"></div>
 <div class="buehne-szene" aria-hidden="true">
  <div class="buehne-bahn"></div><div class="buehne-bahn"></div>
  <div class="buehne-planet"><img src="{self.u('docs/artwork/hero-ozeanwelt.webp')}" alt="" width="1200" height="1200" fetchpriority="high" decoding="async"></div>
  <div class="buehne-film" data-kopffilm data-quelle="{medien}" data-standbild="{medien}-standbild.webp" data-marken='{esc(marken)}'>
   <img src="{medien}-standbild.webp" alt="" hidden width="960" height="540">
  </div>
 </div>
 <div class="buehne-kino" aria-hidden="true" data-kinofilm data-quelle="{self.u('docs/website/medien/kopffilm')}" data-standbild="{self.u('docs/website/medien/kopffilm-standbild.webp')}" data-titel="Aus dem Spiel: ein Mond bricht"></div>
 <div class="rahmen buehne-text">
  <a class="version" href="#spiel"><b>Online</b>Browser · Android · KI-Rat</a>
  <h1 id="titel">Weltraumstrategie<span>für Menschen und KI</span></h1>
  <p class="einleitung">Du fängst mit einem Planeten und knappen Vorräten an. Eine Runde dauert 365 Spieltage. In der Zeit forschst du, baust Flotten,
  gewinnst Monde, öffnest Hyperraumlöcher und errichtest mit deiner Allianz eine Dysonsphäre. Wer einen gegnerischen Mond zerstören will, braucht einen Weltenkrümmer.</p>
  <div class="knopfreihe"><a class="button" href="{SPIEL_URL}">Im Browser spielen <span aria-hidden="true">↗</span></a>
  <a class="button secondary trailer-knopf" href="Trailer.html"><span aria-hidden="true">▶</span> Trailer ansehen</a><a class="button secondary" href="Download.html">Android-App laden</a></div>
  <p class="buehne-fuss">Menschen, Serverbots und KI-Agenten spielen in derselben Galaxie nach denselben Regeln. Zuschauen geht ohne Konto.</p>
 </div>
 <div class="buehne-hud" aria-live="off"><span>Im Film</span><b>Weltenkrümmer</b><div class="balken">{balken}</div><button type="button">Film anhalten</button></div>
</section>
<div class="kennzahlen" aria-label="Sternenepoche in Zahlen">
 <div><b>4</b><span>Völker mit eigenen Stärken, Schiffen und Weltenkrümmern</span></div>
 <div><b>{len(units)}</b><span>Schiffe, Abwehr, Schilde und Megaschiffe je Volk</span></div>
 <div><b>{len(boden)}</b><span>Bodentruppen mit Kontertabelle für die Invasion</span></div>
 <div><b>{len(monde.get('gebaeude', {}))} + 10</b><span>Mondgebäude und Mondtypen</span></div>
 <div><b>{zeichnungen}</b><span>technische Zeichnungen: Blaupause, Explosion, Schnitt</span></div>
 <div><b>{archiv}</b><span>Archiv-Einträge mit Dossier, Chronik und Volksakten</span></div>
</div>"""
        def karte(klasse, bild, kennung, titel, text, punkte, link, linktext, ganz=False):
            li = "".join(f"<li>{p}</li>" for p in punkte)
            return (f'<article class="neu glas {klasse}"><div class="neu-bild">{bild if bild.startswith("<") else f"<img src=\"{bild}\" alt=\"\" loading=\"lazy\" decoding=\"async\"{" class=\"ganz\"" if ganz else ""}>"}</div>'
                    f'<div class="neu-text"><span class="marke-klein">{esc(kennung)}</span><h3>{titel}</h3><p>{text}</p>{f"<ul>{li}</ul>" if li else ""}'
                    f'<a class="weiter" href="{link}">{linktext} →</a></div></article>')
        f = self.ausschnitt
        wk = KODEX["units"]["weltentruemmer"]
        dy = REGELN["dyson"]["regeln"]
        neu = f"""
<section class="abschnitt" id="spiel" aria-labelledby="spiel-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Das Spiel</span><h2 id="spiel-titel">Was dich in der Galaxie erwartet.</h2></div>
 <p>Du führst Flotten über Sektoren hinweg, gewinnst Monde aus großen Schlachten, belagerst feindliche Kolonien und fasst mit deiner Allianz eine Sonne ein.
 Zu jeder Einheit gibt es Werte, Zeichnungen und ihre Geschichte. Alle Bilder sind Ansichten aus dem Spiel.</p></div>
 <div class="neu-raster">
 {karte("breit", f("flotte"), "Flottenführung", "Versand in drei Schritten", "Der Flottenversand führt dich von Ziel und Auftrag über die Flotte bis zu Zeit und Verband – mit Prüfung bei jedem Schritt.",
   ["<b>Ziel &amp; Auftrag:</b> Schnellziele, Zielkarte und Aufklärungskette – unpassende Aufträge stehen abgeblendet mit Grund.",
    "<b>Flotte:</b> Bedarfsrechner für Beute und Frachtraum, Simulation mit 64 Läufen samt Trümmerfeld und Mondchance.",
    "<b>Zeit &amp; Verband:</b> zehn Tempostufen mit Ankunft und Deuterium, Ankunft koppeln, Terminflug bis 7 Spieltage voraus.",
    "<b>Farbige Auftragszeichen</b> in acht Gruppen überall im Spiel; Verbände bis 16 Flotten aus deiner Allianz."],
   "Sternenepoche-Handbuch.html#kapitel-32", "Kapitel 32 lesen")}
 {karte("", self.u("docs/website/medien/filme/krath.jpg"), "Megaschiff", "Weltenkrümmer", f"Eine mondgroße Kampfstation aus verschmolzenen Trümmern mit {zahl_de(wk['struktur'])} Struktur und {zahl_de(wk['angriff'])} Angriff auf vier Ziele je Runde. Sie entsteht im Werftdock eines Mondes.",
   ["Nach einem vollständigen Sieg bricht sie den Mond des Gegners.", "Drei Schlachtkreuzer ergänzen die Linie: Linien-, Abfang- und schwerer Schlachtkreuzer."],
   "Weltenkruemmer.html", "Die vier Weltenkrümmer")}
 {karte("", f("mond"), "Monde", "Monde, Hyperraumtor, Phalanx", "Aus großen Schlachten entstehen Monde. Jeder der zehn Mondtypen hat einen eigenen Vor- und Nachteil, sieben Mondgebäude machen ihn zu Hafen, Werft und Torstation.",
   ["Der Hyperraumtor-Generator öffnet neben seinem Mond ein öffentliches Hyperraumtor zu einem zweiten Generator.", "Die Tiefenphalanx zeigt die Flüge eines Planeten in Reichweite."],
   "Sternenepoche-Handbuch.html#kapitel-26", "Kapitel 26 und 27")}
 {karte("", f("lage"), "Invasion", "Bomben, landen, belagern", "Eine Kolonie fällt in drei Abschnitten: Bomber senken alle Gebäude auf 30 % Integrität, Truppentransporter landen, dann 48 Spielstunden Belagerung.",
   [f"{len(boden)} Bodentruppen mit Kontertabelle, Kaserne und Garnison.", "Verbündete schicken Bodenhilfe; einmal je Belagerung die Volksmiliz."],
   "Sternenepoche-Handbuch.html#kapitel-30", "Kapitel 30 lesen")}
 {karte("", f("grossprojekt"), "Allianz-Großprojekt", "Die Dysonsphäre", f"Das größte Bauwerk des Spiels: Eine Allianz umschließt eine Sonne. Nach {dy['bau_sekunden'] // 86400} Spieltagen Bau liefert sie den {zahl_de(dy['sonnenfaktor'])}-fachen Sonnenfaktor und versorgt bis zu {dy['anschluesse']} angeschlossene Planeten.",
   [f"Allianzen mit bis zu {REGELN['allianz_max']} Mitgliedern.", "Die Sphäre wirkt, solange die Allianz sechs der zwölf Plätze hält."],
   "Sternenepoche-Handbuch.html#kapitel-31", "Kapitel 31 lesen")}
 {karte("", self.art("blueprints", "explosion.ships.kreuzer", "krath"), "Techinfo „Technik“", "Technische Zeichnungen", f"Zu jedem Schiff, jeder Bodentruppe und jedem Weltenkrümmer gibt es Blaupause, Explosionszeichnung und Schnitt – in der Bauart jedes Volkes, zusammen {zeichnungen} Zeichnungen.",
   [], "Sternenepoche-Kodex.html", "Im Kodex ansehen")}
 {karte("", self.art("backgrounds", "archive"), "Techinfo „Archiv“", "Lore-Archiv", f"Jeder der {archiv} Einträge hat ein technisches Dossier, eine Chronik und die Akten aller vier Völker. Dazu der Kanon der Weite und die Stimmen des Rates.",
   [f'Zum Hören: die <a href="Hoerchronik.html">Hörchronik</a> – {self.hc_kurz()}.'], "Lore.html", "Zum Lore-Archiv")}
 {karte("", f("techbaum-karte"), "Gesamtkarte", "Technologiebaum", f"Alle Gebäude, {forschungen} Forschungen und Einheiten aller Völker auf einer Karte, mit Bildern, Voraussetzungen und Kampfauswertung – auf- und zuklappbar.",
   [], "Technologiebaum.html", "Technologiebaum öffnen")}
 {karte("halb", f("versorgung"), "Wirtschaft", "Wachstum, Klima und Sonne", "Jedes Volk hat seine Heimatbahn. Der Klimafaktor eines Platzes bestimmt das Wachstum, der Sonnenfaktor die Energie aus Solarkraftwerk und Solarsatelliten.",
   ["In jedem Sonnensystem liegt ein Asteroidengürtel für Bergbauschiffe.", "Die Akademie bildet Fachkräfte und Forscher kostenlos aus.", "Plasmatechnik steigert die Förderung, Antriebsforschung bringt stärkere Antriebe."],
   "Sternenepoche-Handbuch.html#kapitel-29", "Kapitel 29 lesen")}
 {karte("halb", f("galaxie-3d"), "Galaxie", "Sonnensysteme in 3D", "Jedes Sonnensystem lässt sich frei drehen und zoomen: Planeten mit Wolken und Stadtlichtern, Monde, Asteroidengürtel, Trümmerfelder und Nebel. Ein Tipp auf ein Objekt zeigt, was dort ist und was du tun kannst.",
   ["Weltenkrümmer erscheinen aus einem Hyperraumloch und sind im System zu sehen.", "Am Handy öffnet die Galaxie direkt in 3D, die Tabelle ist einen Tipp entfernt."],
   "Sternenepoche-Start.html#bereich-galaxie", "Zur Galaxie im Spielguide")}
 </div></div></section>"""
        tb = json.loads((lambda s: s[s.index("{"):s.rindex("}") + 1])((ROOT / "web-client/techbaum/daten.js").read_text(encoding="utf-8")))
        tbk = list(tb["knoten"].values())
        zahl = lambda *typen: sum(1 for x in tbk if x.get("typ") in typen)
        techbaum = f"""
<section class="abschnitt techbaum-teaser" id="technologiebaum" aria-labelledby="tb-titel"><div class="rahmen">
 <div class="tb-raster">
  <div class="tb-text"><img class="tb-zeichen" src="seite/techbaum-logo.webp" alt="" width="120" height="120">
   <span class="kennung">Gesamtkarte</span><h2 id="tb-titel">Der ganze Technologiebaum auf einer Karte.</h2>
   <p>{len(tbk)} Einträge in {len(tb['bereiche'])} Bereichen – {zahl('gebaeude')} Gebäude, {zahl('forschung')} Forschungen, {zahl('schiff', 'megaschiff')} Schiffe, {zahl('boden')} Bodentruppen,
   {zahl('verteidigung', 'schild', 'rakete')} Abwehranlagen und Raketen, {zahl('mond')} Mondanlagen, {zahl('mission')} Missionen – für alle vier Völker.</p>
   <ul class="tb-liste"><li><b>Pfade:</b> Wähle ein Ziel, und die Karte zeigt den Weg dorthin über alle Voraussetzungen.</li>
   <li><b>Dossier:</b> Motiv, Blaupause, Schnitt und Explosion, Werte mit Volksbonus, Kosten und Konter.</li>
   <li><b>Meine Stufe &amp; Volksänderungen:</b> Was ist bei deiner Zivilisationsstufe offen, was macht dein Volk anders?</li>
   <li><b>Suche, Minikarte, Klappen</b> – auf dem Handy mit zwei Fingern zoomen.</li></ul>
   <div class="knopfreihe"><a class="button" href="Technologiebaum.html">Technologiebaum öffnen</a></div></div>
  <div class="tb-bilder">{self.foto("techbaum-nah", "Technologiebaum · Dossier", "Weltenkrümmer der Syntheten mit Pfad und Werten")}
   {self.foto("techbaum-karte", "Kampfflotte und Verteidigung", "Ansicht der Syntheten")}</div>
 </div></div></section>"""
        galerie = f"""
<section class="abschnitt" id="ansicht" aria-labelledby="ansicht-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Aus dem Spiel</span><h2 id="ansicht-titel">Deine Brücke.</h2></div>
 <p>Links das Menü, oben Vorräte und Ereignisse, rechts deine Kolonien. Jede Seite folgt demselben Aufbau: Banner, Kacheln, Detailfeld mit Bauknopf.</p></div>
 <div class="galerie">{self.foto("uebersicht", "Übersicht", "Lagebericht, Zivilisationsstufe, Grundausbildung und Flottenbewegungen")}
 <div class="stapel">{self.foto("werft", "Schiffswerft", "Alle Schiffe des Volkes mit Bestand")}{self.foto("galaxie-3d", "Galaxie in 3D", "Sonnensystem mit Planeten, Monden und Asteroidengürtel")}</div></div>
</div></section>"""
        vtab = self.voelker_tabelle()
        voelker = "".join(f"""
 <article class="volk glas" style="--farbe:{farbe}"><div class="volk-bild"><img src="{self.art('portraits', v, v)}" alt="{name}" loading="lazy" width="480" height="480">
 <img class="volk-wappen" src="{self.art('emblems', v, v)}" alt="" loading="lazy" width="44" height="44"></div>
 <div class="volk-text"><h3>{name}</h3><dl><dt>Stärken</dt><dd>{vtab[name][0]}</dd><dt>Schwächen</dt><dd class="minus">{vtab[name][1]}</dd>
 <dt>Heimatbahn</dt><dd>Platz {vtab[name][2]} · Weltenkrümmer-Strahl in <span style="color:{farbe}">{vtab[name][3]}</span></dd></dl></div></article>""" for v, name, farbe in VOELKER)
        voelker = f"""
<section class="abschnitt" id="voelker" aria-labelledby="voelker-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Vier Zivilisationen</span><h2 id="voelker-titel">Wähle dein Volk.</h2></div>
 <p>Das Volk verändert Spielwerte und Aussehen – Schiffe, Gebäude, Monde, Zeichnungen und Weltenkrümmer haben je Volk eine eigene Bauart. Es bleibt für die ganze Epoche.</p></div>
 <div class="voelker">{voelker}</div>
 <p style="margin-top:20px"><a href="Sternenepoche-Handbuch.html#kapitel-4">Alle Volkswerte im Handbuch</a> · <a href="Lore.html#6-die-vier-völker">Geschichte der Völker im Lore-Archiv</a> · <a href="Hoerchronik.html">Jedes Volk erzählt seine Geschichte – zum Hören</a></p>
</div></section>"""
        rat = f"""
<section class="abschnitt" id="rat" aria-labelledby="rat-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Dein Rat · vier Stimmen je Volk</span><h2 id="rat-titel">Hör deinem Rat zu.</h2></div>
 <p>Stratege, Verwalter, Diplomat und Feldherr: Jedes Volk hat vier Ratsfiguren mit eigener Herkunft und eigener Stimme. Sie sprechen im Spiel bei Angriffen, Erfolgen und Rückschlägen. Klick auf eine Figur – sie stellt sich vor.</p></div>
 <div class="rat-volk" role="tablist" aria-label="Volk des Rates wählen">{"".join(f'<button type="button" role="tab" data-rat-volk="{v}"{" class=\"an\" aria-selected=\"true\"" if i == 0 else " aria-selected=\"false\""}><img src="{self.art("emblems", v, v)}" alt="" width="30" height="30">{n}</button>' for i, (v, n, _) in enumerate(VOELKER))}</div>
 <div class="rat-buehne" id="rat-buehne" data-basis="{self.u('docs/rat/')}" data-three="{self.u('web-client/three.min.js')}" data-volk="aurelianer"><div class="rat-leinwand"></div><span class="rat-hinweis">Ton an · Klick auf eine Figur</span><div class="rat-untertitel" aria-live="polite"></div></div>
 <div class="rat-karten" id="rat-karten"><noscript><p>Die Ratsbühne braucht JavaScript.</p></noscript></div>
</div></section>"""
        android = android_daten()
        wege = f"""
<section class="abschnitt" id="spielen" aria-labelledby="spielen-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Drei Wege ins Spiel</span><h2 id="spielen-titel">Spiel, wo du willst.</h2></div>
 <p>Online spielst du in der gemeinsamen Galaxie. Die Android-App und das Harness-Cockpit bringen dieselbe Spielwelt auch lokal mit – allein, im WLAN oder mit deinem eigenen KI-Rat.</p></div>
 <div class="wege">
  <article class="weg glas"><span class="weg-symbol">01</span><h3>Im Browser</h3><p>Anmelden, Volk wählen, Platz belegen – und los. Bauen, forschen und Flotten führen direkt im Browser, auf dem Handy mit eigener Menüleiste.</p><a class="button" href="{SPIEL_URL}">Im Browser spielen ↗</a></article>
  <article class="weg glas"><span class="weg-symbol">02</span><h3>Android-App</h3><p>Die ganze Spielwelt auf dem Telefon: Einzelspiel, WLAN-Partie für Freunde oder online. Als Lite-Fassung für Handys und als Vollversion auch für x86-Geräte.</p><a class="button secondary" href="Download.html#android">Zur App</a></article>
  <article class="weg glas"><span class="weg-symbol">03</span><h3>Mit KI regieren</h3><p>Verbinde Ollama oder OpenRouter mit deinem Reich, gib Rollen an Modelle ab oder lass im Harness-Cockpit einen Rat aus vier Agenten regieren.</p><a class="button secondary" href="Harness-Cockpit.html">Mit KI spielen</a></article>
 </div></div></section>"""
        wissen = f"""
<section class="abschnitt" id="wissen" aria-labelledby="wissen-titel"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Nachschlagen</span><h2 id="wissen-titel">Alles, was du wissen willst.</h2></div>
 <p>Die Website liest dieselben Quellen wie das Spiel: Regelwerte aus dem Kern, Texte und Archiv aus dem Inhaltskatalog, Bilder aus dem Bildverzeichnis.</p></div>
 <div class="neu-raster">
 {karte("", f("uebersicht"), "Einstieg", "Spielguide", f"Die erste Spielsitzung in acht Schritten und alle {len(BEREICHSFOTO)} Spielbereiche mit Bild und Bedienweg.", [], "Sternenepoche-Start.html", "Zum Spielguide")}
 {karte("", f("forschung"), "Regeln", "Handbuch", f"{len(kapitel_liste())} Kapitel: Wirtschaft, Forschung, Flüge, Kampf, Monde, Invasion, Dysonsphäre, Flottenführung – mit Rechenbeispielen.", [], "Sternenepoche-Handbuch.html", "Zum Handbuch")}
 {karte("", f("codex"), "Einheiten", "Kodex", f"{len(units)} Einheiten je Volk, {len(boden)} Bodentruppen, Raketen, Mondanlagen und Dysonsphäre – Werte, Konter und Zeichnungen.", [], "Sternenepoche-Kodex.html", "Zum Kodex")}
 </div></div></section>"""
        inhalt = buehne + neu + techbaum + galerie + voelker + rat + wege + wissen
        doc = self.huelle("index.html", "Sternenepoche – Weltraumstrategie für Menschen und KI",
                          f"Sternenepoche: Online-Weltraumstrategie in einer gemeinsamen Galaxie. Flottenführung, Monde, Weltenkrümmer, Invasion, Dysonsphäre – im Browser, als Android-App und mit eigenem KI-Rat.",
                          inhalt, kopf_extra=f'<link rel="preload" as="image" href="{self.u("docs/artwork/hero-ozeanwelt.webp")}">',
                          fuss_extra=f'<script src="{self.u("docs/rat/rat-buehne.js")}" defer></script>')
        self.schreibe("index.html", doc)

    def voelker_tabelle(self):
        abschnitt = dict((n, t) for n, _, t in [(k[0], k[1], k[2]) for k in kapitel_liste()])[4]
        zeilen = {}
        for m in re.finditer(r"^\| \*\*(\w+)\*\* \| ([^|]+) \| ([^|]+) \|", abschnitt, re.M):
            zeilen[m[1]] = [m[2].strip(), m[3].strip()]
        heimat = {"Aurelianer": 6, "Krath": 3, "Veyari": 8, "Syntheten": 10}
        strahl = {"Aurelianer": "Gold", "Krath": "Glutrot", "Veyari": "Grün", "Syntheten": "Blau"}
        for n in zeilen:
            zeilen[n] += [heimat[n], strahl[n]]
        if len(zeilen) != 4:
            raise ValueError("Völkertabelle in Kapitel 4 nicht gefunden")
        return zeilen

    # ===================================================================== Spielguide
    def guide(self):
        kap = {n: (t, x) for n, t, x in kapitel_liste()}
        erste = kap[7][1]
        intro = erste.split("### ", 1)[0]
        schritte = list(re.finditer(r"^### (\d+) ([^\n]+)\n(.*?)(?=^### |\Z)", erste, re.M | re.S))
        schrittfoto = {"01": "uebersicht", "02": "versorgung", "03": "anlagen", "04": "forschung", "05": "werft", "06": "flotte", "07": "bericht", "08": "auftraege"}
        teile = [f'<section data-suche><h2 id="erste-schritte"><span class="nr">Kapitel 7</span>Deine erste Spielsitzung</h2>{self.md(intro)}']
        for m in schritte:
            teile.append(f'<h3 id="schritt-{m[1]}">{m[1]} · {esc(m[2])}</h3>{self.foto(schrittfoto[m[1]], m[2], "", "glas kapitel-bild")}{self.md(m[3])}')
        teile.append("</section>")
        ober = kap[8][1]
        oben = ober.split("### ", 1)[0]
        bereiche = list(re.finditer(r"^### ([^\n]+)\n(.*?)(?=^### |\Z)", ober, re.M | re.S))
        teile.append(f'<section data-suche><h2 id="bereiche"><span class="nr">Kapitel 8</span>Alle Spielbereiche</h2>{self.md(oben)}</section>')
        leiste_b = []
        for m in bereiche:
            name = m[1].strip()
            sid = "bereich-" + slug(name)
            bild = self.foto(BEREICHSFOTO[name], name, "", "glas kapitel-bild") if name in BEREICHSFOTO else ""
            teile.append(f'<section data-suche><h3 id="{sid}">{esc(name)}</h3>{bild}{self.md(m[2])}</section>')
            leiste_b.append(f'<li><a href="#{sid}">{esc(name)}</a></li>')
        # Eigene Modelle einrichten (Kapitel 6): Pflicht im Spielguide (tools/player_publication_test.py, Anker agent-setup).
        teile.append(f'<section data-suche id="agent-setup"><h2 id="kapitel-6"><span class="nr">Kapitel 6</span>{esc(kap[6][0])}</h2>'
                     f'{self.foto("rat", "Rat · Modelle & Team", "Rollen an Modelle vergeben", "glas kapitel-bild")}{self.md(kap[6][1])}</section>')
        extra = [(22, "Die wichtigsten Begriffe"), (21, "Häufige Probleme")]
        for n, kurz in extra:
            teile.append(f'<section data-suche><h2 id="kapitel-{n}"><span class="nr">Kapitel {n}</span>{esc(kap[n][0])}</h2>{self.md(kap[n][1])}</section>')
        leiste = (f'<aside class="leiste glas"><details open><summary>Inhalt</summary>'
                  f'<label class="leiste-suche">Im Spielguide suchen<input type="search" placeholder="z. B. Mond, Verband, Kaserne"></label><p id="suche-status" class="sr-only" role="status"></p>'
                  f'<p class="gruppe">Einstieg</p><ol><li><a href="#erste-schritte">Erste Spielsitzung</a><ol>'
                  + "".join(f'<li><a href="#schritt-{m[1]}">{m[1]} · {esc(m[2])}</a></li>' for m in schritte)
                  + f'</ol></li></ol><p class="gruppe">Spielbereiche</p><ol><li><a href="#bereiche">So liest du die Oberfläche</a></li>{"".join(leiste_b)}</ol>'
                  f'<p class="gruppe">Mit KI spielen</p><ol><li><a href="#agent-setup">Eigene Modelle einrichten</a></li></ol>'
                  f'<p class="gruppe">Hilfe</p><ol><li><a href="#kapitel-22">Begriffe</a></li><li><a href="#kapitel-21">Häufige Probleme</a></li></ol></details></aside>')
        inhalt = (self.seitenkopf("Spielguide", "Vom ersten Bauauftrag zur eigenen Flotte.",
                                  f"Die erste Spielsitzung in acht Schritten und alle {len(bereiche)} Spielbereiche – jeder mit einem Bild der heutigen Oberfläche und dem Weg dorthin. Die vollständigen Regeln stehen im <a href=\"Sternenepoche-Handbuch.html\">Handbuch</a>.",
                                  self.u("web-client/assets/backgrounds.command.webp"), "Spielguide")
                  + f'<div class="rahmen lesen">{leiste}<article class="text">{"".join(teile)}</article></div>')
        self.schreibe("Sternenepoche-Start.html", self.huelle("Sternenepoche-Start.html", "Spielguide – Sternenepoche",
                      "Erste Schritte und alle Spielbereiche der Sternenepoche mit Bildern der aktuellen Oberfläche.", inhalt))

    # ===================================================================== Handbuch
    def handbuch(self):
        teile, leiste = [], []
        gruppen = [(1, "Grundlagen"), (7, "Spielen"), (12, "Galaxie, Flotten, Kampf"), (17, "Diplomatie und Wertung"), (20, "Hilfe"), (24, "Flotten, Monde und Großprojekte")]
        for n, titel, text in kapitel_liste():
            if n == 23:
                continue
            for ab, g in gruppen:
                if ab == n:
                    leiste.append(f'</ol><p class="gruppe">{g}</p><ol>')
            bild = ""
            if n in KAPITELFOTO:
                bild = self.foto(KAPITELFOTO[n], titel, "", "glas kapitel-bild")
            elif n == 28:
                bild = (f'<figure class="foto glas kapitel-bild"><img src="{self.art("megaships", "todesmond", "aurelianer")}" alt="Weltenkrümmer der Aurelianer" loading="lazy" style="aspect-ratio:1;object-fit:contain;background:#02060c">'
                        f'<figcaption><b>Weltenkrümmer der Aurelianer</b>Alle vier Völker im Film: <a href="Weltenkruemmer.html">Weltenkrümmer</a></figcaption></figure>')
            body = self.md(text)
            unter = re.findall(r'<h3 id="([^"]+)">(.*?)</h3>', body)
            leiste.append(f'<li><a href="#kapitel-{n}">{n} · {esc(titel)}</a>'
                          + (f'<ol>{"".join(f"<li><a href=\"#{i}\">{t}</a></li>" for i, t in unter[:12])}</ol>' if n >= 24 else "") + "</li>")
            teile.append(f'<section data-suche><h2 id="kapitel-{n}"><span class="nr">Kapitel {n}</span>{esc(titel)}</h2>{bild}{body}</section>')
        aside = (f'<aside class="leiste glas"><details open><summary>32 Kapitel</summary>'
                 f'<label class="leiste-suche">Im Handbuch suchen<input type="search" placeholder="z. B. Phalanx, Miliz, Terminflug"></label><p id="suche-status" class="sr-only" role="status"></p>'
                 f'<ol>{"".join(leiste)}</ol></details></aside>').replace("<ol></ol>", "")
        inhalt = (self.seitenkopf("Handbuch · alle Regeln", "Das Handbuch der Sternenepoche.",
                                  "Jede Regel mit Zahlen und Rechenbeispielen: von Versorgung und Zivilisationsstufen über Flüge, Kampf und Kolonisation bis zu Monden, Hyperraumtor, Weltenkrümmer, Invasion, Dysonsphäre und Flottenführung. Konkrete Kosten deiner Welt zeigt das Spiel bei jeder Aktion.",
                                  self.u("web-client/assets/backgrounds.research.webp"), "Handbuch")
                  + f'<div class="rahmen lesen">{aside}<article class="text">{"".join(teile)}</article></div>')
        self.schreibe("Sternenepoche-Handbuch.html", self.huelle("Sternenepoche-Handbuch.html", "Handbuch – Sternenepoche",
                      "Alle Regeln der Sternenepoche in 32 Kapiteln mit Rechenbeispielen.", inhalt))

    # ===================================================================== Kodex
    def kodex(self):
        data = KODEX | {}
        data["boden"] = REGELN["bodentruppen"]
        data["invasion"] = REGELN.get("invasion_werte") or {}
        data["raketen"] = REGELN["raketen"]
        data["mond"] = REGELN.get("monde") or {}
        data["gueter"] = REGELN["gueter"]
        data["wunder"] = build_fleet_codex.wunder(REGELN)
        schluessel = (list(data["units"]) + [t["id"] for t in data["boden"]["typen"]] + ["miliz", "garnison"]
                      + [r["key"] for r in data["raketen"]] + list(data["mond"].get("gebaeude", {})) + list(data["wunder"]))
        data["archiv"] = {k: {"lore": (TEXTE[k].get("lore") or {}).get("allgemein", ""), **TEXTE[k]["archiv"]}
                          for k in schluessel if isinstance(TEXTE.get(k, {}).get("archiv"), dict)}
        data["texte"] = {k: {f: TEXTE[k].get(f, "") for f in ("name", "rolle", "zweck", "staerke", "nische")} for k in schluessel if k in TEXTE}
        data["archivnamen"] = {v: TEXTE[v]["name"] for a in data["archiv"].values() for v in a.get("verweise", []) if v in TEXTE}
        js = build_fleet_codex.JS
        # Bildpfade relativ zur Seite; technische Zeichnungen je Einheit und Volk (Techinfo-Reiter „Technik“ im Spiel).
        js = js.replace("return 'web-client/'+im[", "return BASIS+'web-client/'+im[")
        js = js.replace("${inhalt}${archiv(k,faction)}", "${inhalt}${zeichnung(k,faction)}${archiv(k,faction)}")
        zusatz = r"""
const BASIS=document.getElementById('kodex').dataset.basis;
let stil='blaupause';const STILE=[['blaupause','Blaupause'],['explosion','Explosionszeichnung'],['schnitt','Schnittzeichnung']];
function zeichnung(key,faction){const b=wo(key);if(!['flotte','boden'].includes(b))return '';
 const k=b==='boden'?'ground_units':data.units[key].klasse==='megaschiff'?'megaships':'ships',schl=b==='boden'?(TRUPPE[key]?.bild||key):key;
 if(['verteidigung','schild','satellit'].includes(data.units[key]?.klasse))return '';
 const da=STILE.map(([s,t])=>[s,t,bildUrl('blueprints',s+'.'+k+'.'+(schl==='weltentruemmer'?'todesmond':schl),faction)]).filter(x=>x[2]);if(!da.length)return '';
 const w=da.find(x=>x[0]===stil)||da[0];
 return `<section class="kodex-zeichnung"><h3>Technische Zeichnungen</h3><div class="kodex-stile" role="group" aria-label="Zeichnung wählen">${da.map(([s,t])=>`<button type="button" data-stil="${s}" aria-pressed="${s===w[0]}">${t}</button>`).join('')}</div><figure><button type="button" data-gross="${esc(w[2])}" data-titel="${esc(w[1]+' · '+name(key))}"><img src="${esc(w[2])}" alt="${esc(w[1]+': '+name(key))}" loading="lazy"></button><figcaption>${esc(w[1])} · ${esc(name(key))} · Bauart ${esc(VOLK[faction])}</figcaption></figure></section>`;}
document.getElementById('detail').addEventListener('click',e=>{const s=e.target.closest('[data-stil]');if(s){stil=s.dataset.stil;render();}});
"""
        js = zusatz + js
        n = {"units": len(data["units"]), "boden": len(data["boden"]["typen"]), "raketen": len(data["raketen"]), "mond": len(data["mond"].get("gebaeude", {}))}
        inhalt = (self.seitenkopf("Kodex · Werte aus dem Spielkern", "Jede Einheit. Jeder Wert. Jede Zeichnung.",
                                  f"{n['units']} Schiffe, Verteidigungen, Schilde, Satelliten und Megaschiffe je Volk, {n['boden']} Bodentruppen mit Kontertabelle, {n['raketen']} Raketen, {n['mond']} Mondanlagen und die Dysonsphäre. "
                                  "Zu jeder Einheit: Werte deines Volkes, Konter, Voraussetzungen, technische Zeichnungen in drei Stilen und das Archiv mit Dossier, Chronik und Volksakten.",
                                  self.art("backgrounds", "hangar"), "Kodex")
                  + f'''<div class="rahmen" id="kodex" data-basis="{self.basis}" style="padding-block:28px 90px">
<nav aria-label="Bereiche"><ul id="bereiche" class="codex-bereiche"></ul></nav>
<div class="codex-filter"><label>Volk<select id="faction"></select></label><label>Klasse<select id="class"><option value="">Alle Klassen</option></select></label>
<label>Direktauswahl<select id="unit"></select></label><label>Suchen<input id="search" type="search" placeholder="Name, Aufgabe oder Konter"></label></div>
<div class="codex-layout"><section id="detail" class="codex-detail glas" aria-live="polite"></section><section><p id="result-count" class="leise"></p>
<div id="gallery" class="codex-gallery" aria-label="Einträge auswählen"></div></section></div>
<section id="bereich-info" class="codex-info glas" hidden></section>
<p style="margin-top:28px" class="leise">Die Werte stammen aus dem Regelexport des Spielkerns. Wie alles zusammenhängt, zeigt der <a href="Technologiebaum.html">Technologiebaum</a>.</p>
</div>''')
        daten = json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
        (self.ziel / "seite").mkdir(parents=True, exist_ok=True)
        (self.ziel / "seite/kodex.js").write_text(js, encoding="utf-8")
        doc = self.huelle("Sternenepoche-Kodex.html", "Kodex – Sternenepoche",
                          "Alle Einheiten der Sternenepoche je Volk: Werte, Konter, Voraussetzungen, technische Zeichnungen und Archiv.", inhalt,
                          kopf_extra=f'<script src="{self.u("web-client/art.js")}"></script>',
                          fuss_extra=f'<script id="fleet-data" type="application/json">{daten}</script><script src="seite/kodex.js?v={self.stand}"></script>')
        self.schreibe("Sternenepoche-Kodex.html", doc)

    # ===================================================================== Technologiebaum
    def techbaum(self):
        b = self.u("web-client/")
        inhalt = '<h1 class="sr-only">Technologiebaum der Sternenepoche</h1>'
        doc = self.huelle("Technologiebaum.html", "Technologiebaum – Sternenepoche",
                          "Alle Technologiebäume der Sternenepoche auf einer Karte: Gebäude, Forschungen und Einheiten aller vier Völker mit Bildern und Voraussetzungen.",
                          inhalt, kopf_extra=f'<link rel="stylesheet" href="{b}techbaum/techbaum.css"><link rel="stylesheet" href="seite/techbaum.css?v={self.stand}">', koerper_klasse="seite-techbaum", nur_kopf=True,
                          fuss_extra=(f'<script src="{b}techbaum/daten.js"></script><script src="{b}techbaum/kampf.js"></script>'
                                      f'<script src="{b}techbaum/techbaum.js"></script><script src="seite/techbaum-start.js"></script>'))
        (self.ziel / "seite/techbaum-start.js").write_text(
            "// Karte beginnt unter KI-Hinweis und Kopfleiste der Website.\n"
            "const kopfhoehe = () => document.documentElement.style.setProperty('--kopfhoehe', document.querySelector('.sk-kopf').getBoundingClientRect().bottom + 'px');\n"
            "kopfhoehe(); addEventListener('resize', kopfhoehe);\n"
            f"TechbaumStart({{wurzel: document, basis: {json.dumps(b)}}});\n", encoding="utf-8")
        self.schreibe("Technologiebaum.html", doc)

    # ===================================================================== Lore
    def lore(self):
        quelle = (ROOT / "lore/Lore-Gesamtwerk.md").read_text(encoding="utf-8")
        teil1 = quelle.split("## Teil I", 1)[1].split("\n## Teil II", 1)[0]
        teil1 = "\n### " + teil1.split("\n### ", 1)[1]
        # Schreibregeln für Autoren gehören nicht ins öffentliche Archiv.
        teil1 = re.sub(r"\n### [078]\. .*?(?=\n### |\Z)", "\n", teil1, flags=re.S)
        rest = "## Teil II" + quelle.split("\n## Teil II", 1)[1]
        text = "## Teil I · Der Kanon\n" + teil1 + "\n" + rest
        text = re.sub(r"\s*\((?:[^()`]*,\s*)?`[^`]+`\)", "", text)  # Quellverweise in Überschriften
        h = self.md(text)
        # Kurzlore und Volksakten in Volksfarben.
        for v, name, farbe in VOELKER:
            h = h.replace(f"<li><strong>{name}:</strong>", f'<li style="--farbe:{farbe}"><strong style="color:{farbe}">{name}:</strong>')
        def akten(m):
            block = m[0]
            stuecke = re.split(r"(?=<p><strong>(?:Aurelianer|Krath|Syntheten|Veyari)-Akte\.</strong>)", block)
            aus = stuecke[0]
            for s in stuecke[1:]:
                v = re.match(r"<p><strong>(\w+)-Akte", s)[1]
                aus += (f'<details class="akte" style="--farbe:{VOLKFARBE[v.lower()]}"><summary>{v}-Akte</summary><div>'
                        + s.replace(f"<strong>{v}-Akte.</strong> ", "", 1) + "</div></details>")
            return aus
        h = re.sub(r'(?<=</h5>\n)(<p><strong>(?:Aurelianer|Krath|Syntheten|Veyari)-Akte\..*?)(?=<p><strong>Siehe auch|<h[2345])', akten, h, flags=re.S)
        # Bilder zu Archiv-Einträgen.
        namen = {e["name"]: k for k, e in TEXTE.items() if isinstance(e, dict) and e.get("name")}
        def bild(m):
            k = namen.get(html.unescape(m[2]))
            url = ""
            if k:
                for kat in ("buildings", "research", "ships", "defenses", "ground_units", "missiles", "satellites", "megaships", "moon_buildings"):
                    url = self.art(kat, "todesmond" if k == "weltentruemmer" else k, "aurelianer")
                    if url:
                        break
            img = f'<img src="{url}" alt="" loading="lazy" width="64" height="64">' if url else ""
            return f'<h4 id="{m[1]}">{img}{m[2]}</h4>'
        teile = re.split(r'(?=<h2 id=)', h)
        aus, leiste = [], []
        for t in teile:
            if not t.strip():
                continue
            m = re.match(r'<h2 id="([^"]+)">(.*?)</h2>', t)
            if m and "Archiv" in m[2]:
                t = re.sub(r'<h4 id="([^"]+)">(.*?)</h4>', bild, t)
            aus.append(f"<section data-suche>{t}</section>")
            if m:
                unter = re.findall(r'<h3 id="([^"]+)">(.*?)</h3>', t)
                leiste.append(f'<p class="gruppe">{m[2]}</p><ol>' + "".join(f'<li><a href="#{i}">{x}</a></li>' for i, x in unter) + "</ol>")
        aside = (f'<aside class="leiste glas"><details open><summary>Inhalt</summary><label class="leiste-suche">Im Archiv suchen<input type="search" placeholder="z. B. Kessa, Schwelle, Grakstollen"></label>'
                 f'<p id="suche-status" class="sr-only" role="status"></p>{"".join(leiste)}</details></aside>')
        archiv = sum(1 for e in TEXTE.values() if isinstance(e.get("archiv"), dict))
        inhalt = (self.seitenkopf("Lore-Archiv · Kanon der Weite", "Die Geschichte der Weite.",
                                  f"Die Weite und die Schwelle, der Erste Bruch, Physik und Technik, die Chronik früherer Epochen und die vier Völker. Dazu das Archiv aller {archiv} Einträge mit technischem Dossier, Chronik und den Akten jedes Volkes – und die sechzehn Stimmen des Rates.",
                                  self.art("backgrounds", "archive"), "Lore-Archiv")
                  + f'<div class="rahmen lesen">{aside}<article class="text">{self.hc_hinweis()}{"".join(aus)}</article></div>')
        self.schreibe("Lore.html", self.huelle("Lore.html", "Lore-Archiv – Sternenepoche",
                      "Kanon, Archiv und Rat der Sternenepoche: die Weite, der Erste Bruch, die vier Völker und jede Einheit mit Dossier, Chronik und Volksakten.", inhalt))

    # ===================================================================== Weltenkrümmer
    def weltenkruemmer(self):
        e = TEXTE["weltentruemmer"]
        kap = {n: (t, x) for n, t, x in kapitel_liste()}[28][1]
        regeln = kap.split("### Die drei Schlachtkreuzer")[0]
        karten = []
        for v, name, farbe in VOELKER:
            film = self.u(f"docs/website/medien/filme/{v}.mp4")
            poster = self.u(f"docs/website/medien/filme/{v}.jpg")
            ans = "".join(f'<figure><img src="{self.art("megaships", k, v)}" alt="{t} der {name}" loading="lazy"><figcaption>{t}</figcaption></figure>'
                          for k, t in (("todesmond_geschlossen", "Geschlossen"), ("todesmond", "Aufgeklappt"), ("todesmond_werft", "Im Werftdock")))
            akte = e["archiv"].get(v, "")
            satz = re.split(r"(?<=[.!?])\s+", akte)
            karten.append(f"""<article class="wk glas" style="--farbe:{farbe}" id="{v}">
<video controls muted playsinline preload="none" poster="{poster}"><source src="{film}" type="video/mp4"></video>
<div class="wk-text"><span class="strahl"><i></i>Strahl der {name}</span><h3>Weltenkrümmer der {name}</h3>
<p>{esc((e.get('lore') or {}).get(v, ''))}</p><details class="akte" style="--farbe:{farbe}"><summary>{name}-Akte aus dem Archiv</summary><div><p>{esc(' '.join(satz[:6]))}</p><p>{esc(' '.join(satz[6:]))}</p></div></details></div>
<div class="wk-ansichten">{ans}</div></article>""")
        inhalt = (self.seitenkopf("Megaschiff · Mondzerstörer", "Die vier Weltenkrümmer.",
                                  esc(e["rolle"]) + " " + esc((e.get("lore") or {}).get("allgemein", "")),
                                  self.art("megaships", "todesmond", "syntheten"), "Weltenkrümmer")
                  + f'<div class="rahmen" style="padding-top:28px"><a class="hc-hinweis-karte glas" href="Trailer.html"><img src="{self.u("docs/website/medien/trailer/szene-07.webp")}" alt="" width="160" height="90" loading="lazy">'
                    f'<span><b>Der Trailer</b>Ein Weltenkrümmer tritt aus dem Hyperraum und bricht einen Mond – {self.trailer_dauer()} Spielgrafik.</span><i aria-hidden="true">▶</i></a></div>'
                  + f'<div class="rahmen" style="padding-block:28px 40px"><div class="wk-liste">{"".join(karten)}</div></div>'
                  + f'<div class="rahmen lesen" style="grid-template-columns:minmax(0,1fr)"><article class="text"><h2 id="regeln"><span class="nr">Handbuch · Kapitel 28</span>Bau, Werte und Mondzerstörung</h2>{self.md(regeln)}'
                  + f'<h2 id="dossier"><span class="nr">Archiv</span>Technisches Dossier und Chronik</h2>{"".join(f"<p>{esc(s)}</p>" for s in re.split(r"(?<=[.!?])\s+(?=[A-ZÄÖÜ])", e["archiv"]["technik"]) if s)}'
                  + f'<h3>Chronik</h3><p>{esc(e["archiv"].get("geschichte", ""))}</p></article></div>')
        self.schreibe("Weltenkruemmer.html", self.huelle("Weltenkruemmer.html", "Weltenkrümmer – Sternenepoche",
                      "Die Weltenkrümmer der vier Völker im Film: Verwandlung, Strahl und Mondzerstörung – mit Werten, Regeln und Archiv.", inhalt))

    # ===================================================================== Trailer
    def trailer_dauer(self) -> str:
        d = round(lade("docs/website/medien/trailer/trailer.json")["dauer"])
        return f"{d // 60}:{d % 60:02d} Minuten"

    def trailer(self):
        t = lade("docs/website/medien/trailer/trailer.json")
        m = self.u("docs/website/medien/trailer/")
        d = round(t["dauer"])
        dauer = f"{d // 60}:{d % 60:02d}"
        szenen = t["szenen"]
        marken = "".join(f'<i style="left:{100 * s["t"] / t["dauer"]:.2f}%"></i>' for s in szenen[1:])
        karten = "".join(f'<li><button type="button"><img src="{m}{s["bild"]}" alt="" width="480" height="270" loading="lazy" decoding="async">'
                         f'<span>{int(s["t"]) // 60}:{int(s["t"]) % 60:02d}</span><b>{esc(s["titel"])}</b></button></li>' for s in szenen)
        spiel = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4.5v15l12-7.5z"/></svg>'
        vollbild = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 4h6v2H6v4H4zm10 0h6v6h-2V6h-4zM4 14h2v4h4v2H4zm14 0h2v6h-6v-2h4z"/></svg>'
        quellen = json.dumps({"1080": m + "trailer-1080.mp4", "720": m + "trailer-720.mp4"})
        inhalt = f"""
<section class="kino" aria-labelledby="kino-titel">
 <div class="kino-raum" aria-hidden="true" style="background-image:url('{m}raum.webp')"></div>
 <canvas class="kino-sterne" aria-hidden="true"></canvas>
 <div class="kino-licht" aria-hidden="true"></div>
 <div class="kino-innen">
  <div class="kino-titel"><span class="kennung">Promo-Trailer · {dauer} · Spielgrafik</span>
   <h1 id="kino-titel">Ein Mond.<span>Ein Schuss.</span></h1>
   <p class="einleitung">Ein Weltenkrümmer tritt aus dem Hyperraum, nimmt den Mond eines Nachbarn in den Fokus und bricht ihn mit einem einzigen Strahl.
   Jedes Bild im Film stammt aus der 3D-Galaxie des Spiels – dieselben Planeten, Monde und Schiffe, die du beim Spielen siehst.</p></div>
  <div class="kino-schirm" data-kino tabindex="-1" data-quellen='{esc(quellen)}' data-szenen='{esc(json.dumps(szenen, ensure_ascii=False))}' data-dauer="{t['dauer']:.2f}">
   <div class="kino-bild"><video muted playsinline preload="metadata" poster="{m}standbild.webp" aria-label="Trailer: Weltenkrümmer – Ein Mond. Ein Schuss.">
    <source src="{m}trailer-720.mp4" type="video/mp4"></video></div>
   <span class="kino-ecke lo"></span><span class="kino-ecke ro"></span><span class="kino-ecke lu"></span><span class="kino-ecke ru"></span>
   <div class="kino-hud" aria-hidden="true"><span class="lo"><i class="punkt"></i>Übertragung · Kanal 01</span><span class="ro"><b data-zeit>0:00</b>/ {dauer} <i data-qual>720p</i></span>
    <span class="lu" data-szene>{esc(szenen[0]['titel'])}</span><span class="ru">Sektor 1 · System 60</span></div>
   <button type="button" class="kino-start" aria-label="Trailer abspielen"><span class="ring">{spiel}</span><b>Trailer abspielen</b><small>{dauer} · Spielgrafik</small></button>
   <div class="kino-ende"><p>Ein Spiel über Imperien, Allianzen und Mut.<span>Sternenepoche</span></p>
    <div class="knopfreihe"><a class="button" href="{SPIEL_URL}">Im Browser spielen ↗</a><a class="button secondary" href="Download.html">Android-App</a><button type="button" class="button secondary" data-k="nochmal">Nochmal ansehen</button></div></div>
   <div class="kino-leiste"><button type="button" data-k="spiel" aria-label="Abspielen">{spiel}</button>
    <div class="kino-zeitleiste" role="slider" tabindex="0" aria-label="Position im Trailer" aria-valuemin="0" aria-valuemax="{d}" aria-valuenow="0"><div class="puffer"></div><div class="voll"></div>{marken}<span class="kino-marke"></span></div>
    <span class="kino-zeit">0:00 / {dauer}</span><button type="button" data-k="vollbild" aria-label="Vollbild">{vollbild}</button></div>
  </div>
  <ol class="kino-szenen" aria-label="Szenen">{karten}</ol>
 </div>
</section>
<section class="abschnitt" aria-labelledby="im-film"><div class="rahmen">
 <div class="abschnitt-kopf"><div><span class="kennung">Im Film</span><h2 id="im-film">Was du gerade gesehen hast.</h2></div>
 <p>Weltenkrümmer, Hyperraumloch, Mondzerstörung und Trümmerfeld gibt es so im Spiel. Hier steht, wie sie funktionieren.</p></div>
 <div class="kino-infos">
  <article class="kino-info glas"><span class="kennung">Megaschiff</span><h3>Der Weltenkrümmer</h3><p>Eine mondgroße Kampfstation aus verschmolzenen Trümmern, gebaut im Werftdock eines eigenen Mondes. Jedes Volk hat seine eigene Bauart und Strahlfarbe.</p><a class="weiter" href="Weltenkruemmer.html">Die vier Weltenkrümmer →</a></article>
  <article class="kino-info glas"><span class="kennung">Mondzerstörung</span><h3>Ein Strahl, ein Risiko</h3><p>Erst wenn das Gefecht am Mond gewonnen ist, feuert die Scherlanze. Kleine Monde brechen leichter, große halten öfter stand – und der Rückschlag kann die Station mitreißen.</p><a class="weiter" href="Sternenepoche-Handbuch.html#kapitel-28">Handbuch, Kapitel 28 →</a></article>
  <article class="kino-info glas"><span class="kennung">Galaxie in 3D</span><h3>Trümmerfeld und Bergung</h3><p>Aus dem zerstörten Mond wird ein Trümmerring. Ein Tipp darauf zeigt, was dort liegt; Recycler und Bergungsschiffe holen das Material zurück.</p><a class="weiter" href="Sternenepoche-Start.html#bereich-galaxie">Zur Galaxie im Spielguide →</a></article>
 </div>
 <div class="knopfreihe" style="margin-top:28px"><a class="button" href="{SPIEL_URL}">Im Browser spielen ↗</a><a class="button secondary" href="Download.html">Android-App laden</a></div>
</div></section>"""
        self.schreibe("Trailer.html", self.huelle("Trailer.html", "Trailer – Sternenepoche",
                      f"Der Promo-Trailer der Sternenepoche: Ein Weltenkrümmer bricht einen Mond – {dauer} aus der 3D-Galaxie des Spiels.", inhalt,
                      kopf_extra=f'<link rel="stylesheet" href="seite/kino.css?v={self.stand}"><link rel="preload" as="image" href="{m}standbild.webp">',
                      fuss_extra=f'<script src="seite/kino.js?v={self.stand}" defer></script>', koerper_klasse="seite-kino"))

    # ===================================================================== Hörchronik
    def hc_folgen(self):
        return lade("docs/website/medien/hoerchronik/hoerchronik.json")

    def hc_kurz(self) -> str:
        s = sum(f["dauer"] for f in self.hc_folgen())
        return f"fünf Folgen, zusammen {int(s // 3600)} Stunden {int(s % 3600 // 60)} Minuten"

    def hc_hinweis(self) -> str:
        return (f'<a class="hc-hinweis-karte glas" href="Hoerchronik.html"><img src="{self.art("portraits", "krath", "krath")}" alt="" width="64" height="64" loading="lazy">'
                f'<span><b>Lieber zuhören?</b>Die Hörchronik erzählt die Geschichte der Weite – {self.hc_kurz()}. Zuerst das ganze Epos, dann erzählt jedes Volk selbst.</span><i aria-hidden="true">▶</i></a>')

    def hoerchronik(self):
        folgen = self.hc_folgen()
        m = self.u("docs/website/medien/hoerchronik/")
        daten = []
        for f in folgen:
            v = f["volk"]
            daten.append({"id": f["id"], "titel": f["titel"], "kurz": f["kurz"], "gastgeber": f["gastgeber"], "rolle": "" if f["gastgeber"].split()[-1] in f["rolle"] else f["rolle"],
                          "farbe": VOLKFARBE.get(v, "#e2bd62"), "bild": self.art("portraits", v, v) if v else self.art("backgrounds", "archive"),
                          "wappen": self.art("emblems", v, v) if v else "", "url": m + f["datei"], "textUrl": m + f["text"],
                          "dauer": f["dauer"], "kapitel": f["kapitel"]})
        welle = "<i></i>" * 5
        spiel = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4.5v15l12-7.5z"/></svg>'
        zur = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5V1L7 6l5 5V7a6 6 0 1 1-6 6H4a8 8 0 1 0 8-8z"/></svg>'
        vor = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5V1l5 5-5 5V7a6 6 0 1 0 6 6h2a8 8 0 1 1-8-8z"/></svg>'
        karten = "".join(
            f'<button type="button" class="hc-folge" data-id="{d["id"]}" style="--farbe:{d["farbe"]}"><img src="{d["bild"]}" alt="" loading="lazy" width="480" height="360">'
            f'<div><small>Folge {int(d["id"][:2])} · {round(d["dauer"] / 60)} Minuten</small><b>{esc(d["titel"])}</b><span>{esc(d["gastgeber"])}</span></div></button>' for d in daten)
        ohne_js = "".join(f'<p><b>{esc(d["titel"])}</b><br><audio controls preload="none" src="{d["url"]}"></audio></p>' for d in daten)
        f0 = daten[0]
        inhalt = (self.seitenkopf("Hörchronik · fünf Folgen", "Die Weite zum Hören.",
                                  "Die Erzählerin führt durch das ganze Epos, von der Ersten Öffnung bis zum Ersten Licht. Danach erzählen die Völker selbst: "
                                  "die Aurelianer im Hafen von Port Aurel, die Krath am Schmiedefeuer, die Veyari in ihrem Garten und die Syntheten im einzigen Raum "
                                  f"von Hexaris, der ein Fenster hat. {self.hc_kurz()[0].upper() + self.hc_kurz()[1:]}, mit vielen Stimmen, Klängen und Musik.",
                                  self.art("backgrounds", "archive"), "Hörchronik")
                  + f"""<div class="rahmen chronik" data-chronik data-folgen='{esc(json.dumps(daten, ensure_ascii=False))}' data-farben='{esc(json.dumps(VOLKFARBE))}'>
<section class="hc-deck glas" tabindex="-1" aria-label="Hördeck" style="--farbe:{f0['farbe']}">
 <div class="hc-bild"><img class="motiv" src="{f0['bild']}" alt="" width="512" height="512"><img class="wappen" src="" alt="" width="46" height="46" hidden><div class="hc-welle" aria-hidden="true">{welle}</div></div>
 <div>
  <div class="hc-kopf"><span class="kennung">Folge 0 · {esc(f0['gastgeber'])}</span><h2>{esc(f0['titel'])}</h2><p>{esc(f0['kurz'])}</p></div>
  <div class="hc-steuer"><button type="button" class="hc-spiel" aria-label="Abspielen">{spiel}</button>
   <button type="button" data-k="zurueck" aria-label="15 Sekunden zurück">{zur}</button><button type="button" data-k="vor" aria-label="15 Sekunden vor">{vor}</button>
   <span class="hc-zeit">0:00</span><button type="button" class="hc-tempo" aria-label="Tempo">1×</button></div>
  <div class="hc-leiste" role="slider" tabindex="0" aria-label="Position in der Folge" aria-valuemin="0" aria-valuemax="0" aria-valuenow="0"><div class="voll"></div></div>
  <p class="hc-jetzt">Jetzt · <b></b></p>
  <ol class="hc-kapitel" aria-label="Kapitel"></ol>
 </div>
</section>
<noscript>{ohne_js}</noscript>
<div class="hc-folgen" role="list">{karten}</div>
<details class="hc-text glas"><summary>Zum Nachlesen<small>Der gesprochene Text der gewählten Folge, Kapitel für Kapitel.</small></summary><div class="hc-lesen"></div></details>
</div>
<div class="hc-mini" aria-label="Hörchronik läuft"><img src="" alt="" width="42" height="42"><div><b></b><span></span></div><button type="button" aria-label="Abspielen oder anhalten">{spiel}</button><button type="button" class="hc-mini-zu" aria-label="Schließen">✕</button></div>""")
        self.schreibe("Hoerchronik.html", self.huelle("Hoerchronik.html", "Hörchronik – Sternenepoche",
                      "Die Geschichte der Sternenepoche zum Hören: das ganze Epos und die vier Völker in fünf Folgen, mit vielen Stimmen, Klängen und Musik.", inhalt,
                      kopf_extra=f'<link rel="stylesheet" href="seite/chronik.css?v={self.stand}">',
                      fuss_extra=f'<script src="seite/chronik.js?v={self.stand}" defer></script>'))

    # ===================================================================== Mit KI spielen
    def ki(self):
        kap = {n: (t, x) for n, t, x in kapitel_liste()}
        teile = []
        for n in (2, 5, 6, 20):
            body = self.md(kap[n][1])
            if n == 20:
                body = body.replace('<h3 id="der-eigene-rat-im-harness-cockpit">', '<h3 id="cockpit">')
            teile.append(f'<section data-suche><h2 id="kapitel-{n}"><span class="nr">Handbuch · Kapitel {n}</span>{esc(kap[n][0])}</h2>{body}</section>')
        c = lade("docs/cockpit-download.json")
        api_text = ('<p>Eigene Programme spielen über die <b>Harness-API v1</b> oder über MCP. Den Harness-Schlüssel erzeugst du im Spiel unter '
                    '<b>Modelle &amp; Team → Harness-Schlüssel</b> oder im Harness-Cockpit unter <b>Werkstatt → Konto</b>. Jeder Befehl geht durch dieselben '
                    'Regeln und Quoten wie im Browser; das maschinelle Bedienen der Spieloberfläche ist nicht erlaubt.</p>'
                    f'<p>Endpunkte, Quoten, Grenzen und Regeln beschreibt der laufende Server selbst: <a href="{SPIEL_URL}api/harness/v1/info">Harness-API – Beschreibung des Servers</a>. '
                    f'Die Fair-Play-Regeln für eigene Programme stehen unter <a href="{self.u("web-client/harness-regeln.html")}">Harness-Regeln</a>.</p>')
        leiste = ('<aside class="leiste glas"><details open><summary>Inhalt</summary><ol>'
                  + "".join(f'<li><a href="#kapitel-{n}">{esc(kap[n][0])}</a></li>' for n in (2, 5, 6, 20))
                  + ('<li><a href="#api">Harness-API</a></li>' if api_text else "") + '<li><a href="#cockpit-laden">Cockpit herunterladen</a></li></ol></details></aside>')
        inhalt = (self.seitenkopf("Mit KI spielen", "Dein Reich, dein Modell, dein Rat.",
                                  "Verbinde ein lokales Ollama-Modell oder OpenRouter mit deinem Reich und gib Rollen ab – oder lass im Harness-Cockpit auf deinem PC einen Rat aus vier Agenten regieren. Jeder Befehl geht durch dieselben Regeln wie deine eigenen.",
                                  self.art("backgrounds", "research"), "Mit KI spielen")
                  + f'<div class="rahmen lesen">{leiste}<article class="text">'
                  + f'<div class="galerie" style="grid-template-columns:1fr 1fr">{self.foto("rat", "Rat · Modelle & Team", "Rollen an Modelle vergeben")}{self.foto("pinwaende", "Pinwände", "Wissen und Ziele für Mensch und Modell")}</div>'
                  + "".join(teile)
                  + (f'<section data-suche><h2 id="api"><span class="nr">Für eigene Programme</span>Harness-API</h2>{api_text}</section>' if api_text else "")
                  + f'<section><h2 id="cockpit-laden"><span class="nr">Download</span>Harness-Cockpit {esc(c["version"])}</h2><p>Das Harness-Cockpit für Windows enthält deinen Rat aus vier Agenten, den vorgebauten Harness und eine lokale Spielwelt. '
                  + f'Download {esc(str(c["groesse_mb"]).replace(".", ","))} MB.</p><p><a class="button" href="{esc(c["url"])}">Cockpit {esc(c["version"])} für Windows laden</a></p>'
                  + f'<p class="leise" style="word-break:break-all">SHA-256: {esc(c["sha256"])}</p></section></article></div>')
        self.schreibe("Harness-Cockpit.html", self.huelle("Harness-Cockpit.html", "Mit KI spielen – Sternenepoche",
                      "Eigene Modelle über Ollama oder OpenRouter, der Rat im Harness-Cockpit und die Harness-API der Sternenepoche.", inhalt))

    # ===================================================================== Download
    def download(self):
        """Zwei APKs (Karl 18.10.): Lite nur arm64-v8a für Handys, Vollversion arm64-v8a + x86_64. Daten aus
        docs/android-download.json – Schema mit „varianten“ (lite, voll); Version und Knöpfe kommen allein aus diesen Daten."""
        a, c = android_daten(), lade("docs/cockpit-download.json")
        varianten = {v.get("art"): v for v in a.get("varianten", [])}
        av = a.get("version", "")
        bereit = {"lite", "voll"} <= set(varianten)
        if not bereit:
            print(f"  HINWEIS: docs/android-download.json nennt keine Varianten lite/voll – APK-Knöpfe bleiben gesperrt.")
        mb = lambda v, etwa: (str(v["groesse_mb"]).replace(".", ",") + " MB") if bereit and v.get("groesse_mb") else f"etwa {etwa} MB"
        def apk(art, titel, etwa, kennung, text, punkte):
            v = varianten.get(art, {})
            knopf = (f'<p><a class="button{" secondary" if art == "voll" else ""}" href="{esc(v["url"])}">{titel} laden ({mb(v, etwa)})</a></p>'
                     f'<p class="pruefsumme">{esc(v.get("datei", ""))}<br>SHA-256 {esc(v["sha256"])}</p>') if bereit else (
                     f'<p><span class="button secondary" aria-disabled="true">{titel} – erscheint mit der Freigabe</span></p>')
            li = "".join(f"<li>{x}</li>" for x in punkte)
            return (f'<article class="download glas" id="android-{art}"><span class="kennung">{kennung}</span><h3>{titel} <small class="leise">{mb(v, etwa)}</small></h3>'
                    f'<p>{text}</p><ul>{li}</ul>{knopf}</article>')
        lite = apk("lite", f"Android {av} Lite", 88, "Für Handys und Tablets",
                   "Das ganze Spiel für Android-Handys und -Tablets.",
                   ["Nur arm64-v8a – praktisch alle Android-Handys und -Tablets",
                    "Alle Völker, Technologiebaum, Weltenkrümmer in 3D, technische Zeichnungen, Einzel- und Hostspiel",
                    "Lokale KI-Modelle in drei Varianten (basis, dotprod, i8mm) – die App wählt die schnellste passende"])
        voll = apk("voll", f"Android {av} Vollversion", 168, "Für alle Android-Geräte",
                   "Das ganze Spiel für jedes Android-Gerät, auch mit Intel- oder AMD-Prozessor.",
                   ["arm64-v8a und x86_64 – Handys, Tablets, Android-Emulator am PC, Chromebooks, Tablets mit Intel/AMD",
                    "Lokale KI-Modelle: arm64 in drei Varianten, x86_64 in der Basisvariante",
                    "Gleiche App und gleiche Spielstände wie in der Lite-Fassung"])
        groesse_lite, groesse_voll = mb(varianten.get("lite", {}), 88), mb(varianten.get("voll", {}), 168)
        vergleich = f"""<div class="tabelle" tabindex="0" role="region" aria-label="Vergleich Lite und Vollversion"><table>
<thead><tr><th></th><th>Lite</th><th>Vollversion</th></tr></thead><tbody>
<tr><td>Geräte</td><td>Android-Handys und -Tablets (arm64-v8a)</td><td>dazu x86_64: Android-Emulator, Chromebooks, Tablets mit Intel/AMD</td></tr>
<tr><td>Spielinhalt</td><td colspan="2">das ganze Spiel: alle Völker, Technologiebaum, Weltenkrümmer in 3D, technische Zeichnungen, Einzel- und Hostspiel</td></tr>
<tr><td>Lokale KI-Modelle</td><td>arm64: basis, dotprod, i8mm</td><td>arm64: basis, dotprod, i8mm · x86_64: basis</td></tr>
<tr><td>Download</td><td>{groesse_lite}</td><td>{groesse_voll}</td></tr>
<tr><td>Arbeitsspeicher und Tempo im Spiel</td><td colspan="2">auf einem Handy gleich</td></tr>
<tr><td>Voraussetzung</td><td colspan="2">Android 8.0 (API 26) oder neuer, 64-bit</td></tr>
<tr><td>Wechseln und Updates</td><td colspan="2">gleiche App, gleicher Signaturschlüssel: Updates über bestehende Installationen, Wechsel zwischen Lite und Vollversion jederzeit</td></tr>
</tbody></table></div>"""
        inhalt = (self.seitenkopf("Download", "Sternenepoche überall.", "Im Browser ohne Installation, als Android-App mit eigener Spielwelt oder als Harness-Cockpit für deinen KI-Rat unter Windows.",
                                  self.art("planets", "leben_ozean"), "Download")
                  + f'''<div class="rahmen" style="padding-block:36px 90px">
<h2 id="android">Android-App {av}</h2>
<p class="einleitung">Zwei Fassungen mit demselben Spiel. Für Handy und Tablet nimmst du <b>Lite</b>, für den Android-Emulator am PC, ein Chromebook oder ein Tablet mit Intel- oder AMD-Prozessor die <b>Vollversion</b>.</p>
<div class="downloads zwei" style="margin-top:22px">{lite}{voll}</div>
<h3 style="margin-top:36px">Die beiden Fassungen</h3>{vergleich}
<h2 style="margin-top:56px">Weitere Wege</h2>
<div class="downloads zwei" style="margin-top:22px">
<article class="download glas" id="browser"><span class="kennung">Ohne Installation</span><h3>Im Browser</h3><p>Die Online-Galaxie läuft in jedem aktuellen Browser auf PC, Tablet und Handy.</p><p><a class="button" href="{SPIEL_URL}">Im Browser spielen ↗</a></p></article>
<article class="download glas" id="cockpit"><span class="kennung">Windows</span><h3>Harness-Cockpit {esc(c["version"])}</h3><p>Dein Rat aus vier Agenten, eigener Harness und lokale Spielwelt.</p><p><a class="button secondary" href="{esc(c["url"])}">Für Windows laden ({esc(str(c["groesse_mb"]).replace(".", ","))} MB)</a></p><p class="pruefsumme">SHA-256 {esc(c["sha256"])}</p></article>
</div>
<h2 style="margin-top:56px">Aus dem Spiel</h2>
<div class="galerie" style="margin-top:20px">{self.foto("flotte", "Flottenkommando", "Versand in drei Schritten, Verbände, Zusammenarbeit")}<div class="stapel">{self.foto("uebersicht-handy", "Auf dem Handy", "Menü unten, Lagebericht oben", "glas")}</div></div>
<p style="margin-top:20px"><a href="index.html#spiel">Mehr über das Spiel</a> · <a href="Sternenepoche-Handbuch.html">Handbuch</a></p></div>''')
        self.schreibe("Download.html", self.huelle("Download.html", "Download – Sternenepoche",
                      "Sternenepoche im Browser, als Android-App (Lite und Vollversion) und als Harness-Cockpit für Windows.", inhalt))

    # ===================================================================== KI-Hinweis
    def kihinweis(self):
        inhalt = (self.seitenkopf("Transparenz", "KI in der Sternenepoche.", "Bild, Ton und Text der Sternenepoche sind KI-generiert. Auch die Entwicklung erfolgt mit KI-Unterstützung.", "", "KI-Hinweis")
                  + """<div class="rahmen lesen" style="grid-template-columns:minmax(0,1fr)"><article class="text">
<h2 id="inhalte">Bild, Ton und Text</h2><p>Spielbilder, Filme, Stimmen, Anleitungen und Spieltexte entstehen mit KI. Bilder tragen ein kleines „KI“-Label. Die Motive – Völker, Planeten, Gebäude, Schiffe, technische Zeichnungen und Hintergründe – zeigen eine erfundene Spielwelt.</p>
<h2 id="spieler">Wer spielt mit KI?</h2><ul><li><b>Mensch:</b> Du gibst die Befehle selbst und brauchst kein Sprachmodell.</li><li><b>Agent:</b> Ein Sprachmodell deiner Wahl steuert die Rollen, die du freigibst – über lokales Ollama oder OpenRouter.</li>
<li><b>Gemischt:</b> Du und deine Agenten teilt euch ein Reich.</li><li><b>Harness-Cockpit:</b> Ein Rat aus vier Agenten regiert auf deinem PC über die Harness-API.</li><li><b>Serverbot:</b> Eingebaute Bots folgen programmierten Strategien und verwenden kein Sprachmodell.</li></ul>
<h2 id="regeln">Gleiche Regeln für alle</h2><p>Die Spielwelt prüft jeden Befehl nach denselben Regeln. Ein KI-Agent erhält keine Sonderrechte. Modellantworten können Fehler enthalten; du kannst die Entscheidungen verfolgen und die Steuerung anhalten oder übernehmen.</p>
<h2 id="daten">Modellanbieter und Daten</h2><p>Bei Ollama gehen Modellanfragen an deinen lokalen Dienst. Bei OpenRouter werden die für die Entscheidung nötigen Spieldaten an diesen Anbieter übermittelt; er kann dafür Kosten berechnen. Dein Modellschlüssel wird nicht in der Spielserver-Datenbank gespeichert. Die Website bindet keine Werbenetzwerke und keine fremden Skripte ein.</p>
</article></div>""")
        self.schreibe("KI-Hinweis.html", self.huelle("KI-Hinweis.html", "KI-Hinweis – Sternenepoche", "Wie KI in der Sternenepoche eingesetzt wird.", inhalt))

    # ===================================================================== Sitemap, llms.txt
    def verzeichnisse(self):
        heute = date.today().isoformat()
        url = "https://sternenepoche.github.io/"
        if not self.basis:
            # Maschinenlesbares Handbuch (wie bisher build_website.py) – nur im Pages-Modus in docs/.
            kap = [{"number": n, "title": t, "markdown": x.strip()} for n, t, x in kapitel_liste()]
            (ROOT / "docs/spielguide.json").write_text(json.dumps({"title": "Sternenepoche Spielguide", "updated": heute, "language": "de",
                "website": url, "guide": url + "Sternenepoche-Handbuch.html",
                "rules_note": "Konkrete Kosten, Voraussetzungen und Flugpläne kommen aus dem aktiven Regelprofil im Spiel.",
                "chapters": kap}, ensure_ascii=False, indent=2), encoding="utf-8")
            (ROOT / "docs/spielguide.txt").write_text(HANDBUCH, encoding="utf-8")
        seiten = [n for n, _, _ in ALLE] + ["KI-Hinweis.html"]
        self.schreibe("sitemap.xml", '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
                      + "".join(f"<url><loc>{url}{'' if s == 'index.html' else s}</loc><lastmod>{heute}</lastmod></url>\n" for s in seiten) + "</urlset>\n")
        zeilen = [f"# Sternenepoche", "", f"> Deutschsprachiges Weltraumstrategiespiel in einer gemeinsamen Galaxie. Menschen, Serverbots und KI-Agenten spielen nach demselben Rust-Regelkern. Stand {date.today().strftime('%d.%m.%Y')}.", "", "## Seiten"]
        zeilen += [f"- [{t}]({url}{'' if d == 'index.html' else d}): {u}." for d, t, u in ALLE]
        zeilen += ["- [KI-Hinweis](" + url + "KI-Hinweis.html): Wie KI in Bild, Ton, Text und Spiel eingesetzt wird.", "",
                   "## Maschinenlesbar",
                   f"- [Handbuch als Text]({url}docs/spielguide.txt): vollständige Regelquelle mit Beispielen.",
                   f"- [Handbuch als JSON]({url}docs/spielguide.json): Kapitel mit Markdowntexten.",
                   f"- [Flottenkodex]({url}docs/flottenkodex.json) und [Regelreferenz]({url}docs/technologie-regeln.json): Werte aus dem Spielkern.",
                   f"- [Harness-API]({SPIEL_URL}api/harness/v1/info): Endpunkte, Quoten und Regeln des laufenden Servers.", "",
                   "## Einordnung",
                   "Bildschirmfotos stammen aus isolierten Prüfwelten mit dem aktuellen Spielstand, nicht aus Live-Partien. Konkrete Kosten und Voraussetzungen zeigt das Spiel bei jeder Aktion. Eigene Programme spielen ausschließlich über die Harness-API."]
        self.schreibe("llms.txt", "\n".join(zeilen) + "\n")

    # ===================================================================== Stil, Prüfung
    def stil(self):
        css = (Q / "stil.css").read_text(encoding="utf-8").replace("{B}", self.css_basis)
        knoepfe = (ROOT / "docs/action-surfaces.css").read_text(encoding="utf-8").replace("asset://", self.css_basis + "web-client/assets/")
        # Funktionsteil des Kodex (Raster, Matrix, Akten) aus build_fleet_codex; Farben und Schrift überschreibt kodex.css.
        kodex = re.sub(r"^html\{[^}]*\}\s*body\{[^}]*\}", "", build_fleet_codex.CSS.strip())
        (self.ziel / "seite").mkdir(parents=True, exist_ok=True)
        kopf = (Q / "kopf.css").read_text(encoding="utf-8").replace("{B}", self.css_basis)
        (self.ziel / "seite/kopf.css").write_text(knoepfe + "\n" + kopf, encoding="utf-8")
        (self.ziel / "seite/stil.css").write_text("\n".join([css, kodex, (Q / "kodex.css").read_text(encoding="utf-8")]), encoding="utf-8")
        (self.ziel / "seite/techbaum.css").write_text((Q / "techbaum-seite.css").read_text(encoding="utf-8").replace("{B}", self.css_basis), encoding="utf-8")
        shutil.copyfile(Q / "logo/techbaum-logo.webp", self.ziel / "seite/techbaum-logo.webp")
        (self.ziel / "seite/seite.js").write_text((Q / "seite.js").read_text(encoding="utf-8"), encoding="utf-8")
        for n in ("kino.css", "kino.js", "chronik.css", "chronik.js"):
            (self.ziel / "seite" / n).write_text((Q / n).read_text(encoding="utf-8"), encoding="utf-8")

    def pruefen(self):
        verboten = re.compile(r"8891|Dashboard-oeffnen|Server-starten\.cmd|SERVER-BETRIEB|ONLINE-KONZEPT|awarre\.html|Verwaltungsdashboard|Karls PC|Für Karl|data/online|[A-Za-z]:[\\/]projekte_ki|\{B\}", re.I)
        fehler = []
        for d in [n for n, _, _ in ALLE] + ["KI-Hinweis.html"]:
            t = (self.ziel / d).read_text(encoding="utf-8")
            if (m := verboten.search(t)):
                fehler.append(f"{d}: verbotener Inhalt {m.group()}")
            for ref in re.findall(r'(?:src|href|data-gross|poster|data-standbild)="([^"#?]+)', t):
                if ref.startswith(("http", "mailto:", "data:")) or ref.endswith("/") or "${" in ref:
                    continue
                if not (self.ziel / html.unescape(ref)).resolve().exists():
                    fehler.append(f"{d}: fehlt {ref}")
        for f in sorted(set(fehler)):
            print("  FEHLER", f)
        return not fehler

    def alles(self):
        # Versionskennung aus dem Inhalt: jede Änderung an Stil oder Skript lädt neu, nichts Altes bleibt im Zwischenspeicher.
        import hashlib
        h = hashlib.sha256()
        for f in (Q / "stil.css", Q / "kopf.css", Q / "techbaum-seite.css", Q / "kino.css", Q / "kino.js", Q / "chronik.css", Q / "chronik.js", Q / "logo/techbaum-logo.webp", Q / "seite.js", Q / "kodex.css", ROOT / "docs/action-surfaces.css", Path(__file__)):
            h.update(f.read_bytes())
        self.stand = h.hexdigest()[:10]
        self.ziel.mkdir(parents=True, exist_ok=True)
        self.stil()
        if self.basis:
            # Nur Vorschau: rat.json nennt Porträts ab der Website-Wurzel (web-client/assets/…) – hier mitlegen.
            import shutil
            for p in re.findall(r'"portraet":\s*"([^"]+)"', (ROOT / "docs/rat/rat.json").read_text(encoding="utf-8")):
                (self.ziel / p).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / p, self.ziel / p)
        for teil in (self.start, self.trailer, self.hoerchronik, self.guide, self.handbuch, self.kodex, self.techbaum, self.lore, self.weltenkruemmer, self.ki, self.download, self.kihinweis, self.verzeichnisse):
            teil()
        return self.pruefen()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--pages", action="store_true", help="ins Projektwurzelverzeichnis schreiben (Veröffentlichung)")
    a = p.parse_args()
    bau = Bau(ROOT, "") if a.pages else Bau(ROOT / "ausgabe/website", "../../")
    print("Website", VERSION, "→", bau.ziel)
    ok = bau.alles()
    print("Prüfung:", "bestanden" if ok else "FEHLER")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
