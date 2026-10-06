"""Kostenschätzung und Vorabprüfung für OpenRouter.

Die Modellliste von OpenRouter ist öffentlich und braucht keinen Schlüssel. Aus ihren
Preisen und dem Mengengerüst der Epoche ergibt sich, was ein Lauf ungefähr kostet.
"""

from __future__ import annotations

import os

import httpx

from .konfig import AUFWAND, OPENROUTER_URL, ROLLEN, Konfig

# Mengengerüst, gemessen im echten Lauf vom 4. Okt. 2026 (laeufe/openrouter-test, 28 Spieltage, ein Reich):
# Entscheidungen je Agent und Spieltag. Der Verwalter lässt sich meist alle zwei Stunden wecken.
AUFRUFE_JE_TAG = {"stratege": 2.0, "verwalter": 10.1, "feldherr": 3.3, "diplomat": 2.0}
AUSGABE_JE_AUFRUF = {"stratege": 730, "verwalter": 390, "feldherr": 320, "diplomat": 630}
LAGEBILD_TOKENS = 2500
# Abfrage- und Korrekturrunden: Modellaufrufe je Entscheidung, gemessen wie oben. Seit die Bauliste Wirkung
# und Bedarf nennt, fragt der Verwalter kaum noch ab; die Werte sind deshalb eher zu hoch als zu niedrig.
ZUSATZRUNDEN = {"stratege": 1.55, "verwalter": 2.25, "feldherr": 2.0, "diplomat": 1.1}
# Anteil von grenzen.denk_tokens, den eine Denkeinstellung im Mittel verbraucht (Schätzung).
DENKANTEIL = {"budget": 1.0, "hoch": 1.0, "mittel": 0.5, "niedrig": 0.25}
# Deutscher Text: grob 3,3 Zeichen je Token.
ZEICHEN_JE_TOKEN = 3.3


def modelle_holen(basis_url: str = OPENROUTER_URL) -> dict[str, dict]:
    r = httpx.get(f"{basis_url}/models", timeout=60)
    r.raise_for_status()
    return {m["id"]: m for m in r.json()["data"]}


def schluessel_stand(basis_url: str, env: str) -> dict | None:
    schluessel = os.environ.get(env, "")
    if not schluessel:
        return None
    r = httpx.get(f"{basis_url}/key", headers={"Authorization": f"Bearer {schluessel}"}, timeout=30)
    r.raise_for_status()
    return r.json().get("data", {})


def pruefe(konfig: Konfig, modelle: dict[str, dict]) -> list[str]:
    """Prüft die OpenRouter-Anbieter der Konfiguration gegen die Modellliste. Liefert Befunde als Text."""
    befunde: list[str] = []
    benutzt = set(konfig.vergleich) if konfig.vergleich else set(konfig.rollen.values())
    for name in sorted(benutzt):
        a = konfig.anbieter[name]
        if a.art != "openrouter":
            befunde.append(f"[ok] {name}: {a.art}, {a.modell or 'ohne Modell'}, {a.basis_url or 'ohne Netz'}")
            continue
        m = modelle.get(a.modell)
        if not m:
            aehnlich = [i for i in modelle if a.modell.split("/")[-1].split("-")[0] in i][:5]
            befunde.append(f"[FEHLER] {name}: Modell '{a.modell}' gibt es bei OpenRouter nicht. Ähnlich: {', '.join(aehnlich) or 'nichts'}")
            continue
        parameter = set(m.get("supported_parameters") or [])
        if a.schema == "json_schema" and "structured_outputs" not in parameter:
            befunde.append(f"[FEHLER] {name}: {a.modell} unterstützt kein erzwungenes Antwortschema. schema = \"json_object\" setzen oder anderes Modell wählen.")
        elif a.schema == "json_object" and "response_format" not in parameter:
            befunde.append(f"[FEHLER] {name}: {a.modell} unterstützt response_format nicht. schema = \"aus\" setzen.")
        elif a.denken != "weglassen" and "reasoning" not in parameter:
            befunde.append(f"[FEHLER] {name}: {a.modell} kennt den Parameter reasoning nicht. denken = \"weglassen\" setzen.")
        else:
            preis = m.get("pricing", {})
            befunde.append(
                f"[ok] {name}: {a.modell}, Kontext {m.get('context_length')}, "
                f"{float(preis.get('prompt', 0)) * 1e6:.3f} / {float(preis.get('completion', 0)) * 1e6:.3f} USD je Million Tokens ein/aus"
            )
        denken = m.get("reasoning") or {}
        stufen = denken.get("supported_efforts") or []
        if denken.get("mandatory") and a.denken == "aus":
            befunde.append(f"[FEHLER] {name}: {a.modell} denkt immer, denken = \"aus\" geht nicht.")
        if a.denken == "budget" and stufen and not denken.get("supports_max_tokens"):
            # Gemessen an DeepSeek V4: ohne Budgetunterstützung denkt das Modell bis zum Tokenlimit und antwortet nicht.
            befunde.append(f"[FEHLER] {name}: {a.modell} hält kein Denkbudget ein, es kennt nur die Stufen {', '.join(stufen)}. "
                           "denken = \"aus\" oder eine passende Stufe setzen.")
        if a.denken in AUFWAND and stufen and AUFWAND[a.denken] not in stufen:
            befunde.append(f"[FEHLER] {name}: {a.modell} kennt die Denkstufe {AUFWAND[a.denken]} nicht, nur {', '.join(stufen)}.")
        if (m.get("reasoning") or {}).get("default_enabled") and a.denken == "weglassen":
            befunde.append(f"[Hinweis] {name}: {a.modell} denkt von sich aus. Ohne denken = \"budget\" oder \"aus\" ist das Denkbudget nicht begrenzt.")
        if not a.provider.get("order") and not a.provider.get("only"):
            befunde.append(f"[Hinweis] {name}: kein Provider festgelegt. Für eine Messung provider.order und allow_fallbacks = false setzen, "
                           "damit alle Agenten dieselben Gewichte bekommen.")
        if not os.environ.get(a.schluessel_env, ""):
            befunde.append(f"[FEHLER] {name}: Umgebungsvariable {a.schluessel_env} ist nicht gesetzt.")
    return befunde


def schaetze(konfig: Konfig, system_zeichen: dict[str, int], modelle: dict[str, dict]) -> tuple[list[dict], dict]:
    """Schätzt Aufrufe, Tokens und Kosten je Rolle und Anbieter für die konfigurierte Epoche."""
    l = konfig.lauf
    agenten = len(l.ki) if l.ki is not None else l.spieler
    zeilen: list[dict] = []
    for rolle in ROLLEN:
        if konfig.vergleich:
            anteile = {n: 1 / len(konfig.vergleich) for n in konfig.vergleich}
        else:
            anteile = {konfig.rollen[rolle]: 1.0}
        for name, anteil in anteile.items():
            a = konfig.anbieter[name]
            aufrufe = AUFRUFE_JE_TAG[rolle] * agenten * l.tage * ZUSATZRUNDEN[rolle] * anteil
            system_tokens = system_zeichen[rolle] / ZEICHEN_JE_TOKEN
            ein = aufrufe * (system_tokens + LAGEBILD_TOKENS)
            aus = aufrufe * (AUSGABE_JE_AUFRUF[rolle] + konfig.grenzen.denk_tokens * DENKANTEIL.get(a.denken, 0.0))
            if a.art == "openrouter" and a.modell in modelle:
                preis = modelle[a.modell].get("pricing", {})
                p_ein, p_aus = float(preis.get("prompt", 0)), float(preis.get("completion", 0))
                p_cache = float(preis["input_cache_read"]) if preis.get("input_cache_read") else None
            else:
                p_ein, p_aus, p_cache = a.preis_ein / 1e6, a.preis_aus / 1e6, None
            usd = ein * p_ein + aus * p_aus
            # Mit Prefix Caching kostet der für alle gleiche Systemtext den Cache-Preis.
            usd_cache = None
            if p_cache is not None:
                usd_cache = aufrufe * (system_tokens * p_cache + LAGEBILD_TOKENS * p_ein) + aus * p_aus
            zeilen.append({"rolle": rolle, "anbieter": name, "art": a.art, "modell": a.modell or "-", "aufrufe": round(aufrufe),
                           "ein_mio": ein / 1e6, "aus_mio": aus / 1e6, "usd": usd, "usd_cache": usd_cache})
    summe = {
        "aufrufe": sum(z["aufrufe"] for z in zeilen),
        "ein_mio": sum(z["ein_mio"] for z in zeilen),
        "aus_mio": sum(z["aus_mio"] for z in zeilen),
        "usd": sum(z["usd"] for z in zeilen),
        "usd_cache": sum(z["usd_cache"] if z["usd_cache"] is not None else z["usd"] for z in zeilen),
    }
    return zeilen, summe


def tabelle(zeilen: list[dict], summe: dict) -> str:
    aus = [f"{'Rolle':<10} {'Anbieter':<18} {'Modell':<42} {'Aufrufe':>9} {'Ein (Mio)':>10} {'Aus (Mio)':>10} {'USD':>9} {'USD mit Cache':>14}"]
    for z in zeilen:
        cache = f"{z['usd_cache']:.2f}" if z["usd_cache"] is not None else "-"
        aus.append(f"{z['rolle']:<10} {z['anbieter']:<18} {z['modell'][:42]:<42} {z['aufrufe']:>9} {z['ein_mio']:>10.1f} {z['aus_mio']:>10.1f} {z['usd']:>9.2f} {cache:>14}")
    aus.append(f"{'Summe':<10} {'':<18} {'':<42} {summe['aufrufe']:>9} {summe['ein_mio']:>10.1f} {summe['aus_mio']:>10.1f} {summe['usd']:>9.2f} {summe['usd_cache']:>14.2f}")
    return "\n".join(aus)
