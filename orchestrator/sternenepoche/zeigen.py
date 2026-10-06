"""Betrachter im Browser, auch während ein Lauf noch rechnet.

`python -m sternenepoche zeigen LAUFORDNER` startet einen kleinen Webserver nur auf 127.0.0.1. Er liefert
betrachter/index.html und die Dateien des Laufs. Läuft der Lauf noch (kein schluss.json), baut er den
Zwischenstand aus dem neuesten Tages-Schnappschuss: die Engine lädt ihn und gibt Rangliste, Statistik und
Galaxie aus, genau wie am Ende eines Laufs. Dazu kommen die Entscheidungen der Modelle in knapper Form.

Der Server liest nur. Er schreibt nichts in den Laufordner und ruft kein Modell auf.
"""

from __future__ import annotations

import json
import threading
import time
import tomllib
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

from .bruecke import Bruecke
from .protokoll import lies_jsonl

WURZEL = Path(__file__).resolve().parents[2]
BETRACHTER = WURZEL / "betrachter" / "index.html"
# Was der Betrachter vom Laufordner lesen darf; alles andere bleibt unerreichbar.
TABELLEN = ("tageswerte.jsonl", "kampfberichte.jsonl", "register.jsonl", "nachrichten.jsonl", "handel.jsonl")


def _kurz(wert, laenge: int = 400) -> str:
    text = wert if isinstance(wert, str) else json.dumps(wert, ensure_ascii=False)
    return text if len(text) <= laenge else text[: laenge - 1] + "…"


def _aktion(a: dict, korrektur: bool) -> dict:
    befehl = a.get("aktion")
    return {"typ": befehl.get("typ", "?") if isinstance(befehl, dict) else "?", "ok": bool(a.get("ok")),
            "text": _kurz(a.get("text", ""), 240), "befehl": _kurz(befehl, 300), "korrektur": korrektur}


def knapp(nr: int, e: dict) -> dict:
    """Eine Entscheidung so, wie der Betrachter sie zeigt: wer, warum, was, mit welchem Ergebnis, was es kostete."""
    aktionen = [_aktion(a, False) for a in e.get("aktionen", [])] + [_aktion(a, True) for a in e.get("korrektur", [])]
    runden = e.get("runden", [])
    return {
        "nr": nr, "zeit": e.get("zeit"), "zeittext": e.get("zeittext"), "spieler": e.get("spieler"), "name": e.get("name"),
        "rolle": e.get("rolle"), "modell": e.get("modell"), "gruende": e.get("gruende", []),
        "begruendung": _kurz(e.get("begruendung") or "", 900), "notiz": _kurz(e.get("notiz") or "", 1500),
        "prognose": _kurz(e.get("prognose") or "", 600), "wecker_stunden": e.get("wecker_stunden"),
        "abfragen": len(e.get("abfragen", [])), "runden": len(runden), "aktionen": aktionen, "fehler": e.get("fehler"),
        "kosten": round(sum(r.get("kosten_usd") or 0.0 for r in runden), 6),
        "punkte": e.get("punkte"), "rang": e.get("rang"), "stufe": e.get("stufe"),
    }


class Lage:
    """Liest einen Laufordner und hält teure Ergebnisse vor, bis sich die Quelldatei ändert."""

    def __init__(self, lauf: Path, engine: Path | None = None):
        self.dir = lauf
        self.engine = engine or self._engine_aus_konfig()
        self._sperre = threading.Lock()
        self._entscheidungen: tuple[tuple, list[dict]] = ((), [])
        self._zwischenstand: tuple[str, dict | None] = ("", None)

    def _engine_aus_konfig(self) -> Path:
        pfad = Path("target/release/sternenepoche.exe")
        konfig = self.dir / "konfig.toml"
        if konfig.exists():
            pfad = Path(tomllib.loads(konfig.read_text(encoding="utf-8")).get("lauf", {}).get("engine", pfad))
        if not pfad.is_absolute():
            pfad = WURZEL / pfad
        if not pfad.exists() and pfad.suffix == ".exe":
            pfad = pfad.with_suffix("")
        return pfad

    def entscheidungen(self) -> list[dict]:
        datei = self.dir / "entscheidungen.jsonl.gz"
        if not datei.exists():
            return []
        st = datei.stat()
        schluessel = (st.st_size, st.st_mtime_ns)
        with self._sperre:
            if self._entscheidungen[0] != schluessel:
                # lies_jsonl verträgt mehrteiliges gzip und eine angefangene letzte Zeile.
                self._entscheidungen = (schluessel, [knapp(i, e) for i, e in enumerate(lies_jsonl(datei))])
            return self._entscheidungen[1]

    def kosten(self) -> dict:
        liste = self.entscheidungen()
        return {"usd": round(sum(e["kosten"] for e in liste), 4), "aufrufe": sum(e["runden"] for e in liste)}

    def schluss(self) -> dict | None:
        """schluss.json, oder während des Laufs der Zwischenstand aus dem neuesten Schnappschuss."""
        datei = self.dir / "schluss.json"
        if datei.exists():
            return json.loads(datei.read_text(encoding="utf-8"))
        bilder = sorted((self.dir / "schnappschuesse").glob("tag-*.bin"))
        if not bilder:
            return None
        neuestes = bilder[-1]
        with self._sperre:
            if self._zwischenstand[0] == neuestes.name:
                stand = self._zwischenstand[1]
            else:
                stand = None
                # Ein Schnappschuss kann gerade geschrieben werden: dann den vorigen nehmen.
                for bild in reversed(bilder[-2:]):
                    try:
                        stand = self._aus_schnappschuss(bild)
                        break
                    except Exception:  # noqa: BLE001 - halb geschriebene Datei, der vorige reicht
                        continue
                self._zwischenstand = (neuestes.name, stand)
        if stand is None:
            return None
        return {**stand, "kosten": self.kosten()}

    def _aus_schnappschuss(self, bild: Path) -> dict:
        with Bruecke(self.engine) as e:
            stand = e.ruf("laden", pfad=str(bild))
            pruef = e.ruf("hash")
            return {
                "startwert": stand["startwert"], "spieler": len(stand["spieler"]), "ki": stand["ki"], "tage": stand["tage"],
                "regel_version": stand["regel_version"], "regel_hash": stand["regel_hash"], "bots": stand["bots"],
                "hash": pruef["hash"], "zeit": stand["zeit"], "beendet": False, "live": True,
                "schnappschuss": bild.name, "stand": stand["zeittext"],
                "rangliste": e.ruf("rangliste")["rangliste"], "statistik": e.ruf("statistik")["spieler"], "welt": e.ruf("welt"),
            }

    def tabelle(self, name: str) -> str:
        datei = self.dir / name
        if not datei.exists():
            return ""
        text = datei.read_text(encoding="utf-8")
        # Während des Laufs kann die letzte Zeile noch unvollständig sein.
        return text if text.endswith("\n") or not text else text[: text.rfind("\n") + 1]

    def stand(self) -> dict:
        liste = self.entscheidungen()
        letzte = liste[-1] if liste else None
        return {"live": not (self.dir / "schluss.json").exists(), "lauf": self.dir.name, "entscheidungen": len(liste),
                "kosten": self.kosten(), "letzte": letzte["zeittext"] if letzte else None, "zeit": time.time()}


def handler(lage: Lage):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):  # leise: der Betrachter fragt regelmäßig nach
            pass

        def _senden(self, status: int, typ: str, daten: bytes) -> None:
            self.send_response(status)
            self.send_header("Content-Type", typ)
            self.send_header("Content-Length", str(len(daten)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(daten)

        def _json(self, wert) -> None:
            self._senden(200, "application/json; charset=utf-8", json.dumps(wert, ensure_ascii=False).encode("utf-8"))

        def do_GET(self) -> None:  # noqa: N802 - Name aus http.server
            url = urlparse(self.path)
            try:
                if url.path in ("/", "/index.html"):
                    self._senden(200, "text/html; charset=utf-8", BETRACHTER.read_bytes())
                elif url.path == "/api/stand":
                    self._json(lage.stand())
                elif url.path == "/api/entscheidungen":
                    ab = int(parse_qs(url.query).get("ab", ["0"])[0])
                    self._json(lage.entscheidungen()[max(0, ab):])
                elif url.path == "/lauf/schluss.json":
                    s = lage.schluss()
                    if s is None:
                        self._senden(404, "text/plain; charset=utf-8", "Noch kein Schnappschuss".encode("utf-8"))
                    else:
                        self._json(s)
                elif url.path.startswith("/lauf/") and url.path[6:] in TABELLEN:
                    self._senden(200, "text/plain; charset=utf-8", lage.tabelle(url.path[6:]).encode("utf-8"))
                else:
                    self._senden(404, "text/plain; charset=utf-8", b"nicht gefunden")
            except Exception as x:  # noqa: BLE001 - der Fehler gehört in die Antwort, nicht in einen Absturz
                self._senden(500, "text/plain; charset=utf-8", f"{type(x).__name__}: {x}".encode("utf-8"))

    return Handler


def zeigen(lauf: str | Path, port: int = 8198, oeffnen: bool = True) -> None:
    lage = Lage(Path(lauf))
    if not lage.dir.is_dir():
        raise FileNotFoundError(f"Laufordner {lage.dir} gibt es nicht")
    server = ThreadingHTTPServer(("127.0.0.1", port), handler(lage))
    adresse = f"http://127.0.0.1:{server.server_address[1]}/"
    print(f"Betrachter für {lage.dir} auf {adresse} (Strg+C beendet)", flush=True)
    if oeffnen:
        webbrowser.open(adresse)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
