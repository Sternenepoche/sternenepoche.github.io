"""Hörchronik für die Website: je Folge eine Opus-Datei (Mono, 40 kbit/s) aus dem Master-WAV und eine Kapitelliste.

Quelle: laeufe/audio/hoerchronik/<folge>.wav (Master, −16 LUFS) und ausgabe/hoerchronik/<folge>.json (Kapitelmarken).
Ziel:   docs/website/medien/hoerchronik/<folge>.ogg + hoerchronik.json – klein genug für GitHub Pages (je Folge < 25 MB) –
        und <folge>.text.json zum Nachlesen: Kapitel mit Sprecher und Sprechtext aus content/hoerchronik/<folge>.drehbuch.md
        (ohne Klang-, Musik- und Regieanweisungen, ohne Belege).
Aufruf: python -X utf8 tools/website/hoerchronik_web.py
"""
import json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUELLE, META = ROOT / "laeufe/audio/hoerchronik", ROOT / "ausgabe/hoerchronik"
ZIEL = ROOT / "docs/website/medien/hoerchronik"
FOLGEN = ["00-chronik-der-weite", "01-aurelianer", "02-krath", "03-veyari", "04-syntheten"]


def dauer(p: Path) -> float:
    r = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(p)], capture_output=True, text=True)
    return float(r.stdout.strip())


DREHBUCH = ROOT / "content/hoerchronik"
GASTGEBER = {"00": "ERZÄHLERIN", "01": "SEVERIN", "02": "GROTHAK", "03": "THALEN", "04": "LUMEN"}


def text(f: str, besetzung: dict) -> tuple[dict, dict]:
    """Drehbuch → Kapitel mit Zeilen {wer, volk, text}; dazu Titel und Kurzbeschreibung der Folge."""
    roh = (DREHBUCH / f"{f}.drehbuch.md").read_text(encoding="utf-8")
    roh = re.sub(r"<!--.*?-->", "", roh, flags=re.S).split("\n## Belege", 1)[0]
    kopf = {"titel": re.search(r"^# (.+)$", roh, re.M)[1].strip(), "kurz": re.search(r"^> (.+)$", roh, re.M)[1].strip()}
    kapitel = []
    for m in re.finditer(r"^## (.+?)\n(.*?)(?=^## |\Z)", roh, re.M | re.S):
        zeilen = []
        for z in m[2].splitlines():
            t = re.match(r"^([A-ZÄÖÜ][A-ZÄÖÜ0-9_]*)(?:\s*\{[^}]*\})?:\s*(.+)$", z.strip())
            if not t:
                continue
            s = besetzung.get(t[1], {})
            zeilen.append({"wer": s.get("name", t[1].title()), "volk": s.get("volk", ""), "text": t[2].strip()})
        kapitel.append({"titel": m[1].strip(), "zeilen": zeilen})
    return kopf, {"kapitel": kapitel}


def main():
    ZIEL.mkdir(parents=True, exist_ok=True)
    liste = []
    besetzung = json.loads((DREHBUCH / "besetzung.json").read_text(encoding="utf-8"))["sprecher"]
    for f in FOLGEN:
        wav, ogg = QUELLE / f"{f}.wav", ZIEL / f"{f}.ogg"
        if not ogg.exists() or ogg.stat().st_mtime < wav.stat().st_mtime:
            print("kodiere", f, flush=True)
            subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", str(wav), "-ac", "1", "-c:a", "libopus", "-b:a", "40k", "-vbr", "on",
                            "-application", "audio", "-frame_duration", "60", "-map_metadata", "-1", str(ogg)], check=True)
        m = json.loads((META / f"{f}.json").read_text(encoding="utf-8"))
        kopf, nachlesen = text(f, besetzung)
        if [k["titel"] for k in nachlesen["kapitel"]] != [k["titel"] for k in m["kapitel"]]:
            raise SystemExit(f"{f}: Kapitel im Drehbuch und in der Aufnahme weichen ab – erst neu rendern")
        (ZIEL / f"{f}.text.json").write_text(json.dumps(nachlesen, ensure_ascii=False, separators=(",", ":")), encoding="utf-8", newline="\n")
        host = besetzung[GASTGEBER[f[:2]]]
        liste.append({"id": f, "titel": kopf["titel"], "kurz": kopf["kurz"], "gastgeber": host["name"], "rolle": host.get("rolle", ""),
                      "volk": host.get("volk", ""), "datei": f"{f}.ogg", "text": f"{f}.text.json", "dauer": round(dauer(ogg), 1),
                      "bytes": ogg.stat().st_size, "kapitel": [{"titel": k["titel"], "start": round(k["start_s"], 1)} for k in m["kapitel"]]})
        print(f"  {f}: {ogg.stat().st_size / 1e6:.1f} MB, {liste[-1]['dauer'] / 60:.0f} min, {len(m['kapitel'])} Kapitel", flush=True)
    (ZIEL / "hoerchronik.json").write_text(json.dumps(liste, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")


if __name__ == "__main__":
    sys.exit(main())
