"""Baut die Seite Harness-Cockpit.html: alle Funktionen des Cockpits mit echten Bildschirmfotos.

Vorlage: docs/cockpitseite.html. Platzhalter:
  <!-- BILD name | Titel | Bildunterschrift -->           Pflichtbild aus docs/bilder/cockpit/<name>.webp
  <!-- BILD_OPTIONAL name | Titel | Bildunterschrift -->  nur, wenn das Bild vorhanden ist
  <!-- COCKPIT_HERO -->, <!-- COCKPIT_BAU -->            feste Bilder für Kopf und Startabschnitt
  <!-- COCKPIT_DOWNLOAD -->                             Download-Kasten aus docs/cockpit-download.json (fehlt die
                                                        Datei, steht dort ehrlich „in Vorbereitung“)
Bilder kommen aus der Cockpit-Abnahme (harness/tests/abnahme_cockpit.py) über tools/cockpit_bilder.py."""
import html
import json
import re

from site_policy import decorate
from website_content import ROOT, cockpit_bild, finish

QUELLE = ROOT / "docs" / "cockpitseite.html"
ZIEL = ROOT / "Harness-Cockpit.html"
BILDER = ROOT / "docs" / "bilder" / "cockpit"
DOWNLOAD = ROOT / "docs" / "cockpit-download.json"


def download_kasten() -> str:
    """Download-Kasten: Version, Datei, Größe, SHA-256 und Link auf das GitHub-Release (nur https://github.com/…)."""
    if not DOWNLOAD.is_file():
        return ('<div class="download-kasten" id="download"><b>Download in Vorbereitung</b><p>Das Cockpit wird gerade für '
                'den öffentlichen Download verpackt. Bis dahin spielst du im Browser – selbst, mit deinem Modellteam oder '
                'gemischt. Eigene Programme können die Harness-API schon heute nutzen.</p></div>')
    d = json.loads(DOWNLOAD.read_text(encoding="utf-8"))
    url = d["url"]
    if not re.fullmatch(r"https://github\.com/Sternenepoche/[A-Za-z0-9_.-]+/releases/download/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\.zip", url):
        raise ValueError("Download-Adresse muss ein GitHub-Release der Sternenepoche sein")
    if not re.fullmatch(r"[0-9a-f]{64}", d["sha256"]):
        raise ValueError("SHA-256 fehlt oder ist ungültig")
    e = lambda k: html.escape(str(d[k]))
    return (f'<div class="download-kasten" id="download"><b>Harness-Cockpit {e("version")} für Windows</b>'
            f'<p>{e("datei")} · {e("groesse_mb").replace(".", ",")} MB · Stand {e("stand")}</p>'
            f'<a class="button" href="{html.escape(url)}" download>Cockpit herunterladen ↓</a>'
            f'<p class="pruefsumme">SHA-256: <code>{e("sha256")}</code></p>'
            '<p>Entpacken, <code>sternenepoche-cockpit.exe</code> starten. Windows kann beim ersten Start warnen, weil das '
            'Programm nicht signiert ist („Weitere Informationen → Trotzdem ausführen“). Prüfe vorher die Prüfsumme.</p></div>')


def build():
    document = QUELLE.read_text(encoding="utf-8")

    def bild(m):
        art, name, titel, text = m[1], m[2].strip(), m[3].strip(), m[4].strip()
        if not (BILDER / f"{name}.webp").is_file():
            if art == "BILD_OPTIONAL":
                return ""
            raise ValueError(f"Cockpit-Bild fehlt: {name}.webp (tools/cockpit_bilder.py)")
        return cockpit_bild(name, titel, text)

    document = re.sub(r"<!-- (BILD|BILD_OPTIONAL) ([^|]+)\|([^|]+)\|(.*?) -->", bild, document)
    document = document.replace("<!-- COCKPIT_HERO -->", cockpit_bild(
        "02-rat-verwalter", "Rat mit Schaltplan", "Jeder Agent mit 3D-Figur, Status, Modell und seinem Harness in zehn Stufen."))
    document = document.replace("<!-- COCKPIT_BAU -->", cockpit_bild(
        "00-werkstatt-bau", "Bauen mit „Let's go“", "Bausteine, Abhängigkeiten, Bauschritte mit Fortschritt und die lokalen Prüfungen."))
    document = document.replace("<!-- COCKPIT_DOWNLOAD -->", download_kasten())
    if "<!-- BILD" in document or "<!-- COCKPIT_" in document:
        raise ValueError("Ungelöster Platzhalter in der Cockpit-Seite")
    document = decorate(finish(document), "KI-Hinweis.html")
    document = "\n".join(line.rstrip() for line in document.splitlines()) + "\n"
    ZIEL.write_text(document, encoding="utf-8")
    print("Cockpit page built:", len(document.encode("utf-8")), "bytes")


if __name__ == "__main__":
    build()
