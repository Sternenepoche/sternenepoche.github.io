"""Ratsbühne der Website auf der CPU rendern (SwiftShader, keine GPU): je Volk ein Bild der Bühne.

Aufruf: python tools/rat_buehne_pruefen.py [BASIS-URL] [ZIELORDNER]
  BASIS-URL  Standard http://127.0.0.1:18780/index.html (Webserver auf die Projektwurzel), auch die Live-Seite geht
  ZIELORDNER Standard laeufe/website-pruefung/rat
Rendert auf der CPU (SwiftShader), damit laufende GPU-Aufträge nicht gestört werden."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
basis = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:18780/index.html"
ziel = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "laeufe" / "website-pruefung" / "rat"
ziel.mkdir(parents=True, exist_ok=True)
with sync_playwright() as p:
    b = p.chromium.launch(channel="chrome", headless=True,
                          args=["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--disable-gpu"])
    s = b.new_page(viewport={"width": 1280, "height": 900}, bypass_csp=True)  # nur für wait_for_function im Test
    fehler = []
    s.on("pageerror", lambda e: fehler.append(str(e)[:200]))
    s.on("console", lambda m: fehler.append(m.text[:200]) if m.type == "error" else None)
    s.goto(basis + "#rat", wait_until="load")
    print("WebGL:", s.evaluate("""() => { const g = document.createElement('canvas').getContext('webgl');
        const e = g && g.getExtension('WEBGL_debug_renderer_info'); return g ? g.getParameter(e ? e.UNMASKED_RENDERER_WEBGL : g.RENDERER) : 'kein WebGL'; }"""))
    s.evaluate("document.getElementById('rat-buehne').scrollIntoView({block: 'center'})")
    for volk in ["veyari", "aurelianer", "krath", "syntheten"]:
        s.evaluate(f"document.querySelector('[data-rat-volk={volk}]').click()")
        s.wait_for_function("document.querySelector('.rat-hinweis').textContent.startsWith('Ton an') && !!document.querySelector('#rat-buehne canvas')", timeout=120000)
        s.wait_for_timeout(2500)
        s.locator("#rat-buehne").screenshot(path=str(ziel / f"buehne-{volk}.png"))
        print(volk, "ok")
    # Nahaufnahme: Orrin stellt sich vor (Kamera fährt zum Kopf).
    s.evaluate("document.querySelector('[data-rat-volk=veyari]').click()")
    s.wait_for_function("document.querySelector('.rat-hinweis').textContent.startsWith('Ton an')", timeout=120000)
    s.wait_for_timeout(1500)
    s.evaluate("[...document.querySelectorAll('#rat-karten button')].find((k) => (k.getAttribute('aria-label') || '').startsWith('Orrin')).click()")
    s.wait_for_timeout(3500)
    s.locator("#rat-buehne").screenshot(path=str(ziel / "buehne-orrin-nah.png"))
    print("fehler:", fehler[:6])
    b.close()
