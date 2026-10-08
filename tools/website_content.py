"""One reviewed image/meaning map for landing page, guide and handbook."""
import base64
import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Actual screenshots captured from this client's isolated Rust demonstration world.
CAPTIONS = {
 'reich': ('Dein Reich im Überblick', 'Links wechselst du den Aufgabenbereich, oben liest du Vorräte, rechts wählst du den Planeten. Beginne mit Bevölkerung, Energie und Stabilität.'),
 'gebaeude': ('Gebäude auswählen und ausbauen', 'Eine Bildkachel öffnet die Anlage. Lies Wirkung, nächste Stufe, Kosten und Voraussetzungen, bevor du den Ausbau in Auftrag gibst.'),
 'kolonie': ('Versorgung statt nur Lagerbestand prüfen', 'Bestand ist dein Vorrat; die Bilanz pro Spielstunde zeigt seine Entwicklung. Eine negative Bilanz macht die Reichweite zum entscheidenden Warnsignal.'),
 'forschen': ('Ein Projekt für dein ganzes Reich', 'Wähle links eine Technologie und prüfe rechts Voraussetzungen und Kosten. Ein Forschungsprojekt läuft reichsweit; bezahlt wird sein Start auf der Heimatwelt.'),
 'werft': ('Bestellen ist noch kein fertiges Schiff', 'Wähle den Schiffstyp, lies seine Aufgabe und stelle die Stückzahl ein. Erst nach der Fertigung steht das Schiff für eine Mission bereit.'),
 'verteidigung': ('Den Planeten absichern', 'Geschütze verteidigen die gewählte Welt. Vergleiche Kosten und Voraussetzungen und plane Energie, Forschung und eine bezahlbare Flotte mit ein.'),
 'flotten': ('Ein Flug besteht aus mehr als dem Hinweg', 'Prüfe Startplanet, Schiffe, Mission, Ziel, Treibstoff, Fracht und Rückkehr. Saven hält Schiffe während ihres Fluges vom bedrohten Planeten fern.'),
 'karte': ('Der Sternenatlas zeigt dein Wissen', 'Gehe von der Galaxie über den Sektor zum System. Unbekannte Welten bleiben verborgen; erst eigene Sondenberichte geben belastbare Informationen.'),
 'kolonien': ('Eine neue Heimat sorgfältig vorbereiten', 'Ein geeignetes Ziel allein genügt nicht. Prüfe Aufklärung, Kolonieschiff, bewaffnete Eskorte, Siedler, Startfracht und deine Koloniegrenze.'),
 'kampf': ('Angriff, Verteidigung und Verbände', 'Eine gewonnene Schlacht überträgt noch keinen Planeten. Flottenkampf, Bodenkampf und die spätere Kolonisierung haben eigene Voraussetzungen.'),
 'aktionen': ('Handel und Diplomatie verbinden Reiche', 'Markt, private Nachrichten und Verträge haben unterschiedliche Wirkungen. Ein Handel ist erst mit der Lieferung abgeschlossen.'),
 'berichte': ('Berichte sind Beobachtungen mit Zeitstempel', 'Hier kontrollierst du abgeschlossene Aktionen und gewonnenes Wissen. Ein alter Spionagebericht beschreibt den beobachteten Zeitpunkt, nicht automatisch den jetzigen Zustand.'),
 'imperium': ('Wachstum an mehreren Werten erkennen', 'Vergleiche die öffentlich sichtbare Entwicklung. Eine große Rohstoffreserve allein ist kein Sieg: Investitionen, Forschung, Bevölkerung und Streitkräfte tragen zur Wertung bei.'),
 'freischaltungen': ('Vom Ziel rückwärts planen', 'Der Technologiebaum macht Abhängigkeiten sichtbar. Prüfe Gebäude, Forschung und Zivilisationsstufe, bevor du Rohstoffe für dein nächstes Ziel zurücklegst.'),
 'agent': ('Aufgaben bewusst an Modelle vergeben', 'Aktiviere zunächst eine Rolle und prüfe ihre Entscheidungen. Der Browser-Agent benötigt den offenen Tab und eine erreichbare Modellverbindung.'),
 'profil': ('Dein Reich bleibt dein Reich', 'Hier änderst du die Spielweise. Volk und Teilnehmerplatz bleiben in der laufenden Epoche gebunden; ein Wechsel erzeugt kein neues Reich.'),
 'regeln': ('Die aktuellen Voraussetzungen im Spiel nachlesen', 'Der Browser zeigt die Regeln der aktiven Welt. Sie sind bei konkreten Bau-, Forschungs- und Flugentscheidungen maßgeblich.'),
}

CHAPTER_SCREENS = {1:'reich',3:'profil',6:'agent',7:'gebaeude',8:'reich',9:'gebaeude',10:'kolonie',11:'freischaltungen',12:'karte',13:'berichte',14:'flotten',15:'kampf',16:'kolonien',17:'aktionen',18:'kolonie',19:'imperium',21:'regeln'}
CHAPTER_ART = {
 2:('backgrounds.command.webp','Eine gemeinsame Welt, drei Zugänge','Du bedienst dein Reich im Browser. Der Rust-Server speichert die Welt. Das private Dashboard dient dem Betreiber.'),
 4:('portraits.veyari.veyari.webp','Dein Volk verändert deine Planung','Volksboni verschieben Stärken und Schwächen. Wähle bewusst: Das Volk bleibt für diese Epoche festgelegt.'),
 5:('role_portraits.stratege.aurelianer.webp','Vier Aufgaben für eine Regierung','Stratege, Verwalter, Feldherr und Diplomat können menschlich oder von einem Modell geführt werden. Serverbots sind eigenständige Reiche.'),
 20:('backgrounds.archive.webp','Der Betreiber bewahrt die Welt','Ein Neustart setzt den gespeicherten Stand fort. Eine neue Epoche ist ein ausdrücklicher Reset; sichere vorher ein Backup.'),
 22:('ships.spionagesonde.aurelianer.webp','Von der Sonde zum Saven','Die Begriffe gehören zusammen: Aufklärung liefert Wissen, Flüge kosten Zeit und Treibstoff, Versorgung hält dein Reich handlungsfähig.'),
}

def screenshot(key, compact=False):
    title, caption = CAPTIONS[key]
    return (f'<figure class="screenshot"><button type="button" class="zoom" aria-label="Screenshot vergrößern: {html.escape(title)}">'
            f'<img src="screen://{key}.webp" alt="Aktuelle Browseroberfläche: {html.escape(title)}" width="1440" height="1000" loading="lazy" decoding="async">'
            '<span class="zoom-label" aria-hidden="true">Vergrößern ↗</span></button>'
            f'<figcaption><b>Echte Spieloberfläche · Beispielwelt · 08.10.2026</b>{html.escape(caption)}</figcaption></figure>')

def illustration(name, title, caption):
    return (f'<figure class="illustration"><img src="asset://{name}" alt="{html.escape(title)} – Spielillustration" loading="lazy" width="480" height="360">'
            f'<figcaption><strong>{html.escape(title)}</strong>{html.escape(caption)}</figcaption></figure>')

def chapter_visual(number):
    if number in CHAPTER_SCREENS: return screenshot(CHAPTER_SCREENS[number])
    return illustration(*CHAPTER_ART[number]) if number in CHAPTER_ART else ''

SPACE = '''<svg width="0" height="0" style="position:absolute" aria-hidden="true"><defs><filter id="asset-cutout" color-interpolation-filters="sRGB"><feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  3 3 3 0 -.3"/></filter></defs></svg><div class="space-backdrop" aria-hidden="true" data-ai-image><div class="starfield"></div><img class="space-rock" src="asset://planets.asteroid.webp" alt="" width="82" height="82"><img class="space-rock second" src="asset://planets.asteroid.webp" alt="" width="36" height="36"><img class="space-rock third" src="asset://planets.asteroid.webp" alt="" width="132" height="132"></div>
<button class="motion-control" id="space-toggle" type="button" aria-pressed="false" hidden>Weltraumbewegung pausieren</button>'''
HERO_ART = '''<div class="hero-art" aria-hidden="true"><div class="hero-orbit"></div><img class="hero-planet" src="asset://planets.leben_ozean.webp" alt="" width="480" height="480"><img class="hero-ship" src="asset://ships.kreuzer.aurelianer.webp" alt="" width="480" height="288"></div><div class="coordinate" aria-hidden="true"><b>DEIN NÄCHSTES KAPITEL</b>HEIMATWELT / ORBIT ERREICHT<br>AUFBAU · FORSCHUNG · ENTDECKUNG</div>'''
DIALOG = '''<dialog class="image-dialog" id="image-dialog" aria-labelledby="image-dialog-title"><div class="dialog-toolbar"><span id="image-dialog-title">Spielansicht</span><button type="button" aria-label="Vergrößerten Screenshot schließen">✕</button></div><img alt=""></dialog>'''

def finish(document):
    document = document.replace('<!-- SITE_THEME -->','<style>'+ (ROOT/'docs/site-theme.css').read_text(encoding='utf-8')+'</style>')
    document = document.replace('<!-- SITE_MOTION -->','<script>'+ (ROOT/'docs/site-motion.js').read_text(encoding='utf-8')+'</script>')
    document = document.replace('<!-- SPACE -->',SPACE).replace('<!-- HERO_ART -->',HERO_ART).replace('<!-- IMAGE_DIALOG -->',DIALOG)
    embedded = {}
    def embed(match):
        kind,name = match.groups()
        file = ROOT/('web-client/assets' if kind=='asset' else 'docs/bilder/online')/name
        if not file.is_file(): raise ValueError('Missing public image: '+str(file))
        if (kind,name) not in embedded:
            embedded[kind,name] = 'data:image/webp;base64,'+base64.b64encode(file.read_bytes()).decode('ascii')
        return embedded[kind,name]
    document = re.sub(r'(asset|screen)://([a-zA-Z0-9_.-]+)',embed,document)
    if re.search(r'<!-- (?:SITE_|SPACE|HERO_ART|IMAGE_DIALOG)',document): raise ValueError('Unresolved website placeholder')
    return document
