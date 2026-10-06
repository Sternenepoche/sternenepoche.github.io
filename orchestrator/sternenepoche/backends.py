"""Modellanbindung: OpenRouter, lokale Server mit OpenAI-kompatibler Schnittstelle, Attrappe.

Alle drei beantworten dieselbe Frage: Nachrichten und Antwortschema hinein, Text heraus.
OpenRouter und lokale Server (vLLM, llama.cpp, Ollama) sprechen dasselbe Protokoll, bei
OpenRouter kommen Provider-Routing, Denkbudget, Kostenzählung und Budgetgrenze dazu.
"""

from __future__ import annotations

import asyncio
import json
import os
import random
import time
from dataclasses import dataclass, field

import httpx

from .konfig import AUFWAND, Anbieter, Grenzen


class FatalerFehler(RuntimeError):
    """Ein Fehler, den Wiederholen nicht behebt: falscher Schlüssel, kein Guthaben, unbekanntes Modell."""


class BudgetErreicht(RuntimeError):
    pass


@dataclass
class Antwort:
    text: str = ""
    denken: str | None = None
    ein_tokens: int = 0
    aus_tokens: int = 0
    denk_tokens: int = 0
    cache_tokens: int = 0
    kosten_usd: float = 0.0
    latenz_s: float = 0.0
    versuche: int = 1
    modell: str = ""
    upstream: str | None = None
    ende: str | None = None
    fehler: str | None = None

    def als_dict(self) -> dict:
        return {k: v for k, v in self.__dict__.items()}


@dataclass
class Kostenzaehler:
    """Zählt die Kosten aller Anbieter eines Laufs und hält die Budgetgrenze."""

    budget_usd: float | None = None
    summe_usd: float = 0.0
    aufrufe: int = 0
    ein_tokens: int = 0
    aus_tokens: int = 0
    je_anbieter: dict = field(default_factory=dict)

    def pruefe(self) -> None:
        if self.budget_usd is not None and self.summe_usd >= self.budget_usd:
            raise BudgetErreicht(f"Budget von {self.budget_usd:.2f} USD erreicht ({self.summe_usd:.2f} USD verbraucht)")

    def buche(self, anbieter: str, a: Antwort) -> None:
        self.summe_usd += a.kosten_usd
        self.aufrufe += 1
        self.ein_tokens += a.ein_tokens
        self.aus_tokens += a.aus_tokens
        e = self.je_anbieter.setdefault(anbieter, {"aufrufe": 0, "ein_tokens": 0, "aus_tokens": 0, "kosten_usd": 0.0, "fehler": 0})
        e["aufrufe"] += 1
        e["ein_tokens"] += a.ein_tokens
        e["aus_tokens"] += a.aus_tokens
        e["kosten_usd"] += a.kosten_usd
        if a.fehler:
            e["fehler"] += 1


def _addiere(ziel: Antwort, quelle: Antwort) -> None:
    """Zählt Tokens und Kosten eines gescheiterten Versuchs zur Antwort hinzu."""
    ziel.ein_tokens += quelle.ein_tokens
    ziel.aus_tokens += quelle.aus_tokens
    ziel.denk_tokens += quelle.denk_tokens
    ziel.cache_tokens += quelle.cache_tokens
    ziel.kosten_usd += quelle.kosten_usd


def _ist_json(text: str) -> bool:
    try:
        json.loads(text)
        return True
    except ValueError:
        return False


class Backend:
    def __init__(self, konfig: Anbieter, grenzen: Grenzen, zaehler: Kostenzaehler):
        self.k = konfig
        self.grenzen = grenzen
        self.zaehler = zaehler
        self._sperre = asyncio.Semaphore(max(1, konfig.parallel))

    async def frage(self, nachrichten: list[dict], schema: dict, saat: int, meta: dict) -> Antwort:
        """Beantwortet eine Anfrage. Scheitert sie endgültig, steht der Grund in `fehler`."""
        async with self._sperre:
            beginn = time.monotonic()
            antwort = await self._frage(nachrichten, schema, saat, meta)
            antwort.latenz_s = round(time.monotonic() - beginn, 3)
        antwort.modell = antwort.modell or self.k.modell
        self.zaehler.buche(self.k.name, antwort)
        return antwort

    async def _frage(self, nachrichten, schema, saat, meta) -> Antwort:
        raise NotImplementedError

    async def schliessen(self) -> None:
        pass


class OpenAIKompatibel(Backend):
    """Lokale Server: vLLM, llama.cpp, Ollama und alles andere mit /chat/completions."""

    def __init__(self, konfig: Anbieter, grenzen: Grenzen, zaehler: Kostenzaehler):
        super().__init__(konfig, grenzen, zaehler)
        self._client = httpx.AsyncClient(
            base_url=konfig.basis_url.rstrip("/"),
            timeout=httpx.Timeout(grenzen.zeitlimit_sekunden, connect=15.0),
            limits=httpx.Limits(max_connections=max(4, konfig.parallel * 2)),
        )

    def _kopfzeilen(self) -> dict:
        kopf = {"Content-Type": "application/json"}
        if self.k.schluessel_env:
            schluessel = os.environ.get(self.k.schluessel_env, "")
            if schluessel:
                kopf["Authorization"] = f"Bearer {schluessel}"
        return kopf

    def _anfrage(self, nachrichten: list[dict], schema: dict, saat: int) -> dict:
        body: dict = {
            "model": self.k.modell,
            "messages": nachrichten,
            "max_tokens": self.grenzen.max_ausgabe_tokens,
            "temperature": self.grenzen.temperatur,
            "seed": saat,
        }
        if self.k.schema == "json_schema":
            body["response_format"] = {"type": "json_schema", "json_schema": {"name": "antwort", "strict": True, "schema": schema}}
        elif self.k.schema == "guided_json":
            body["guided_json"] = schema
        elif self.k.schema == "json_object":
            body["response_format"] = {"type": "json_object"}
        body.update(self.k.zusatz)
        return body

    def _kosten(self, usage: dict, a: Antwort) -> float:
        return (a.ein_tokens * self.k.preis_ein + a.aus_tokens * self.k.preis_aus) / 1_000_000

    @staticmethod
    def _fehlertext(daten) -> tuple[int | None, str, dict]:
        fehler = daten.get("error") if isinstance(daten, dict) else None
        if isinstance(fehler, dict):
            code = fehler.get("code")
            return (code if isinstance(code, int) else None), str(fehler.get("message", fehler)), fehler.get("metadata") or {}
        if fehler:
            return None, str(fehler), {}
        return None, "", {}

    def _fatal(self, status: int, text: str, metadata: dict) -> FatalerFehler | None:
        if status in (401, 403):
            return FatalerFehler(f"{self.k.name}: Zugriff abgelehnt ({status}). Schlüssel in {self.k.schluessel_env or 'der Umgebung'} prüfen. {text}")
        if status in (400, 404, 422):
            return FatalerFehler(
                f"{self.k.name}: Anfrage für Modell '{self.k.modell}' abgelehnt ({status}): {text}. "
                "Häufige Ursache: das Modell oder sein Provider unterstützt das Antwortschema nicht; "
                "dann schema = \"json_object\" setzen oder ein anderes Modell wählen."
            )
        return None

    def _mehr_platz(self, body: dict) -> dict:
        """Anfrage für den nächsten Versuch, nachdem das Tokenlimit erreicht wurde: doppelter Platz, höchstens
        das Vierfache des Ausgangswerts. Derselbe Aufruf noch einmal endete fast immer genauso."""
        neu = dict(body)
        grenze = 4 * (self.grenzen.max_ausgabe_tokens + self.grenzen.denk_tokens)
        neu["max_tokens"] = min(2 * int(body.get("max_tokens") or self.grenzen.max_ausgabe_tokens), grenze)
        return neu

    async def _frage(self, nachrichten, schema, saat, meta) -> Antwort:
        body = self._anfrage(nachrichten, schema, saat)
        letzter = "unbekannt"
        # Was gescheiterte Versuche verbraucht haben, gehört zur Antwort: bezahlt ist es trotzdem.
        verbraucht = Antwort()
        for versuch in range(1, self.grenzen.versuche + 1):
            warte = min(30.0, 1.5 ** versuch) + random.random()
            try:
                r = await self._client.post("/chat/completions", json=body, headers=self._kopfzeilen())
            except (httpx.TimeoutException, httpx.TransportError) as e:
                letzter = f"Verbindung: {type(e).__name__}"
                await asyncio.sleep(warte)
                continue
            try:
                daten = r.json()
            except ValueError:
                daten = {}
            code, text, metadata = self._fehlertext(daten)
            status = code or r.status_code
            if status >= 400 or text:
                fatal = self._fatal(status, text or r.text[:300], metadata)
                if fatal:
                    raise fatal
                letzter = f"Status {status}: {text or r.text[:200]}"
                if r.headers.get("retry-after", "").isdigit():
                    warte = max(warte, min(60.0, float(r.headers["retry-after"])))
                await asyncio.sleep(warte)
                continue
            wahl = (daten.get("choices") or [{}])[0]
            if isinstance(wahl.get("error"), dict):
                letzter = f"Provider: {wahl['error'].get('message', wahl['error'])}"
                await asyncio.sleep(warte)
                continue
            nachricht = wahl.get("message") or {}
            usage = daten.get("usage") or {}
            a = Antwort(
                text=nachricht.get("content") or "",
                denken=nachricht.get("reasoning") or nachricht.get("reasoning_content"),
                ein_tokens=int(usage.get("prompt_tokens") or 0),
                aus_tokens=int(usage.get("completion_tokens") or 0),
                denk_tokens=int((usage.get("completion_tokens_details") or {}).get("reasoning_tokens") or usage.get("reasoning_tokens") or 0),
                cache_tokens=int((usage.get("prompt_tokens_details") or {}).get("cached_tokens") or usage.get("cached_tokens") or 0),
                versuche=versuch,
                modell=daten.get("model") or self.k.modell,
                upstream=daten.get("provider"),
                ende=wahl.get("finish_reason"),
            )
            a.kosten_usd = self._kosten(usage, a)
            leer = not a.text.strip()
            if leer or (a.ende == "length" and not _ist_json(a.text)):
                # Tokenlimit erreicht: meist hat das Denken alles verbraucht, oder die Antwort endet mitten im JSON.
                _addiere(verbraucht, a)
                letzter = f"{'leere' if leer else 'abgeschnittene'} Antwort (Ende: {a.ende})"
                if a.ende == "length":
                    body = self._mehr_platz(body)
                continue
            _addiere(a, verbraucht)
            return a
        fehlschlag = Antwort(fehler=f"nach {self.grenzen.versuche} Versuchen aufgegeben: {letzter}", versuche=self.grenzen.versuche)
        _addiere(fehlschlag, verbraucht)
        return fehlschlag

    async def schliessen(self) -> None:
        await self._client.aclose()


class OpenRouter(OpenAIKompatibel):
    """OpenRouter: ein Schlüssel, viele Modelle.

    Für eine Messung zählt, dass alle Agenten einer Rolle dieselben Gewichte bekommen.
    Dafür pinnt `provider` in der Konfiguration Anbieter und Quantisierung
    (order, allow_fallbacks, quantizations); der tatsächliche Anbieter jeder Antwort
    steht im Datensatz.
    """

    def _kopfzeilen(self) -> dict:
        schluessel = os.environ.get(self.k.schluessel_env, "")
        if not schluessel:
            raise FatalerFehler(
                f"{self.k.name}: Umgebungsvariable {self.k.schluessel_env} ist nicht gesetzt. "
                "Schlüssel unter https://openrouter.ai/keys anlegen und in der Umgebung setzen, nie in eine Datei schreiben."
            )
        kopf = {"Content-Type": "application/json", "Authorization": f"Bearer {schluessel}"}
        if self.k.referer:
            kopf["HTTP-Referer"] = self.k.referer
        if self.k.titel:
            kopf["X-OpenRouter-Title"] = self.k.titel
        return kopf

    def _anfrage(self, nachrichten: list[dict], schema: dict, saat: int) -> dict:
        body = super()._anfrage(nachrichten, schema, saat)
        provider = dict(self.k.provider)
        if self.k.schema == "json_schema":
            # Nur Endpunkte, die das Antwortschema wirklich erzwingen.
            provider.setdefault("require_parameters", True)
        if provider:
            body["provider"] = provider
        if self.k.denken == "budget":
            # Denktokens zählen als Ausgabe und gegen max_tokens: Budget hart begrenzen, Platz für die Antwort lassen.
            body["reasoning"] = {"max_tokens": self.grenzen.denk_tokens}
            body["max_tokens"] = self.grenzen.max_ausgabe_tokens + self.grenzen.denk_tokens
        elif self.k.denken == "aus":
            body["reasoning"] = {"effort": "none"}
        elif self.k.denken in AUFWAND:
            body["reasoning"] = {"effort": AUFWAND[self.k.denken]}
            body["max_tokens"] = self.grenzen.max_ausgabe_tokens + self.grenzen.denk_tokens
        return body

    def _mehr_platz(self, body: dict) -> dict:
        """Nach erreichtem Tokenlimit: doppelter Platz und weniger Denken. Ein Budget, das das Modell nicht
        einhält, wird zu "ohne Denken"; eine Denkstufe sinkt auf die niedrigste, die jedes Denkmodell kennt."""
        neu = super()._mehr_platz(body)
        if self.k.denken == "budget":
            neu["reasoning"] = {"effort": "none"}
        elif self.k.denken in AUFWAND:
            neu["reasoning"] = {"effort": "low"}
        return neu

    def _kosten(self, usage: dict, a: Antwort) -> float:
        kosten = usage.get("cost")
        return float(kosten) if isinstance(kosten, (int, float)) else 0.0

    def _fatal(self, status: int, text: str, metadata: dict) -> FatalerFehler | None:
        if status == 402:
            quelle = metadata.get("limit_source", "")
            return FatalerFehler(f"{self.k.name}: OpenRouter meldet kein Guthaben mehr (402 {quelle}). {text}")
        return super()._fatal(status, text, metadata)


class Mock(Backend):
    """Attrappe ohne Modell: gültige, einfache Antworten aus dem Lagebild. Für Tests des Ablaufs."""

    async def _frage(self, nachrichten, schema, saat, meta) -> Antwort:
        rolle = meta.get("rolle")
        sicht = meta.get("sicht") or {}
        phase = meta.get("phase", "entscheidung")
        aktionen: list[dict] = []
        abfragen: list[dict] = []
        notiz = sicht.get("notiz") or ""
        planeten = sicht.get("planeten") or []
        heimat = planeten[0] if planeten else None
        erster_aufruf = not notiz
        if phase in ("entscheidung", "abfragen") and heimat:
            k = heimat["koord"]
            if rolle == "stratege" and erster_aufruf:
                aktionen.append({"typ": "doktrin", "anteile": {"wirtschaft": 65, "militaer": 10, "forschung": 15, "reserve": 10},
                                 "text": "Erst Wirtschaft und Bevölkerung, dann Stufe II."})
            elif rolle == "verwalter":
                if erster_aufruf:
                    aktionen.append({"typ": "steuersatz", "prozent": 15})
                    # Einmal eine Abfrage und einmal eine ungültige Aktion, damit beide Wege geprüft werden.
                    abfragen.append({"typ": "kosten", "gebaeude": "solarkraftwerk", "forschung": None, "einheit": None, "stufe": None, "planet": None})
                stufe = sicht.get("naechste_stufe") or {}
                if stufe and all(b["erfuellt"] for b in stufe.get("bedingungen", [])) and stufe.get("erfuellt_seit_stunden", 0) >= stufe.get("haltezeit_stunden", 48):
                    aktionen.append({"typ": "stufenaufstieg"})
                if len(heimat["bauschleife"]) < 2:
                    e = heimat["energie"]
                    g = heimat["gebaeude"]
                    if e["erzeugung"] < e["verbrauch"] + 40:
                        was = "solarkraftwerk"
                    elif sicht.get("volk") != "syntheten" and heimat["rate"]["nahrung"] < heimat["bevoelkerung"] * 0.01:
                        was = "farm"
                    elif heimat["bevoelkerung"] > heimat["wohnraum"] * 0.55:
                        was = "wohnblock"
                    elif heimat["arbeit"]["bedarf"] > heimat["arbeit"]["verfuegbar"] * 0.93:
                        was = None
                    else:
                        was = "kristallmine" if g.get("kristallmine", 0) < g.get("erzmine", 0) else "erzmine"
                    if was:
                        aktionen.append({"typ": "bauen", "planet": k, "gebaeude": was})
                if "ungueltig geprueft" not in notiz and not erster_aufruf:
                    aktionen.append({"typ": "bauen", "planet": "9:9:9", "gebaeude": "erzmine"})
            elif rolle == "diplomat" and erster_aufruf:
                nachbarn = sicht.get("nachbarn") or []
                if nachbarn:
                    aktionen.append({"typ": "nachricht", "an": [nachbarn[0]["spieler"]], "allianz": False, "text": "Gruß von nebenan. Friedliche Nachbarschaft?"})
            for v in sicht.get("vertraege") or []:
                if rolle == "diplomat" and v["status"] == "angebot an dich":
                    aktionen.append({"typ": "vertrag_annehmen", "vertrag": v["vertrag"]})
        if phase == "abfragen":
            # Nach den Ergebnissen der Abfragen folgt die eigentliche Entscheidung.
            abfragen = []
        if phase == "korrektur" and "ungueltig geprueft" not in notiz:
            notiz += " ungueltig geprueft"
        inhalt = {
            "begruendung": f"Attrappe, Rolle {rolle}, Phase {phase}.",
            "abfragen": abfragen,
            "aktionen": [] if phase == "korrektur" else aktionen,
            "prognose": "Die Bauschleife bleibt gefüllt.",
            "notiz": (notiz or "begonnen").strip(),
            "wecker_stunden": None,
        }
        text = json.dumps(inhalt, ensure_ascii=False)
        ein = sum(len(n["content"]) for n in nachrichten) // 4
        a = Antwort(text=text, ein_tokens=ein, aus_tokens=len(text) // 4, modell="attrappe")
        a.kosten_usd = (a.ein_tokens * self.k.preis_ein + a.aus_tokens * self.k.preis_aus) / 1_000_000
        return a


def erzeuge(konfig: Anbieter, grenzen: Grenzen, zaehler: Kostenzaehler) -> Backend:
    if konfig.art == "openrouter":
        return OpenRouter(konfig, grenzen, zaehler)
    if konfig.art == "openai":
        return OpenAIKompatibel(konfig, grenzen, zaehler)
    return Mock(konfig, grenzen, zaehler)


def json_aus_text(text: str) -> dict:
    """Liest das JSON-Objekt einer Antwort, auch wenn es in Zäunen oder Begleittext steckt."""
    t = text.strip()
    try:
        wert = json.loads(t)
    except ValueError:
        anfang, ende = t.find("{"), t.rfind("}")
        if anfang < 0 or ende <= anfang:
            raise ValueError("kein JSON-Objekt in der Antwort")
        wert = json.loads(t[anfang : ende + 1])
    if not isinstance(wert, dict):
        raise ValueError("die Antwort ist kein JSON-Objekt")
    return wert
