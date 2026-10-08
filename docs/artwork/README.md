# Transparente Hero-Grafiken

Stand: 8. Oktober 2026. Die Startseite und der Spielguide verwenden zwei echte Alpha-Freistellungen bestehender Spielgrafiken. Der frühere Helligkeitsfilter machte dunkle Rumpfteile durchsichtig. Beide Bilder werden jetzt normal zusammengesetzt, ohne diesen Filter und ohne Screen-Mischmodus.

| Datei | Abmessungen | Größe |
|---|---|---|
| `hero-cruiser-v2.webp` | 960 × 576 | 83.626 Bytes |
| `hero-ocean-v2.webp` | 960 × 960 | 152.166 Bytes |

Zusammen 235.792 Bytes. Die öffentliche Seite verwendet gecachte WebP-Dateien. Beim Pages-Build wird für Bilder mit Alpha ein transparenter PNG-Rückfall erzeugt; JPEG würde einen rechteckigen Hintergrund hinzufügen. Die hochauflösenden PNG-Master liegen lokal unter `content/website/`. Die ursprünglichen Spielassets bleiben erhalten.

Erstellt mit dem eingebauten ImageGen-Werkzeug, Modus `background-extraction`, `transparent_background: true`. Ausgangsbilder: `content/assets/ships/ships.kreuzer.aurelianer.png` und `content/assets/planets/planets.leben_ozean.png`. Anschließend ausschließlich verkleinert und als WebP mit Qualität 84, Alphaqualität 100 und Methode 6 codiert.

## Prompt für das Schiff

Use case: background-extraction. Edit target: the provided original Aurelianer cruiser game asset. Deliver one production cutout for the Sternenepoche website. Remove ONLY the dark navy studio background outside the spaceship, replacing it with actual transparent alpha. Preserve this exact spaceship, its nose at lower left and engines at upper right, the entire silhouette, orientation, proportions, panel detail, original pale champagne hull, gray armor, bronze engines, lighting and shadows. All dark engine interiors, windows, recesses and shadowed hull panels must remain OPAQUE; do not punch holes into dark parts of the ship. Preserve fine antennas and gun barrels with clean antialiased edges, no dark background halo, no jagged outline. Retain the full ship with a small transparent safety margin around it, landscape canvas closely matching the original 1280 x 768 aspect ratio. This is careful background extraction, not a redesign. No stars, no planet, no text, no glow, no extra parts, no checkerboard painted into the image.

## Prompt für den Planeten

Use case: background-extraction. Edit target: the provided ocean planet game asset. Deliver one production cutout for the Sternenepoche website. Remove ONLY the black background outside the circular planet and replace it with actual transparent alpha. Preserve the exact blue ocean planet, continent arrangement, white cloud layers, the original directional lighting and dark night side. The full planet surface including dark shadowed areas must remain OPAQUE. Preserve a very fine soft blue atmospheric rim with clean antialiased outer alpha, without a black rectangular background or a wide fuzzy halo. Keep the circular sphere centered, the entire globe visible with a small even transparent safety margin around it, square high-resolution canvas. No redesign, no extra moons, no rings, no text, no stars, no checkerboard painted into the image.
