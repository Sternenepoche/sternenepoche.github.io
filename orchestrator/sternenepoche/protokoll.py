"""Datenerfassung: alles, was ein Agent sieht, begründet und tut, samt Wirkung.

Die Entscheidungen enthalten nur, was der Agent zum Zeitpunkt der Entscheidung sah.
Der vollständige Weltzustand liegt getrennt in den Schnappschüssen. So bekommt ein
später trainiertes Modell keine Information als Eingabe, die dem Spieler verborgen war.
"""

from __future__ import annotations

import gzip
import json
import zlib
from pathlib import Path

TABELLEN = ("tageswerte", "kampfberichte", "register", "nachrichten", "handel")


class Ausgabe:
    def __init__(self, verzeichnis: str | Path, fortsetzen: bool = False):
        self.dir = Path(verzeichnis)
        self.dir.mkdir(parents=True, exist_ok=True)
        (self.dir / "schnappschuesse").mkdir(exist_ok=True)
        modus = "at" if fortsetzen else "wt"
        self._entscheidungen = gzip.open(self.dir / "entscheidungen.jsonl.gz", modus, encoding="utf-8")
        self._protokoll = open(self.dir / "protokoll.jsonl", "a" if fortsetzen else "w", encoding="utf-8")
        self._tabellen = {n: open(self.dir / f"{n}.jsonl", "a" if fortsetzen else "w", encoding="utf-8") for n in TABELLEN}
        self.zeiger = {n: 0 for n in TABELLEN}
        self.entscheidungen = 0
        fortschritt = self.dir / "fortschritt.json"
        if fortsetzen and fortschritt.exists():
            daten = json.loads(fortschritt.read_text(encoding="utf-8"))
            self.zeiger.update(daten.get("zeiger", {}))
            self.entscheidungen = daten.get("entscheidungen", 0)

    def entscheidung(self, datensatz: dict) -> None:
        self._entscheidungen.write(json.dumps(datensatz, ensure_ascii=False) + "\n")
        self.entscheidungen += 1

    def protokoll(self, eintraege: list[dict]) -> None:
        """Aktionsprotokoll der Engine: daraus und aus dem Startwert lässt sich der Lauf nachspielen."""
        for e in eintraege:
            self._protokoll.write(json.dumps(e, ensure_ascii=False) + "\n")

    def tabelle(self, name: str, zeilen: list[dict]) -> None:
        for z in zeilen:
            self._tabellen[name].write(json.dumps(z, ensure_ascii=False) + "\n")
        self.zeiger[name] += len(zeilen)

    def lies(self, name: str) -> dict:
        pfad = self.dir / name
        return json.loads(pfad.read_text(encoding="utf-8")) if pfad.exists() else {}

    def schreibe(self, name: str, daten) -> None:
        (self.dir / name).write_text(json.dumps(daten, ensure_ascii=False, indent=2), encoding="utf-8")

    def sichern(self) -> None:
        self._entscheidungen.flush()
        self._protokoll.flush()
        for f in self._tabellen.values():
            f.flush()
        self.schreibe("fortschritt.json", {"zeiger": self.zeiger, "entscheidungen": self.entscheidungen})

    def schliessen(self) -> None:
        self.sichern()
        self._entscheidungen.close()
        self._protokoll.close()
        for f in self._tabellen.values():
            f.close()


def _gzip_text(pfad: Path) -> str:
    """Entpackt alle gzip-Glieder einer Datei (jedes Fortsetzen hängt eines an). Endet die Datei mitten in
    einem Glied, weil der Prozess hart beendet wurde, gilt das Lesbare davor."""
    daten = pfad.read_bytes()
    teile: list[bytes] = []
    while daten:
        d = zlib.decompressobj(16 + zlib.MAX_WBITS)
        try:
            teile.append(d.decompress(daten))
        except zlib.error:
            break
        if not d.eof:
            break
        daten = d.unused_data
    return b"".join(teile).decode("utf-8", "replace")


def lies_jsonl(pfad: Path) -> list[dict]:
    """Liest vollständige Zeilen; eine beim Abbruch abgerissene letzte Zeile zählt nicht."""
    text = _gzip_text(pfad) if pfad.suffix == ".gz" else pfad.read_text(encoding="utf-8")
    zeilen = text.splitlines(keepends=True)
    if zeilen and not zeilen[-1].endswith("\n"):
        zeilen.pop()
    return [json.loads(z) for z in zeilen if z.strip()]


def auf_schnappschuss_kuerzen(verzeichnis: str | Path, zeit: int, zeilen: dict[str, int], log_anzahl: int) -> int:
    """Nach einem harten Abbruch: Protokoll, Tabellen und Entscheidungen auf den Stand eines Schnappschusses
    kürzen. Danach passen sie wieder genau zum Weltzustand, und Nachspielen trifft denselben Zustandshash.
    Liefert die Zahl der behaltenen Entscheidungen."""
    d = Path(verzeichnis)

    def kuerzen(pfad: Path, n: int) -> None:
        vollstaendig = [z for z in pfad.read_text(encoding="utf-8").splitlines(keepends=True) if z.endswith("\n")] if pfad.exists() else []
        if len(vollstaendig) < n:
            raise ValueError(f"{pfad.name} hat {len(vollstaendig)} Zeilen, der Schnappschuss kennt {n}")
        pfad.write_text("".join(vollstaendig[:n]), encoding="utf-8")

    kuerzen(d / "protokoll.jsonl", log_anzahl)
    for name, n in zeilen.items():
        kuerzen(d / f"{name}.jsonl", n)
    # Ein Fenster gehört zum Schnappschuss, wenn es zu dessen Zeit oder davor lief.
    behalten = [e for e in lies_jsonl(d / "entscheidungen.jsonl.gz") if e["zeit"] <= zeit]
    with gzip.open(d / "entscheidungen.jsonl.gz", "wt", encoding="utf-8") as f:
        for e in behalten:
            f.write(json.dumps(e, ensure_ascii=False) + "\n")
    (d / "fortschritt.json").write_text(json.dumps({"zeiger": zeilen, "entscheidungen": len(behalten)}, ensure_ascii=False, indent=2), encoding="utf-8")
    return len(behalten)


def etiketten(verzeichnis: str | Path) -> int:
    """Hängt jeder Entscheidung ihre Folgen an: Punktedifferenz nach 1, 7 und 30 Spieltagen und den Endrang."""
    d = Path(verzeichnis)
    tageswerte = lies_jsonl(d / "tageswerte.jsonl")
    punkte: dict[tuple[int, int], int] = {}
    for t in tageswerte:
        p = t["punkte"]
        punkte[(t["tag"], t["spieler"])] = p["wirtschaft"] + p["forschung"] + p["militaer"] + p["zivilisation"]
    schluss = json.loads((d / "schluss.json").read_text(encoding="utf-8"))
    endrang = {s["id"]: s["rang"] for s in schluss["statistik"]}
    endpunkte = {s["id"]: sum(s["punkte"].values()) for s in schluss["statistik"]}
    n = 0
    with open(d / "etiketten.jsonl", "w", encoding="utf-8") as aus:
        for e in lies_jsonl(d / "entscheidungen.jsonl.gz"):
            tag = e["zeit"] // 86_400
            zeile = {"zeit": e["zeit"], "spieler": e["spieler"], "rolle": e["rolle"], "punkte": e["punkte"],
                     "endrang": endrang.get(e["spieler"]), "endpunkte": endpunkte.get(e["spieler"])}
            for spanne in (1, 7, 30):
                spaeter = punkte.get((tag + spanne, e["spieler"]))
                zeile[f"delta_{spanne}"] = None if spaeter is None else spaeter - e["punkte"]
            aus.write(json.dumps(zeile, ensure_ascii=False) + "\n")
            n += 1
    return n
