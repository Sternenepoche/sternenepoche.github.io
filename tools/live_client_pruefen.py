"""Live-Spielclient prüfen, ohne anzumelden: lädt ohne Fehler, zeigt den Server mit Namen statt Adresse, Cockpit-Look aktiv.

Aufruf: python tools/live_client_pruefen.py [SPIEL-URL] [BILD]
  SPIEL-URL Standard https://sternenepoche.taila4f584.ts.net/ (auch https://sternenepoche.github.io/web-client/ prüfen)
Rendert auf der CPU (SwiftShader), damit laufende GPU-Aufträge nicht gestört werden."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
url = sys.argv[1] if len(sys.argv) > 1 else "https://sternenepoche.taila4f584.ts.net/"
bild = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "laeufe" / "website-pruefung" / "live-client.png"
bild.parent.mkdir(parents=True, exist_ok=True)
with sync_playwright() as p:
    b = p.chromium.launch(channel="chrome", headless=True,
                          args=["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--disable-gpu"])
    s = b.new_page(viewport={"width": 1440, "height": 900})
    # Auf dem Server-PC selbst löst Tailscale die Spieladresse lokal auf; Chrome verlangt dann die Freigabe für das
    # lokale Netz (bei fremden Spielern läuft es über das öffentliche Relay). Für die Prüfung wird sie erteilt.
    herkunft = "/".join(url.split("/")[:3])
    cdp = b.new_browser_cdp_session()
    kontext = cdp.send("Target.getBrowserContexts")["browserContextIds"][0]
    for recht in ["local-network-access", "loopback-network", "local-network"]:
        try:
            cdp.send("Browser.setPermission", {"permission": {"name": recht}, "setting": "granted", "origin": herkunft, "browserContextId": kontext})
        except Exception:
            pass
    fehler = []
    s.on("pageerror", lambda e: fehler.append(str(e)[:200]))
    s.on("response", lambda r: fehler.append(f"HTTP {r.status} {r.url}") if r.status >= 400 else None)
    s.goto(url, wait_until="load")
    s.wait_for_timeout(6000)
    verbindung = s.evaluate("document.getElementById('connection')?.textContent || ''")
    look = s.evaluate("[...document.styleSheets].some((x) => (x.href || '').endsWith('cockpit-look.css'))")
    adresse_sichtbar = s.evaluate("/\\.ts\\.net/.test(document.body.innerText)")
    s.screenshot(path=str(bild))
    print("Verbindung:", verbindung)
    print("Cockpit-Look:", look, "| Adresse im sichtbaren Text:", adresse_sichtbar)
    print("Fehler:", fehler[:8] or "keine")
    b.close()
    ok = verbindung.startswith("Verbunden · Sternenepoche-Server") and look and not adresse_sichtbar and not fehler
    print("PASS" if ok else "FEHLER")
    sys.exit(0 if ok else 1)
