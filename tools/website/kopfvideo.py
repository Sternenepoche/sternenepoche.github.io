"""Kopf-Video der Website: die vier Weltenkrümmer freigestellt (Alpha), damit sie vor dem blauen Planeten erscheinen.

Quelle: ausgabe/weltenkruemmer/filme/<volk>_mondzerstoerung.mp4 (tools/content/weltenkruemmer_film.py, 960×540, 24 fps).
Je Volk zwei Ausschnitte (Verwandlung, Strahl + Mondbruch), weich ein- und ausgeblendet, hintereinander geschnitten.
Freistellung: Schwarz des Weltraums → transparent (colorkey), Sterne per Öffnen (Erosion/Dilation) entfernt, Kante weich.

Ergebnis in docs/website/medien/:
  weltenkruemmer.webm / -klein.webm  VP9 mit Alphakanal (Chrome, Edge, Firefox, Android)
  weltenkruemmer.mp4  / -klein.mp4   H.264 auf Schwarz vormultipliziert – Safari/iOS legt es mit mix-blend-mode: screen auf
  weltenkruemmer-standbild.webp      Standbild mit Alpha (prefers-reduced-motion, Plakat)
  weltenkruemmer.json                Zeitmarken je Volk für die Beschriftung
Aufruf: python -X utf8 tools/website/kopfvideo.py   (--filme: Filme je Volk, --kopffilm: Spielfilm im Wechsel)
"""
import json, shutil, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUELLE = ROOT / "ausgabe/weltenkruemmer/filme"
ZIEL = ROOT / "docs/website/medien"
FFMPEG = shutil.which("ffmpeg") or r"C:\ffmpeg\bin\ffmpeg.exe"
# Reihenfolge und Ausschnitte (Sekunden im Quellfilm). Strahlfarben aus docs/LORE-BIBEL.md / Weltenkrümmer-Abnahme.
VOELKER = [("aurelianer", "Aurelianer", "#ffcf33"), ("krath", "Krath", "#ff4a2e"),
           ("veyari", "Veyari", "#5dff7a"), ("syntheten", "Syntheten", "#5ab8ff")]
AUSSCHNITTE = [(9.0, 15.0), (16.6, 22.6)]
BLENDE = 0.6
STANDBILD = ("aurelianer", 14.2)
# Maske als volle Silhouette: Sterne weg (Öffnen), dann Lücken und dunkle Schattierung schließen (Schließen), halbe Deckung
# auf voll anheben – sonst scheint der Planet durch dunkle Rumpfteile und der Weltenkrümmer wirkt flach.
MASKE = ("erosion,dilation,dilation,dilation,dilation,erosion,erosion,erosion,"
         "lut=y='min(255,val*3)',gblur=sigma=0.6")


def freistellen(eingang: str, a: float, b: float, name: str) -> str:
    d = b - a
    return (f"[{eingang}:v]trim=start={a}:end={b},setpts=PTS-STARTPTS,format=rgba,"
            f"colorkey=0x00030a:0.022:0.03,split[{name}c][{name}k];"
            f"[{name}k]alphaextract,{MASKE}[{name}m];"
            f"[{name}c][{name}m]alphamerge,fade=t=in:st=0:d={BLENDE}:alpha=1,fade=t=out:st={d - BLENDE:.2f}:d={BLENDE}:alpha=1[{name}]")


def main():
    ZIEL.mkdir(parents=True, exist_ok=True)
    eingaben, teile, marken, t = [], [], [], 0.0
    for i, (volk, name, farbe) in enumerate(VOELKER):
        eingaben += ["-i", str(QUELLE / f"{volk}_mondzerstoerung.mp4")]
        start = t
        for j, (a, b) in enumerate(AUSSCHNITTE):
            teile.append((i, a, b, f"v{i}{j}")); t += b - a
        marken.append({"volk": volk, "name": name, "farbe": farbe, "von": round(start, 2), "bis": round(t, 2)})
    graph = ";".join(freistellen(str(i), a, b, n) for i, a, b, n in teile)
    graph += ";" + "".join(f"[{n}]" for *_, n in teile) + f"concat=n={len(teile)}:v=1:a=0[alle];[alle]split=4[g][k][p][pk];"
    graph += "[k]scale=640:360:flags=lanczos[klein];"
    graph += "[p]premultiply=inplace=1,format=yuv420p[mp4];[pk]scale=640:360:flags=lanczos,premultiply=inplace=1,format=yuv420p[mp4k]"
    vp9 = ["-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-auto-alt-ref", "0", "-row-mt", "1", "-deadline", "good", "-cpu-used", "2"]
    x264 = ["-c:v", "libx264", "-preset", "slow", "-profile:v", "high", "-movflags", "+faststart", "-pix_fmt", "yuv420p"]
    befehl = [FFMPEG, "-v", "error", "-y", *eingaben, "-filter_complex", graph,
              "-map", "[g]", *vp9, "-b:v", "650k", "-crf", "44", "-an", str(ZIEL / "weltenkruemmer.webm"),
              "-map", "[klein]", *vp9, "-b:v", "300k", "-crf", "47", "-an", str(ZIEL / "weltenkruemmer-klein.webm"),
              "-map", "[mp4]", *x264, "-crf", "26", "-an", str(ZIEL / "weltenkruemmer.mp4"),
              "-map", "[mp4k]", *x264, "-crf", "28", "-an", str(ZIEL / "weltenkruemmer-klein.mp4")]
    subprocess.run(befehl, check=True)
    volk, sek = STANDBILD
    standbild = (f"[0:v]trim=start={sek}:end={sek + 0.1},setpts=PTS-STARTPTS,format=rgba,colorkey=0x00030a:0.022:0.03,split[c][k];"
                 "[k]alphaextract,"+MASKE+"[m];[c][m]alphamerge")
    subprocess.run([FFMPEG, "-v", "error", "-y", "-i", str(QUELLE / f"{volk}_mondzerstoerung.mp4"), "-filter_complex", standbild,
                    "-frames:v", "1", "-c:v", "libwebp", "-quality", "86", str(ZIEL / "weltenkruemmer-standbild.webp")], check=True)
    (ZIEL / "weltenkruemmer.json").write_text(json.dumps({"dauer": round(t, 2), "voelker": marken}, ensure_ascii=False, indent=1), encoding="utf-8")
    for f in sorted(ZIEL.glob("weltenkruemmer*")):
        print(f.name, f"{f.stat().st_size / 1e6:.2f} MB")


KINO = ROOT / "laeufe/weltenkruemmer-film"
SAAT = {"aurelianer": 1803, "krath": 4711, "syntheten": 9020, "veyari": 3317}


def kopf_rgba():
    """Kopf aus echten RGBA-Bildfolgen der Kinoseite (tools/weltenkruemmer_kino/film.py kopf …): kein Freistellen nötig.
    Je Volk 12 s, 1600×900, 24 fps, Ein-/Ausblenden aus Alpha 0 bereits gerendert."""
    eingaben, marken, t = [], [], 0.0
    for volk, name, farbe in VOELKER:
        # laeufe/weltenkruemmer-film/kopf ist eine Junction nach F:\Auslagerung_D\… (Karl 18.10.: Großes auf F:, Verweis auf D:).
        ordner = KINO / "kopf" / volk if any((KINO / "kopf" / volk).glob("0001.png")) else None
        if not ordner:
            raise SystemExit(f"Keine Kopf-Bildfolge für {volk}")
        bilder = sorted(ordner.glob("[0-9][0-9][0-9][0-9].png"))
        eingaben += ["-framerate", "24", "-i", str(ordner / "%04d.png")]
        dauer = len(bilder) / 24
        marken.append({"volk": volk, "name": name, "farbe": farbe, "von": round(t, 2), "bis": round(t + dauer, 2)})
        t += dauer
    n = len(VOELKER)
    graph = "".join(f"[{i}:v]format=rgba,setpts=PTS-STARTPTS[s{i}];" for i in range(n))
    graph += "".join(f"[s{i}]" for i in range(n)) + f"concat=n={n}:v=1:a=0[alle];[alle]split=4[g][k][p][pk];"
    graph += "[g]scale=1280:720:flags=lanczos[gross];[k]scale=640:360:flags=lanczos[klein];"
    graph += "[p]scale=1280:720:flags=lanczos,premultiply=inplace=1,format=yuv420p[mp4];[pk]scale=640:360:flags=lanczos,premultiply=inplace=1,format=yuv420p[mp4k]"
    vp9 = ["-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-auto-alt-ref", "0", "-row-mt", "1", "-deadline", "good", "-cpu-used", "2"]
    x264 = ["-c:v", "libx264", "-preset", "slow", "-profile:v", "high", "-movflags", "+faststart", "-pix_fmt", "yuv420p"]
    subprocess.run([FFMPEG, "-v", "error", "-y", *eingaben, "-filter_complex", graph,
                    "-map", "[gross]", *vp9, "-b:v", "900k", "-crf", "42", "-an", str(ZIEL / "weltenkruemmer.webm"),
                    "-map", "[klein]", *vp9, "-b:v", "320k", "-crf", "46", "-an", str(ZIEL / "weltenkruemmer-klein.webm"),
                    "-map", "[mp4]", *x264, "-crf", "26", "-an", str(ZIEL / "weltenkruemmer.mp4"),
                    "-map", "[mp4k]", *x264, "-crf", "28", "-an", str(ZIEL / "weltenkruemmer-klein.mp4")], check=True)
    subprocess.run([FFMPEG, "-v", "error", "-y", "-i", str(KINO / "kopf" / STANDBILD[0] / "poster.png"), "-vf", "scale=1280:-2",
                    "-c:v", "libwebp", "-quality", "86", str(ZIEL / "weltenkruemmer-standbild.webp")], check=True)
    (ZIEL / "weltenkruemmer.json").write_text(json.dumps({"dauer": round(t, 2), "voelker": marken, "quelle": "rgba"}, ensure_ascii=False, indent=1), encoding="utf-8")
    for f in sorted(ZIEL.glob("weltenkruemmer*")):
        print(f.name, f"{f.stat().st_size / 1e6:.2f} MB")


def filme():
    """Ganze Filme je Volk für die Weltenkrümmer-Seite: 960×540 H.264 ohne Ton, Plakat aus dem geladenen Zustand.
    Bevorzugt die neuen Kinofilme (laeufe/weltenkruemmer-film/<volk>-<saat>.mp4), sobald alle-fertig.txt da ist."""
    ziel = ZIEL / "filme"
    ziel.mkdir(parents=True, exist_ok=True)
    kino = (KINO / "alle-fertig.txt").exists()
    for volk, _, _ in VOELKER:
        q = str(KINO / f"{volk}-{SAAT[volk]}.mp4") if kino else str(QUELLE / f"{volk}_mondzerstoerung.mp4")
        print("Quelle", q)
        subprocess.run([FFMPEG, "-v", "error", "-y", "-i", q, "-an", "-vf", "scale=960:-2", "-c:v", "libx264", "-preset", "slow",
                        "-crf", "27", "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(ziel / f"{volk}.mp4")], check=True)
        poster = KINO / "kopf" / volk / "poster.png"
        if poster.exists():
            # Plakat aus der Kopf-Fassung: aufgeklappt und geladen, kurz vor dem Schuss, auf Weltraumschwarz.
            subprocess.run([FFMPEG, "-v", "error", "-y", "-f", "lavfi", "-i", "color=c=0x03060c:s=1600x900", "-i", str(poster),
                            "-filter_complex", "[0][1]overlay=format=auto,scale=960:-2", "-frames:v", "1", "-q:v", "4",
                            str(ziel / f"{volk}.jpg")], check=True)
        else:
            subprocess.run([FFMPEG, "-v", "error", "-y", "-ss", "14.2", "-i", q, "-frames:v", "1", "-vf", "scale=960:-2", "-q:v", "4",
                            str(ziel / f"{volk}.jpg")], check=True)
        print(volk, f"{(ziel / f'{volk}.mp4').stat().st_size / 1e6:.2f} MB")


KOPFFILM = ROOT / "ausgabe/pr-film/kopffilm-quelle.mp4"  # Karls Fassung vom 19.10. (Downloads/5gT4_6Q9HLiA0XkvaTrE9A.mp4)


def kopffilm():
    """Der ganze Spielfilm (82 s, Mond Asche bricht) für den Kopf der Startseite, im Wechsel mit den Weltenkrümmern
    (docs/website/seite.js): 960×540 für breite Fenster (das Kinofenster ist höchstens ~900 px breit), 640×360 fürs Handy, beide H.264 ohne Ton mit faststart."""
    x264 = ["-c:v", "libx264", "-preset", "slow", "-profile:v", "high", "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-an"]
    for name, hoehe, crf in (("kopffilm", 540, "27"), ("kopffilm-klein", 360, "28")):
        subprocess.run([FFMPEG, "-v", "error", "-y", "-i", str(KOPFFILM), "-vf", f"scale=-2:{hoehe}:flags=lanczos", *x264,
                        "-crf", crf, str(ZIEL / f"{name}.mp4")], check=True)
    # Standbild: der Strahl trifft den Mond.
    subprocess.run([FFMPEG, "-v", "error", "-y", "-ss", "56", "-i", str(KOPFFILM), "-frames:v", "1", "-vf", "scale=1280:-2",
                    "-c:v", "libwebp", "-quality", "82", str(ZIEL / "kopffilm-standbild.webp")], check=True)
    for f in sorted(ZIEL.glob("kopffilm*")):
        print(f.name, f"{f.stat().st_size / 1e6:.2f} MB")


if __name__ == "__main__":
    import sys
    if "--kopffilm" in sys.argv:
        kopffilm()
    elif "--filme" in sys.argv:
        filme()
    elif "--rgba" in sys.argv or (KINO / "kopf/kopf-fertig.txt").exists():
        kopf_rgba()
    else:
        main()
