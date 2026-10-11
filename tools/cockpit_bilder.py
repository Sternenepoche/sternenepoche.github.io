"""Bildschirmfotos des Harness-Cockpits für die Website: PNG aus der Cockpit-Abnahme -> WebP (1600 px breit).

  python tools/cockpit_bilder.py [QUELLORDNER]

Quelle: docs/bilder/cockpit/*.png (die Abnahme legt sie dort ab) oder ein angegebener Ordner.
Ziel:   docs/bilder/cockpit/<name>.webp. Titelleiste und Rahmen von Windows werden abgeschnitten,
wenn das Bild sie enthält (erkennbar an der Leiste oben, die nicht nachtblau ist)."""
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ZIEL = ROOT / "docs" / "bilder" / "cockpit"


def umwandeln(quelle: Path) -> list[str]:
    ZIEL.mkdir(parents=True, exist_ok=True)
    fertig = []
    for png in sorted(quelle.glob("*.png")):
        with Image.open(png) as bild:
            bild = bild.convert("RGB")
            b, h = bild.size
            # Fensterrahmen und Titelleiste von Windows abschneiden: Das Cockpit beginnt in der Bildmitte beim ersten
            # nachtblauen Bildpunkt (blau deutlich über rot, dunkel). Liegt darüber eine Titelleiste, hat das Fenster
            # auch links, rechts und unten einen Rahmen von einem Bildpunkt. Inhalte am Rand (Bildlaufleisten,
            # helle Karten) dürfen den Zuschnitt nicht verschieben, deshalb wird nur oben gesucht.
            nacht = lambda p: p[2] - p[0] >= 8 and sum(p) < 160
            oben = next((y for y in range(0, min(90, h)) if nacht(bild.getpixel((b // 2, y)))), 0)
            if oben:
                bild = bild.crop((1, oben, b - 1, h - 1))
            if bild.width > 1600:
                bild = bild.resize((1600, round(bild.height * 1600 / bild.width)), Image.LANCZOS)
            ziel = ZIEL / (png.stem + ".webp")
            bild.save(ziel, "WEBP", quality=84, method=6)
            fertig.append(f"{ziel.name} {bild.width}x{bild.height} {ziel.stat().st_size // 1024} KB")
    return fertig


if __name__ == "__main__":
    for zeile in umwandeln(Path(sys.argv[1]) if len(sys.argv) > 1 else ZIEL):
        print(zeile)
