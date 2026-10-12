"""Baut die Seite Download.html: Systemwahl (Android, Windows, Linux, macOS) mit der Android-App.

Vorlage: docs/downloadseite.html. Platzhalter:
  <!-- ANDROID_DOWNLOAD -->  Download-Kasten aus docs/android-download.json (dieselbe Datei liest die App für Updates)
  <!-- ANDROID_BILDER -->    echte Bildschirmfotos der App aus docs/bilder/android/<name>.webp
  <!-- COCKPIT_DOWNLOAD -->  Windows-Cockpit wie auf der Cockpit-Seite (docs/cockpit-download.json)
Die APK liegt als GitHub-Release der Sternenepoche; die Seite verlinkt nur https://github.com/Sternenepoche/…"""
import html
import json
import re

from build_cockpitpage import download_kasten as cockpit_kasten
from site_policy import decorate
from website_content import ROOT, finish

QUELLE = ROOT / "docs" / "downloadseite.html"
ZIEL = ROOT / "Download.html"
ANDROID = ROOT / "docs" / "android-download.json"
BILDER = ROOT / "docs" / "bilder" / "android"
GALERIE = [
    ("intro", "Das Sternentor öffnet sich", "Beim Start läuft das Logo einmal ganz durch."),
    ("einrichtung", "Einrichtung in drei Schritten", "Name, Spielweise, Let's go – wie im Cockpit."),
    ("spiel", "Dein Reich auf dem Handy", "Das vollständige Spiel, angepasst an den kleinen Bildschirm."),
    ("host", "Handy als WLAN-Server", "Adresse und QR-Code für Mitspieler, Tempo und Pause."),
    ("spionage", "Spionageordner", "Jeder Bericht wird abgelegt, je Ziel mit Verlauf; Flotte oder Sonde mit einem Tipp."),
    ("flotte", "Ziel mit einem Tipp", "Über der Tastatur: Koordinate aus der Zwischenablage und gemerkte Ziele."),
    ("graph", "Notizen mit Wissensgraph", "Markdown mit [[Verweisen]] wie in Obsidian."),
    ("start", "Eigene Welt einstellen", "Größe der Galaxie und Tempo der Spielzeit."),
]


def android_kasten() -> str:
    if not ANDROID.is_file():
        return ('<div class="download-kasten" id="android-download"><b>Android-App in Vorbereitung</b>'
                '<p>Bis dahin spielst du im Browser deines Handys.</p></div>')
    d = json.loads(ANDROID.read_text(encoding="utf-8"))
    if not re.fullmatch(r"https://github\.com/Sternenepoche/[A-Za-z0-9_.-]+/releases/download/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\.apk", d["url"]):
        raise ValueError("APK-Adresse muss ein GitHub-Release der Sternenepoche sein")
    if not re.fullmatch(r"[0-9a-f]{64}", d["sha256"]) or int(d["code"]) < 1:
        raise ValueError("android-download.json: SHA-256 oder Versionscode ungültig")
    e = lambda k: html.escape(str(d[k]))
    return (f'<div class="download-kasten" id="android-download"><b>Sternenepoche für Android {e("version")}</b>'
            f'<p>{e("datei")} · {str(d["groesse_mb"]).replace(".", ",")} MB · ab Android {e("min_android")} · Stand {e("stand")}</p>'
            f'<a class="button" href="{html.escape(d["url"])}">App herunterladen ↓</a>'
            f'<p class="pruefsumme">SHA-256: <code>{e("sha256")}</code></p></div>')


def galerie() -> str:
    teile = []
    for name, titel, text in GALERIE:
        if not (BILDER / f"{name}.webp").is_file():
            raise ValueError(f"App-Bild fehlt: docs/bilder/android/{name}.webp")
        teile.append(f'<figure><img src="docs/bilder/android/{name}.webp" alt="Sternenepoche-App: {html.escape(titel)}" width="540" height="1145" loading="lazy" decoding="async">'
                     f'<figcaption><b>{html.escape(titel)}</b>{html.escape(text)}</figcaption></figure>')
    return "".join(teile)


def build():
    document = QUELLE.read_text(encoding="utf-8")
    document = document.replace("<!-- ANDROID_DOWNLOAD -->", android_kasten())
    document = document.replace("<!-- ANDROID_BILDER -->", galerie())
    document = document.replace("<!-- COCKPIT_DOWNLOAD -->", cockpit_kasten().replace('id="download"', 'id="windows-download"'))
    if "<!-- ANDROID_" in document or "<!-- COCKPIT_" in document:
        raise ValueError("Ungelöster Platzhalter in der Download-Seite")
    document = decorate(finish(document), "KI-Hinweis.html")
    document = "\n".join(line.rstrip() for line in document.splitlines()) + "\n"
    ZIEL.write_text(document, encoding="utf-8")
    print("Download page built:", len(document.encode("utf-8")), "bytes")


if __name__ == "__main__":
    build()
