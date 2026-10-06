"""Lagebild: aus den Daten der Engine wird knapper, strukturierter Text.

Das Lagebild enthält keine Ratschläge, nur das, was ein Spieler auf seinen Bildschirmen sähe.
Jede Rolle bekommt den Ausschnitt, der zu ihren Entscheidungen gehört.
"""

from __future__ import annotations

# Was der Verwalter fertigen darf (Welt::fertigen): zivile Schiffe, keine Kriegsschiffe und keine Anlagen.
ZIVILE_SCHIFFE = ("kleiner_transporter", "grosser_transporter", "bergbauschiff", "recycler", "kolonieschiff")


def _z(n) -> str:
    """Zahl mit Tausenderpunkt."""
    if isinstance(n, float):
        return f"{n:.2f}".rstrip("0").rstrip(".")
    return f"{n:,}".replace(",", ".")


def _paare(d: dict, null: bool = False) -> str:
    teile = [f"{k} {_z(v)}" for k, v in d.items() if null or v]
    return ", ".join(teile) if teile else "nichts"


def _planet(p: dict, rolle: str) -> list[str]:
    art = "Heimatwelt" if p["heimat"] else "Kolonie"
    nebel = ", Nebelsystem" if p["nebel"] else ""
    z = [f"## Planet {p['koord']} ({art}, Zone {p['zone']}{nebel}, Felder {p['felder']['belegt']}/{p['felder']['gesamt']})"]
    if p.get("blockade"):
        b = p["blockade"]
        z.append(f"Im Orbit: Flotte {b['flotte']} von {b['durch']} ({b['art']})")
    z.append(
        f"Einwohner {_z(p['bevoelkerung'])} / Wohnraum {_z(p['wohnraum'])}, Stabilität {p['stabilitaet']} (Ziel {p['stabilitaet_ziel']}), "
        f"Versorgung {p['nahrung_deckung']} %, Konsumgüter {p['konsum_deckung']} %"
    )
    if rolle in ("verwalter", "stratege"):
        z.append(
            f"Strom {_z(p['energie']['erzeugung'])} erzeugt / {_z(p['energie']['verbrauch'])} verbraucht, "
            f"Arbeitskräfte {_z(p['arbeit']['bedarf'])} gebraucht / {_z(p['arbeit']['verfuegbar'])} vorhanden, "
            f"Fachkräfte {_z(p['fachkraefte']['bedarf'])} / {_z(p['fachkraefte']['verfuegbar'])}"
        )
    if rolle == "verwalter":
        f = p["faktoren"]
        if not p["heimat"]:
            z.append("Ertragsfaktoren in Promille: " + _paare(f, null=True))
        z.append("Gut: Bestand | je Stunde | Lagergrenze | voll in Stunden")
        for gut, bestand in p["bestand"].items():
            rate = p["rate"][gut]
            if bestand == 0 and rate == 0:
                continue
            voll = p["voll_in_stunden"].get(gut)
            z.append(f"{gut}: {_z(bestand)} | {rate:+d} | {_z(p['lager'][gut])} | {voll if voll is not None else '-'}")
    else:
        z.append("Bestand: " + _paare(p["bestand"]))
        if rolle == "feldherr":
            z.append("Je Stunde: " + ", ".join(f"{g} {r:+d}" for g, r in p["rate"].items() if r))
    z.append("Gebäude: " + _paare(p["gebaeude"]))
    plaetze = f"{len(p['bauschleife'])} von {p['bauschleife_plaetze']} Plätzen"
    if p["bauschleife"]:
        teile = [
            f"{a['gebaeude']} {a['stufe']} ({'wartet' if a.get('wartet') else 'noch ' + str(a['rest_min']) + ' min'})" for a in p["bauschleife"]
        ]
        z.append(f"Bauschleife ({plaetze}): " + ", ".join(teile))
    else:
        z.append(f"Bauschleife ({plaetze}): leer")
    if rolle == "verwalter" and p["baubar"]:
        z.append("Baubar, nächste Stufe (Kosten; Bauzeit; Wirkung; Bedarf; was fehlt):")
        z.extend(f"- {_baubar(b)}" for b in p["baubar"])
    if p["fertigung"]:
        teile = [f"{f['rest']} x {f['produkt']} ({f['schleife']}, nächstes in {f['naechstes_in_min']} min)" for f in p["fertigung"]]
        z.append("Fertigung: " + ", ".join(teile))
    if p["schiffe"] or rolle == "feldherr":
        z.append("Schiffe: " + _paare(p["schiffe"]))
    if p["verteidigung"] or rolle == "feldherr":
        z.append(f"Verteidigung: {_paare(p['verteidigung'])}; Garnison {_z(p['garnison'])}; Bunker schützt {_z(p['bunkerschutz'])} je Rohstoff")
    r = p.get("raketen") or {}
    if r.get("kapazitaet") and rolle in ("feldherr", "verwalter"):
        z.append(
            f"Raketensilo: {_z(r['abfang'])} Abfang-, {_z(r['interplanetar'])} Interplanetarraketen, "
            f"{_z(r['belegt_mit_bau'])} von {_z(r['kapazitaet'])} Plätzen belegt (mit Bau)"
        )
    return z


def _baubar(b: dict) -> str:
    zeile = f"{b['gebaeude']} {b['stufe']}: {_paare(b['kosten'])}; {b['bauzeit_min']} min"
    if b.get("ertrag"):
        zeile += f"; bringt +{_z(b['ertrag']['plus'])} {b['ertrag']['art']}"
    bedarf = [f"+{_z(b[f])} {n}" for f, n in (("strom_plus", "Strom"), ("arbeiter_plus", "Arbeitskräfte"), ("fachkraefte_plus", "Fachkräfte")) if b.get(f)]
    if bedarf:
        zeile += "; braucht " + ", ".join(bedarf)
    if b.get("kostenfaktor"):
        # Dezimalkomma: Tausender stehen im Lagebild mit Punkt.
        faktor = f"{float(b['kostenfaktor']):g}".replace(".", ",")
        zeile += f"; jede weitere Stufe x{faktor} teurer"
    if b["braucht"]:
        zeile += "; Voraussetzung " + ", ".join(b["braucht"])
    if b["fehlt"]:
        zeile += "; fehlt " + ", ".join(b["fehlt"])
    return zeile


def _forschbar(f: dict) -> str:
    dauer = f"{f['dauer_stunden']} h" if f["dauer_stunden"] is not None else "ohne Forschungspunkte nie fertig"
    zeile = f"{f['forschung']} {f['stufe']}: {_paare(f['kosten'])}; {dauer}; Labor {f['labor']}"
    if f["labor_heimat"] < f["labor"]:
        zeile += f" (Heimatwelt hat {f['labor_heimat']})"
    if f["fehlt"]:
        zeile += "; fehlt " + ", ".join(f["fehlt"])
    return zeile


def _kampf(k: dict) -> str:
    ausgang = {"angreifer": "Sieg des Angreifers", "verteidiger": "Sieg des Verteidigers", "unentschieden": "unentschieden"}[k["sieger"]]
    zeile = (
        f"{k['zeit']} {k['ort']}, {k['mission']}: {', '.join(k['angreifer'])} gegen {', '.join(k['verteidiger'])}, "
        f"{ausgang} nach {k['runden']} Runden (du warst {'Verteidiger' if k['seite'] == 'verteidigung' else 'Angreifer'}). "
        f"Verluste Angreifer {_paare(k['verluste_angreifer'])}; Verteidiger {_paare(k['verluste_verteidiger'])}"
    )
    if k["beute"]:
        zeile += f"; Beute {_paare(k['beute'])}"
    if k["truemmer"]["erz"] or k["truemmer"]["kristall"]:
        zeile += f"; Trümmer {_z(k['truemmer']['erz'])} Erz, {_z(k['truemmer']['kristall'])} Kristall"
    return zeile


def _bericht(b: dict) -> str:
    teile = [f"{b['ziel']} ({b['besitzer']}, {b['alter_stunden']} h alt): Bestand {_paare(b['bestand'])}"]
    for feld, name in (("schiffe", "Schiffe"), ("verteidigung", "Verteidigung"), ("gebaeude", "Gebäude"), ("forschung", "Forschung")):
        if b.get(feld) is not None:
            teile.append(f"{name} {_paare(b[feld])}")
        else:
            teile.append(f"{name} unbekannt")
    return "; ".join(teile)


def lagebild(s: dict, rolle: str) -> str:
    z: list[str] = []
    z.append(f"# Lage am {s['zeit']} (Epoche: {s['epoche_tage']} Spieltage)")
    z.append(f"Zivilisation {s['name']}, Volk {s['volk']}. Deine Rolle: {rolle}.")
    p = s["punkte"]
    z.append(
        f"Zivilisationsstufe {s['stufe']}, Rang {s['rang']} von {s['spielerzahl']}, Punkte {_z(p['gesamt'])} "
        f"(Wirtschaft {_z(p['wirtschaft'])}, Forschung {_z(p['forschung'])}, Militär {_z(p['militaer'])}, Zivilisation {_z(p['zivilisation'])})"
    )
    kopf = f"Credits {_z(s['credits'])}, Steuersatz {s['steuersatz']} %"
    if s.get("anfaengerschutz_bis"):
        kopf += f", Anfängerschutz bis {s['anfaengerschutz_bis']}"
    z.append(kopf)
    z.append("Töpfe (Guthaben in Werteinheiten, Anteil am Einkommen): " + ", ".join(f"{t} {_z(v['guthaben'])} ({v['anteil']} %)" for t, v in s["toepfe"].items()))
    k = s["kolonien"]
    z.append(
        f"Kolonien {k['anzahl']} von {k['erlaubt']} erlaubten, Verwaltungsgrenze {k['verwaltungsgrenze']}; "
        f"Flotten unterwegs {s['flottenplaetze']['belegt']} von {s['flottenplaetze']['gesamt']}"
    )
    st = s.get("naechste_stufe")
    if st:
        z.append(f"Nächste Stufe {st['name']}, Bedingungen erfüllt seit {st['erfuellt_seit_stunden']} von {st['haltezeit_stunden']} Stunden:")
        for b in st["bedingungen"]:
            z.append(f"  [{'x' if b['erfuellt'] else ' '}] {b['text']}")
        z.append("  Kosten: " + _paare(st["kosten"]))

    if s["warnungen"]:
        z.append("\n## Warnungen")
        z.extend(f"- {w}" for w in s["warnungen"])

    z.append("")
    for pl in s["planeten"]:
        z.extend(_planet(pl, rolle))
        z.append("")

    f = s["forschung"]
    if rolle in ("verwalter", "stratege", "feldherr"):
        zeile = "## Forschung\nStufen: " + _paare(f["stufen"])
        if f["aktiv"]:
            rest = f["aktiv"]["rest_stunden"]
            zeile += f"\nLäuft: {f['aktiv']['forschung']} {f['aktiv']['stufe']} (noch {rest if rest is not None else '?'} h)"
        else:
            zeile += "\nLäuft: nichts"
        if f["schlange"]:
            zeile += ", danach " + ", ".join(f["schlange"])
        zeile += f"; Forschungspunkte je Stunde {_z(f['punkte_je_stunde'])}"
        z.append(zeile)
        if rolle == "verwalter" and f["moeglich"]:
            z.append("Möglich, nächste Stufe (Kosten; Dauer; Labor; was auf der Heimatwelt fehlt):")
            z.extend(f"- {_forschbar(x)}" for x in f["moeglich"])

    if rolle in ("feldherr", "verwalter") and s["einheiten_kosten"]:
        z.append("\n## Einheiten (Stückkosten; nötige Werftstufe)")
        for e in s["einheiten_kosten"]:
            if rolle == "verwalter" and e["einheit"] not in ZIVILE_SCHIFFE:
                continue
            braucht = f"; braucht {', '.join(e['braucht'])}" if e["braucht"] else ""
            werft = f"Werft {e['werft']}" if e["schiff"] else "ohne Werft"
            z.append(f"- {e['einheit']}: {_paare(e['kosten'])}; {werft}{braucht}")

    if s["flotten"]:
        z.append("\n## Eigene Flotten")
        for fl in s["flotten"]:
            ladung = f", Ladung {_paare(fl['ladung'])}" if fl["ladung"] else ""
            verband = ""
            if fl.get("verbandsfuehrung"):
                verband = ", führt einen Verband"
            elif fl.get("verband") is not None:
                verband = f", im Verband von Flotte {fl['verband']}"
            z.append(f"- Flotte {fl['flotte']}: {fl['mission']} {fl['start']} -> {fl['ziel']}, {fl['zustand']} bis {fl['bis']}, {_paare(fl['schiffe'])}{ladung}{verband}")
    if s.get("verbaende") and rolle == "feldherr":
        z.append("\n## Offene Verbände (eigene und verbündete)")
        z.extend(f"- Führung Flotte {v['fuehrung']} von {v['spieler']}: Ziel {v['ziel']}, Ankunft {v['ankunft']}" for v in s["verbaende"])
    if s.get("raketensalven") and rolle in ("feldherr", "stratege"):
        z.append("\n## Raketen im Flug")
        z.extend(f"- {_z(r['anzahl'])} Raketen von {r['von']} auf {r['ziel']} ({r['zieltyp']}), Einschlag {r['ankunft']}" for r in s["raketensalven"])
    if s.get("kampfberichte") and rolle in ("feldherr", "stratege"):
        z.append("\n## Kampfberichte (neueste zuerst)")
        z.extend(f"- {_kampf(k)}" for k in s["kampfberichte"][: 8 if rolle == "feldherr" else 3])

    if rolle in ("feldherr", "stratege"):
        if s["berichte"]:
            z.append("\n## Spionageberichte")
            z.extend(f"- {_bericht(b)}" for b in s["berichte"][: 12 if rolle == "feldherr" else 5])
        if s["truemmer"] and rolle == "feldherr":
            z.append("\n## Trümmerfelder im Sektor")
            z.extend(f"- {t['koord']}: {_z(t['erz'])} Erz, {_z(t['kristall'])} Kristall" for t in s["truemmer"])
    if s["erkundet"] and rolle in ("verwalter", "stratege", "feldherr"):
        z.append("\n## Erkundete Plätze")
        for e in s["erkundet"]:
            frei = "frei" if e["frei"] else "belegt"
            z.append(
                f"- {e['koord']}: {e['felder']} Felder, Zone {e['zone']}, Erz {e['reich_erz']} ‰, Kristall {e['reich_kristall']} ‰"
                f"{', Nebel' if e['nebel'] else ''}, {frei}"
            )

    if s["nachbarn"] and rolle != "verwalter":
        z.append("\n## Nachbarschaft (belegte Plätze, nächste zuerst)")
        for n in s["nachbarn"][: 25 if rolle in ("feldherr", "diplomat") else 12]:
            zusatz = ", Heimatwelt" if n["heimat"] else ", Kolonie"
            if n["schutz"]:
                zusatz += ", Anfängerschutz"
            z.append(f"- {n['koord']}: {n['spieler']}, {_z(n['punkte'])} Punkte{zusatz}")

    rang = s["rangliste"]
    if rolle in ("stratege", "diplomat", "feldherr"):
        eigener = s["rang"]
        zeigen = [r for r in rang if r[0] <= 10 or abs(r[0] - eigener) <= 2]
        z.append("\n## Rangliste (Rang, Name, Punkte, Stufe)")
        z.extend(f"- {r[0]}. {r[1]}: {_z(r[2])} Punkte, Stufe {r[3]}" for r in zeigen)

    if s["vertraege"] or s["allianz"] or s["einladungen"]:
        z.append("\n## Verträge und Allianz")
        for v in s["vertraege"]:
            zeile = f"- Vertrag {v['vertrag']}: {v['art']} mit {v['partner']}, {v['status']}, Kaution {_z(v['kaution'])}"
            if v.get("tribut"):
                t = v["tribut"]
                zeile += f", {t['zahler']} zahlt {_z(t['menge_je_tag'])} {t['gut']} je Tag"
            z.append(zeile)
        if s["allianz"]:
            z.append(f"- Allianz {s['allianz']['name']}: {', '.join(s['allianz']['mitglieder'])}")
        for e in s["einladungen"]:
            z.append(f"- Einladung in die Allianz {e}")
    if s["register"] and rolle in ("diplomat", "stratege"):
        z.append("\n## Vertragsregister (öffentlich, neueste zuerst)")
        z.extend(f"- {e['zeit']}: {e['art']} {e['a']} / {e['b']}: {e['vorgang']}" for e in s["register"][: 12 if rolle == "diplomat" else 6])

    if s["nachrichten"] and rolle in ("diplomat", "stratege"):
        z.append("\n## Nachrichten (neueste zuerst)")
        for n in s["nachrichten"][: 20 if rolle == "diplomat" else 5]:
            neu = "NEU " if n["neu"] else ""
            wohin = "Allianz" if n["allianz"] else ", ".join(n["an"])
            z.append(f"- {neu}{n['zeit']} {n['von']} an {wohin}: {n['text']}")

    markt = s["markt"]
    if rolle in ("verwalter", "diplomat") and (markt["preise"] or markt["orders"]):
        z.append("\n## Markt")
        for gut, pr in markt["preise"].items():
            z.append(f"- {gut}: Verkauf ab {pr['verkauf_ab'] if pr['verkauf_ab'] is not None else '-'}, Kauf bis {pr['kauf_bis'] if pr['kauf_bis'] is not None else '-'} Credits")
        for o in markt["orders"]:
            z.append(f"- eigene Order {o['order']}: {o['seite']} {_z(o['menge'])} {o['gut']} zu {o['preis']}")

    if s["ereignisse"]:
        z.append("\n## Ereignisse seit deinem letzten Aufruf (neueste zuerst)")
        z.extend(f"- {e['zeit']}: {e['text']}" for e in s["ereignisse"][:30])
    if s["chronik"] and rolle in ("stratege", "feldherr", "diplomat"):
        z.append("\n## Chronik der letzten Tage")
        z.extend(f"- {e['zeit']}: {e['text']}" for e in s["chronik"][:15])

    z.append("\n## Doktrin des Strategen")
    z.append(s["doktrin"] or "(noch keine)")
    if s["meldungen"] and rolle == "stratege":
        z.append("\n## Meldungen der anderen Rollen")
        z.extend(f"- {m['zeit']} {m['von']}: {m['text']}" for m in s["meldungen"])
    z.append("\n## Dein Notizbuch")
    z.append(s["notiz"] or "(leer)")
    return "\n".join(z)
