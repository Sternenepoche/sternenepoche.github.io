"""Browserprüfung der neuen Website (Vorschau ausgabe/website/ über einen lokalen Webserver auf dem Projektordner).

Je Seite Desktop 1440×900 und Handy 375×812 (Edge, ohne „Bewegung reduzieren“): Konsolen- und Seitenfehler, fehlgeschlagene
Anfragen, Querscrollen (scrollWidth > Fensterbreite), Bildschirmfoto. Startseite zusätzlich Filmbilder des Kopfes,
Kodex mit Auswahl und Zeichnung, Handbuch-Suche, Technologiebaum geladen.
Aufruf: python -X utf8 tools/website/pruefen.py [http://127.0.0.1:8765/ausgabe/website/]
Ergebnis: laeufe/website-2026-10-18/pruefung/ (Fotos + pruefung.json)
"""
import json, sys, time
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[2]
BASIS = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8765/ausgabe/website/"
OUT = ROOT / "laeufe/website-2026-10-18/pruefung"
SEITEN = ["index.html", "Sternenepoche-Start.html", "Sternenepoche-Handbuch.html", "Sternenepoche-Kodex.html", "Technologiebaum.html",
          "Weltenkruemmer.html", "Lore.html", "Harness-Cockpit.html", "Download.html", "KI-Hinweis.html"]
GROESSEN = {"desktop": (1440, 900), "handy": (375, 812)}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    bericht, fehlerzahl = {}, 0
    with sync_playwright() as pw:
        b = pw.chromium.launch(channel="msedge", args=["--autoplay-policy=no-user-gesture-required"])
        for art, (w, h) in GROESSEN.items():
            ctx = b.new_context(viewport={"width": w, "height": h}, reduced_motion="no-preference",
                                is_mobile=w < 768, has_touch=w < 768, device_scale_factor=1)
            for seite in SEITEN:
                p = ctx.new_page()
                fehler, kaputt = [], []
                p.on("console", lambda m: fehler.append(m.text[:200]) if m.type == "error" else None)
                p.on("pageerror", lambda e: fehler.append("Seitenfehler: " + str(e)[:200]))
                p.on("response", lambda r: kaputt.append(f"{r.status} {r.url}") if r.status >= 400 else None)
                p.goto(BASIS + seite, wait_until="load")
                time.sleep(2.5)
                breite = p.evaluate("() => ({doc: document.documentElement.scrollWidth, fenster: innerWidth,"
                                    " zu_breit: [...document.querySelectorAll('body *')].filter(e => { const r = e.getBoundingClientRect();"
                                    " return r.width && r.right > innerWidth + 1 && getComputedStyle(e).position !== 'fixed' && !e.closest('.tabelle,.matrix-scroll,.tabelle-scroll,.app,.buehne-szene'); })"
                                    ".slice(0, 6).map(e => e.tagName + '.' + String(e.className).slice(0, 40) + ':' + Math.round(e.getBoundingClientRect().right))})")
                eintrag = {"fehler": fehler[:], "kaputt": kaputt[:], "breite": breite}
                name = seite.replace(".html", "")
                if seite == "index.html":
                    for t in (1.5, 4.5, 8.0):
                        time.sleep(t if t == 1.5 else 3.5)
                        p.screenshot(path=str(OUT / f"{art}-{name}-film-{t:.0f}.png"))
                    eintrag["film"] = p.evaluate("() => { const v = document.querySelector('.buehne-film video');"
                                                 " return v ? {src: v.currentSrc.split('/').pop(), zeit: v.currentTime, laeuft: !v.paused} : null; }")
                    eintrag["hud"] = p.evaluate("() => document.querySelector('.buehne-hud b')?.textContent")
                if seite == "Sternenepoche-Kodex.html":
                    p.locator("#gallery [data-unit='kreuzer']").first.click()
                    time.sleep(1.5)
                    eintrag["kodex"] = p.evaluate("() => ({titel: document.querySelector('#detail h2')?.textContent,"
                                                  " zeichnung: !!document.querySelector('.kodex-zeichnung img'), archiv: !!document.querySelector('.codex-archiv')})")
                    p.locator("[data-stil='explosion']").first.click() if p.locator("[data-stil='explosion']").count() else None
                    time.sleep(1)
                if seite == "Sternenepoche-Handbuch.html":
                    if w >= 900:
                        p.fill(".leiste input[type=search]", "Phalanx")
                        time.sleep(.5)
                        eintrag["suche"] = p.evaluate("() => document.getElementById('suche-status')?.textContent")
                        p.fill(".leiste input[type=search]", "")
                if seite == "Technologiebaum.html":
                    time.sleep(2)
                    eintrag["techbaum"] = p.evaluate("() => ({knoten: document.querySelectorAll('.knoten').length,"
                                                     " app_oben: Math.round(document.querySelector('.app')?.getBoundingClientRect().top || -1),"
                                                     " kopf_unten: Math.round(document.querySelector('.sk-kopf').getBoundingClientRect().bottom)})")
                p.screenshot(path=str(OUT / f"{art}-{name}.png"), full_page=False)
                if seite not in ("Technologiebaum.html",):
                    p.screenshot(path=str(OUT / f"{art}-{name}-ganz.png"), full_page=True) if seite in ("index.html", "Download.html", "Weltenkruemmer.html") else None
                quer = breite["doc"] > breite["fenster"]
                if fehler or kaputt or quer:
                    fehlerzahl += 1
                print(f"{art:7} {seite:30} Fehler {len(fehler)} · kaputt {len(kaputt)} · {'QUERSCROLLEN ' + str(breite) if quer else 'ok'}"
                      + (f" · {eintrag.get('film') or eintrag.get('kodex') or eintrag.get('techbaum') or eintrag.get('suche') or ''}"), flush=True)
                for f in (fehler + kaputt)[:6]:
                    print("        ", f)
                bericht[f"{art}/{seite}"] = eintrag
                p.close()
            ctx.close()
        b.close()
    (OUT / "pruefung.json").write_text(json.dumps(bericht, ensure_ascii=False, indent=1), encoding="utf-8")
    print("Seiten mit Befund:", fehlerzahl)
    sys.exit(1 if fehlerzahl else 0)


if __name__ == "__main__":
    main()
