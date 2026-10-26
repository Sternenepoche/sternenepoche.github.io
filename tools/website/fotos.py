"""Frische Bildschirmfotos des aktuellen Spiels für die neue Website (docs/website/fotos/<name>.webp).

Nur echte Oberfläche aus isolierten Prüfwelten (crates/server/examples/*_ui_fixture.rs) mit dem aktuellen Server-Build.
Aufruf: python -X utf8 tools/website/fotos.py WELT=SERVER [WELT=SERVER ...]
  z. B. flotten=http://127.0.0.1:8921 monde=http://127.0.0.1:8923 dyson=http://127.0.0.1:8925 lage=http://127.0.0.1:8927
Schreibt zusätzlich docs/website/fotos/fotos.json (Zeitpunkt, Welt, Größe je Foto).
"""
import io, json, os, sys, time, urllib.request
from datetime import datetime
from pathlib import Path
from PIL import Image
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[2]
# STERN_FOTOS_ZIEL: Probelauf in einen anderen Ordner, ohne die Website-Fotos zu überschreiben (20.10.2026).
OUT = Path(os.environ["STERN_FOTOS_ZIEL"]) if os.environ.get("STERN_FOTOS_ZIEL") else ROOT / "docs/website/fotos"
PASSWORT = "geheimes-testpasswort-1"  # Testwert aus harness/tests/e2e_harness.py
DESKTOP, HANDY = (1440, 900), (390, 844)

# (Fotoname, Welt, Ziel, Handy?) – Ziel: Seiten-ID aus web-client/rahmen.js MENUE, „mond“ oder ein Sonderfall „technik:<einheit>“.
FOTOS = [
    ("uebersicht", "flotten", "reich", True), ("auftraege", "flotten", "auftraege", False),
    ("versorgung", "flotten", "versorgung", False), ("produktion", "flotten", "kolonie", False),
    ("anlagen", "flotten", "gebaeude", False), ("forschung", "flotten", "forschen", False),
    ("technologiebaum", "flotten", "freischaltungen", False), ("werft", "flotten", "werft", True),
    ("verteidigung", "flotten", "verteidigung", False), ("truppen", "flotten", "truppen", True),
    ("flotte", "flotten", "flotten", True), ("kampfsimulator", "flotten", "warsim", False),
    ("galaxie", "flotten", "karte", True), ("kolonisation", "flotten", "kolonien", False),
    ("imperium", "flotten", "imperium", False), ("codex", "flotten", "codex", False),
    ("allianz", "flotten", "allianzbereich", False), ("briefkasten", "flotten", "briefkasten", False),
    ("berichte", "flotten", "berichte", False), ("handel", "flotten", "aktionen", False),
    ("rat", "flotten", "agent", False), ("pinwaende", "flotten", "pinwaende", False),
    ("technik-blaupause", "flotten", "technik:kreuzer:blaupause", False),
    ("technik-explosion", "flotten", "technik:kreuzer:explosion", False),
    ("technik-schnitt", "flotten", "technik:kreuzer:schnitt", False),
    ("archiv", "flotten", "archiv:kreuzer", False),
    ("infodisplay", "flotten", "info:kreuzer", False), ("bericht", "flotten", "bericht", True),
    ("galaxie-3d", "flotten", "atlas", True),
    ("mond", "monde", "mond", True), ("grossprojekt", "dyson", "grossprojekt", True),
    ("lage", "lage", "reich", False),
    # 0.4.8: Einsatzblatt (Flottenauftrag mit Lage und Simulation) und Invasion in Wellen (invasion_ui_fixture).
    ("einsatz", "flotten", "einsatz:angriff", True), ("invasion", "invasion", "reich", True),
    # 0.4.9: Kräfteatlas im Spiel (Vollbild über Forschung), für Startseite und Nachschlagen (Karl 26.10.2026).
    ("kraefteatlas", "flotten", "kraefteatlas", True),
]


def anmelden(server):
    req = urllib.request.Request(server + "/api/login", data=json.dumps({"name": "Monitorpilot", "password": PASSWORT}).encode(), method="POST")
    req.add_header("Content-Type", "application/json"); req.add_header("X-Sternenepoche-Oberflaeche", "browser")
    return {"server": server, "token": json.loads(urllib.request.urlopen(req).read())["token"], "name": "Monitorpilot"}


def seite(browser, server, groesse, fehler):
    sitzung = anmelden(server)
    ctx = browser.new_context(viewport={"width": groesse[0], "height": groesse[1]}, device_scale_factor=1,
                              is_mobile=groesse[0] < 768, has_touch=groesse[0] < 768)
    ctx.add_init_script(f"localStorage.setItem('sternenepoche-server',{json.dumps(server)});"
                        f"sessionStorage.setItem('sternenepoche-session',{json.dumps(json.dumps(sitzung))});")
    p = ctx.new_page()
    p.on("pageerror", lambda e: fehler.append(str(e)[:200]))
    p.on("console", lambda m: fehler.append(m.text[:200]) if m.type == "error" else None)
    p.goto(server + "/", timeout=90000)  # 20.10.2026: ein langsamer erster Seitenaufbau brach sonst den ganzen Lauf ab
    for _ in range(80):
        if p.evaluate("() => typeof view !== 'undefined' && !!view && !document.getElementById('game').hidden"):
            break
        time.sleep(0.5)
    p.evaluate("() => { window.SternenLoading?.ready?.(); document.getElementById('sternen-loader')?.remove(); }")
    time.sleep(2.5)
    return ctx, p


def gehe(p, ziel):
    if ziel == "kraefteatlas":
        # Wie ein Spieler: Forschung, Knopf „Kräfteatlas“; das Foto zeigt Kopf und Klassenmatrix.
        gehe(p, "forschen"); time.sleep(1.5)
        p.evaluate("() => document.querySelector('#forschen .kraefteatlas-knopf')?.click()"); time.sleep(9)
        p.evaluate("() => document.querySelector('#kraefteatlas-overlay')?.shadowRoot?.querySelector('#klassen')?.scrollIntoView({block: 'center'})")
        time.sleep(2); return
    if ziel.startswith("einsatz:"):
        # Einsatzblatt wie aus der Galaxie „Angriff planen“: ausgespähtes Ziel, Vorlage, Simulation (Lage am Ziel).
        k = p.evaluate("() => (view.berichte || []).map(b => b.ziel)[0] || ''")
        p.evaluate("(o) => window.Einsatz?.oeffnen(o)", {"ziel": k, "mission": ziel.split(":")[1]}); time.sleep(2.5)
        for _ in range(3):
            if p.evaluate("() => document.getElementById('einsatz-blatt')?.dataset.esStufe") == "flotte":
                break
            p.evaluate("() => document.querySelector('#einsatz-blatt [data-es-weiter]')?.click()"); time.sleep(1.5)
        p.evaluate("() => document.querySelector('#einsatz-blatt [data-es-vorlage]')?.click()"); time.sleep(1.5)
        p.evaluate("() => document.querySelector('#einsatz-blatt [data-es-sim]')?.click()"); time.sleep(5)
        return
    if ziel == "mond":
        # Seit 0.4.1 öffnet die Planetenwahl den Mond (Mondseite.oeffne); der alte Seitenleisten-Link bleibt Rückfall.
        k = p.evaluate("() => { const m = view.monde || {}; const l = [...(m.eigene || []), ...(m.monde || [])];"
                       " return l[0]?.koord || (view.planeten || []).find(x => x.mond)?.koord || null; }")
        if k and p.evaluate("() => !!window.Mondseite?.oeffne"):
            p.evaluate("(k) => window.Mondseite.oeffne(k)", k); return
        p.click("#planet-sidebar .planet-mond, .planet-mond", timeout=8000); return
    if ziel == "atlas":
        # 3D-Sonnensystem (galaxy.js): am Handy öffnet die Galaxie direkt die 3D-Ansicht, am Desktop der Knopf „3D-Ansicht“.
        gehe(p, "karte"); time.sleep(2)
        p.evaluate("() => { const k = document.querySelector('[data-gt-atlas]'); if (k && k.getClientRects().length) k.click(); }")
        time.sleep(14)
        p.evaluate("() => { const c = [...document.querySelectorAll('canvas')].find(x => x.getClientRects().length && x.width > 300); c?.scrollIntoView({block: 'end'}); }")
        time.sleep(4); return
    if ziel == "bericht":
        # Einen Bericht in der Berichtansicht öffnen (erster Eintrag der Liste).
        gehe(p, "briefkasten"); time.sleep(2)
        p.evaluate("() => { const l = document.getElementById('messages') || document;"
                   " const b = l.querySelector('[data-report-open], [data-report], .report-row button, .report-row, li button, details summary');"
                   " b?.click(); }")
        time.sleep(2.5); return
    if ziel.startswith(("technik:", "archiv:", "info:")):
        teile = ziel.split(":"); gehe(p, "werft"); time.sleep(1.5)
        # Kachel wählen: das Infodisplay der Einheit erscheint auf der Seite.
        p.evaluate("(k) => document.querySelector(`[data-bs-key='${k}']`)?.click()", teile[1]); time.sleep(1.5)
        if teile[0] == "info":
            p.evaluate("() => document.querySelector('.bs-detail, [id^=bs-detail]')?.scrollIntoView({block: 'center'})")
            time.sleep(1); return
        # Techinfo öffnet das Techbaum-Dossier (Shadow DOM, Playwright-Locator durchdringt es).
        p.evaluate("(k) => document.querySelector(`[data-bs-techinfo='${k}']`)?.click()", teile[1]); time.sleep(3)
        if not p.locator("text=DOSSIER").first.is_visible():  # Dossier liegt im Shadow DOM; der Locator durchdringt es
            p.evaluate("(k) => document.querySelector(`[data-bs-techinfo='${k}']`)?.click()", teile[1]); time.sleep(3)
        if p.evaluate("() => !!window.TechbaumSpiel && document.querySelector('#techinfo[open]') === null"):
            # Reiter im Dossier des Technologiebaums (Shadow DOM): den sichtbaren Knopf mit genau diesem Text antippen –
            # der Playwright-Locator traf am 26.10.2026 einen verdeckten gleichnamigen Knopf, das Foto zeigte dann nur das Motiv.
            tippe = """([text, art]) => { const r = document.getElementById('techbaum-overlay')?.shadowRoot; if (!r) return false;
                const b = [...r.querySelectorAll(art)].find(x => x.textContent.trim() === text && x.getClientRects().length);
                if (!b) return false; b.scrollIntoView({block: 'center'}); b.click(); return true; }"""
            if teile[0] == "technik":
                text = {"blaupause": "Blaupause", "explosion": "Explosion", "schnitt": "Schnitt"}[teile[2]]
                if not p.evaluate(tippe, [text, "button"]):
                    print("Zeichnungsreiter nicht gefunden:", text)
            else:
                if not p.evaluate(tippe, ["Archiv", "button"]):
                    print("Archiv nicht gefunden")
                time.sleep(1)
                p.evaluate("""() => { const r = document.getElementById('techbaum-overlay')?.shadowRoot;
                    const s = r && [...r.querySelectorAll('summary')].find(x => /Chronik/.test(x.textContent) && x.getClientRects().length);
                    if (s) { s.click(); s.scrollIntoView({block: 'start'}); } }""")
            time.sleep(2.5); return
        p.evaluate("(k) => document.querySelector(`[data-bs-techinfo='${k}'][data-bs-reiter='techinfo'], [data-bs-techinfo='${k}']`)?.click()", teile[1])
        if not p.evaluate("(k) => !!document.querySelector(`[data-bs-techinfo='${k}']`)", teile[1]):
            # Detailfeld der Einheit erst öffnen (Kachel), dann Techinfo.
            p.evaluate("(k) => document.querySelector(`[data-bs-key='${k}'], [data-key='${k}']`)?.click()", teile[1]); time.sleep(1)
            p.evaluate("(k) => document.querySelector(`[data-bs-techinfo='${k}']`)?.click()", teile[1])
        time.sleep(1)
        reiter = "technik" if teile[0] == "technik" else "archiv"
        p.evaluate("(r) => document.querySelector(`#techinfo [data-ti-reiter='${r}']`)?.click()", reiter); time.sleep(1)
        if len(teile) > 2:
            p.evaluate("(z) => document.querySelector(`#techinfo [data-ti-zeichnung='${z}']`)?.click()", teile[2])
        time.sleep(2); return
    p.evaluate("(id) => { const r = document.querySelector(`#seiten-reiter [data-tab='${id}'], [data-tab='${id}']`);"
               " if (r) r.click(); else document.querySelector(`#tabs [data-menue='${id}']`)?.click(); }", ziel)
    if not p.evaluate("(id) => !!document.getElementById(id)?.getClientRects().length", ziel):
        gruppe = p.evaluate("(id) => (window.Rahmen?.MENUE || []).find(([, , b]) => b.some(([x]) => x === id))?.[0] || ''", ziel)
        if gruppe:
            p.evaluate("(g) => document.querySelector(`#tabs [data-menue='${g}']`)?.click()", gruppe); time.sleep(1)
        p.evaluate("(id) => document.querySelector(`[data-tab='${id}']`)?.click()", ziel)


def techbaum(url: str):
    """Bilder der Technologiebaum-Gesamtkarte (web-client/techbaum über ausgabe/techbaum/index.html)."""
    OUT.mkdir(parents=True, exist_ok=True)
    liste = json.loads((OUT / "fotos.json").read_text(encoding="utf-8")) if (OUT / "fotos.json").exists() else {}
    with sync_playwright() as pw:
        b = pw.chromium.launch(channel="msedge", args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"])
        # 21.10.2026 (Karl: „der coole neue Techtree“): lesbare Nahansichten statt der winzigen Gesamtkarte –
        # ein Bereich mit Bildern und Linien, der Weltenkrümmer mit Dossier und hervorgehobenem Pfad, am Handy ein Kreuzer.
        for name, groesse, volk, schritt in (("techbaum-karte", DESKTOP, "syntheten", "bereich:flotte"), ("techbaum-nah", DESKTOP, "syntheten", "objekt:weltentruemmer"),
                                             ("techbaum-raumfahrt", DESKTOP, "veyari", "bereich:raumfahrt"), ("techbaum-handy", HANDY, "aurelianer", "objekt:kreuzer")):
            p = b.new_page(viewport={"width": groesse[0], "height": groesse[1]}, is_mobile=groesse[0] < 768, has_touch=groesse[0] < 768)
            p.goto(url + "#volk=" + volk); time.sleep(5)
            # Website-Seite: Kopfleiste und KI-Zeile fürs Foto ausblenden, Karte füllt das Bild.
            p.evaluate("() => { document.querySelector('.sk-kopf')?.remove(); document.querySelector('.site-ai-notice')?.remove();"
                       " document.documentElement.style.setProperty('--kopfhoehe', '0px'); dispatchEvent(new Event('resize')); }")
            time.sleep(2)
            p.evaluate("(v) => document.querySelector(`[data-volk='${v}']`)?.click()", volk); time.sleep(1.5)
            art, ziel = schritt.split(":")
            if art == "bereich":
                # Lesbarer Ausschnitt: Kamera auf die Mitte des Bereichs (Stufen IV und V, wo er am dichtesten ist), Maßstab 0,85.
                p.evaluate("""(ber) => { const T = window.__techbaum, L = T.LAY(), K = window.TECHBAUM.knoten;
                  const ids = Object.keys(K).filter(i => K[i].bereich === ber && L.pos[i] && !L.pos[i].zu);
                  const xs = ids.map(i => L.pos[i].x), ys = ids.map(i => L.pos[i].y), s = .85;
                  const cx = (Math.min(...xs) + Math.max(...xs) + 96) / 2, cy = (Math.min(...ys) + Math.max(...ys) + 110) / 2 + 260;
                  const v = T.sicht(); T.zustand.cam = {s, x: v.links + (v.w - v.links - v.rechts) / 2 - cx * s, y: v.oben + (v.h - v.oben - v.unten) / 2 - cy * s};
                  document.getElementById('buehne').dispatchEvent(new WheelEvent('wheel', {deltaY: 0, clientX: 700, clientY: 400, bubbles: true})); }""", ziel)
            else:
                p.evaluate("(z) => window.__techbaum.zeigeObjekt(z)", ziel)
            time.sleep(3.5)
            p.evaluate("() => { const h = document.getElementById('hinweis'); if (h) h.style.opacity = 0; }")
            Image.open(io.BytesIO(p.screenshot(timeout=150000))).convert("RGB").save(OUT / f"{name}.webp", "WEBP", quality=86, method=6)
            liste[name + ".webp"] = {"welt": "techbaum", "ziel": url, "breite": groesse[0], "hoehe": groesse[1],
                                      "zeit": datetime.now().isoformat(timespec="seconds")}
            print(name, (OUT / f"{name}.webp").stat().st_size)
            p.close()
        b.close()
    (OUT / "fotos.json").write_text(json.dumps(liste, ensure_ascii=False, indent=1), encoding="utf-8")


def main():
    if len(sys.argv) > 2 and sys.argv[1] == "--techbaum":
        return techbaum(sys.argv[2])
    welten = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
    nur = {a for a in sys.argv[1:] if "=" not in a}
    OUT.mkdir(parents=True, exist_ok=True)
    liste = json.loads((OUT / "fotos.json").read_text(encoding="utf-8")) if (OUT / "fotos.json").exists() else {}
    with sync_playwright() as pw:
        b = pw.chromium.launch(channel="msedge", args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"])
        for welt, server in welten.items():
            for groesse in (DESKTOP, HANDY):
                auftrag = [f for f in FOTOS if f[1] == welt and (groesse == DESKTOP or f[3]) and (not nur or f[0] in nur)]
                if not auftrag:
                    continue
                fehler = []
                ctx, p = seite(b, server, groesse, fehler)
                for name, _, ziel, _ in auftrag:
                    try:
                        # Vorheriges Foto aufräumen: Techbaum-Dossier, Techinfo, Berichtansicht und andere Dialoge schließen.
                        p.evaluate("() => { window.KraefteatlasSpiel?.schliessen?.(); window.TechbaumSpiel?.schliessen?.(); window.Einsatz?.schliessen?.(); document.querySelectorAll('dialog[open]').forEach(d => d.close()); }")
                        time.sleep(1.5)
                        gehe(p, ziel)
                    except Exception as e:
                        print("NICHT ERREICHT", name, str(e)[:120]); continue
                    time.sleep(3)
                    if not ziel.startswith(("technik:", "archiv:", "info:", "atlas", "einsatz:")):
                        p.evaluate("() => window.scrollTo(0, 0)")
                    datei = name + ("-handy" if groesse == HANDY else "") + ".webp"
                    Image.open(io.BytesIO(p.screenshot(timeout=150000))).convert("RGB").save(OUT / datei, "WEBP", quality=84, method=6)
                    liste[datei] = {"welt": welt, "ziel": ziel, "breite": groesse[0], "hoehe": groesse[1],
                                    "zeit": datetime.now().isoformat(timespec="seconds")}
                    print(datei, (OUT / datei).stat().st_size)
                if fehler:
                    print("Konsole/Seitenfehler", welt, groesse, fehler[:5])
                ctx.close()
        b.close()
    (OUT / "fotos.json").write_text(json.dumps(liste, ensure_ascii=False, indent=1), encoding="utf-8")


if __name__ == "__main__":
    main()
