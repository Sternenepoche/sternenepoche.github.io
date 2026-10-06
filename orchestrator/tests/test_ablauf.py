"""Tests des Orchestrators. Ohne Modell und ohne echten Schlüssel:

- ein ganzer Lauf mit der Attrappe, danach Nachspielen aus dem Protokoll,
- Anhalten an der Budgetgrenze und Fortsetzen mit demselben Endzustand,
- die Anfrage an OpenRouter gegen einen nachgebauten Server im eigenen Prozess,
- die Prompts enthalten keine Wörter, die auf eine Messung hinweisen.

Aufruf aus dem Projektverzeichnis:  .venv/Scripts/python -m unittest discover -s orchestrator/tests -v
"""

from __future__ import annotations

import asyncio
import json
import os
import subprocess
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from sternenepoche.backends import Antwort, FatalerFehler, Kostenzaehler, OpenRouter, json_aus_text
from sternenepoche.bruecke import Bruecke
from sternenepoche.konfig import ROLLEN, Anbieter, Grenzen, laden
from sternenepoche.lagebild import lagebild
from sternenepoche.lauf import Abbruch, Lauf
from sternenepoche.prompt import systemtext, verbotene_woerter, verdacht
from sternenepoche.protokoll import etiketten, lies_jsonl

WURZEL = Path(__file__).resolve().parents[2]
ENGINE = WURZEL / "target" / "release" / "sternenepoche.exe"
if not ENGINE.exists():
    ENGINE = WURZEL / "target" / "release" / "sternenepoche"

PROBE = """
[lauf]
startwert = 11
spieler = 5
tage = 3
ki = [0, 1]
bottypen = ["oekonom", "raeuber", "igel"]
ausgabe = "{aus}"
engine = "{engine}"
schnappschuss_tage = 1

[grenzen]
{budget}

[anbieter.attrappe]
art = "mock"
preis_ein = 1.0
preis_aus = 1.0

[rollen]
stratege = "attrappe"
verwalter = "attrappe"
feldherr = "attrappe"
diplomat = "attrappe"
"""


def schreibe_konfig(tmp: Path, name: str, budget: str = "") -> Path:
    pfad = tmp / f"{name}.toml"
    text = PROBE.format(aus=(tmp / name).as_posix(), engine=ENGINE.as_posix(), budget=budget)
    pfad.write_text(text, encoding="utf-8")
    return pfad


class Ablauf(unittest.TestCase):
    def test_lauf_nachspielen_und_etiketten(self):
        with tempfile.TemporaryDirectory() as t:
            tmp = Path(t)
            schluss = asyncio.run(Lauf(laden(schreibe_konfig(tmp, "a")), leise=True).fahre())
            self.assertTrue(schluss["beendet"])
            entscheidungen = lies_jsonl(tmp / "a" / "entscheidungen.jsonl.gz")
            self.assertGreater(len(entscheidungen), 40)
            self.assertEqual({e["rolle"] for e in entscheidungen}, set(ROLLEN))
            phasen = {r["phase"] for e in entscheidungen for r in e["runden"]}
            self.assertEqual(phasen, {"entscheidung", "abfragen", "korrektur"})
            self.assertTrue(any(e["abfragen"] for e in entscheidungen))
            # Aus Startwert und Aktionsprotokoll entsteht derselbe Zustand.
            r = subprocess.run([str(ENGINE), "replay", str(tmp / "a")], capture_output=True, text=True, encoding="utf-8")
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertEqual(etiketten(tmp / "a"), len(entscheidungen))
            zeile = lies_jsonl(tmp / "a" / "etiketten.jsonl")[0]
            self.assertIn("delta_1", zeile)
            # Das Regelwerk liegt beim Lauf. Ein geändertes eingebautes Regelwerk stört das Nachspielen nicht:
            # replay nimmt das des Laufs, und dessen Hash steht in schluss.json.
            regelwerk = (tmp / "a" / "regelwerk.ron").read_bytes()
            self.assertIn(b"stufen", regelwerk)
            # Ein zweiter neuer Lauf in denselben Ordner wird abgelehnt statt alte Stände liegen zu lassen.
            with self.assertRaises(FileExistsError):
                Lauf(laden(schreibe_konfig(tmp, "a")), leise=True)

    def test_budgetgrenze_haelt_an_und_fortsetzen_trifft_denselben_zustand(self):
        with tempfile.TemporaryDirectory() as t:
            tmp = Path(t)
            ganz = asyncio.run(Lauf(laden(schreibe_konfig(tmp, "ganz")), leise=True).fahre())
            self.assertGreater(ganz["kosten"]["usd"], 0.05)
            teil = schreibe_konfig(tmp, "teil", budget=f"budget_usd = {ganz['kosten']['usd'] / 3}")
            with self.assertRaises(Abbruch):
                asyncio.run(Lauf(laden(teil), leise=True).fahre())
            angehalten = json.loads((tmp / "teil" / "schluss.json").read_text(encoding="utf-8"))
            self.assertFalse(angehalten["beendet"])
            self.assertLess(angehalten["zeit"], ganz["zeit"])
            # Auch ein angehaltener Lauf lässt sich nachspielen: bis zu seiner letzten Zeit, nicht bis zum Epochenende.
            r = subprocess.run([str(ENGINE), "replay", str(tmp / "teil")], capture_output=True, text=True, encoding="utf-8")
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            # Budget aufheben und fortsetzen: der Endzustand gleicht dem des ununterbrochenen Laufs.
            schreibe_konfig(tmp, "teil")
            fertig = asyncio.run(Lauf(laden(teil), fortsetzen=True, leise=True).fahre())
            self.assertTrue(fertig["beendet"])
            self.assertEqual(fertig["hash"], ganz["hash"])
            r = subprocess.run([str(ENGINE), "replay", str(tmp / "teil")], capture_output=True, text=True, encoding="utf-8")
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


    def test_fortsetzen_nach_hartem_abbruch_trifft_denselben_zustand(self):
        with tempfile.TemporaryDirectory() as t:
            tmp = Path(t)
            ganz = asyncio.run(Lauf(laden(schreibe_konfig(tmp, "ganz")), leise=True).fahre())
            konfig = schreibe_konfig(tmp, "absturz")
            asyncio.run(Lauf(laden(konfig), leise=True).fahre())
            d = tmp / "absturz"
            # Harter Abbruch nach dem ersten Tag: kein gespeicherter Stand, kein Schluss, nur der erste
            # Schnappschuss; das gzip endet mitten im Glied, das Protokoll mit einer halben Zeile.
            (d / "schluss.json").unlink()
            bilder = sorted((d / "schnappschuesse").glob("tag-*.bin"))
            self.assertGreater(len(bilder), 1)
            for b in bilder[1:]:
                b.unlink()
            roh = (d / "entscheidungen.jsonl.gz").read_bytes()
            (d / "entscheidungen.jsonl.gz").write_bytes(roh[: len(roh) - 40])
            with open(d / "protokoll.jsonl", "a", encoding="utf-8") as f:
                f.write('{"zeit": 999999, "abgeris')
            fertig = asyncio.run(Lauf(laden(konfig), fortsetzen=True, leise=True).fahre())
            self.assertTrue(fertig["beendet"])
            self.assertEqual(fertig["hash"], ganz["hash"])
            entscheidungen = lies_jsonl(d / "entscheidungen.jsonl.gz")
            schluessel = [(e["zeit"], e["spieler"], e["rolle"]) for e in entscheidungen]
            self.assertEqual(len(schluessel), len(set(schluessel)), "keine Entscheidung doppelt")
            self.assertEqual(len(entscheidungen), len(lies_jsonl(tmp / "ganz" / "entscheidungen.jsonl.gz")))
            # Die Kosten des ganzen Laufs zählen, auch die vor dem Abbruch.
            self.assertAlmostEqual(fertig["kosten"]["usd"], ganz["kosten"]["usd"], places=6)
            r = subprocess.run([str(ENGINE), "replay", str(d)], capture_output=True, text=True, encoding="utf-8")
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


class Prompts(unittest.TestCase):
    def test_kein_hinweis_auf_messung_oder_modelle(self):
        with Bruecke(ENGINE) as engine:
            engine.ruf("neu", startwert=3, spieler=4, tage=2, ki=[0, 1])
            for rolle in ROLLEN:
                r = engine.ruf("regeltext", rolle=rolle)
                system = systemtext(rolle, r["text"], 365, r["limits"])
                self.assertEqual(verbotene_woerter(system), [], f"Systemtext {rolle}")
                sicht = engine.ruf("sicht", spieler=0, rolle=rolle)["sicht"]
                text = lagebild(sicht, rolle)
                self.assertEqual(verbotene_woerter(text), [], f"Lagebild {rolle}")
                self.assertIn("Dein Notizbuch", text)
                # Das Antwortschema erzwingt jede Aktion der Rolle und nichts darüber hinaus.
                typen = {a["properties"]["typ"]["enum"][0] for a in r["schema"]["properties"]["aktionen"]["items"]["anyOf"]}
                self.assertEqual(typen, set(r["aktionstypen"]))

    def test_lagebild_kampf_raketen_verbaende_und_zivile_schiffe(self):
        """Felder wie in kern::sicht (der Kerntest angriff_pluendert_nach_beutequote_und_bricht_den_pakt
        prüft dieselben Schlüssel). Vorher fehlten Kampfberichte, Raketen und Verbände im Lagebild, und
        der Verwalter sah Verteidigungsanlagen, die er nicht fertigen darf."""
        with Bruecke(ENGINE) as engine:
            engine.ruf("neu", startwert=3, spieler=4, tage=2, ki=[0, 1])
            sicht = engine.ruf("sicht", spieler=0, rolle="feldherr")["sicht"]
        sicht["kampfberichte"] = [{
            "zeit": "Tag 3, 04:15", "ort": "1:7:6", "mission": "angriff", "seite": "verteidigung",
            "angreifer": ["Rabor", "Kessa"], "verteidiger": [sicht["name"]], "sieger": "verteidiger", "runden": 4,
            "verluste_angreifer": {"leichter_jaeger": 12}, "verluste_verteidiger": {"raketenwerfer": 3},
            "beute": {}, "truemmer": {"erz": 3600, "kristall": 1200},
        }]
        sicht["raketensalven"] = [{"von": "Rabor", "ziel": "1:7:6", "anzahl": 5, "zieltyp": "raketenwerfer", "ankunft": "Tag 3, 05:00"}]
        sicht["verbaende"] = [{"fuehrung": 17, "spieler": "Kessa", "ziel": "1:9:4", "ankunft": "Tag 3, 09:30"}]
        sicht["planeten"][0]["raketen"] = {"abfang": 4, "interplanetar": 2, "belegt_mit_bau": 8, "kapazitaet": 10}
        text = lagebild(sicht, "feldherr")
        self.assertIn("## Kampfberichte", text)
        self.assertIn("Rabor, Kessa gegen", text)
        self.assertIn("Sieg des Verteidigers nach 4 Runden (du warst Verteidiger)", text)
        self.assertIn("Trümmer 3.600 Erz", text)
        self.assertIn("5 Raketen von Rabor auf 1:7:6", text)
        self.assertIn("Führung Flotte 17 von Kessa", text)
        self.assertIn("Raketensilo: 4 Abfang-, 2 Interplanetarraketen, 8 von 10 Plätzen", text)
        self.assertNotIn("## Kampfberichte", lagebild(sicht, "diplomat"))
        einheit = lambda name, schiff: {"einheit": name, "schiff": schiff, "kosten": {"erz": 100}, "werft": 1, "braucht": []}
        sicht["einheiten_kosten"] = [einheit("kleiner_transporter", True), einheit("recycler", True), einheit("kreuzer", True),
                                     einheit("bergbauschiff", True), einheit("raketenwerfer", False)]
        verwalter = lagebild(sicht, "verwalter")
        einheiten = [z[2:].split(":")[0] for z in verwalter.split("## Einheiten")[1].split("\n## ")[0].splitlines() if z.startswith("- ")]
        self.assertEqual(einheiten, ["kleiner_transporter", "recycler", "bergbauschiff"])
        self.assertIn("kreuzer", lagebild(sicht, "feldherr"))

    def test_verdacht_und_json(self):
        self.assertTrue(verdacht(["Vermutlich ist das hier ein Benchmark."]))
        self.assertFalse(verdacht(["Kristall ist der Engpass.", None]))
        self.assertEqual(json_aus_text('```json\n{"a": 1}\n```'), {"a": 1})
        with self.assertRaises(ValueError):
            json_aus_text("keine Antwort")


class NachgebauterServer(BaseHTTPRequestHandler):
    """Verhält sich wie /chat/completions von OpenRouter, soweit der Orchestrator es nutzt."""

    anfragen: list[dict] = []
    plan: list[int] = []

    def log_message(self, *_):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        type(self).anfragen.append({"pfad": self.path, "auth": self.headers.get("Authorization"), "body": body,
                                    "titel": self.headers.get("X-OpenRouter-Title")})
        status = type(self).plan.pop(0) if type(self).plan else 200
        # "leer": das Denken hat das Tokenlimit verbraucht; "abgeschnitten": die Antwort endet mitten im JSON.
        if status in ("leer", "abgeschnitten"):
            inhalt = "" if status == "leer" else '{"begruendung": "ok", "aktio'
            antwort = {"id": "gen-0", "model": body["model"], "provider": "Beispielprovider",
                       "choices": [{"finish_reason": "length", "message": {"role": "assistant", "content": inhalt}}],
                       "usage": {"prompt_tokens": 1000, "completion_tokens": body["max_tokens"], "cost": 0.0001,
                                 "completion_tokens_details": {"reasoning_tokens": body["max_tokens"] if status == "leer" else 100}}}
            status = 200
        elif status == 200:
            antwort = {"id": "gen-1", "model": body["model"], "provider": "Beispielprovider",
                       "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json.dumps(
                           {"begruendung": "ok", "abfragen": [], "aktionen": [], "prognose": "", "notiz": "n", "wecker_stunden": None})}}],
                       "usage": {"prompt_tokens": 1200, "completion_tokens": 80, "cost": 0.00042,
                                 "prompt_tokens_details": {"cached_tokens": 900}, "completion_tokens_details": {"reasoning_tokens": 30}}}
        elif status == 402:
            antwort = {"error": {"code": 402, "message": "Insufficient credits", "metadata": {"limit_source": "openrouter_credits"}}}
        else:
            antwort = {"error": {"code": status, "message": "Rate limited"}}
        daten = json.dumps(antwort).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(daten)))
        if status == 429:
            self.send_header("Retry-After", "0")
        self.end_headers()
        self.wfile.write(daten)


class OpenRouterAnfrage(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), NachgebauterServer)
        threading.Thread(target=cls.server.serve_forever, daemon=True).start()
        cls.url = f"http://127.0.0.1:{cls.server.server_port}/api/v1"
        # Kein echter Schlüssel: der nachgebaute Server prüft nur, dass der Kopf ankommt.
        os.environ["STERNENEPOCHE_TESTSCHLUESSEL"] = "nur-fuer-den-test"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()

    def backend(self, **felder) -> tuple[OpenRouter, Kostenzaehler]:
        zaehler = Kostenzaehler()
        k = Anbieter(name="t", art="openrouter", modell="meta-llama/llama-3.3-70b-instruct", basis_url=self.url,
                     schluessel_env="STERNENEPOCHE_TESTSCHLUESSEL", **felder)
        return OpenRouter(k, Grenzen(versuche=3, max_ausgabe_tokens=900, denk_tokens=500), zaehler), zaehler

    def frage(self, b: OpenRouter) -> Antwort:
        async def lauf():
            try:
                return await b.frage([{"role": "system", "content": "s"}, {"role": "user", "content": "u"}],
                                     {"type": "object", "properties": {}}, 5, {})
            finally:
                await b.schliessen()
        return asyncio.run(lauf())

    def test_anfrage_schema_provider_denkbudget_und_kosten(self):
        NachgebauterServer.anfragen.clear()
        NachgebauterServer.plan[:] = [429, 200]
        b, zaehler = self.backend(denken="budget", provider={"order": ["Beispielprovider"], "allow_fallbacks": False, "data_collection": "deny"})
        a = self.frage(b)
        self.assertIsNone(a.fehler)
        self.assertEqual(a.versuche, 2)  # der erste Versuch lief ins Ratenlimit
        self.assertEqual(a.upstream, "Beispielprovider")
        self.assertEqual((a.ein_tokens, a.aus_tokens, a.denk_tokens, a.cache_tokens), (1200, 80, 30, 900))
        self.assertAlmostEqual(zaehler.summe_usd, 0.00042)
        anfrage = NachgebauterServer.anfragen[-1]
        self.assertEqual(anfrage["pfad"], "/api/v1/chat/completions")
        self.assertEqual(anfrage["auth"], "Bearer nur-fuer-den-test")
        body = anfrage["body"]
        self.assertEqual(body["response_format"]["type"], "json_schema")
        self.assertTrue(body["response_format"]["json_schema"]["strict"])
        self.assertEqual(body["provider"], {"order": ["Beispielprovider"], "allow_fallbacks": False, "data_collection": "deny", "require_parameters": True})
        self.assertEqual(body["reasoning"], {"max_tokens": 500})
        self.assertEqual(body["max_tokens"], 1400)
        self.assertEqual(body["seed"], 5)

    def test_leere_antwort_naechster_versuch_ohne_denken_mit_mehr_platz(self):
        NachgebauterServer.anfragen.clear()
        NachgebauterServer.plan[:] = ["leer", 200]
        b, zaehler = self.backend(denken="budget")
        a = self.frage(b)
        self.assertIsNone(a.fehler)
        self.assertEqual(a.versuche, 2)
        erste, zweite = (x["body"] for x in NachgebauterServer.anfragen[-2:])
        self.assertEqual((erste["reasoning"], erste["max_tokens"]), ({"max_tokens": 500}, 1400))
        self.assertEqual((zweite["reasoning"], zweite["max_tokens"]), ({"effort": "none"}, 2800))
        # Der gescheiterte Versuch ist bezahlt: er gehört zur Antwort und wird genau einmal gebucht.
        self.assertAlmostEqual(a.kosten_usd, 0.0001 + 0.00042)
        self.assertAlmostEqual(zaehler.summe_usd, 0.0001 + 0.00042)
        self.assertEqual(a.ein_tokens, 1000 + 1200)

    def test_abgeschnittene_antwort_mit_niedrigster_denkstufe(self):
        NachgebauterServer.anfragen.clear()
        NachgebauterServer.plan[:] = ["abgeschnitten", 200]
        b, _ = self.backend(denken="mittel")
        a = self.frage(b)
        self.assertIsNone(a.fehler)
        erste, zweite = (x["body"] for x in NachgebauterServer.anfragen[-2:])
        self.assertEqual((erste["reasoning"], erste["max_tokens"]), ({"effort": "medium"}, 1400))
        self.assertEqual((zweite["reasoning"], zweite["max_tokens"]), ({"effort": "low"}, 2800))

    def test_endgueltiger_fehlschlag_traegt_seine_kosten(self):
        NachgebauterServer.anfragen.clear()
        NachgebauterServer.plan[:] = ["leer", "leer", "leer"]
        b, zaehler = self.backend(denken="budget")
        a = self.frage(b)
        self.assertIn("leere Antwort", a.fehler)
        self.assertAlmostEqual(a.kosten_usd, 0.0003)
        self.assertAlmostEqual(zaehler.summe_usd, 0.0003)
        # Mehr Platz, aber höchstens das Vierfache von Antwort plus Denkbudget.
        self.assertEqual([x["body"]["max_tokens"] for x in NachgebauterServer.anfragen[-3:]], [1400, 2800, 5600])

    def test_kein_guthaben_ist_fatal(self):
        NachgebauterServer.plan[:] = [402]
        b, _ = self.backend()
        with self.assertRaises(FatalerFehler) as x:
            self.frage(b)
        self.assertIn("openrouter_credits", str(x.exception))

    def test_ohne_schluessel_keine_anfrage(self):
        NachgebauterServer.anfragen.clear()
        b, _ = self.backend()
        b.k.schluessel_env = "STERNENEPOCHE_GIBT_ES_NICHT"
        with self.assertRaises(FatalerFehler):
            self.frage(b)
        self.assertEqual(NachgebauterServer.anfragen, [])

    def test_dauerhaftes_ratenlimit_wird_zur_fehlenden_antwort(self):
        NachgebauterServer.plan[:] = [429, 429, 429]
        b, zaehler = self.backend()
        a = self.frage(b)
        self.assertIn("aufgegeben", a.fehler)
        self.assertEqual(zaehler.je_anbieter["t"]["fehler"], 1)


class Betrachter(unittest.TestCase):
    """`sternenepoche zeigen`: der Server liefert einen fertigen Lauf, und einen laufenden aus dem neuesten Schnappschuss."""

    @staticmethod
    def _hole(port: int, pfad: str):
        import urllib.error
        import urllib.request

        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{port}{pfad}", timeout=30) as r:
                return r.status, r.read().decode("utf-8")
        except urllib.error.HTTPError as x:
            return x.code, ""

    def test_fertiger_und_laufender_lauf(self):
        import shutil

        from sternenepoche.zeigen import Lage, handler

        with tempfile.TemporaryDirectory() as t:
            tmp = Path(t)
            schluss = asyncio.run(Lauf(laden(schreibe_konfig(tmp, "a")), leise=True).fahre())
            # Derselbe Lauf, als liefe er noch: ohne schluss.json, mit Schnappschüssen.
            shutil.copytree(tmp / "a", tmp / "live")
            (tmp / "live" / "schluss.json").unlink()
            for verzeichnis, live in ((tmp / "a", False), (tmp / "live", True)):
                server = ThreadingHTTPServer(("127.0.0.1", 0), handler(Lage(verzeichnis, ENGINE)))
                threading.Thread(target=server.serve_forever, daemon=True).start()
                port = server.server_address[1]
                try:
                    status, text = self._hole(port, "/")
                    self.assertEqual(status, 200)
                    self.assertIn("Entscheidungen der Modelle", text)
                    stand = json.loads(self._hole(port, "/api/stand")[1])
                    self.assertEqual(stand["live"], live)
                    s = json.loads(self._hole(port, "/lauf/schluss.json")[1])
                    self.assertEqual(s.get("live", False), live)
                    self.assertEqual(len(s["welt"]["spieler"]), 5)
                    self.assertEqual(s["regel_hash"], schluss["regel_hash"])
                    if live:
                        # Zwischenstand aus dem letzten Tagesschnappschuss, Kosten aus den Entscheidungen.
                        self.assertTrue(s["schnappschuss"].startswith("tag-"))
                        self.assertGreater(s["kosten"]["aufrufe"], 0)
                    entscheidungen = json.loads(self._hole(port, "/api/entscheidungen?ab=0")[1])
                    self.assertEqual(len(entscheidungen), stand["entscheidungen"])
                    self.assertEqual(len(json.loads(self._hole(port, "/api/entscheidungen?ab=5")[1])), len(entscheidungen) - 5)
                    e = next(x for x in entscheidungen if x["aktionen"])
                    self.assertTrue({"typ", "ok", "text", "befehl", "korrektur"} <= set(e["aktionen"][0]))
                    self.assertIn("\n", self._hole(port, "/lauf/tageswerte.jsonl")[1])
                    # Nur der Betrachter und die freigegebenen Dateien sind erreichbar.
                    for pfad in ("/lauf/konfig.toml", "/lauf/../schluss.json", "/lauf/schnappschuesse/tag-0001.bin", "/lauf/entscheidungen.jsonl.gz"):
                        self.assertEqual(self._hole(port, pfad)[0], 404, pfad)
                finally:
                    server.shutdown()
                    server.server_close()


if __name__ == "__main__":
    unittest.main()
