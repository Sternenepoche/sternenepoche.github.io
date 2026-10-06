"""Der Lauf einer Epoche: fällige Rollen wecken, Antworten als Stapel holen, Aktionen anwenden.

Ablauf je Entscheidungsfenster:
1. Alle fälligen Rollen bekommen ihr Lagebild und antworten gleichzeitig. Die Weltuhr steht.
2. Wer Abfragen gestellt hat, bekommt die Ergebnisse und entscheidet danach.
3. Die Aktionen wirken in der von der Engine ausgelosten Reihenfolge.
4. Wer abgelehnte Aktionen hat, darf einmal korrigieren.
"""

from __future__ import annotations

import asyncio
import json
import shutil
import time
from pathlib import Path

from .backends import Antwort, Backend, BudgetErreicht, FatalerFehler, Kostenzaehler, erzeuge, json_aus_text
from .bruecke import Bruecke
from .konfig import ROLLEN, Konfig
from .lagebild import lagebild
from .prompt import systemtext, text_hash, verdacht
from .protokoll import TABELLEN, Ausgabe, auf_schnappschuss_kuerzen, lies_jsonl

TAG = 86_400


class Abbruch(RuntimeError):
    """Der Lauf wurde sauber angehalten und kann mit --fortsetzen weiterlaufen."""


class Lauf:
    def __init__(self, konfig: Konfig, fortsetzen: bool = False, leise: bool = False):
        self.k = konfig
        self.fortsetzen = fortsetzen
        self.leise = leise
        self.zaehler = Kostenzaehler(budget_usd=konfig.grenzen.budget_usd)
        benutzt = set(konfig.vergleich) if konfig.vergleich else set(konfig.rollen.values())
        self.backends: dict[str, Backend] = {n: erzeuge(konfig.anbieter[n], konfig.grenzen, self.zaehler) for n in sorted(benutzt)}
        self.dir = Path(konfig.lauf.ausgabe)
        # Ein neuer Lauf überschreibt Protokoll und Tabellen, ließe aber Schnappschüsse und gespeicherten Stand
        # eines früheren Laufs liegen; ein späteres --fortsetzen griffe dann auf fremde Stände zurück.
        if not fortsetzen and any((self.dir / n).exists() for n in ("protokoll.jsonl", "stand.bin", "schnappschuesse")):
            raise FileExistsError(
                f"In {self.dir} liegt schon ein Lauf. Mit --fortsetzen weiterlaufen lassen, "
                f"einen anderen Ausgabeordner wählen oder den alten Ordner selbst entfernen."
            )
        # Nach einem harten Abbruch (kein gespeicherter Stand) wird erst auf den letzten Schnappschuss gekürzt
        # und dann zum Anhängen geöffnet; vorher würde ein neues gzip-Glied hinter das abgerissene geschrieben.
        self.absturz = fortsetzen and not (self.dir / "stand.bin").exists()
        self.aus = None if self.absturz else Ausgabe(self.dir, fortsetzen)
        self.system: dict[str, str] = {}
        self.schema: dict[str, dict] = {}
        self.limits: dict[str, dict] = {}
        self.fehler_je_anbieter: dict[str, int] = {}

    def _sag(self, text: str) -> None:
        if not self.leise:
            print(text, flush=True)

    def _saat(self, zeit: int, spieler: int, rolle: str, runde: int) -> int:
        return (self.k.lauf.startwert * 1_000_003 + zeit * 131 + spieler * 17 + ROLLEN.index(rolle) * 5 + runde) % 2_147_483_647

    async def _frage(self, e: dict, runde: int, phase: str) -> Antwort:
        backend = self.backends[e["anbieter"]]
        meta = {"rolle": e["rolle"], "sicht": e["sicht"], "phase": phase}
        return await backend.frage(e["nachrichten"], self.schema[e["rolle"]], self._saat(e["zeit"], e["spieler"], e["rolle"], runde), meta)

    async def _stapel(self, eintraege: list[dict], runde: int, phase: str) -> list[Antwort]:
        """Fragt alle Einträge gleichzeitig. Fatale Fehler und die Budgetgrenze brechen den Stapel ab."""
        ergebnisse = await asyncio.gather(*(self._frage(e, runde, phase) for e in eintraege), return_exceptions=True)
        for r in ergebnisse:
            if isinstance(r, BaseException):
                raise r
        return list(ergebnisse)

    @staticmethod
    def _lies(antwort: Antwort) -> tuple[dict | None, str | None]:
        if antwort.fehler:
            return None, antwort.fehler
        try:
            return json_aus_text(antwort.text), None
        except ValueError as x:
            return None, f"Antwort nicht lesbar: {x}"

    @staticmethod
    def _runde(phase: str, a: Antwort) -> dict:
        d = a.als_dict()
        d["phase"] = phase
        d["antwort_roh"] = d.pop("text")
        return d

    async def _nachrunde(self, eintraege: list[dict], runde: int, phase: str) -> list[Antwort]:
        """Abfrage- und Korrekturrunde. Ein fataler Fehler hier beendet das Fenster noch ordentlich:
        die Betroffenen bleiben ohne Antwort, danach hält der Lauf an."""
        try:
            return await self._stapel(eintraege, runde, phase)
        except FatalerFehler as x:
            self._stopp = x
            return [Antwort(fehler=str(x)) for _ in eintraege]

    async def _fenster(self, engine: Bruecke, stand: dict) -> None:
        # Die Budgetgrenze greift nur zwischen zwei Fenstern, nie mitten in einem.
        self.zaehler.pruefe()
        zeit = stand["zeit"]
        eintraege: list[dict] = []
        for f in stand["faellig"]:
            rolle, spieler = f["rolle"], f["spieler"]
            sicht = engine.ruf("sicht", spieler=spieler, rolle=rolle)["sicht"]
            nutzer = lagebild(sicht, rolle)
            anbieter = self.k.anbieter_fuer(spieler, rolle)
            eintraege.append({
                "zeit": zeit, "zeittext": stand["zeittext"], "spieler": spieler, "name": f["name"], "rolle": rolle,
                "gruende": f["gruende"], "anbieter": anbieter, "sicht": sicht, "nutzer": nutzer,
                "nachrichten": [{"role": "system", "content": self.system[rolle]}, {"role": "user", "content": nutzer}],
                "runden": [], "abfragen": [], "aktionen": [], "korrektur": [], "inhalt": None, "fehler": None,
            })

        # 1. Entscheidung, alle gleichzeitig.
        antworten = await self._stapel(eintraege, 0, "entscheidung")
        for e, a in zip(eintraege, antworten):
            e["runden"].append(self._runde("entscheidung", a))
            e["inhalt"], e["fehler"] = self._lies(a)
            e["letzte"] = a

        # 2. Abfragen: lesend, alle gegen denselben Zustand vor den Aktionen dieses Fensters.
        mit_abfragen = [e for e in eintraege if e["inhalt"] and isinstance(e["inhalt"].get("abfragen"), list) and e["inhalt"]["abfragen"]]
        for e in mit_abfragen:
            max_abfragen = self.limits[e["rolle"]]["abfragen"]
            for abfrage in [x for x in e["inhalt"]["abfragen"] if isinstance(x, dict)][:max_abfragen]:
                ergebnis = engine.ruf("werkzeug", spieler=e["spieler"], abfrage=abfrage)["ergebnis"]
                e["abfragen"].append({"abfrage": abfrage, "ergebnis": ergebnis})
            e["nachrichten"] = e["nachrichten"] + [
                {"role": "assistant", "content": e["letzte"].text},
                {"role": "user", "content": "Ergebnisse deiner Abfragen:\n"
                    + "\n".join(json.dumps(x, ensure_ascii=False) for x in e["abfragen"])
                    + "\n\nEntscheide jetzt. Weitere Abfragen sind in diesem Aufruf nicht möglich: abfragen muss [] sein."},
            ]
        if mit_abfragen:
            antworten = await self._nachrunde(mit_abfragen, 1, "abfragen")
            for e, a in zip(mit_abfragen, antworten):
                e["runden"].append(self._runde("abfragen", a))
                e["inhalt"], e["fehler"] = self._lies(a)
                e["letzte"] = a

        # 3. Aktionen anwenden, in der ausgelosten Reihenfolge der Engine.
        for e in eintraege:
            if not e["inhalt"]:
                continue
            aktionen = e["inhalt"].get("aktionen") or []
            if not isinstance(aktionen, list):
                aktionen = []
            aktionen = [a for a in aktionen if isinstance(a, dict)]
            if aktionen:
                ergebnisse = engine.ruf("handeln", spieler=e["spieler"], rolle=e["rolle"], aktionen=aktionen)["ergebnisse"]
                e["aktionen"] = [{"aktion": a, **r} for a, r in zip(aktionen, ergebnisse)]

        # 4. Eine Korrektur für abgelehnte Aktionen.
        mit_fehlern = [e for e in eintraege if any(not a["ok"] for a in e["aktionen"])]
        for e in mit_fehlern:
            abgelehnt = [a for a in e["aktionen"] if not a["ok"]]
            e["nachrichten"] = e["nachrichten"] + [
                {"role": "assistant", "content": e["letzte"].text},
                {"role": "user", "content": "Diese Aktionen wurden abgelehnt:\n"
                    + "\n".join(f"- {json.dumps(a['aktion'], ensure_ascii=False)}: {a['text']}" for a in abgelehnt)
                    + "\n\nDie übrigen Aktionen sind ausgeführt. Du darfst einmal korrigieren: sende in aktionen nur die "
                      "korrigierten Aktionen oder [], abfragen muss [] sein."},
            ]
        if mit_fehlern and self._stopp is None:
            antworten = await self._nachrunde(mit_fehlern, 2, "korrektur")
            for e, a in zip(mit_fehlern, antworten):
                e["runden"].append(self._runde("korrektur", a))
                inhalt, _ = self._lies(a)
                if not inhalt:
                    continue
                # Notizbuch, Wecker und Prognose gelten in der Fassung der Korrektur.
                for feld in ("notiz", "wecker_stunden", "prognose"):
                    if feld in inhalt:
                        e["inhalt"][feld] = inhalt[feld]
                aktionen = [a for a in (inhalt.get("aktionen") or []) if isinstance(a, dict)]
                if aktionen:
                    ergebnisse = engine.ruf("handeln", spieler=e["spieler"], rolle=e["rolle"], aktionen=aktionen)["ergebnisse"]
                    e["korrektur"] = [{"aktion": x, **r} for x, r in zip(aktionen, ergebnisse)]

        # Aufruf abschließen und Datensatz schreiben.
        for e in eintraege:
            inhalt = e["inhalt"] or {}
            notiz = inhalt.get("notiz") if isinstance(inhalt.get("notiz"), str) else None
            wecker = inhalt.get("wecker_stunden")
            wecker_s = int(wecker * 3600) if isinstance(wecker, (int, float)) and wecker > 0 else None
            if e["korrektur"]:
                offen = [a for a in e["korrektur"] if not a["ok"]]
            else:
                offen = [a for a in e["aktionen"] if not a["ok"]]
            hinweise = [f"Aktion {a['aktion'].get('typ', '?')} abgelehnt: {a['text']}" for a in offen]
            engine.ruf("aufruf_ende", spieler=e["spieler"], rolle=e["rolle"], notiz=notiz, wecker_sekunden=wecker_s, hinweise=hinweise)
            if e["fehler"]:
                self.fehler_je_anbieter[e["anbieter"]] = self.fehler_je_anbieter.get(e["anbieter"], 0) + 1
            letzte = e["letzte"]
            self.aus.entscheidung({
                "zeit": e["zeit"], "zeittext": e["zeittext"], "spieler": e["spieler"], "name": e["name"], "rolle": e["rolle"],
                "gruende": e["gruende"], "anbieter": e["anbieter"], "modell": letzte.modell, "upstream": letzte.upstream,
                "system_hash": text_hash(self.system[e["rolle"]]), "nutzer": e["nutzer"],
                "runden": e["runden"], "abfragen": e["abfragen"], "aktionen": e["aktionen"], "korrektur": e["korrektur"],
                "begruendung": inhalt.get("begruendung"), "prognose": inhalt.get("prognose"), "notiz": notiz,
                "wecker_stunden": wecker if wecker_s else None, "fehler": e["fehler"],
                "verdacht": verdacht([inhalt.get("begruendung"), notiz] + [r.get("denken") for r in e["runden"]]),
                "punkte": e["sicht"]["punkte"]["gesamt"], "rang": e["sicht"]["rang"], "stufe": e["sicht"]["stufe"],
            })

    def _tabellen(self, engine: Bruecke) -> None:
        for name in TABELLEN:
            zeilen = engine.ruf("tabelle", name=name, ab=self.aus.zeiger[name])["zeilen"]
            self.aus.tabelle(name, zeilen)

    def _nach_absturz(self, engine: Bruecke) -> dict:
        """Kein sauber gespeicherter Stand, weil der Prozess hart beendet wurde: beim letzten Schnappschuss
        weitermachen und alle Ausgaben auf ihn kürzen, damit Protokoll, Tabellen und Entscheidungen genau zum
        Weltzustand passen. Verloren ist höchstens die Zeit seit dem Schnappschuss."""
        bilder = sorted((self.dir / "schnappschuesse").glob("tag-*.bin"))
        if not bilder:
            raise Abbruch(f"kein gespeicherter Stand und kein Schnappschuss in {self.dir}")
        stand = engine.ruf("laden", pfad=str(bilder[-1]))
        zeilen = {n: len(engine.ruf("tabelle", name=n, ab=0)["zeilen"]) for n in TABELLEN}
        log_anzahl = engine.ruf("hash")["log_anzahl"]
        behalten = auf_schnappschuss_kuerzen(self.dir, stand["zeit"], zeilen, log_anzahl)
        self.aus = Ausgabe(self.dir, fortsetzen=True)
        self._sag(f"Fortsetzen nach Abbruch ab {bilder[-1].name} ({stand['zeittext']}), {behalten} Entscheidungen behalten")
        # Ein Schnappschuss entsteht nach dem Fenster zu seiner Zeit: weiter mit dem nächsten Schritt, sonst
        # liefe dieses Fenster ein zweites Mal.
        if stand["ende"]:
            stand["faellig"] = []
            return stand
        # weiter liefert nur den neuen Zeitpunkt; die Kopfdaten des Laufs stammen aus dem geladenen Stand.
        return {**stand, **engine.ruf("weiter")}

    def _kosten_uebernehmen(self) -> None:
        """Die Budgetgrenze gilt für den ganzen Lauf: bisherige Kosten aus den Entscheidungen übernehmen."""
        for e in lies_jsonl(self.aus.dir / "entscheidungen.jsonl.gz"):
            for r in e["runden"]:
                a = Antwort(ein_tokens=r.get("ein_tokens") or 0, aus_tokens=r.get("aus_tokens") or 0,
                            kosten_usd=r.get("kosten_usd") or 0.0, fehler=r.get("fehler"))
                self.zaehler.buche(e["anbieter"], a)

    async def fahre(self) -> dict:
        l = self.k.lauf
        stand_datei = self.dir / "stand.bin"
        beginn = time.monotonic()
        self._stopp: FatalerFehler | None = None
        with Bruecke(l.engine) as engine:
            if self.fortsetzen:
                if self.absturz:
                    stand = self._nach_absturz(engine)
                else:
                    stand = engine.ruf("laden", pfad=str(stand_datei))
                    # Verbraucht: ein späterer Absturz darf nicht auf diesen älteren Stand zurückfallen, während die
                    # Ausgaben schon weiter sind; dann gilt der letzte Schnappschuss.
                    stand_datei.unlink()
                self._kosten_uebernehmen()
            else:
                felder = {"startwert": l.startwert, "spieler": l.spieler, "tage": l.tage, "bottypen": l.bottypen}
                if l.ki is not None:
                    felder["ki"] = l.ki
                if l.regeln:
                    felder["regeln"] = l.regeln
                stand = engine.ruf("neu", **felder)
                shutil.copyfile(self.k.pfad, self.aus.dir / "konfig.toml")
                # Das Regelwerk, mit dem die Engine rechnet, gehört zum Lauf: `replay` nimmt es von hier,
                # auch wenn regeln/regelwerk.ron sich später ändert. Bytegenau, der Hash hängt daran.
                if "regelwerk" in stand:
                    (self.aus.dir / "regelwerk.ron").write_text(stand.pop("regelwerk"), encoding="utf-8", newline="")
            for rolle in ROLLEN:
                r = engine.ruf("regeltext", rolle=rolle)
                self.limits[rolle] = r["limits"]
                self.schema[rolle] = r["schema"]
                self.system[rolle] = systemtext(rolle, r["text"], stand["tage"], r["limits"])
            # Beim Fortsetzen bleiben frühere Systemtexte erhalten: ältere Entscheidungen verweisen auf ihren Hash.
            prompts = self.aus.lies("prompts.json") if self.fortsetzen else {}
            prompts.update({text_hash(t): {"rolle": r, "text": t} for r, t in self.system.items()})
            self.aus.schreibe("prompts.json", prompts)
            kopf = {"startwert": stand["startwert"], "spieler": len(stand["spieler"]), "ki": stand["ki"], "tage": stand["tage"],
                    "regel_version": stand["regel_version"], "regel_hash": stand["regel_hash"], "bots": stand["bots"],
                    "anbieter": {n: {"art": b.k.art, "modell": b.k.modell, "provider": b.k.provider} for n, b in self.backends.items()},
                    "rollen": self.k.rollen, "vergleich": self.k.vergleich}
            self._sag(f"Sternenepoche: {kopf['spieler']} Spieler, davon {len(kopf['ki'])} mit Modellen, {kopf['tage']} Spieltage, "
                      f"Regelwerk {kopf['regel_version']} ({kopf['regel_hash'][:12]})")
            letzter_tag = stand["zeit"] // TAG
            grund = None
            try:
                while True:
                    if stand["faellig"]:
                        await self._fenster(engine, stand)
                    self.aus.protokoll(engine.ruf("log")["eintraege"])
                    if self._stopp is not None:
                        raise self._stopp
                    tag = stand["zeit"] // TAG
                    if tag != letzter_tag:
                        letzter_tag = tag
                        self._tabellen(engine)
                        if l.schnappschuss_tage > 0 and tag % l.schnappschuss_tage == 0:
                            engine.ruf("speichern", pfad=str(self.aus.dir / "schnappschuesse" / f"tag-{tag:04d}.bin"))
                        self.aus.sichern()
                        erster = engine.ruf("rangliste")["rangliste"][0]
                        self._sag(f"Tag {tag:>3}: {self.aus.entscheidungen} Entscheidungen, {self.zaehler.summe_usd:.2f} USD, "
                                  f"vorn {erster[1]} mit {erster[2]} Punkten")
                    if stand["ende"]:
                        break
                    stand = engine.ruf("weiter")
            except (BudgetErreicht, FatalerFehler, KeyboardInterrupt, asyncio.CancelledError) as x:
                # Sauber anhalten: der gespeicherte Stand enthält alles bis zum letzten vollständigen Fenster.
                grund = f"{type(x).__name__}: {x}" if str(x) else type(x).__name__
                self.aus.protokoll(engine.ruf("log")["eintraege"])
                self._tabellen(engine)
                engine.ruf("speichern", pfad=str(stand_datei))
            self._tabellen(engine)
            pruefsumme = engine.ruf("hash")
            schluss = {**kopf, "hash": pruefsumme["hash"], "log_hash": pruefsumme["log_hash"], "aktionen": pruefsumme["log_anzahl"],
                       "zeit": stand["zeit"], "beendet": grund is None, "angehalten": grund,
                       "rangliste": engine.ruf("rangliste")["rangliste"], "statistik": engine.ruf("statistik")["spieler"],
                       "welt": engine.ruf("welt"),
                       "kosten": {"usd": round(self.zaehler.summe_usd, 4), "aufrufe": self.zaehler.aufrufe, "ein_tokens": self.zaehler.ein_tokens,
                                  "aus_tokens": self.zaehler.aus_tokens, "je_anbieter": self.zaehler.je_anbieter},
                       "entscheidungen": self.aus.entscheidungen, "ohne_antwort": self.fehler_je_anbieter,
                       "laufzeit_s": round(time.monotonic() - beginn, 1)}
            self.aus.schreibe("schluss.json", schluss)
        self.aus.schliessen()
        for b in self.backends.values():
            await b.schliessen()
        if grund:
            raise Abbruch(f"Lauf angehalten ({grund}). Fortsetzen mit --fortsetzen.")
        self._sag(f"Fertig: {self.aus.entscheidungen} Entscheidungen, {self.zaehler.aufrufe} Modellaufrufe, {self.zaehler.summe_usd:.2f} USD, "
                  f"Zustandshash {schluss['hash'][:16]}. Daten in {self.aus.dir}")
        return schluss
