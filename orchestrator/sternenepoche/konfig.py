"""Konfiguration eines Laufs aus einer TOML-Datei."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path

ROLLEN = ("stratege", "verwalter", "feldherr", "diplomat")
DENKEN = ("weglassen", "aus", "budget", "niedrig", "mittel", "hoch")
# Denkstufen der Konfiguration und ihr Name bei OpenRouter (reasoning.effort)
AUFWAND = {"niedrig": "low", "mittel": "medium", "hoch": "high"}
OPENROUTER_URL = "https://openrouter.ai/api/v1"


class KonfigFehler(ValueError):
    pass


@dataclass
class Anbieter:
    """Ein Modell hinter einer Schnittstelle: OpenRouter, ein lokaler Server oder die Attrappe."""

    name: str
    art: str  # "openrouter", "openai" (vLLM, llama.cpp, Ollama) oder "mock"
    modell: str = ""
    basis_url: str = ""
    schluessel_env: str = ""
    parallel: int = 8
    # json_schema: response_format mit Schema; guided_json: vLLM; json_object; aus: nur Prompt
    schema: str = "json_schema"
    # weglassen: Parameter nicht senden; aus: Denken abschalten; budget: auf grenzen.denk_tokens begrenzen;
    # niedrig, mittel, hoch: Denkaufwand als Stufe, für Modelle, die ein Tokenbudget nicht einhalten
    denken: str = "weglassen"
    provider: dict = field(default_factory=dict)
    zusatz: dict = field(default_factory=dict)
    # Preise je Million Tokens für lokale Server, falls Kosten mitgezählt werden sollen
    preis_ein: float = 0.0
    preis_aus: float = 0.0
    titel: str = ""
    referer: str = ""


@dataclass
class Grenzen:
    max_ausgabe_tokens: int = 1200
    denk_tokens: int = 1000
    temperatur: float = 0.7
    zeitlimit_sekunden: float = 180.0
    versuche: int = 3
    budget_usd: float | None = None


@dataclass
class Lauf:
    startwert: int = 1
    spieler: int = 50
    tage: int = 365
    ki: list[int] | None = None  # None: alle Spieler sind Agenten
    bottypen: list[str] = field(default_factory=lambda: ["oekonom", "raeuber", "igel", "haendler"])
    ausgabe: str = "laeufe/lauf"
    engine: str = "target/release/sternenepoche.exe"
    regeln: str | None = None
    schnappschuss_tage: int = 5


@dataclass
class Konfig:
    lauf: Lauf
    grenzen: Grenzen
    anbieter: dict[str, Anbieter]
    rollen: dict[str, str]
    # Vergleichsvariante: jedes genannte Modell steuert seine Agenten in allen vier Rollen
    vergleich: list[str]
    pfad: Path

    def anbieter_fuer(self, spieler: int, rolle: str) -> str:
        if self.vergleich:
            ki = self.lauf.ki if self.lauf.ki is not None else list(range(self.lauf.spieler))
            # Fester Versatz aus dem Startwert, damit die Zuordnung je Epoche wechselt und nachvollziehbar bleibt.
            platz = (ki.index(spieler) + self.lauf.startwert) % len(self.vergleich)
            return self.vergleich[platz]
        return self.rollen[rolle]


def _nur_bekannte(klasse, daten: dict, wo: str) -> dict:
    bekannt = set(klasse.__dataclass_fields__)
    fremd = set(daten) - bekannt
    if fremd:
        raise KonfigFehler(f"{wo}: unbekannte Felder {sorted(fremd)}")
    return daten


def laden(pfad: str | Path) -> Konfig:
    pfad = Path(pfad)
    with open(pfad, "rb") as f:
        roh = tomllib.load(f)

    lauf_roh = dict(roh.get("lauf", {}))
    if lauf_roh.get("ki") == "alle":
        lauf_roh["ki"] = None
    lauf = Lauf(**_nur_bekannte(Lauf, lauf_roh, "[lauf]"))
    grenzen = Grenzen(**_nur_bekannte(Grenzen, dict(roh.get("grenzen", {})), "[grenzen]"))

    anbieter: dict[str, Anbieter] = {}
    for name, daten in roh.get("anbieter", {}).items():
        a = Anbieter(name=name, **_nur_bekannte(Anbieter, dict(daten), f"[anbieter.{name}]"))
        if a.art == "openrouter":
            a.basis_url = a.basis_url or OPENROUTER_URL
            a.schluessel_env = a.schluessel_env or "OPENROUTER_API_KEY"
        if a.art not in ("openrouter", "openai", "mock"):
            raise KonfigFehler(f"[anbieter.{name}]: art muss openrouter, openai oder mock sein")
        if a.art != "mock" and not a.modell:
            raise KonfigFehler(f"[anbieter.{name}]: modell fehlt")
        if a.art == "openai" and not a.basis_url:
            raise KonfigFehler(f"[anbieter.{name}]: basis_url fehlt")
        if a.schema not in ("json_schema", "guided_json", "json_object", "aus"):
            raise KonfigFehler(f"[anbieter.{name}]: schema muss json_schema, guided_json, json_object oder aus sein")
        if a.denken not in DENKEN:
            raise KonfigFehler(f"[anbieter.{name}]: denken muss {', '.join(DENKEN)} sein")
        anbieter[name] = a

    vergleich = list(roh.get("vergleich", {}).get("anbieter", []))
    rollen = dict(roh.get("rollen", {}))
    if vergleich:
        for name in vergleich:
            if name not in anbieter:
                raise KonfigFehler(f"[vergleich]: Anbieter '{name}' ist nicht definiert")
    else:
        for rolle in ROLLEN:
            if rolle not in rollen:
                raise KonfigFehler(f"[rollen]: {rolle} fehlt")
            if rollen[rolle] not in anbieter:
                raise KonfigFehler(f"[rollen]: Anbieter '{rollen[rolle]}' für {rolle} ist nicht definiert")
    if lauf.ki is not None and any(i < 0 or i >= lauf.spieler for i in lauf.ki):
        raise KonfigFehler("[lauf]: ki enthält Spieler außerhalb von 0 bis spieler-1")
    return Konfig(lauf=lauf, grenzen=grenzen, anbieter=anbieter, rollen=rollen, vergleich=vergleich, pfad=pfad)
