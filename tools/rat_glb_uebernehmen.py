"""Neu gebackene Web-GLBs der Cockpit-Sitzung für die Website übernehmen.

Quelle: harness/cockpit/assets/figuren/web/<fid>.glb; Ziel: docs/rat/<fid>/figur.glb, Eintrag „glb“ in docs/rat/rat.json.
Prüft vor dem Kopieren den GLB-Kopf (Magie, Version 2, Länge) und meldet Größe und Dreieckszahl."""
import json
import shutil
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUELLE = ROOT / "harness/cockpit/assets/figuren/web"
RAT = ROOT / "docs/rat"


def pruefen(pfad: Path) -> str:
    daten = pfad.read_bytes()
    magie, version, laenge = struct.unpack_from("<4sII", daten)
    assert magie == b"glTF" and version == 2 and laenge == len(daten), (pfad.name, magie, version, laenge, len(daten))
    n = struct.unpack_from("<I", daten, 12)[0]
    gltf = json.loads(daten[20:20 + n])
    dreiecke = 0
    for netz in gltf.get("meshes", []):
        for teil in netz["primitives"]:
            if "indices" in teil:
                dreiecke += gltf["accessors"][teil["indices"]]["count"] // 3
    bilder = [b.get("mimeType") for b in gltf.get("images", [])]
    return f"{len(daten) / 1e6:.1f} MB, {dreiecke:,} Dreiecke, Bilder {bilder}"


rat = json.loads((RAT / "rat.json").read_text(encoding="utf-8"))
for glb in sorted(QUELLE.glob("*.glb")):
    fid = glb.stem
    if fid not in rat:
        print("übersprungen (nicht im Rat):", fid)
        continue
    info = pruefen(glb)
    (RAT / fid).mkdir(exist_ok=True)
    shutil.copyfile(glb, RAT / fid / "figur.glb")
    rat[fid]["glb"] = f"{fid}/figur.glb"
    print(fid, info)
(RAT / "rat.json").write_text(json.dumps(rat, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
print("mit 3D:", sorted(k for k, v in rat.items() if v.get("glb")))
