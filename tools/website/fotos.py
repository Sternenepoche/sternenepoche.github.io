"""Frische Bildschirmfotos des aktuellen Spiels für die neue Website (docs/website/fotos/<name>.webp).

Nur echte Oberfläche aus isolierten Prüfwelten (crates/server/examples/*_ui_fixture.rs) mit dem aktuellen Server-Build.
Aufruf: python -X utf8 tools/website/fotos.py WELT=SERVER [WELT=SERVER ...]
  z. B. flotten=http://127.0.0.1:8921 monde=http://127.0.0.1:8923 dyson=http://127.0.0.1:8925 lage=http://127.0.0.1:8927
Schreibt zusätzlich docs/website/fotos/fotos.json (Zeitpunkt, Welt, Größe je Foto).
"""
import io, json, sys, time, urllib.request
from datetime import datetime
from pathlib import Path
from PIL import Image
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/website/fotos"
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
    ("mond", "monde", "mond", True), ("grossprojekt", "dyson", "grossprojekt", True),
    ("lage", "lage", "reich", False),
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
    p.goto(server + "/")
    for _ in range(80):
        if p.evaluate("() => typeof view !== 'undefined' && !!view && !document.getElementById('game').hidden"):
            break
        time.sleep(0.5)
    p.evaluate("() => { window.SternenLoading?.ready?.(); document.getElementById('sternen-loader')?.remove(); }")
    time.sleep(2.5)
    return ctx, p


def gehe(p, ziel):
    if ziel == "mond":
        p.click("#planet-sidebar .planet-mond, .planet-mond", timeout=8000); return
    if ziel.startswith(("technik:", "archiv:")):
        teile = ziel.split(":"); gehe(p, "werft"); time.sleep(1.5)
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
        for name, groesse, schritte in (("techbaum-karte", DESKTOP, []), ("techbaum-nah", DESKTOP, ["naeher"]), ("techbaum-handy", HANDY, [])):
            p = b.new_page(viewport={"width": groesse[0], "height": groesse[1]}, is_mobile=groesse[0] < 768, has_touch=groesse[0] < 768)
            p.goto(url); time.sleep(5)
            # Website-Seite: Kopfleiste und KI-Zeile fürs Foto ausblenden, Karte füllt das Bild.
            p.evaluate("() => { document.querySelector('.sk-kopf')?.remove(); document.querySelector('.site-ai-notice')?.remove();"
                       " document.documentElement.style.setProperty('--kopfhoehe', '0px'); dispatchEvent(new Event('resize')); }")
            time.sleep(2)
            for s in schritte:
                if s == "naeher":
                    # Hineinzoomen und einen Knoten wählen: Dossier mit Bild, Werten und Pfaden.
                    p.mouse.move(groesse[0] * .42, groesse[1] * .45)
                    for _ in range(4):
                        p.mouse.wheel(0, -240); time.sleep(.4)
                    time.sleep(1)
                    p.mouse.click(466, 440)
                    time.sleep(2.5)
            Image.open(io.BytesIO(p.screenshot())).convert("RGB").save(OUT / f"{name}.webp", "WEBP", quality=86, method=6)
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
                        if p.evaluate("() => !!document.querySelector('#techinfo[open]')"):
                            p.evaluate("() => document.getElementById('techinfo')?.close()")
                        gehe(p, ziel)
                    except Exception as e:
                        print("NICHT ERREICHT", name, str(e)[:120]); continue
                    time.sleep(3)
                    if not ziel.startswith(("technik:", "archiv:")):
                        p.evaluate("() => window.scrollTo(0, 0)")
                    datei = name + ("-handy" if groesse == HANDY else "") + ".webp"
                    Image.open(io.BytesIO(p.screenshot())).convert("RGB").save(OUT / datei, "WEBP", quality=84, method=6)
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
