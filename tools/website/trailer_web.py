"""Promo-Trailer für die Website: Webfassungen, Standbild, Szenenbilder und Hintergrund aus dem Film selbst.

Quelle: ausgabe/pr-film/sternenepoche-weltenkruemmer.mp4 (Master 1080p, tools/film/sonnensystem_film.py).
Ziel:   docs/website/medien/trailer/ – trailer-1080.mp4 (≈ 4 Mbit/s), trailer-720.mp4 (≈ 2 Mbit/s), standbild.webp,
        raum.webp (Nebel ohne Objekte, für den Seitenhintergrund), szene-<n>.webp und trailer.json (Szenen mit Zeit).
Aufruf: python -X utf8 tools/website/trailer_web.py
"""
import json, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUELLE = ROOT / "ausgabe/pr-film/sternenepoche-weltenkruemmer.mp4"
ZIEL = ROOT / "docs/website/medien/trailer"
# Szenen nach der Zeitleiste des Films (T_RIG = 36 s; Laden, Feuer und Bruch aus dem Rig des Weltenkrümmers).
SZENEN = [(0.0, "Weltenkrümmer", "Ein Mond. Ein Schuss.", 2.4),
          (6.0, "Die Nachbarschaft", "Heimatwelt, Verbündete, Handelspartner", 7.6),
          (18.0, "Nachbar Vex", "Mond Asche", 19.6),
          (24.0, "Anomalie", "Etwas regt sich im Hyperraum", 28.6),
          (30.0, "Hyperraumloch", "Der Weltenkrümmer tritt aus", 32.4),
          (36.0, "Entfaltung", "Er nimmt den Mond in den Fokus", 43.0),
          (45.0, "Aufladen", "Die Fokussierschale lädt", 49.0),
          (53.5, "Scherlanze", "Der Strahl trifft den Mond", 55.4),
          (59.5, "Bruch", "Mond Asche zerstört", 61.0),
          (66.0, "Trümmerfeld", "Recycler und Bergungsschiffe bergen das Material", 70.0),
          (77.5, "Sternenepoche", "Ein Spiel über Imperien, Allianzen und Mut", 80.5)]


def ff(*a):
    subprocess.run(["ffmpeg", "-v", "error", "-y", *map(str, a)], check=True)


def bild(t, breite, ziel, q=82, extra=""):
    ff("-ss", t, "-i", QUELLE, "-frames:v", 1, "-vf", f"scale={breite}:-2:flags=lanczos{extra}", "-c:v", "libwebp", "-quality", q, ziel)


def main():
    ZIEL.mkdir(parents=True, exist_ok=True)
    for name, hoehe, rate in (("trailer-1080", 1080, "4200k"), ("trailer-720", 720, "2000k")):
        z = ZIEL / f"{name}.mp4"
        if z.exists() and z.stat().st_mtime > QUELLE.stat().st_mtime:
            continue
        log = ZIEL / f"{name}-pass"
        gemeinsam = ["-i", QUELLE, "-an", "-vf", f"scale=-2:{hoehe}:flags=lanczos", "-c:v", "libx264", "-preset", "slow", "-profile:v", "high",
                     "-pix_fmt", "yuv420p", "-b:v", rate, "-maxrate", str(int(rate[:-1]) * 2) + "k", "-bufsize", str(int(rate[:-1]) * 4) + "k",
                     "-g", 60, "-threads", 4, "-passlogfile", log]
        ff(*gemeinsam, "-pass", 1, "-f", "mp4", "NUL")
        ff(*gemeinsam, "-pass", 2, "-movflags", "+faststart", z)
        for p in ZIEL.glob(f"{name}-pass*"):
            p.unlink()
        print(name, f"{z.stat().st_size / 1e6:.1f} MB", flush=True)
    bild(61.0, 1920, ZIEL / "standbild.webp", 84)
    bild(27.0, 1600, ZIEL / "raum.webp", 70, ",gblur=sigma=6")
    liste = []
    for i, (t, titel, text, bildzeit) in enumerate(SZENEN):
        bild(bildzeit, 480, ZIEL / f"szene-{i:02d}.webp", 78)
        liste.append({"t": t, "titel": titel, "text": text, "bild": f"szene-{i:02d}.webp"})
    dauer = float(subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(QUELLE)],
                                 capture_output=True, text=True).stdout)
    (ZIEL / "trailer.json").write_text(json.dumps({"dauer": dauer, "szenen": liste}, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("fertig", len(liste), "Szenen", flush=True)


if __name__ == "__main__":
    main()
