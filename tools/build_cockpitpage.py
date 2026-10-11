"""Baut die Seite Harness-Cockpit.html: alle Funktionen des Cockpits mit echten Bildschirmfotos.

Vorlage: docs/cockpitseite.html. Platzhalter:
  <!-- BILD name | Titel | Bildunterschrift -->           Pflichtbild aus docs/bilder/cockpit/<name>.webp
  <!-- BILD_OPTIONAL name | Titel | Bildunterschrift -->  nur, wenn das Bild vorhanden ist
  <!-- COCKPIT_HERO -->, <!-- COCKPIT_BAU -->            feste Bilder für Kopf und Startabschnitt
Bilder kommen aus der Cockpit-Abnahme (harness/tests/abnahme_cockpit.py) über tools/cockpit_bilder.py."""
import re

from site_policy import decorate
from website_content import ROOT, cockpit_bild, finish

QUELLE = ROOT / "docs" / "cockpitseite.html"
ZIEL = ROOT / "Harness-Cockpit.html"
BILDER = ROOT / "docs" / "bilder" / "cockpit"


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
    if "<!-- BILD" in document or "<!-- COCKPIT_" in document:
        raise ValueError("Ungelöster Platzhalter in der Cockpit-Seite")
    document = decorate(finish(document), "KI-Hinweis.html")
    ZIEL.write_text(document, encoding="utf-8")
    print("Cockpit page built:", len(document.encode("utf-8")), "bytes")


if __name__ == "__main__":
    build()
