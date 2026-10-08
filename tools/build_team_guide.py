"""Keep the public handbook and in-game beginner guide in sync with annotated UI shots."""
from pathlib import Path
import re,shutil
from bs4 import BeautifulSoup
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
ILLUSTRATIONS=[
 ('start','01-modelle','Die Modellzentrale im Überblick','1 Modell hinzufügen · 2 Anbieter wählen · 3 lokale API-Adresse · 4 Modellliste laden · 5 exakte Modell-ID auswählen · 6 diese Verbindung testen.'),
 ('ollama','02-ollama','Ollama: die richtigen Felder','1 Teamname frei wählen · 2 Adresse des laufenden lokalen Ollama · 3 installierte Modelle abfragen · 4 Modell-ID übernehmen · 5 Antwort testen. Der gezeigte Modellname ist ein Beispiel aus der Testinstallation; wähle ein auf deinem Rechner installiertes Modell.'),
 ('openrouter','03-openrouter','OpenRouter: Schlüssel und Modell sind zwei verschiedene Angaben','1 Anbieter OpenRouter · 2 hier den eigenen API-Key eintragen · 3 Modellkatalog öffnen · 4 echte Modell-ID wählen · 5 Test ausführen. „openai/gpt-4.1-mini“ ist eine beim Erstellen im OpenRouter-Katalog geprüfte Beispiel-ID. Wähle dein gewünschtes Modell aus der aktuellen Liste. Das Schlüsselfeld ist im Bild leer.'),
 ('rollen','04-rollen','Eine Rolle individuell einrichten','1 Teammitglied zuordnen · 2 Persönlichkeit in soul.md · 3 konkrete Aufgaben in rollen.md · 4 eigene Markdown-Datei importieren; vorher daneben das Importziel wählen. Änderungen anschließend speichern.'),
 ('test','05-browserfreigabe','Die passende Browserfreigabe ablesen','1 Ursprung der tatsächlich geöffneten Spielseite · 2 dazu passender PowerShell-Befehl. Im Bild ist ein lokaler Spieleinstieg geöffnet. Auf der öffentlichen Website steht hier deren HTTPS-Ursprung; übernimm deshalb den Text aus deinem eigenen Spiel.'),
 ('pinwaende','06-pinwaende','Wissen als Pinwände und Karten organisieren','1 weitere Pinwand erstellen · 2 Bearbeitungen dauerhaft speichern · 3 Reihenfolge ändern · 4 Karte für Wissen, Ziel oder Übergabe hinzufügen. Beispielinhalte stammen aus einer isolierten Testwelt.')]

def build():
    source=ROOT/'web-client/agenten-hilfe.html'
    soup=BeautifulSoup(source.read_text(encoding='utf-8'),'html.parser')
    beginner_sections={
      'ollama':'''<h3 id="ollama-erste-installation">Noch kein Ollama und noch kein Modell? Beginne hier.</h3><ol><li>Öffne <a href="https://ollama.com/download">ollama.com/download</a>, wähle dein Betriebssystem und lade den offiziellen Installer. Unter Windows heißt er OllamaSetup.exe. Öffne ihn und führe die Installation zu Ende.</li><li>Plane freien Speicher für Programm und Modelle ein. Modelle sind eigene Downloads und können deutlich größer als der Installer sein. Ein bereits vorhandenes Modell musst du nicht nochmals herunterladen.</li><li>Starte Ollama aus dem Startmenü. Es kann im Hintergrund laufen. Öffne anschließend eine <strong>neue</strong> PowerShell: Windows-Taste drücken, „PowerShell“ eingeben und öffnen. Gib <code>ollama --version</code> ein und drücke Enter. Erwartet wird eine Versionsnummer. Wird der Befehl nicht gefunden, schließe die PowerShell nach der Installation und öffne sie erneut.</li><li>Gib <code>ollama list</code> ein. Siehst du Modellnamen, kannst du eines davon später im Spiel auswählen. Ist die Liste leer, benötigst du zunächst ein Modell.</li><li>Öffne die <a href="https://ollama.com/library">Modellbibliothek</a>. Wähle ein Text-/Chatmodell und einen konkreten Tag. Embedding-, Sprachtranskriptions- oder reine Bildmodelle sind keine Regierungsassistenten.</li><li>Als kleines, tatsächlich getestetes Einrichtungsbeispiel zeigen unsere Bilder <a href="https://ollama.com/library/qwen2.5-coder:1.5b">qwen2.5-coder:1.5b</a>. Zum Herunterladen lautet der vollständige Befehl <code>ollama pull qwen2.5-coder:1.5b</code>. Warte, bis der Download erfolgreich abgeschlossen ist. Dieses kleine Modell ist ein Technikbeispiel, keine Garantie für gute Spielstrategie. Du kannst stattdessen ein anderes geeignetes Modell aus der Bibliothek wählen.</li><li>Prüfe mit <code>ollama list</code>, dass der Name jetzt erscheint. Optional öffnest du <code>ollama run qwen2.5-coder:1.5b</code>, gibst eine kurze Frage ein und beendest diesen Terminalchat mit <code>/bye</code>. Ollama selbst soll weiterlaufen.</li><li>Öffne auf demselben Rechner <a href="http://127.0.0.1:11434/api/tags" target="_blank" rel="noopener">http://127.0.0.1:11434/api/tags</a>. Erwartet wird JSON-Text mit einer Modellliste. Eine erreichbare API ist die Grundlage; die separate Freigabe für die Spielwebsite folgt weiter unten. Starte nicht zusätzlich ein zweites <code>ollama serve</code>, wenn die Desktop-App den Port bereits bedient.</li></ol><p>Danach wechselst du zurück ins Spiel und arbeitest die fünf Pfeile im Bild ab. Die <a href="https://docs.ollama.com/windows">offizielle Windows-Anleitung</a> erklärt abweichende Installationen und Speicherorte.</p>''',
      'openrouter':'''<h3 id="openrouter-erstes-konto">Noch kein OpenRouter-Konto und noch kein API-Key?</h3><ol><li>Öffne <a href="https://openrouter.ai">OpenRouter</a> in einem eigenen Tab. Melde dich an oder erstelle über „Sign Up“ ein Konto und schließe die dort verlangten Bestätigungsschritte ab.</li><li>Öffne nach der Anmeldung die <a href="https://openrouter.ai/settings/keys">API-Schlüssel-Verwaltung</a>. Ein normales Websitepasswort gehört nicht in Sternenepoche: Du brauchst einen eigenen API-Key.</li><li>Erstelle einen neuen Schlüssel, gib ihm einen wiedererkennbaren Namen wie „Sternenepoche“ und setze das angebotene Ausgaben-/Credit-Limit. Prüfe bei einem kostenpflichtigen Modell außerdem, ob dein Anbieter-Konto Guthaben bzw. eine nutzbare Abrechnung hat. Sternenepoche kauft kein Guthaben für dich.</li><li>Kopiere den neu angezeigten Schlüssel über die Kopierfunktion. Verwahre ihn privat, wenn du ihn später erneut verwenden möchtest. Poste ihn weder auf einer Pinwand noch in einem Rollentext oder Chat.</li><li>Wechsle zurück zur Modellkarte im Spiel. Pfeil 2 zeigt das Passwortfeld <strong>OpenRouter API-Key</strong>. Füge dort ausschließlich den Schlüssel ein.</li><li>Die Modellkennung ist eine zweite, unabhängige Angabe. Klicke <strong>Modelle laden</strong>, wähle den gewünschten Katalogeintrag und prüfe die übernommene Modell-ID unter Pfeil 4. Jetzt erst führst du den Test unter Pfeil 5 aus. Bei HTTP 401 prüfst du den Schlüssel, bei HTTP 402 Guthaben/Limits, bei HTTP 404 die Modell-ID.</li></ol><p>Ein im Browser eingeloggtes OpenRouter-Konto ersetzt das Schlüsselfeld nicht. Weitere Einzelheiten stehen in der <a href="https://openrouter.ai/docs/api_reference/authentication">offiziellen OpenRouter-Anleitung zu API-Schlüsseln</a>.</p>'''}
    for anchor,content in beginner_sections.items():
        marker='ollama-erste-installation' if anchor=='ollama' else 'openrouter-erstes-konto'
        if not soup.find(id=marker):
            extra=BeautifulSoup(content,'html.parser')
            for node in reversed(list(extra.contents)):soup.find(id=anchor).insert_after(node)
    for p in soup.select('p'):
        if 'Jede Rolle liest vor jedem Zug' in p.get_text():
            p.clear();p.append('Jede Rolle liest vor jedem Zug das aktuelle Lagebild, ihre soul.md und rollen.md, die Namen und Rollen der Kollegen sowie die letzten bestätigten Übergaben. Bei überschaubaren Pinwänden erhält sie alle Karten. Bei umfangreichem Wissen werden offene und an die Rolle gerichtete Karten bevorzugt; die Modelle können weitere Karten über ein lesendes Pinwandwerkzeug seitenweise nachschlagen. Der vollständige Inhalt bleibt gespeichert. Modelle dürfen bis zu vier Karten pro Zug anlegen oder aktualisieren. Sie können keine ganze Wand löschen. Handelswünsche auf einer Karte sind interne Absprachen; der eigentliche Handel benötigt gültige Spielaktionen. Abgelehnte Aktionen erscheinen ausdrücklich im Übergabeprotokoll und können neue Karten nicht als erledigt bestätigen.')
    if not soup.find(id='spielprobe'):
        extra=BeautifulSoup('''<h3 id="spielprobe">Vor dem ersten echten Zug: eine Spielentscheidung proben</h3><p>Ein kurzer Verbindungstest prüft die Technik. Mit <strong>Spielentscheidung proben · ohne Befehle</strong> prüfst du zusätzlich, ob dein Modell mit deinem aktuellen Spielstand und den Rollentexten umgehen kann. Dafür musst du angemeldet sein und einen Spielplatz besitzen; die Probe selbst verändert das Spiel nicht.</p><ol><li>Bestehe zuerst den Verbindungstest. Weise das Modell nach Wunsch einer Rolle zu und bearbeite deren Texte. Die Probe verwendet die erste dem Modell zugeordnete Rolle; ohne Zuordnung verwendet sie den Verwalter.</li><li>Klicke direkt auf der Modellkarte auf <strong>Spielentscheidung proben · ohne Befehle</strong>.</li><li>Das Modell liest das aktuelle Lagebild, die Rollenbeschreibung und deine Pinwände. Falls es lesende Werkzeuge anfordert, holt die Probe diese Informationen und erlaubt eine zweite Antwort.</li><li>Im Protokoll unterhalb der Einstellungen steht deutlich <strong>SPIELPROBE · KEINE BEFEHLE AUSGEFÜHRT</strong>. Lies die Begründung, vorgeschlagenen Aktionen und die Übergabe. Passt die Absicht zu deinen Vorgaben?</li><li>Bei einem Formatfehler oder einer unpassenden Antwort passe Modell oder Rollenbeschreibung an und wiederhole die Probe. Erst wenn du zufrieden bist, speichere und starte das Team.</li></ol><p>Die Probe benötigt höchstens zwei Modellaufrufe und kann bei OpenRouter Kosten verursachen. Sie sendet keine Spielbefehle und speichert keine Modellnotizen. Ob ein Vorschlag im Ausführungszeitpunkt bezahlbar und regelkonform ist, prüft weiterhin der Spielserver. Ein bestandener Test ist daher keine Garantie für fehlerfreie Strategie.</p>''','html.parser')
        for node in list(extra.contents):soup.find(id='rollen').insert_before(node)
    if not soup.find(id='browser-berechtigung'):
        extra=BeautifulSoup('''<h3 id="browser-berechtigung">Die Websiteberechtigung im Browser finden</h3><ol><li>Klicke links neben der Adresse deiner Spielseite auf das Symbol für Websiteinformationen / Website-Steuerelemente. Die genaue Form unterscheidet sich je nach Browser.</li><li>Öffne die Websiteeinstellungen bzw. Berechtigungen dieser Seite. Suche nach „Lokales Netzwerk“, „Local network“ oder „Loopback network“. Neuere Chrome-Versionen können den Zugriff auf den eigenen Rechner separat als Loopback führen.</li><li>Erlaube den Zugriff auf den eigenen Rechner für deine Spielseite. Ändere nicht pauschal die Berechtigungen aller Websites.</li><li>Lade die Spielseite neu, öffne „Modelle &amp; Team“ und lade die Modellliste erneut. Bei einem neuen Testprofil kann der Browser die Nachfrage auch erst bei „Modelle laden“ oder beim Test anzeigen.</li></ol><p>Diese Browserberechtigung und OLLAMA_ORIGINS sind zwei getrennte Freigaben: Der Browser erlaubt die lokale Verbindung; Ollama erlaubt deiner konkreten Website das Lesen seiner Antwort. Beide müssen passen. <a href="https://github.com/GoogleChrome/modern-web-guidance/blob/main/skills/modern-web-guidance/guides/security/local-network-access.md">Technische Erläuterung von Google Chrome</a>.</p>''','html.parser')
        heading=next(h for h in soup.select('h3') if 'Zwei lokale Modelle' in h.get_text())
        for node in list(extra.contents):heading.insert_before(node)
    for old in soup.select('.team-guide-figure'):old.decompose()
    for anchor,name,title,caption in ILLUSTRATIONS:
        img=ROOT/'docs/bilder/team'/f'{name}.png'
        out=img.with_suffix('.webp')
        with Image.open(img) as im:im.save(out,'WEBP',quality=88,method=6)
        (ROOT/'web-client/guide').mkdir(exist_ok=True)
        shutil.copy2(out,ROOT/'web-client/guide'/f'team-{name}.webp')
        figure=soup.new_tag('figure',attrs={'class':'team-guide-figure'})
        link=soup.new_tag('a',href=f'guide/team-{name}.webp',target='_blank',rel='noopener')
        link.append(soup.new_tag('img',src=f'guide/team-{name}.webp',alt=title+' – '+caption,loading='lazy'));figure.append(link)
        c=soup.new_tag('figcaption');c.string=title+'. '+caption+' Zum Vergrößern das Bild anklicken oder antippen.';figure.append(c)
        soup.find(id=anchor).insert_after(figure)
    source.write_text(str(soup),encoding='utf-8')
    main=soup.select_one('main')
    for node in list(main.children):
        if getattr(node,'name',None)=='h2':break
        node.extract()
    for node in main.select('h3'):node.name='h4'
    for node in main.select('h2'):node.name='h3'
    for node in main.select('[id]'):node['id']='team-'+node['id']
    for node in main.select('a[href^="#"]'):node['href']='#team-'+node['href'][1:]
    for node in main.select('img'):node['src']=node['src'].replace('guide/team-','bilder/team/')
    for node in main.select('a[href^="guide/team-"]'):node['href']=node['href'].replace('guide/team-','bilder/team/')
    # Raw HTML keeps the same lists, exact labels, illustrations and anchors in both readers.
    chapter='## 20 Deinen eigenen KI-Assistenten verbinden\n\nDiese bebilderte Anleitung führt dich durch die neue Oberfläche **Modelle & Team**. Folge den nummerierten Pfeilen und prüfe nach jedem Schritt das beschriebene Ergebnis.\n\n'+''.join(str(n) for n in main.children)+'\n\n'
    p=ROOT/'docs/HANDBUCH.md';text=p.read_text(encoding='utf-8')
    text=re.sub(r'^## 20 .*?(?=^## 21 )',lambda _:chapter,text,flags=re.M|re.S)
    text=text.replace('**Agentensteuerung**','**Modelle & Team**').replace('### Agentensteuerung','### Modelle & Team').replace('**Lokale Modelle suchen**','**Modelle laden**').replace('**Modell testen · keine Spielaktionen**','**Verbindung & JSON-Antwort testen**').replace('**Agent starten**','**Team starten / fortsetzen**').replace('**Agent stoppen / Steuerung übernehmen**','**Team stoppen**').replace('**Steuerung übernehmen**','**Team stoppen**')
    a=text.index('### Der Ablauf im Browser',text.index('## 6 '));b=text.index('### Lokale Modelle über Ollama',a)
    text=text[:a]+'''### Der Ablauf im Browser

Die ausführliche Anleitung mit markierten Bildern, genauen Eingaben und Fehlerhilfe steht in **Kapitel 20**. Du kannst deine Verbindung schon vor Anmeldung und Spielplatz unter **Eigene KI vorab einrichten und testen** prüfen.

1. Öffne **Modelle & Team**, füge bis zu vier benannte Modellkonfigurationen hinzu und wähle jeweils Ollama oder OpenRouter.
2. Lade die Modellliste, wähle die genaue Modell-ID und teste jede verwendete Konfiguration über **Verbindung & JSON-Antwort testen**.
3. Wähle im Spielerprofil **Gemischt**, wenn du Rollen selbst zuordnen möchtest, oder **Agent**, wenn die Modelle ihre Aufgaben beim Start miteinander beraten sollen.
4. Ordne Modelle den vier Rollen zu. Ein Modell darf mehrere Rollen übernehmen. Bearbeite bei Bedarf **soul.md** und **rollen.md** direkt an der Rolle.
5. Speichere Team und Pinwände, prüfe die Limits und klicke **Team starten / fortsetzen**. Ein reines Agententeam benötigt genügend Aufrufe für seine Beratung und die anschließenden Züge.
6. Kontrolliere bestätigte Übergaben und Spielzustand. Stoppen nimmt bereits bestätigte Spielaufträge nicht zurück.

Der Test führt keine Spielbefehle aus. Bei OpenRouter ist er trotzdem ein echter, möglicherweise kostenpflichtiger Modellaufruf. Nach Neuladen werden Schlüssel erneut eingegeben und Modelle erneut getestet; gespeicherte Texte und Pinwände bleiben im Konto.

'''+text[b:]
    a=text.index('### Modelle & Team',text.index('## 8 '));b=text.index('### Spielerprofil',a)
    text=text[:a]+'''### Modelle & Team

Hier verwaltest du bis zu vier benannte Modelle aus Ollama und OpenRouter, testest jeden Zugang und ordnest Modelle den Regierungsrollen zu. Ein Modell kann mehrere Rollen übernehmen. Die Texte **soul.md** und **rollen.md** sind pro Rolle editierbar, importierbar und als Markdown herunterladbar.

Öffne **＋ Modell hinzufügen**, wähle Anbieter und genaue Modell-ID, teste die Verbindung, besetze die gewünschten Rollen und speichere. Mit **Team starten / fortsetzen** beginnt die Ausführung. **Team stoppen** gibt die Steuerung zurück. Die bebilderte Anleitung in Kapitel 20 erklärt jeden Klick einschließlich Browserfreigabe und Fehlerbehebung.

### Pinwände

Für alle Spielweisen: Erstelle mehrere Wände, ergänze Karten mit Wissen, Zielen und Empfängern, ändere ihre Reihenfolge mit ↑/↓ und bearbeite oder lösche alte Inhalte. **Pinwände speichern** schreibt die Änderungen dauerhaft ins Konto. Alle Modelle lesen sie vor jeder neuen Entscheidung zusammen mit den bestätigten Übergaben. Ausführliche Beispiele stehen in Kapitel 20.

'''+text[b:]
    text=text.replace('| **Modelle & Team** | Modelle, Rollen, Tests, Start, Stopp und Aufrufprotokoll |','| **Modelle & Team** | Bis zu vier Modelle, Verbindungstests, soul.md/rollen.md, Start und bestätigte Übergaben |\n| **Pinwände** | Dauerhaftes Wissen, mehrere Wände, Karten, Ziele und Teamabsprachen |')
    p.write_text(text,encoding='utf-8')
    p=ROOT/'tools/build_startpage.py';text=p.read_text(encoding='utf-8').replace('("agent", "Agentensteuerung", 6)','("agent", "Modelle & Team", 20), ("pinwaende", "Pinwände", 20)').replace('alle 20 Bereiche','alle 21 Bereiche').replace('Die 20 Bereiche','Die 21 Bereiche');p.write_text(text,encoding='utf-8')
    for f in ['docs/startseite.html','docs/landingpage.html']:
        p=ROOT/f;text=p.read_text(encoding='utf-8').replace('unter Agentensteuerung','unter Modelle &amp; Team').replace('Lokale Modelle suchen','Modelle laden');p.write_text(text,encoding='utf-8')
    p=ROOT/'docs/startseite.html';text=p.read_text(encoding='utf-8')
    section='''<section class="section" id="agent-setup" aria-labelledby="agent-setup-title"><div class="wrap"><div class="section-heading"><span class="eyebrow">Dein Reich · bis zu vier eigene Modelle</span><h2 id="agent-setup-title">Schritt für Schritt<br>zu deinem Modellteam.</h2><p class="lead">Öffne im Spiel <strong>Modelle &amp; Team</strong>. Vor der Anmeldung kannst du bereits <strong>Eigene KI vorab einrichten und testen</strong> öffnen. Die Bilder markieren die echten Eingabefelder; die vollständige Anleitung darunter erklärt jeden Schritt, erwartete Ergebnisse und Fehler.</p></div><div class="guide-grid"><article class="operator-box"><h3>Ollama auf deinem Rechner</h3><figure class="team-guide-figure"><img src="team://02-ollama.webp" alt="Ollama mit fünf nummerierten Pfeilen: Teamname, API-Adresse, Modellliste, genaue Modell-ID und Test" loading="lazy"><figcaption>1 Name · 2 API-Adresse · 3 Modelle laden · 4 Modell auswählen · 5 Verbindung testen.</figcaption></figure><p>Ein lokales Modell kann alle Rollen übernehmen. Zwei lokale Modelle werden nacheinander geladen und nach der Antwort entladen. Welche Browserfreigaben erforderlich sind, zeigen wir in der Anleitung mit dem passenden Befehl.</p></article><article class="operator-box"><h3>OpenRouter – auch im gemischten Team</h3><figure class="team-guide-figure"><img src="team://03-openrouter.webp" alt="OpenRouter mit fünf nummerierten Pfeilen: Anbieter, persönlicher API-Key, Katalog, Modell-ID und Test" loading="lazy"><figcaption>1 Anbieter · 2 eigener Schlüssel · 3 Modellkatalog · 4 echte Modell-ID · 5 Test. Modell-ID im Bild: Beispiel aus dem OpenRouter-Katalog.</figcaption></figure><p>Ollama und OpenRouter lassen sich in einem Team kombinieren. Jeder Eintrag hat seinen eigenen Test und gegebenenfalls Schlüssel. Schlüssel bleiben nur in deinem Tab; ein Test kann Anbietergebühren verursachen.</p></article></div><a class="button" href="#guide-20-deinen-eigenen-ki-assistenten-verbinden">Bebilderte Anleitung: Einrichtung, Tests, Rollen und Pinwände ↓</a><p>Zusätzlich zur Verbindungsprüfung kannst du eine <strong>Spielentscheidung proben · ohne Befehle</strong>. Das Team startet erst bewusst nach deinen Tests und der gespeicherten Zuordnung.</p></div></section>'''
    text=re.sub(r'<section class="section" id="agent-setup".*?</section>',lambda _:section,text,flags=re.S);p.write_text(text,encoding='utf-8')
    p=ROOT/'tools/website_content.py';text=p.read_text(encoding='utf-8')
    text=text.replace("'comm':'docs/bilder/kommunikation'","'comm':'docs/bilder/kommunikation','team':'docs/bilder/team'")
    text=text.replace("document = re.sub(r'(asset|screen|logo|artwork|comm)://", "document = document.replace('src=\"bilder/team/', 'src=\"team://')\n    document = re.sub(r'(asset|screen|logo|artwork|comm|team)://")
    p.write_text(text,encoding='utf-8')
    p=ROOT/'tools/build_pages.py';text=p.read_text(encoding='utf-8').replace("'docs/bilder/kommunikation','docs/branding'","'docs/bilder/kommunikation','docs/bilder/team','docs/branding'")
    if "'guide/team-01-modelle.webp'" not in text:text=text.replace("'agenten-hilfe.html',","'agenten-hilfe.html',"+','.join(repr('guide/team-'+name+'.webp') for _,name,_,_ in ILLUSTRATIONS)+',')
    p.write_text(text,encoding='utf-8')
    p=ROOT/'crates/server/src/main.rs';text=p.read_text(encoding='utf-8')
    if '(false,"/guide/team-01-modelle.webp")' not in text:
        i=text.index('            (false,"/agent-team.js")')
        text=text[:i]+''.join(f'            (false,"/guide/team-{name}.webp")=>Some((include_bytes!("../../../web-client/guide/team-{name}.webp").as_slice(),"image/webp")),\n' for _,name,_,_ in ILLUSTRATIONS)+text[i:]
    p.write_text(text,encoding='utf-8')
    p=ROOT/'tools/player_publication_test.py';text=p.read_text(encoding='utf-8').replace("'Lokale Modelle suchen'","'Modelle laden','Pinwände','soul.md','rollen.md'");p.write_text(text,encoding='utf-8')
    print('Team guide synchronized: six annotated screenshots and full beginner instructions.')
if __name__=='__main__':build()
