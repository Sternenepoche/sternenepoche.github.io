"""Logo des Technologiebaums mit dem lokalen Bildmodell (Karl, 18.10.2026: „mach ein Bild mit lokalem Modell“).

Qwen-Image 2.1 im Edit-Modus über ComfyUI 8191 (wie tools/content/blaupausen.py), Vorlage ist das echte Zeichen der
Sternenepoche (docs/branding/sternenepoche-mark.png): derselbe facettierte Vierstern, aus dem ein Technologiebaum wächst.
Danach Freistellen mit BiRefNet (RemoveBackground) → PNG mit Alpha.

GPU-Regeln: Start nur bei Koordinator idle (120 s) und leerer Queue, Stapel höchstens 8, danach /free.
Aufruf:
  python -X utf8 tools/website/techbaum_logo.py [--saaten 1801,1802,…]   Kandidaten nach docs/website/logo/kandidaten/
  python -X utf8 tools/website/techbaum_logo.py --waehlen <saat>          freistellen → docs/website/logo/techbaum-logo.png/.webp
"""
from __future__ import annotations

import argparse, json, sys, time
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/content"))
from planeten_karten import api, gpu_frei  # noqa: E402
import weltenkruemmer_ansichten as wk  # noqa: E402

ZIEL = ROOT / "docs/website/logo"
PROMPT = (
    "Redesign this exact emblem into the logo of a TECHNOLOGY TREE. Keep the very same four-point star unchanged in shape and "
    "material: four separated tapered faceted segments of brushed titanium and dark graphite with fine champagne-gold chamfered "
    "edges and a small icy cyan glowing diamond core. Place the star in the lower centre. From its top point grows a stylised "
    "tree of knowledge: precise machined circuit traces branching upward and outward like a symmetric tree crown, ending in seven "
    "to nine small glowing hexagonal nodes; the central trunk path glows warm gold, the outer branches glow cool cyan, built from "
    "the same metal, gold and light as the star. Premium industrial space-navigation hardware, restrained dimensional depth, "
    "front view, centred, compact square composition filling 75 percent of the canvas, iconic and readable at 32 pixels. "
    "Plain flat medium grey background.")
NEGATIV = ("text, letters, numbers, words, typography, watermark, signature, frame, border, circle, ring, shield, wings, planet, "
           "scenery, stars in background, collage, multiple emblems, cartoon, flat vector clipart, blurry, cut off, cropped")
wk.NEGATIV = NEGATIV


def vorlage() -> Path:
    """Zeichen auf mittelgrauem Grund (LoadImage verwirft Alpha)."""
    datei = ZIEL / "vorlage-stern.png"
    ZIEL.mkdir(parents=True, exist_ok=True)
    with Image.open(ROOT / "docs/branding/sternenepoche-mark.png") as m:
        m = m.convert("RGBA").resize((768, 768), Image.LANCZOS)
        grund = Image.new("RGBA", (1024, 1024), (118, 122, 128, 255))
        grund.alpha_composite(m, (128, 200))
        grund.convert("RGB").save(datei)
    return datei


def kandidaten(saaten: list[int]) -> None:
    ziel = ZIEL / "kandidaten"
    ziel.mkdir(parents=True, exist_ok=True)
    print("warte auf Koordinator idle (120 s) …", flush=True)
    gpu_frei(120)
    ref = wk.hochladen(vorlage())
    for s in saaten[:8]:
        datei = ziel / f"techbaum-{s}.png"
        g = wk.graph([ref], PROMPT, s, "sternenepoche/website/techbaum-logo", schritte=30)
        t = time.monotonic()
        wk.speichern(datei, g, PROMPT, s)
        print(f"  {datei.name} {time.monotonic() - t:.0f} s", flush=True)
    frei()
    bogen(ziel)


def frei() -> None:
    try:
        q = json.loads(api("/queue", timeout=10))
        if not q["queue_running"] and not q["queue_pending"]:
            api("/free", {"unload_models": True, "free_memory": True})
            print("VRAM freigegeben", flush=True)
    except Exception as e:  # noqa: BLE001
        print("Freigabe nicht möglich:", e, flush=True)


def bogen(ordner: Path) -> None:
    bilder = sorted(ordner.glob("techbaum-*.png"))
    if not bilder:
        return
    b = Image.new("RGB", (4 * 384, ((len(bilder) + 3) // 4) * 384), (10, 14, 20))
    for i, p in enumerate(bilder):
        with Image.open(p) as im:
            b.paste(im.convert("RGB").resize((384, 384)), ((i % 4) * 384, (i // 4) * 384))
    b.save(ordner / "bogen.jpg", quality=88)
    print("Kontaktbogen", ordner / "bogen.jpg")


def waehlen(saat: int) -> None:
    """Freistellen mit BiRefNet (ComfyUI-Kernknoten), dann zuschneiden und als PNG/WebP ablegen."""
    quelle = ZIEL / "kandidaten" / f"techbaum-{saat}.png"
    name = wk.hochladen(quelle)
    g = {"1": {"class_type": "LoadImage", "inputs": {"image": name}},
         "2": {"class_type": "LoadBackgroundRemovalModel", "inputs": {"bg_removal_name": "BiRefNet.safetensors"}},
         "3": {"class_type": "RemoveBackground", "inputs": {"image": ["1", 0], "bg_removal_model": ["2", 0]}},
         "4": {"class_type": "InvertMask", "inputs": {"mask": ["3", 0]}},
         "5": {"class_type": "JoinImageWithAlpha", "inputs": {"image": ["1", 0], "alpha": ["4", 0]}},
         "6": {"class_type": "SaveImage", "inputs": {"images": ["5", 0], "filename_prefix": "sternenepoche/website/techbaum-logo-frei"}}}
    roh = wk.lauf(g)
    if not roh:
        raise SystemExit("Freistellen fehlgeschlagen")
    frei_png = ZIEL / f"techbaum-{saat}-frei.png"
    frei_png.write_bytes(roh)
    with Image.open(frei_png) as im:
        im = im.convert("RGBA")
        box = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
        im = im.crop(box)
        seite = max(im.size) + 24
        q = Image.new("RGBA", (seite, seite), (0, 0, 0, 0))
        q.alpha_composite(im, ((seite - im.width) // 2, (seite - im.height) // 2))
        q = q.resize((512, 512), Image.LANCZOS)
        q.save(ZIEL / "techbaum-logo.png", optimize=True)
        q.save(ZIEL / "techbaum-logo.webp", "WEBP", quality=90, method=6)
        q.resize((128, 128), Image.LANCZOS).save(ZIEL / "techbaum-logo-128.png", optimize=True)
    (ZIEL / "techbaum-logo.json").write_text(json.dumps({"saat": saat, "quelle": str(quelle.relative_to(ROOT)), "prompt": PROMPT,
                                                        "negativ": NEGATIV, "modell": "qwen_image_2.1 edit, BiRefNet"},
                                                       ensure_ascii=False, indent=1), encoding="utf-8")
    frei()
    print("Logo:", ZIEL / "techbaum-logo.png")


if __name__ == "__main__":
    a = argparse.ArgumentParser()
    a.add_argument("--saaten", default="1801,1802,1803,1804,1805,1806,1807,1808")
    a.add_argument("--waehlen", type=int)
    x = a.parse_args()
    waehlen(x.waehlen) if x.waehlen else kandidaten([int(s) for s in x.saaten.split(",")])
