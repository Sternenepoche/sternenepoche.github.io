"""Kommandozeile des Orchestrators."""

from __future__ import annotations

import argparse
import asyncio
import sys

from . import kosten as kosten_modul
from .backends import FatalerFehler
from .bruecke import Bruecke, EngineFehler
from .konfig import ROLLEN, KonfigFehler, laden
from .lauf import Abbruch, Lauf
from .prompt import systemtext
from .protokoll import etiketten


def _system_zeichen(konfig) -> dict[str, int]:
    """Länge des Systemtexts je Rolle, aus der Engine."""
    with Bruecke(konfig.lauf.engine) as engine:
        felder = {"startwert": 1, "spieler": 2, "tage": konfig.lauf.tage}
        if konfig.lauf.regeln:
            felder["regeln"] = konfig.lauf.regeln
        engine.ruf("neu", **felder)
        laengen = {}
        for rolle in ROLLEN:
            r = engine.ruf("regeltext", rolle=rolle)
            laengen[rolle] = len(systemtext(rolle, r["text"], konfig.lauf.tage, r["limits"]))
    return laengen


def cmd_lauf(args) -> int:
    konfig = laden(args.konfig)
    try:
        asyncio.run(Lauf(konfig, fortsetzen=args.fortsetzen).fahre())
    except Abbruch as x:
        print(x, file=sys.stderr)
        return 2
    return 0


def cmd_kosten(args) -> int:
    konfig = laden(args.konfig)
    modelle = kosten_modul.modelle_holen() if any(a.art == "openrouter" for a in konfig.anbieter.values()) else {}
    zeilen, summe = kosten_modul.schaetze(konfig, _system_zeichen(konfig), modelle)
    agenten = len(konfig.lauf.ki) if konfig.lauf.ki is not None else konfig.lauf.spieler
    print(f"Schätzung für {agenten} Agenten über {konfig.lauf.tage} Spieltage (Preise von OpenRouter, Stand jetzt):\n")
    print(kosten_modul.tabelle(zeilen, summe))
    print("\nLokale Anbieter ohne preis_ein und preis_aus zählen mit 0 USD. Die Schätzung nimmt das Mengengerüst des Konzepts an;"
          "\ngemessen wird im Lauf selbst, die Budgetgrenze [grenzen] budget_usd hält ihn an.")
    return 0


def cmd_pruefen(args) -> int:
    konfig = laden(args.konfig)
    print(f"Konfiguration {args.konfig}: in Ordnung")
    try:
        with Bruecke(konfig.lauf.engine) as engine:
            stand = engine.ruf("neu", startwert=1, spieler=2, tage=1)
            print(f"Engine: in Ordnung, Regelwerk {stand['regel_version']} ({stand['regel_hash'][:12]})")
    except EngineFehler as x:
        print(f"Engine: {x}")
        return 1
    fehler = 0
    if any(a.art == "openrouter" for a in konfig.anbieter.values()):
        modelle = kosten_modul.modelle_holen()
        print(f"OpenRouter: {len(modelle)} Modelle in der Liste")
        for b in kosten_modul.pruefe(konfig, modelle):
            print("  " + b)
            fehler += b.startswith("[FEHLER]")
        for env in sorted({a.schluessel_env for a in konfig.anbieter.values() if a.art == "openrouter"}):
            try:
                stand = kosten_modul.schluessel_stand(kosten_modul.OPENROUTER_URL, env)
            except Exception as x:  # noqa: BLE001 - jede Antwort außer Erfolg ist ein Befund
                print(f"  [FEHLER] Schlüssel aus {env} wird nicht angenommen: {x}")
                fehler += 1
                continue
            if stand is not None:
                rest = stand.get("limit_remaining")
                print(f"  [ok] Schlüssel aus {env} gültig, Restguthaben des Schlüssels: {rest if rest is not None else 'ohne Limit'}")
    else:
        for name, a in konfig.anbieter.items():
            print(f"  [ok] {name}: {a.art} {a.modell} {a.basis_url}")
    return 1 if fehler else 0


def cmd_etiketten(args) -> int:
    n = etiketten(args.verzeichnis)
    print(f"{n} Entscheidungen mit Folgen versehen: {args.verzeichnis}/etiketten.jsonl")
    return 0


def cmd_zeigen(args) -> int:
    from .zeigen import zeigen

    zeigen(args.verzeichnis, port=args.port, oeffnen=not args.ohne_browser)
    return 0


def main() -> None:
    p = argparse.ArgumentParser(prog="sternenepoche", description="Orchestrator für Sternenepoche")
    sub = p.add_subparsers(dest="befehl", required=True)
    s = sub.add_parser("lauf", help="fährt eine Epoche nach einer Konfiguration")
    s.add_argument("konfig")
    s.add_argument("--fortsetzen", action="store_true", help="setzt einen angehaltenen Lauf fort")
    s.set_defaults(f=cmd_lauf)
    s = sub.add_parser("kosten", help="schätzt Aufrufe, Tokens und Kosten einer Konfiguration")
    s.add_argument("konfig")
    s.set_defaults(f=cmd_kosten)
    s = sub.add_parser("pruefen", help="prüft Konfiguration, Engine, Modelle und Schlüssel, ohne ein Modell aufzurufen")
    s.add_argument("konfig")
    s.set_defaults(f=cmd_pruefen)
    s = sub.add_parser("zeigen", help="öffnet den Betrachter für einen Lauf im Browser, auch während er läuft")
    s.add_argument("verzeichnis")
    s.add_argument("--port", type=int, default=8198)
    s.add_argument("--ohne-browser", action="store_true", help="nur den Server starten")
    s.set_defaults(f=cmd_zeigen)
    s = sub.add_parser("etiketten", help="hängt den Entscheidungen eines Laufs ihre Folgen an")
    s.add_argument("verzeichnis")
    s.set_defaults(f=cmd_etiketten)
    args = p.parse_args()
    try:
        sys.exit(args.f(args))
    except (KonfigFehler, EngineFehler, FatalerFehler, FileExistsError, FileNotFoundError) as x:
        print(f"Fehler: {x}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
