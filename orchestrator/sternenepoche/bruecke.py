"""Anbindung der Engine als Unterprozess: zeilenweises JSON über Standardein- und -ausgabe."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


class EngineFehler(RuntimeError):
    pass


class Bruecke:
    def __init__(self, engine: str | Path):
        pfad = Path(engine)
        if not pfad.exists():
            raise EngineFehler(f"Engine nicht gefunden: {pfad}. Erst bauen: cargo build --release")
        self._p = subprocess.Popen(
            [str(pfad), "bruecke"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            bufsize=1,
        )

    def ruf(self, cmd: str, **felder) -> dict:
        """Schickt einen Befehl und liefert die Antwort. Fehler der Engine werden zur Ausnahme."""
        if self._p.poll() is not None:
            raise EngineFehler("die Engine läuft nicht mehr")
        self._p.stdin.write(json.dumps({"cmd": cmd, **felder}, ensure_ascii=False) + "\n")
        self._p.stdin.flush()
        zeile = self._p.stdout.readline()
        if not zeile:
            raise EngineFehler(f"die Engine hat auf '{cmd}' nicht geantwortet")
        antwort = json.loads(zeile)
        if not antwort.get("ok"):
            raise EngineFehler(f"{cmd}: {antwort.get('fehler')}")
        return antwort

    def schliessen(self) -> None:
        if self._p.poll() is None:
            try:
                self.ruf("ende")
            except (EngineFehler, OSError, ValueError):
                pass
            try:
                self._p.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._p.kill()
        for strom in (self._p.stdin, self._p.stdout):
            try:
                strom.close()
            except OSError:
                pass

    def __enter__(self) -> "Bruecke":
        return self

    def __exit__(self, *_) -> None:
        self.schliessen()
