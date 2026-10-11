"""Cockpit-Paket vor der Veröffentlichung prüfen: Inhalt, Prüfsumme, keine Entwicklerpfade, keine Schlüssel.

Aufruf: python tools/paket_pruefen.py [ZIP]   (Standard: neuestes harness/paket/aus/*.zip)"""
import hashlib
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
zip_pfad = Path(sys.argv[1]) if len(sys.argv) > 1 else max((ROOT / "harness/paket/aus").glob("*.zip"), key=lambda p: p.stat().st_mtime)
# Jede Tailscale-Adresse außer der offiziellen Spieladresse verrät einen Rechnernamen.
MUSTER = [rb"C:\\Users\\[A-Za-z]+", rb"projekte_ki", rb"D:\\Projekte", rb"D:\\Temp", rb"(?i)\b(?!sternenepoche\.)[a-z0-9-]+\.[a-z0-9-]+\.ts\.net",
          rb"sk-or-v1-[0-9a-f]{20}", rb"sk-ant-[A-Za-z0-9_-]{20}", rb"sep_[0-9a-f]{64}", rb"@gmail\.com"]

sha = hashlib.sha256(zip_pfad.read_bytes()).hexdigest()
print(zip_pfad.name, f"{zip_pfad.stat().st_size / 1e6:.1f} MB", "SHA-256", sha)
summen = zip_pfad.with_name("SHA256SUMS.txt")
if summen.is_file():
    print("SHA256SUMS.txt stimmt:", sha in summen.read_text(encoding="utf-8"))
funde = 0
with zipfile.ZipFile(zip_pfad) as z:
    for i in z.infolist():
        print(f"  {i.file_size / 1e6:8.2f} MB  {i.filename}")
        daten = z.read(i)
        # Lizenztexte nennen die Urheber der Bibliotheken mit ihren öffentlichen Adressen; das verlangen die Lizenzen.
        if i.filename.endswith("THIRD-PARTY-NOTICES.txt"):
            daten = re.sub(rb"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[a-z]{2,}", b"", daten)
        for m in MUSTER:
            n = len(re.findall(m, daten))
            if n:
                funde += 1
                beispiel = re.search(m, daten).group(0)[:40]
                print(f"    FUND {m.decode()}: {n}× (z. B. {beispiel!r})")
    if any(i.filename.endswith(".pdb") for i in z.infolist()):
        funde += 1; print("    FUND: .pdb im Paket")
print("PASS: keine Entwicklerpfade, Schlüssel oder Debug-Dateien" if not funde else f"FEHLER: {funde} Funde")
sys.exit(1 if funde else 0)
