"""Website 0.4 für GitHub Pages vorbereiten und (erst wenn alles fertig ist) pushen.

Karl 18.10.2026: „wenn alles fertig ist push auf sternenepoche.github.io“. Regeln (COORDINATION.md, Gedächtnis
pages-veroeffentlichen): nur aus einem frischen Worktree von origin/main, nur geprüfte Dateien; Website, Spielserver und
APK-Downloadseite gehen gemeinsam raus – deshalb verweigert --push, solange docs/android-download.json nicht 0.4.0 mit
beiden Varianten (lite, voll) nennt oder die APK-Dateien noch nicht als Release erreichbar sind.

  python -X utf8 tools/website/veroeffentlichen.py --vorbereiten   Worktree repos/website-0-4 (Branch website-0-4) von origin/main,
                                                                   Dateien übernehmen, bauen.py --pages, build_pages, Prüfungen, Commit
  python -X utf8 tools/website/veroeffentlichen.py --push          Bedingungen prüfen, dann git push origin website-0-4:main
Übernommen werden nur Website-Dateien: die Seiten und ihre Quellen, Bildverzeichnis (art.js + Motive), Technologiebaum,
Schriften. Der gespiegelte Spielclient (web-client/*.js außer techbaum) bleibt unverändert – der geht mit dem Server-Update.
"""
from __future__ import annotations

import argparse, filecmp, json, os, re, shutil, subprocess, sys, urllib.request
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAGES = ROOT / "repos/pages"
WT = ROOT / "repos/website-0-4"
ZWEIG = "website-0-4"
UMGEBUNG = dict(os.environ, MSYS_NO_PATHCONV="1", PYTHONUTF8="1")

DATEIEN = [
    "tools/website/bauen.py", "tools/website/fotos.py", "tools/website/kopfvideo.py", "tools/website/pruefen.py",
    "tools/website/techbaum_logo.py", "tools/website/veroeffentlichen.py", "tools/website/heute.py", "tools/website/trailer_web.py", "tools/website/hoerchronik_web.py", "tools/build_pages.py", "tools/build_fleet_codex.py",
    "tools/website_content.py", "tools/site_policy.py", "tools/player_publication_test.py",
    "docs/action-surfaces.css", "docs/HANDBUCH.md", "docs/AGENTEN-SCHNITTSTELLE.md", "docs/flottenkodex.json", "docs/technologie-regeln.json",
    "docs/android-download.json", "docs/cockpit-download.json", "docs/artwork/hero-ozeanwelt.webp",
    "docs/branding/sternenepoche-mark.png", "docs/branding/favicon.png", "content/beschreibungen.json", "lore/Lore-Gesamtwerk.md",
    "web-client/art.js", "web-client/ai-labels.js", "web-client/three.min.js",
]
ORDNER = [  # (Ordner, erlaubte Endungen, auszuschließende Namensteile)
    ("docs/website", {".css", ".js", ".webp", ".png", ".jpg", ".json", ".webm", ".mp4", ".ogg"}, ("kandidaten", "vorlage-stern", "-frei.png", "techbaum-1801")),
    ("docs/rat", {".js", ".json", ".glb", ".ogg", ".m4a", ".webp"}, ()),
    ("web-client/techbaum", {".css", ".js"}, ()),
    ("web-client/assets/klein", {".webp"}, ()),  # Vorschaubilder der Techbaum-Kacheln (techbaum_kleinbilder.py)
    ("web-client/fonts", {".woff2", ".txt"}, ()),
]


def git(*a, cwd=WT, pruefen=True):
    r = subprocess.run(["git", *a], cwd=cwd, capture_output=True, text=True, encoding="utf-8", env=UMGEBUNG)
    if pruefen and r.returncode:
        sys.exit(f"git {' '.join(a)} fehlgeschlagen:\n{r.stdout}{r.stderr}")
    return r.stdout


def lauf(*a):
    print(">", " ".join(map(str, a)), flush=True)
    r = subprocess.run([sys.executable, "-X", "utf8", *map(str, a)], cwd=WT, env=UMGEBUNG)
    if r.returncode:
        sys.exit(f"Abbruch: {a[0]} meldet Fehler {r.returncode}")


def dateiliste() -> list[str]:
    liste = [d for d in DATEIEN if (ROOT / d).is_file()]
    for ordner, endungen, aus in ORDNER:
        for p in sorted((ROOT / ordner).rglob("*")):
            rel = p.relative_to(ROOT).as_posix()
            if p.is_file() and p.suffix.lower() in endungen and not any(x in rel for x in aus):
                liste.append(rel)
    art = json.loads((ROOT / "web-client/art.js").read_text(encoding="utf-8").removeprefix("window.STERNEN_ART=").strip().removesuffix(";"))
    liste += sorted({"web-client/" + v["url"].split("?")[0] for v in art["images"].values()}
                    | {"web-client/" + u.split("?")[0] for m in art.get("maps", {}).values() for u in (m.get("karte"), m.get("kugel")) if u})
    return liste


def vorbereiten():
    git("fetch", "origin", cwd=PAGES)
    if WT.exists():
        if git("status", "--porcelain").strip():
            sys.exit(f"{WT} hat ungesicherte Änderungen – erst ansehen")
        if git("merge-base", "HEAD", "origin/main").strip() != git("rev-parse", "origin/main").strip():
            # origin/main ist weitergelaufen (z. B. APK-Release): eigenen Worktree frisch anlegen – alles wird ohnehin neu gebaut.
            print("origin/main ist weitergelaufen – Worktree wird frisch angelegt", flush=True)
            git("worktree", "remove", "--force", str(WT), cwd=PAGES)
            git("branch", "-D", ZWEIG, cwd=PAGES)
    if not WT.exists():
        git("worktree", "add", "-b", ZWEIG, str(WT), "origin/main", cwd=PAGES)
    neu = geaendert = 0
    for rel in dateiliste():
        q, z = ROOT / rel, WT / rel
        if z.exists() and filecmp.cmp(q, z, shallow=False):
            continue
        neu += not z.exists(); geaendert += z.exists()
        z.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(q, z)
    print(f"Übernommen: {neu} neue, {geaendert} geänderte Dateien", flush=True)
    workflow_anpassen()
    lauf("tools/website/bauen.py", "--pages")
    # Neue Dateien müssen versioniert sein, bevor build_pages sie als öffentlich anerkennt.
    git("add", "-A", "--", "index.html", "Sternenepoche-Start.html", "Sternenepoche-Handbuch.html", "Sternenepoche-Kodex.html",
        "Technologiebaum.html", "Trailer.html", "Hoerchronik.html", "Weltenkruemmer.html", "Lore.html", "Harness-Cockpit.html", "Download.html", "KI-Hinweis.html",
        "sitemap.xml", "llms.txt", "seite", "docs", "tools", "content", "lore", "web-client", ".github")
    ziel = WT / ".pages-artifact" / ("pruefung-" + datetime.now().strftime("%Y%m%d-%H%M%S"))
    lauf("tools/build_pages.py", "--out", ziel)
    lauf("tools/player_publication_test.py", "--source", ziel)
    if not git("diff", "--cached", "--name-only").strip():
        print("Nichts zu committen – Stand ist schon vorbereitet.")
        return
    git("commit", "-m", f"Rebuild the public website for Sternenepoche {VERSION_TEXT()}\n\n"
        "New pages built by tools/website/bauen.py from today's sources: start page with the Weltenkruemmer film in front of\n"
        "the planet, guide, handbook (32 chapters), codex with technical drawings, the complete technology tree, Weltenkruemmer\n"
        "films, lore archive, playing with AI, downloads (Lite and full APK) and AI notice. Old screenshots are no longer packaged.\n\n"
        "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>")
    print(git("log", "--oneline", "-1"))
    print(git("diff", "--stat", "origin/main", "HEAD").splitlines()[-1])


def workflow_anpassen():
    """static.yml: Website-Pfade lösen keine Spielkern-Prüfung aus (die liefe sonst mit altem Code gegen das neue art.js)."""
    w = WT / ".github/workflows/static.yml"
    s = w.read_text(encoding="utf-8")
    alt = "not p.startswith(('docs/', 'web-client/assets/'))"
    neu = ("not p.startswith(('docs/', 'web-client/assets/', 'web-client/techbaum/', 'web-client/fonts/', 'seite/', "
           "'tools/website/', 'lore/'))")
    if alt in s:
        s = s.replace(alt, neu)
    zusatz = ("'Trailer.html', 'Hoerchronik.html', 'Technologiebaum.html', 'Weltenkruemmer.html', 'Lore.html', 'content/beschreibungen.json', "
              "'web-client/ai-labels.js', 'web-client/three.min.js',\n              ")
    if "'Trailer.html'" not in s:
        if "'Technologiebaum.html'" in s:  # frühere Fassung: nur die zwei neuen Seiten ergänzen
            s = s.replace("public_files = {'Technologiebaum.html',", "public_files = {'Trailer.html', 'Hoerchronik.html', 'Technologiebaum.html',", 1)
        else:
            s = s.replace("public_files = {'index.html',", "public_files = {" + zusatz + "'index.html',", 1)
    if neu not in s or "'Trailer.html'" not in s:
        sys.exit("static.yml hat eine unerwartete Form – Website-Pfade bitte von Hand eintragen")
    w.write_text(s, encoding="utf-8")


def VERSION_TEXT() -> str:
    return re.search(r'^VERSION = "([^"]+)"', (ROOT / "tools/website/bauen.py").read_text(encoding="utf-8"), re.M)[1]


def push():
    a = json.loads((WT / "docs/android-download.json").read_text(encoding="utf-8"))
    varianten = {v.get("art"): v for v in a.get("varianten", [])}
    if a.get("version") != VERSION_TEXT() or {"lite", "voll"} - set(varianten):
        sys.exit(f"Nicht fertig: android-download.json nennt {a.get('version')} ohne beide Varianten – kein Push.")
    for v in varianten.values():
        try:
            with urllib.request.urlopen(urllib.request.Request(v["url"], method="HEAD"), timeout=30) as r:
                if r.status >= 400:
                    raise OSError(r.status)
        except Exception as e:  # noqa: BLE001
            sys.exit(f"Nicht fertig: APK {v['art']} ist nicht öffentlich erreichbar ({e}) – kein Push.")
    if git("status", "--porcelain").strip():
        sys.exit("Worktree hat ungesicherte Änderungen – erst --vorbereiten")
    git("fetch", "origin")
    if git("merge-base", "HEAD", "origin/main").strip() != git("rev-parse", "origin/main").strip():
        sys.exit("origin/main ist weitergelaufen – erst neu vorbereiten")
    print(git("push", "origin", f"{ZWEIG}:main"))
    print("Gepusht – GitHub Pages baut jetzt (Workflow static.yml).")


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--vorbereiten", action="store_true")
    g.add_argument("--push", action="store_true")
    a = p.parse_args()
    vorbereiten() if a.vorbereiten else push()
