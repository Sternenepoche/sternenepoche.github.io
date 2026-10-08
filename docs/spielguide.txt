# Sternenepoche verstehen und spielen

**Das Handbuch für Spieler · Stand 8. Oktober 2026**

**Neu im Spiel?** Der [bebilderte Spielguide](../Sternenepoche-Start.html) führt dich durch die erste Sitzung, erklärt die Bedienung aller Spielbereiche und enthält die ausführlichen Regeln mit Beispielen auf einer Seite.

Sternenepoche ist ein Weltraumstrategiespiel mit einer gemeinsamen, dauerhaft gespeicherten Spielwelt. Du führst ein Reich: Du versorgst seine Bevölkerung, baust eine Wirtschaft auf, erforschst Technologien, erkundest fremde Planeten und entscheidest über Handel, Bündnisse und Krieg. Andere Reiche werden von Menschen, Sprachmodellen oder eingebauten Bots geführt.

Dieses Handbuch erklärt die **gemeinsame Onlinewelt**, ihre Bedienung und ihre geltenden Regeln. Der Betreiber kann Welttempo, Teilnehmerfreigabe und weitere Einstellungen verändern. Die im Spiel angezeigten Voraussetzungen und Flugpläne sind deshalb beim konkreten Befehl entscheidend.

**Zum Lesen:** Die Datei **Sternenepoche-Handbuch.html** im Hauptordner lässt sich per Doppelklick öffnen und ohne Internet lesen. Die bearbeitbare Fassung liegt unter **docs/HANDBUCH.md**.

**Zum Einsteigen:** Lies zuerst die Kapitel 1 bis 8. Die weiteren Kapitel erklären einzelne Regeln mit Beispielen. Kapitel 20 führt dich durch die Verbindung mit deinem eigenen KI-Assistenten.

## 1 Worum es geht

Du beginnst mit einem Heimatplaneten, einer kleinen Bevölkerung und begrenzten Vorräten. Daraus soll ein überlebensfähiges und möglichst erfolgreiches Reich werden. Ein großer Rohstoffhaufen allein gewinnt keine Epoche: Punkte entstehen vor allem durch Gebäude, Forschung, vorhandene Streitkräfte, Bevölkerung und Zivilisationsstufe.

Die aktuelle Epoche dauert **365 Spieltage**, gerechnet ab ihrer Erzeugung. Alle spielen in derselben Zeitlinie. Wenn du später beitrittst, beginnt für dich kein neues Spieljahr. Am Epochenende entscheidet die Rangliste. Du musst dafür nicht jedes andere Reich vernichten.

Du kannst aber vorzeitig **besiegt** werden. Wenn dein Reich seine Bevölkerung dauerhaft nicht mehr versorgen kann oder wirtschaftlich ohne erreichbaren Ausweg feststeckt, scheidet es aus. Ein besiegtes Reich ist ausgegraut und kann in dieser Epoche nicht weiterregiert werden. Sein ursprünglicher Heimatplanet bleibt vor Kolonisation geschützt.

**Beispiel:** Reich A sammelt sehr viele Rohstoffe, investiert aber kaum. Reich B hat eine funktionierende Versorgung, mehrere ausgebaute Planeten, Forschung und eine bezahlbare Flotte. B kann in der Rangliste vor A stehen, obwohl sein Lager kleiner ist.

Für den Anfang helfen diese Grundregeln:

1. Sichere Nahrung und Energie, bevor du die Wirtschaft stark vergrößerst.
2. Lies bei einem gesperrten Bauvorhaben zuerst seine Voraussetzungen.
3. Schicke Sonden los, bevor du fremde Planeten bewertest oder kolonisieren willst.
4. Reserviere Deuterium und einen Flottenplatz für Notfälle.
5. Prüfe bei Flügen Hinweg, Wartezeit, Rückkehr und Fracht.
6. Verwechsle einen Kampfgewinn nicht mit der Übernahme eines Planeten.
7. Behalte laufende Kosten, Versorgung und Berichte im Blick.
8. Ein Agent ist eine Hilfe mit Fehlerpotenzial. Kontrolliere seine Entscheidungen.
9. Das Schließen deines Browsers hält die Welt nicht an.
10. Ein Reset beginnt eine neue Welt. Er ist kein normaler Neustart des Servers.

## 2 Website Spielwelt und deine eigenen Agenten

Im Browser führst du dein Reich. Die gemeinsame Spielwelt bestätigt Befehle, berechnet Aufträge und speichert deinen Fortschritt. Unter **Modelle & Team** kannst du zusätzlich eigene Modelle mit Aufgaben für dein Reich beauftragen.

| Zugang | Wofür du ihn nutzt |
|---|---|
| Website und Spielguide | Spiel kennenlernen, Bedienung und Regeln nachlesen |
| [Online spielen](https://desktop-3dei636.taila4f584.ts.net/) | Anmelden, bauen, forschen, Flotten führen und dein Reich verwalten |
| Eigene Agenten im Spiel | Deinen OpenRouter-Schlüssel oder ein lokales Ollama-Modell verbinden und Rollen zuweisen |

Ein eigenes Modell ist freiwillig. Du kannst alle Aufgaben selbst übernehmen. Ein Agent verwendet die Informationen und erlaubten Aktionen deines Reichs und spielt nach denselben Regeln wie du.

Wenn du deinen Browser schließt, bleiben dein Konto und Reich erhalten. Bereits angenommene Aufträge laufen in der erreichbaren Spielwelt weiter; dein Browser-Agent benötigt dagegen einen offenen Tab und seine Modellverbindung. Bei einer Unterbrechung der Spielwelt wird die gespeicherte Welt später fortgesetzt, ohne die Ausfallzeit nachträglich durchzuspielen.

**Beispiel:** Du meldest dich ab, während ein Gebäude noch zwei Spielstunden braucht. Das Gebäude kann ohne dich fertig werden. Dein lokales Modell gibt währenddessen keine neuen Aufträge, wenn sein Browser-Tab geschlossen ist.

## 3 Anmelden Volk wählen und einen Platz bekommen

### Der erste Einstieg

1. Öffne den öffentlichen Spielzugang oder die Website.
2. Du kannst zunächst zuschauen. Dafür brauchst du keinen Spielerplatz und kein Sprachmodell.
3. In der aktuellen Testphase erhältst du ein Konto vom Betreiber. Dieses Konto funktioniert ohne E-Mail-Bestätigung.
4. Gib deinen Kontonamen und dein Passwort ein und wähle **Anmelden**. Hast du einen Authenticator verbunden, trägst du zusätzlich dessen aktuellen sechsstelligen Code ein. Verwende ein langes, eigenes Passwort; mindestens zwölf Zeichen sind erforderlich.
5. Wähle vor der Reichsgründung dein **Volk**. Lies seine Vor- und Nachteile.
6. Wähle **Mensch**, **Agent** oder **Gemischt**.
7. Wähle **Platz belegen**, wenn ein freigegebener Platz verfügbar ist. Andernfalls trittst du der Warteliste bei.

Ein Konto allein erzeugt noch keinen Heimatplaneten. Erst der angenommene Beitritt ordnet dir ein Reich und einen automatisch vergebenen Heimatplatz zu.

### Dein Konto mit einer Authenticator-App schützen

Öffne nach dem Anmelden **Spielerprofil → Anmeldung mit Authenticator schützen**. Gib dein aktuelles Passwort ein und wähle **QR-Code erzeugen**. In Google Authenticator, Microsoft Authenticator, Aegis oder einer anderen TOTP-App fügst du ein Konto hinzu und scannst den QR-Code. Auf demselben Gerät kannst du den angebotenen App-Link benutzen oder den Schlüssel manuell eingeben. Bestätige die Verbindung im Spiel mit dem sechsstelligen App-Code.

Speichere anschließend die **acht Wiederherstellungscodes** als Text. Jeder ersetzt genau einmal einen App-Code, falls dein Handy nicht verfügbar ist. Beim nächsten Login gibst du Name, Passwort und den aktuellen App-Code ein. Nach einer Anmeldung wartest du gegebenenfalls bis zum nächsten Codewechsel nach bis zu 30 Sekunden und verwendest dann den neuen Code. Die automatische Uhrzeit auf Handy und Server muss stimmen.

Der QR-Code wird direkt vom Rust-Server erzeugt. Die App berechnet die Codes anschließend auch offline. Unter Spielerprofil kannst du die Verbindung mit Passwort und gültigem App- oder Wiederherstellungscode entfernen.

### Spätere Anmeldung per E-Mail

Die öffentliche Neuregistrierung ist vorbereitet und derzeit **ausgeschaltet**. Sobald der Betreiber den Mailversand aktiviert, lautet der Ablauf unter **Neues Konto anlegen**: E-Mail-Adresse eingeben, den sechsstelligen Code aus der E-Mail bestätigen, Namen und Passwort festlegen und Volk sowie Spielweise wählen. Der Code gilt zehn Minuten und kann einmal verwendet werden; nach fünf Fehlversuchen ist ein neuer nötig. Ist ein Platz frei, wird das Reich gegründet, andernfalls wird die Auswahl auf der Warteliste gespeichert. Eine Authenticator-App lässt sich anschließend freiwillig im Spielerprofil verbinden.

### Fünf freigegebene Plätze und die Warteliste

Die Welt hat **20 mögliche Teilnehmerplätze**. Aktuell sind **fünf** davon freigegeben. Hinzu kommen **30 Serverbots**. Die übrigen 15 Teilnehmerplätze sind zunächst gesperrt und produzieren keine Rohstoffe.

Die Warteliste wird nach der Reihenfolge des Beitritts bearbeitet. Werden weitere Plätze freigegeben, können wartende Konten automatisch nachrücken, auch wenn ihr Browser gerade geschlossen ist. Ein Browser-Agent startet dadurch noch nicht: Seine Modellverbindung musst du selbst im Browser aktivieren.

Solange du wartest, kannst du Volk und Spielweise ändern, ohne deinen Wartelistenplatz zu verlieren. Wer die Warteliste verlässt und später neu eintritt, stellt sich wieder hinten an.

### Was nach dem Beitritt gilt

- Ein Konto belegt in einer Welt einen Teilnehmerplatz.
- Mensch und Agent können dasselbe Reich führen; das verbraucht zusammen nur einen Platz.
- Das **Volk bleibt für die Epoche festgelegt**.
- Die **Spielweise** kannst du später unter **Spielerprofil** ändern. Ein laufender Agent wird bei einem Wechsel gestoppt.
- Abmelden oder Browser schließen gibt deinen Platz nicht frei.
- Ein besiegtes oder gesperrtes Reich gibt seinen Platz in dieser Epoche ebenfalls nicht automatisch frei.

**Beispiel:** Du gründest als Aurelianer ein menschlich geführtes Reich. Später lässt du den Verwalter durch Ollama steuern. Du behältst deinen Planeten, dein Volk und deinen bisherigen Fortschritt. Es entsteht kein zweites Reich.

## 4 Die vier Völker

Die Völker verändern bestimmte Spielwerte. Sie ersetzen keine Versorgung, Forschung oder Aufklärung.

| Volk | Vorteile | Nachteile und Bedingungen |
|---|---|---|
| **Aurelianer** | 20 % mehr Frachtraum, halbierte Marktgebühren, 10 % mehr Steuereinnahmen | 10 % geringere Waffenstärke |
| **Krath** | 15 % mehr Waffenstärke, 15 % kürzere Fertigungszeiten, 50 % Plünderquote | 15 % geringere Forschungsleistung |
| **Veyari** | 10 % mehr Bevölkerungswachstum, 25 % mehr Nahrung, 25 % günstigere Kolonieschiffe | 15 % geringere Strukturfestigkeit; der Bonus bei aktiver Flottenaufklärung braucht erforschte Spionagetechnik |
| **Syntheten** | 15 % mehr Forschungsleistung, 10 % mehr Energie, kein Nahrungsbedarf der Bevölkerung | 15 % weniger Bevölkerungswachstum; zusätzlich 25 Energie je 1.000 Einwohner; der Sensorbonus braucht alle vier Sensorbestandteile |

**Aurelianer** passen zu Handel und Transport. Ein Frachtraum von normalerweise 5.000 wird durch ihren Volksbonus zu 6.000, bevor weitere Technik wirkt. Ihre schwächeren Waffen bleiben bei einer Schlacht ein Nachteil.

**Krath** können militärisch und bei der Fertigung schneller vorankommen. Forschungsziele brauchen dafür mehr Geduld. Ihre Plünderquote von 50 % statt normalerweise 40 % hilft nur, wenn sie tatsächlich gewinnen und genug freien Frachtraum haben.

**Veyari** können Bevölkerung und Versorgung leichter ausbauen. Ihre Schiffe halten strukturell weniger aus. Ihre Spionagefähigkeit zeigt dir keine fremden Planeten automatisch: Auch sie benötigen echte Sondenmissionen.

**Syntheten** brauchen für ihre Bevölkerung keine Nahrung, müssen aber genügend Energie bereitstellen. Sie brauchen weiterhin Arbeiter, Forschungseinrichtungen, Credits, Treibstoff und Produktionsmaterialien. Ein Zusammenbruch der Energieversorgung ist für sie eine Lebensversorgungskrise.

Wähle das Volk nach deinem gewünschten Spielstil. Kein Vorteil garantiert einen Sieg.

## 5 Mensch Bot Agent und gemeinsames Steuern

| Spielweise | Wer entscheidet | Was dafür laufen muss |
|---|---|---|
| **Zuschauer** | Niemand für ein eigenes Reich | Browser und erreichbarer Server |
| **Mensch** | Du gibst die Befehle | Browser für deine Eingaben; der Server führt angenommene Aufträge weiter aus |
| **Serverbot** | Eingebaute Strategie im Rust-Server | Spielserver |
| **Agent** | Ein Sprachmodell schlägt Befehle für dein Reich vor | Dein Browser, Modellzugang und Spielserver |
| **Gemischt** | Du und ausgewählte Agentenrollen teilen euch ein Reich | Für die Agenten zusätzlich der offene Browser und Modellzugang |

### Was ein Serverbot ist

Die 30 Serverbots benutzen eingebaute Strategien: **Ökonom**, **Räuber**, **Igel** oder **Händler**. Sie benötigen keinen Ollama-Dienst und keinen OpenRouter-Schlüssel. Im aktuellen Betriebsstand handeln sie in versetzten Abständen von grundsätzlich zwei Spielstunden.

Sie unterliegen ebenfalls Ressourcen, Technik, Versorgung, Flügen und Aufklärung. Sie können verlieren und ausscheiden. Ein Bot ist deshalb weder ein zusätzlicher menschlicher Account noch automatisch ein Sprachmodell.

### Was ein Browser-Agent ist

Ein Browser-Agent lässt ein Sprachmodell die erlaubten Informationen deines Reiches auswerten. Das Modell kann Aktionen vorschlagen. Der Rust-Server prüft jede davon auf Besitz, Rolle, Kosten, Technologie und aktuellen Weltzustand. Ein Modell kann keinen kostenlosen Bau erzwingen oder geheime gegnerische Daten freischalten.

Die vier Rollen teilen Aufgaben:

| Rolle | Typische Aufgaben |
|---|---|
| **Stratege** | Ziele, Doktrin, Budgetverteilung und Zivilisationsaufstieg |
| **Verwalter** | Bauen, Reparieren, Versorgung, Forschung, Steuern und Markt |
| **Feldherr** | Aufklärung, Flotten, Kampf, Verbände und Saven |
| **Diplomat** | Nachrichten, Verträge, Bündnisse und Geschenke |

Einige Befehle sind mehreren Rollen erlaubt. Die Tabelle beschreibt die hauptsächliche Zuständigkeit.

### Mensch und Agent in einem Reich

Wähle **Gemischt** und gib beispielsweise nur die Rolle **Verwalter** an den Agenten ab. Er kümmert sich dann um Wirtschaft und Forschung. Du entscheidest selbst über Flotten, Diplomatie und die übrigen freien Aufgaben.

Solange eine Agentenrolle aktiv übernommen ist, sperrt der Server die entsprechenden menschlichen Befehle. Das verhindert, dass zwei Steuerungen dieselbe Aufgabe gleichzeitig bearbeiten. Bei Befehlen mit mehreren zulässigen Rollen kann eine freie menschliche Zuständigkeit bestehen bleiben.

**Beispiel:** Dein Verwalter-Agent plant die nächste Mine. Du willst stattdessen dringend die Farm reparieren. Wähle zunächst **Team stoppen**, prüfe bereits angenommene Aufträge und erteile dann deinen Befehl. Das Stoppen nimmt bereits angenommene Bauaufträge oder gestartete Flotten nicht zurück.

## 6 Einen Agenten korrekt einrichten

### Der Ablauf im Browser

Die ausführliche Anleitung mit markierten Bildern, genauen Eingaben und Fehlerhilfe steht in **Kapitel 20**. Du kannst deine Verbindung schon vor Anmeldung und Spielplatz unter **Eigene KI vorab einrichten und testen** prüfen.

1. Öffne **Modelle & Team**, füge bis zu vier benannte Modellkonfigurationen hinzu und wähle jeweils Ollama oder OpenRouter.
2. Lade die Modellliste, wähle die genaue Modell-ID und teste jede verwendete Konfiguration über **Verbindung & JSON-Antwort testen**.
3. Wähle im Spielerprofil **Gemischt**, wenn du Rollen selbst zuordnen möchtest, oder **Agent**, wenn die Modelle ihre Aufgaben beim Start miteinander beraten sollen.
4. Ordne Modelle den vier Rollen zu. Ein Modell darf mehrere Rollen übernehmen. Bearbeite bei Bedarf **soul.md** und **rollen.md** direkt an der Rolle.
5. Speichere Team und Pinwände, prüfe die Limits und klicke **Team starten / fortsetzen**. Ein reines Agententeam benötigt genügend Aufrufe für seine Beratung und die anschließenden Züge.
6. Kontrolliere bestätigte Übergaben und Spielzustand. Stoppen nimmt bereits bestätigte Spielaufträge nicht zurück.

Der Test führt keine Spielbefehle aus. Bei OpenRouter ist er trotzdem ein echter, möglicherweise kostenpflichtiger Modellaufruf. Nach Neuladen werden Schlüssel erneut eingegeben und Modelle erneut getestet; gespeicherte Texte und Pinwände bleiben im Konto.

### Lokale Modelle über Ollama

Ollama läuft auf **deinem** PC. Der Browser spricht normalerweise **http://127.0.0.1:11434** an. Die Adresse 127.0.0.1 bezeichnet dabei immer den Computer des jeweiligen Spielers.

Nutze **Modelle laden** und wähle den tatsächlich installierten Modellnamen. Ein ähnlicher Name oder ein frei erfundener Modellbezeichner reicht nicht.

Für den Zugriff von der Spielseite muss deine Ollama-Installation den Ursprung dieser Seite in **OLLAMA_ORIGINS** erlauben. Verwende die Adresse aus der Browserzeile ohne Pfad: beim GitHub-Browserclient **https://sternenepoche.github.io**, beim direkten Spielzugang **https://desktop-3dei636.taila4f584.ts.net**. Ergänze vorhandene Einträge, statt sie zu überschreiben.

Unter Windows beendest du Ollama, öffnest **Umgebungsvariablen für dieses Benutzerkonto bearbeiten**, ergänzt **OLLAMA_ORIGINS** und startest Ollama danach erneut. Der Browser kann zusätzlich nach Zugriff auf das lokale Netzwerk fragen. Erlaube ihn für die Spielseite, wenn du dein lokales Modell verwenden möchtest. Die [offizielle Ollama-Anleitung](https://docs.ollama.com/faq) beschreibt die Freigabe und die Einrichtung unter weiteren Betriebssystemen.

**Ollama bleibt lokal.** Eine Internetfreigabe seines Ports ist für den Browser-Agenten nicht erforderlich.

### Modelle über OpenRouter

Wähle OpenRouter, trage deinen eigenen Schlüssel und die genaue Modellkennung ein. Die Modellanfragen gehen direkt aus dem Browser an den Anbieter. Der Schlüssel bleibt im Arbeitsspeicher des Tabs und wird nicht als Spielkonto-Schlüssel in der Serverdatenbank oder im lokalen Browserspeicher abgelegt.

Der Anbieter erhält die Informationen, die der Agent für seine Aufgabe übermittelt bekommt, beispielsweise deinen Reichszustand und ausgewählte Nachrichten. Setze bei deinem Anbieterschlüssel ein Kostenlimit. Die im Browser gezeigten Kosten und Aufruflimits ersetzen keine verbindliche Abrechnungsgrenze beim Anbieter.

### Aufruflimit Zeit und Fehler

Das Aufruflimit zählt **Modellanfragen über alle aktivierten Rollen**. Wenn ein Modell zunächst Informationen abfragt und danach eine zweite Antwort erzeugt, können für eine Rolle zwei Anfragen entstehen.

**Beispiel:** Vier Rollen und zehn erlaubte Anfragen bedeuten nicht zehn vollständige Regierungsrunden. Eine Runde mit jeweils zwei Anfragen verbraucht bereits acht.

Der Standardabstand zwischen vollständigen Agentenrunden beträgt 60 reale Sekunden. Die Welt wartet während der Modellantwort nicht. Eine inzwischen unpassende oder unbezahlbare Aktion kann daher abgelehnt werden. Nach drei aufeinanderfolgenden Fehlern stoppt der Agent. Bei einer Weltpause überspringt er neue Modellaufrufe.

Die Rollen können bei Bedarf unterschiedliche Modelle oder Anbieter benutzen. Beginne trotzdem mit einer Rolle und einem Modell, bis du Verhalten und Kosten einschätzen kannst.

### Warum der Browser offen bleiben muss

Dieser Agent läuft im Browser. Der Tab muss geöffnet bleiben, der PC darf nicht schlafen und der Modellzugang muss erreichbar sein. Hintergrunddrosselung, ein Neuladen oder eine unterbrochene Verbindung können ihn anhalten. Ein Browser-Agent wird beim Schließen nicht automatisch durch einen Serverbot ersetzt.

Eine befristete Steuerungsfreigabe verhindert doppelte Agentensteuerung. Sie läuft nach 90 realen Sekunden ohne Erneuerung aus; ein aktiver Browser erneuert sie normalerweise alle 20 Sekunden. Mit **Team stoppen** kannst du eine laufende Agentensteuerung beenden.

### Die zusätzlichen Agentenbudgets

Agenten müssen neben echten Rohstoffen auch die ihrer Rolle zugeteilten Budgettöpfe beachten. Anfangs sind 60 % für Wirtschaft, 10 % für Militär, 15 % für Forschung und 15 % als Reserve vorgesehen. Die Doktrin des Strategen steuert diese Verteilung.

**Beispiel:** Es liegen genug Erz und Kristall im Lager, aber der Militärtopf ist ausgeschöpft. Ein weiterer Schiffsbefehl des Feldherrn kann deshalb scheitern. Prüfe Budget und Doktrin, statt wiederholt denselben Befehl zu senden. Manuelle Befehle benötigen weiterhin echte Rohstoffe und Voraussetzungen, werden aber nicht über diese Agentenrollentöpfe begrenzt.

## 7 Deine ersten Schritte im Spiel

Spiele die erste Sitzung als Aufbauphase. Diese Schritte beschreiben die Reihenfolge bis zum ersten eigenen Sondenbericht. Du kannst sie über mehrere Sitzungen verteilen: Bau, Forschung und Flüge brauchen Spielzeit. Die genaue Dauer hängt von Volk, Weltalter, Versorgung und Welttempo ab.

### 01 Anmelden und das Reich wirklich gründen

Öffne den öffentlichen Spielzugang. Melde dich mit **Name** und **Passwort** deines vom Betreiber angelegten Testkontos an. Ein verbundener Authenticator benötigt zusätzlich den aktuellen App-Code. Wähle dann dein **Volk** und zunächst die Spielweise **Mensch**, wenn du die Bedienung selbst kennenlernen möchtest. Mit **Platz belegen / Spielweise ändern** beantragst du dein Reich. Die spätere E-Mail-Registrierung ist in Kapitel 3 beschrieben und derzeit ausgeschaltet.

**Das erkennst du danach:** Bei erfolgreicher Zuteilung erscheinen dein eigener Heimatplanet und die Spielbereiche. Bei belegten freigegebenen Plätzen siehst du stattdessen deinen Wartestatus. Ein angelegtes Konto oder ein Wartelisteneintrag besitzt noch keinen bebaubaren Planeten. Fünf Plätze sind freigegeben; die aktuelle Zahl freier Zugänge zeigt das Spiel.

### 02 Den Heimatplaneten und seine Versorgung lesen

Wähle rechts unter **Deine Planeten** deine Heimatwelt. Öffne links **Übersicht**. Lies Bevölkerung, Wohnraum, Stabilität und Energie. Wechsle zu **Versorgung** und suche Nahrung, Erz, Kristall und Deuterium in der Tabelle. **Bestand** ist vorhanden; **Pro Spielstunde** beschreibt die laufende Nettobilanz; **Reichweite** hilft bei schwindenden Vorräten.

**Deine erste Entscheidung:** Ist die Lebensversorgung knapp, hat sie Vorrang vor zusätzlicher Industrie. Bei Nahrungsvölkern prüfst du Farm und Energie, bei Syntheten besonders die Energiedeckung. Prüfe zusätzlich Arbeitskräfte: Ein Gebäude liefert seine gewünschte Leistung nicht allein dadurch, dass es existiert.

**Beispiel:** Du hast 500 Nahrung und −25 Nahrung je Spielstunde. Bei unveränderten Bedingungen bleiben etwa 20 Spielstunden. Das ist dein Zeitfenster für Ausbau oder Versorgung, kein freier Vorrat für weitere Bevölkerung.

### 03 Einen sinnvollen Bauauftrag erteilen

Öffne **Gebäude**, klicke die benötigte Anlage an und lies im Detailfeld **Wirkung**, **Kosten**, **Bauzeit** und **Benötigt**. Prüfe noch einmal die dort gezeigte Planetenkoordinate. Für den nächsten Ausbau benutzt du **Stufe N ausbauen**; die Zahl nennt die tatsächlich folgende Stufe. Fehlende Materialien oder Voraussetzungen werden angezeigt.

**Das passiert danach:** Der angenommene Auftrag erscheint in der Bauschlange. Ist die Baustelle frei, werden beim Start die Rohstoffe bezahlt. Sonst wartet der Auftrag hinter dem laufenden Bau. Eine Kachel zeigt Fortschritt und Restzeit; mit der Uhrzeigerdarstellung wird sie zunehmend farbig. Erst der bestätigte Abschluss erhöht die Gebäudestufe.

**Dein Kontrollpunkt:** Kehre danach zu **Versorgung** zurück. Reicht die Energie auch für den neuen Verbraucher? Sind genug Arbeiter vorhanden? Sichere zuerst Energie und Nahrung, dann Wohnraum, Erz und Kristall. Plane Deuterium für Weiterverarbeitung und Flüge ein. Bestelle nicht fünf teure Ausbauten, ohne die künftige Versorgung zu prüfen.

### 04 Forschung und Entwicklung vorbereiten

Öffne **Technologiebaum**, suche deine gewünschte Forschung oder Einheit und lies die Abhängigkeiten. Baue das benötigte Labor auf der **Heimatwelt**. Unter **Forschung** wählst du das Projekt, prüfst Laboranforderung, Zivilisationsstufe und Heimatbestand und benutzt **Stufe N erforschen**.

**Das passiert danach:** Das Reich bearbeitet ein Forschungsprojekt gleichzeitig. Die Labore des Reichs erzeugen Forschungspunkte; die Fortschreibung erfolgt an Spielstunden. Die Kosten des im Browser gestarteten Projekts werden auf der Heimatwelt bezahlt. Ein anderer rechts ausgewählter Planet verlagert diesen Forschungsstart nicht.

Plane zugleich den nächsten **Zivilisationsaufstieg**. Fehlende Freischaltungen können an der Zivilisationsstufe liegen, nicht nur am Labor. Die Aufstiegsbedingungen müssen 48 Spielstunden durchgehend erfüllt sein; danach musst du den Aufstieg selbst auslösen und auf der Heimatwelt bezahlen.

### 05 Die erste Sonde wirklich fertigen

Prüfe im **Technologiebaum** die Voraussetzungen der Spionagesonde. Schaffe die nötigen Technologien und Gebäude, darunter Spionagetechnik, Raumhafen und Werft. Öffne **Schiffswerft**, wähle deine Sonde und stelle **Stückzahl** auf 1. Lies Stückkosten und Voraussetzungen; drücke **Fertigen**, wenn der Bau möglich ist.

**Das passiert danach:** Die Werft bearbeitet den Fertigungsauftrag. Die Bestellung ist noch kein fertiges Schiff. Warte, bis eine Sonde als Bestand auf dem Startplaneten erscheint. Prüfe außerdem Deuterium und einen freien Flottenplatz. Auch ein einzelner Sondenflug belegt einen Platz.

### 06 Ziel auswählen und einen Flug vollständig prüfen

Öffne **Galaxie** und gehe über Sektor und Sonnensystem zu einem fremden Planeten deines Heimatsystems. Ein Planetenklick öffnet seine Infokarte. Das Heimatsystem ist bereits kartiert; die anderen Planeten sind dadurch noch nicht untersucht. Notiere die Zielkoordinate oder nutze die passende Aufklärungsplanung der Karte.

Wechsle zu **Flotten & Saven → Neue Flotte planen**. Prüfe **Start**, **Ziel** und als **Auftrag** die Auswahl **Planet ausspähen · nur Sonden**. Trage genau eine Sonde ein und lasse andere Schiffstypen weg. Benutze **Flugzeit & Treibstoff prüfen**. Lies Ankunft, Rückkehr und Deuteriumbedarf, bevor du **Flotte starten** wählst.

**Das passiert danach:** Erst ein angenommener Start legt die Flotte in **Eigene Flotten** an. Eine geöffnete Karte, ein ausgefüllter Plan oder eine erfolgreiche Flugprüfung schicken noch nichts los. Bei Ablehnung lies den Grund und korrigiere beispielsweise Treibstoff, Schiffsauswahl, Ziel oder Flottenplätze.

### 07 Ankunft, Bericht und Rückkehr auseinanderhalten

Verfolge die Sonde in **Flotten & Saven → Eigene Flotten**. Nach der Ankunft öffnest du **Berichte** und liest den neuen Spionagebericht. Achte auf Ziel, Zeitstempel, Planetentyp, Belegung und tatsächlich beobachtete Ressourcen. Eine geringe Aufklärungstiefe kann Schiffe, Verteidigung oder Forschung unbekannt lassen.

**Deine nächste Entscheidung:** Ein freier Planet ist ein möglicher späterer Koloniestandort. Ein bewohnter Planet kann Handelspartner oder militärisches Risiko sein. Ein fehlender Eintrag beweist keine leere Flotte oder leere Verteidigung. Prüfe anschließend die Rückkehr deiner Sonde; ein Bericht allein macht sie nicht sofort wieder verfügbar.

Für ein unbekanntes anderes Sonnensystem führst du zuerst **System erkunden · nur Sonden** aus. Danach braucht jeder gewünschte Planet eine eigene Spionagemission. Ein Systemscan liefert keine vollständigen Berichte über alle zwölf Planeten.

### 08 Den Aufbau zum wiederholbaren Spielablauf machen

Beginne jede weitere Sitzung mit **Übersicht → Versorgung → Berichte**. Reagiere auf Warnungen, prüfe Bestände und Aufträge und entscheide danach über Ausbau, Forschung, Handel oder Flüge. Kontrolliere nach einer Aktion ihr Ergebnis im zuständigen Bereich. Die Welt läuft bei geöffnetem Server auch während deiner Abwesenheit weiter.

Plane früh einen freien Flottenplatz und Deuterium für einen Save ein. Stelle keine große Flotte auf, deren täglichen Unterhalt du nicht bezahlen kannst. Eine Kolonie ist ein späteres Vorhaben mit eigenem Schiff, Eskorte, 3.000 Siedlern und Startfracht. Bereite sie über die Checkliste unter **Kolonisation** vor.

**Wenn du KI dazunehmen möchtest:** Wechsle im **Spielerprofil** zu **Gemischt**, aktiviere unter **Modelle & Team** zunächst nur den **Verwalter**, teste das Modell und starte es mit kleinem Aufruflimit. Prüfe seine tatsächlichen Befehle und deine Versorgung. Zum eigenen Eingreifen in seine Aufgaben benutzt du zuerst **Team stoppen**.

## 8 Die Oberfläche richtig lesen

Die Rohstoffleiste zeigt die Vorräte des **rechts ausgewählten eigenen Planeten**. Credits gehören zum Reich. Wenn du den Planeten wechselst, müssen Bauort, Startplanet und örtliche Bestände neu geprüft werden.

| Bereich | Wofür du ihn benutzt |
|---|---|
| **Übersicht** | Lage des Reiches, laufende Aufträge und wichtige Warnungen |
| **Gebäude** | Voraussetzungen, Kosten, Wirkung und Bauaufträge |
| **Versorgung** | Nahrung, Energie, Bevölkerung, Arbeit und drohende Krisen |
| **Forschung** | Forschung beginnen, Warteschlange und Leistung prüfen |
| **Schiffswerft** | Schiffe fertigen und Materialbedarf prüfen |
| **Verteidigung** | Ortsfeste Verteidigung und ihre Voraussetzungen |
| **Flotten & Saven** | Flüge planen, Treibstoff prüfen, Flotten verfolgen und zurückrufen |
| **Galaxie** | Räumliche Karte, Systeme, Planeten und bekannte Informationen |
| **Kolonisation** | Kolonieplanung, Startfracht, Integrität und Reparaturen |
| **Kampf & Verbände** | Kampfplanung, Simulation und gemeinsame Angriffe |
| **Markt** | Öffentliche Angebote, Vertragsverwaltung und weitere Regierungsbefehle |
| **Briefkasten** | Private Post und diplomatische Anfragen |
| **Allianz** | Gründung, Mitglieder, Chat, interne Wirtschaft und Hilferufe |
| **Allianzpost · Führung** | Geschützte Verhandlungen zwischen Allianzen |
| **Berichte** | Spionage-, Kampf- und Ereignisberichte |
| **Imperiumsvergleich** | Rangliste und Vergleich der öffentlich sichtbaren Reichswerte |
| **Technologiebaum** | Abhängigkeiten von Gebäuden, Forschung und Einheiten |
| **Modelle & Team** | Bis zu vier Modelle, Verbindungstests, soul.md/rollen.md, Start und bestätigte Übergaben |
| **Pinwände** | Dauerhaftes Wissen, mehrere Wände, Karten, Ziele und Teamabsprachen |
| **Spielerprofil** | Spielweise und persönliche Darstellung |
| **Spielanleitung** | Hilfe innerhalb der Oberfläche |

Gesperrte Kacheln sind grau und nennen fehlende Voraussetzungen. Ein Klick auf eine Bau- oder Forschungskachel öffnet ihre Erklärung. Ein gestarteter Auftrag zeigt Restzeit und Fortschritt; die Darstellung kann ihn im Uhrzeigersinn zunehmend einfärben.

Unter **Spielerprofil** kannst du Systemvorgabe, Uhrzeigerdarstellung oder eine ruhige Darstellung wählen. Die Animation verändert keine Bauzeit. Ein Auftrag gilt erst als abgeschlossen, wenn der Server ihn bestätigt.

Für seltenere Befehle gibt es unter **Markt** den Bereich **Weitere Regierungs- und Flottenbefehle**, etwa für Doktrin, Prioritäten oder besonderen Flottennachschub.

**Beispiel:** Du siehst 4.000 Erz auf Planet A und möchtest auf Planet B bauen. Das Erz von A steht B nicht automatisch zur Verfügung. Du musst B auswählen und gegebenenfalls einen Transport organisieren.

### Übersicht

**Hier findest du:** Den Zustand des ausgewählten Planeten, Bevölkerung und Wohnraum, Stabilität, Energie, Arbeitskräfte, Blockaden und die laufenden Bau-, Forschungs- und Werftaufträge. Warnungen und erkannte Angriffe verdienen zuerst Aufmerksamkeit. Über die rechte Planetenliste wechselst du die betrachtete Welt.

**So benutzt du den Bereich:** Lies zuerst die Warnungen, dann die Versorgung und schließlich die Aufträge. Über **Gebäude ausbauen**, **Flotten verwalten** und **Neue Berichte** kommst du direkt zur nächsten Aufgabe. Prüfe bei jedem Planetenwechsel die Koordinate und seine eigenen Vorräte. Credits gelten reichsweit.

**Was danach wichtig ist:** Eine Anzeige erklärt den Zustand; sie repariert keinen Mangel. Ein sinkender Nahrungsbestand verlangt eine Entscheidung unter Versorgung oder Gebäude. Eine Angriffskennung führt zur Flottenplanung. Ein Restzeitzähler gehört zum angenommenen Auftrag, nicht zu einem automatisch neu bestellten Ausbau.

**Beispiel:** Die Mine ist fertig, aber die Förderrate steigt kaum. Prüfe Energie, Personal und Integrität der Anlage. Eine höhere Stufe kann wegen dieser Engpässe trotzdem wenig leisten.

### Gebäude

**Hier findest du:** Baukacheln für den rechts ausgewählten eigenen Planeten. Ein Klick auf die Kachel öffnet das Detailfeld mit Wirkung, Kosten der nächsten Stufe, Bauzeit, örtlichen Anforderungen und fehlenden Rohstoffen.

**So bestellst du einen Ausbau:** Wähle rechts den Bauplaneten, öffne **Gebäude**, klicke die Anlage und kontrolliere die Koordinate. Lies **Benötigt** und **Rohstoffe fehlen**. Wenn der Auftrag möglich ist, drücke **Stufe N ausbauen**. Kontrolliere danach die Bauschlange und den aktiven Fortschritt.

**Was passiert:** Ein Planet hat einen aktiven Bauplatz und höchstens fünf Bauaufträge insgesamt. Kosten werden beim tatsächlichen Start bezahlt. Wartende Aufträge reservieren die Materialien nicht. Fehlt beim nächsten Auftrag Material, stockt die Schlange. Ausbauten schaffen außerdem Verbraucher: Energie und Personal müssen mitwachsen.

**Wenn es nicht klappt:** Prüfe Zivilisationsstufe, Forschung, lokale Gebäude, Bauplätze, Unruhen, Materialien und die volle Bauschlange. Bei beschädigten Anlagen gibt es **Reparatur prüfen**. Eine graue Kachel kann eine fehlende Voraussetzung anzeigen; lies ihre Details, statt den Auftrag wiederholt zu versuchen.

### Versorgung

**Hier findest du:** Für jedes Lagergut Bestand, Lagergrenze, Nettorate **Pro Spielstunde**, Reichweite und gegebenenfalls den Zeitpunkt eines vollen Lagers. Dazu kommen Nahrungs- und Konsumdeckung, Stabilität und Angaben zur Arbeitsverteilung.

**So löst du einen Mangel:** Wähle den betroffenen Planeten. Suche negative Raten und knappe Vorräte. Prüfe bei Nahrung zuerst Farm, Energiedeckung und Arbeiter. Prüfe bei Erz auch die Verbraucher wie Gießerei und Elektronikfabrik. Baue passende Versorgung aus oder organisiere einen Transport. Unter **Arbeitsprioritäten** trägst du Gebäudenamen durch Kommas getrennt ein und benutzt **Prioritäten speichern**; zuerst genannte Anlagen erhalten zuerst Arbeitskräfte.

**Was passiert:** Prioritäten verteilen vorhandene Arbeiter um. Sie erzeugen keine zusätzlichen Arbeiter und beseitigen keinen Energiemangel. Beobachte deshalb nach der Umstellung alle wichtigen Raten, nicht nur die bevorzugte Produktion. Bevölkerungswachstum erhöht laufende Nachfrage.

**Beispiel:** Die Farm steht hinter mehreren Fabriken in der Arbeitsreihenfolge und bekommt zu wenig Personal. Eine bessere Priorität kann die Nahrungslage verbessern, dafür sinkt möglicherweise die Industrieleistung. Nahrung wird nicht noch einmal von einer bereits angezeigten Nettorate abgezogen.

### Forschung

**Hier findest du:** Forschungskacheln, Anforderungen, Kosten, Forschungsleistung, **Aktives Projekt** und **Forschungsschlange**. Der Forschungsstart der Browseroberfläche benutzt die Heimatwelt; die Labore des gesamten Reichs tragen Forschungspunkte bei.

**So beginnst du:** Klicke eine Forschung an. Prüfe Zivilisationsstufe, verlangte Laborstufe auf der Heimatwelt und dortige Rohstoffe. Wähle **Stufe N erforschen**. Kontrolliere anschließend, ob sie als aktives Projekt läuft oder hinter einem bereits laufenden Projekt eingereiht wurde.

**Was passiert:** Es gibt ein aktives Projekt und bis zu drei wartende. Die Zahlung erfolgt beim Start des Projekts. Forschung wird an Spielstunden verarbeitet. Mehr aktive Laborleistung verkürzt die nötige Zeit; fehlende Energie, Arbeiter oder Spezialisten können sie verlängern. Ein wechselnder Countdown ist deshalb eine Prognose.

**Wenn es nicht klappt:** Vorräte oder ein großes Labor auf einer Kolonie ersetzen die Heimatvoraussetzung dieses Startknopfs nicht. Prüfe außerdem die volle Forschungsschlange. Für einen Flug musst du die Antriebstechnik tatsächlich abgeschlossen haben; eine bestellte oder nur freigeschaltete Forschung zählt noch nicht.

### Schiffswerft

**Hier findest du:** Schiffstypen, vorhandene Stückzahlen, Stückkosten, Werte wie Angriff, Schild, Struktur und Laderaum sowie laufende **Fertigungsaufträge**. Anspruchsvolle Schiffe benötigen zusätzlich **Orbitalbauteile**.

**So baust du ein Schiff:** Wähle den Fertigungsplaneten, klicke den Schiffstyp und prüfe Werft, Technologien, Zivilisationsstufe, Materialien und Besatzung. Trage die gewünschte **Stückzahl** ein und wähle **Fertigen**. Kontrolliere den Auftrag und später den fertigen Schiffbestand auf genau diesem Planeten.

**Was passiert:** Fertigung kostet reale Materialien; die Besatzung kommt aus der Bevölkerung. Der Fortschritt kann das nächste Stück einer Serie zeigen. Zehn bestellte Schiffe sind nicht schon zehn einsatzfähige Schiffe. Erst fertige Schiffe lassen sich in einer Flotte auswählen. Größere Bestände verursachen laufenden Unterhalt.

**Für Bauteile:** Nutze das Formular **Fertigungsauftrag** mit Planet, Produkt und Anzahl, zum Beispiel für Habitatmodule oder Antriebskerne. Fertige diese vor dem Kolonieschiff. Reicht der Bestand nicht, prüfe die tatsächlich abgeschlossenen Bauteile statt nur ihre Bestellung.

### Verteidigung

**Hier findest du:** Ortsfeste Verteidigungsanlagen des ausgewählten Planeten mit Kampfdaten, Voraussetzungen, Stückkosten und Fertigungsfortschritt. Sie schützen ihren Standort und fliegen bei einem Save nicht mit.

**So baust du sie:** Wähle rechts den zu schützenden Planeten. Klicke die Anlage, prüfe freigeschaltete Technik und Fertigung, wähle **Stückzahl** und drücke **Fertigen**. Sieh nach dem Abschluss nach, ob die fertige Zahl im Verteidigungsbestand steht.

**Was passiert:** Die Anlage nimmt an Kämpfen an ihrem Planeten teil. Ein Teil zerstörter normaler Verteidigung wird nach einem Gefecht wiederaufgebaut; das ist keine Garantie für vollständige Wiederherstellung. Gegen Interplanetarraketen gelten eigene Regeln, insbesondere keine normale Wiederaufbauquote für die getroffene Verteidigung.

**Deine Entscheidung:** Stimme Verteidigung auf Gegnerberichte, eigene Technologie und die zu schützenden Vorräte ab. Ein Bunker schützt einen begrenzten Teil der Güter; Sensoren liefern Vorwarnung. Eine große Verteidigung ersetzt weder Versorgung noch das Saven einer beweglichen Flotte. Raketenbefehle findest du bei den weiteren Regierungs- und Flottenbefehlen.

### Flotten & Saven

**Hier findest du:** **Neue Flotte planen**, **Eigene Flotten** und erkannte gegnerische Bewegungen. Flottenkarten zeigen Kennung, Mission, Start, Ziel, Flugphase, Zeiten, Schiffe und Fracht.

**So startest du:** Öffne den Planer. Wähle **Start**, **Ziel**, **Auftrag**, **Geschwindigkeit**, bei Bedarf **Aufenthalt / Save-Wartezeit**, Schiffe und Fracht. Benutze **Flugzeit & Treibstoff prüfen**, lies Ankunft und Rückkehr und wähle dann **Flotte starten**. Erst die angenommene Aktion erscheint unter **Eigene Flotten**. **Rückrufen** leitet eine Rückkehr ein; es stellt die Schiffe nicht sofort wieder auf den Planeten.

**So savest du:** Wähle Auftrag **Saven**, ein anderes Ziel und eine passende Wartezeit. **Alle vorhandenen Schiffe auswählen** übernimmt den Bestand; **Save-Fracht automatisch verladen** füllt verfügbaren Frachtraum und berücksichtigt Treibstoff. Kontrolliere die Auswahl selbst. Fliege vor dem Angriff ab und plane die Rückkehr mit Abstand nach der Gefahr.

**Wenn es nicht klappt:** Prüfe Deuterium, Frachtraum, langsamstes Schiff, Flottenplatz und eine mögliche Blockade. Auch Sonden belegen Slots. Ein verbündeter Transport und ein feindlicher Angriff haben unterschiedliche Zielbedingungen. Eine erkannte Angriffsflotte lässt sich bei rechtzeitiger Sondenankunft über ihre Aufklärungsaktion untersuchen.

### Galaxie

**Hier findest du:** Die räumliche Ansicht **Galaxie → Sektor → Sonnensystem → Planet**. Sonnensysteme liegen als Sternpunkte im Sektor; im System bewegen sich Planeten optisch um ihre Sonne. **Galaxie**, **Zurück** und **Kamera zentrieren** helfen bei der Orientierung. **Umläufe pausieren** stoppt nur die sichtbare Animation.

**So untersuchst du einen Ort:** Öffne ein System und klicke einen Planeten oder seinen Listeneintrag. Seine Infokarte zeigt eigenes Wissen oder den bekannten Sondenbericht. Schließe sie über das Kreuz, Escape oder einen Klick außerhalb. Über **Systemliste & direkter Zugang** kannst du einen Ort gezielt statt durch Kamerabewegung auswählen.

**Was du mit der Information machst:** Plane bei einem unbekannten System zuerst einen Systemscan, danach eine Spionagemission zum einzelnen Planeten. Eine Kartenaktion kann das Flottenformular vorbereiten. Prüfe dort die Sonde, Zielkoordinate und Treibstoff und bestätige **Flotte starten**. **Berichte aktualisieren** liest vorhandene Beobachtungen neu ein; es ersetzt keinen Sondenflug.

**Beispiel:** Du siehst eine Planetenbahn, aber keinen Typ und keine Bewohner. Das ist eine Informationsgrenze. Ein dargestellter Körper ist weder automatisch leer noch als Kolonie reserviert. Erst ein eigener Bericht hilft bei deiner Entscheidung.

### Kolonisation

**Hier findest du:** Aktuelle Koloniezahl, durch Astrophysik erlaubte Kapazität, Verwaltungsgrenze, die Checkliste **Weg zur nächsten Kolonie**, laufende Besetzungen und **Integrität & Reparaturen** des ausgewählten eigenen Planeten.

**So bereitest du eine freie Kolonie vor:** Arbeite die Checkliste ab. Benutze **Startfracht anzeigen** und lies **Mindest-Startfracht für dein Volk**. Prüfe eigenes Wissen über System und Ziel, freien Kolonieplatz, Kolonieschiff, zusätzliche bewaffnete Eskorte, 3.000 Siedler, Startgüter und Flugtreibstoff. Über **Freie Kolonie planen** gelangst du zum Flottenplaner. Kontrolliere alle Angaben und bestätige den Start dort.

**Nach der Ankunft:** Eine erfolgreiche Gründung erscheint als eigener Planet. Wähle ihn rechts aus, lies Versorgung und stelle seine ersten Energie-, Nahrungs- und Förderanlagen in Auftrag. Die Startfracht baut keine Gebäude automatisch. Ein anderer Spieler kann einen freien Planeten vor dir erreichen.

**So reparierst du:** Wähle bei einer beschädigten Anlage **Reparatur prüfen**, kontrolliere Kosten und Zeit und drücke **Reparatur starten**. Die Reparatur benötigt die örtlichen Materialien und eine verfügbare Baustelle. Für eine feindliche Kolonie gelten zusätzlich Orbitkontrolle, höchstens 30 % Gebäudeintegrität und die Besetzungsfrist; ihr ursprünglicher Heimatplanet ist ausgeschlossen.

### Kampf & Verbände

**Hier findest du:** **Kampfsimulator**, Verbandsangriffe, Kampfberichte und bekannte Trümmerfelder. Eine Simulation ist eine Schätzung aus deiner Auswahl und dem letzten ausreichend vollständigen Spionagebericht.

**So prüfst du einen Angriff:** Benutze **Schiffe & Ziel auswählen** und fülle das Flottenformular mit deiner geplanten Streitmacht und Zielkoordinate aus. Kehre hierher zurück und wähle **Gewählte Flotte simulieren**. Lies Siegchance, Unentschieden, Verlustwerte, Alter des Berichts und die angenommene Gegnertechnik. Fehlende Schiffs- oder Verteidigungsdaten werden nicht einfach als Null gerechnet.

**Danach:** Entscheide anhand erwarteter Verluste, möglicher Beute und Frachtraum, ob du wirklich angreifen willst. Starte die Flotte im Flottenplaner ausdrücklich. Der Simulator sendet keinen Angriff. Nach dem Gefecht liest du den Kampfbericht; über **Recycler planen** kannst du eine Bergung bekannter Trümmer vorbereiten.

**Gemeinsam angreifen:** Bei einer eigenen Angriffsflotte gibt es **Verband eröffnen**. Ein berechtigter Partner trägt im Verbandsformular die eigene Flottenkennung und die **Führungsflotte** ein und benutzt **Verband beitreten**. Ziel, Beziehung und rechtzeitige gemeinsame Ankunft müssen passen. Ein Bündnis allein synchronisiert keine Flotten.

### Markt

**Hier findest du:** Den öffentlichen Markt, Vertragsverwaltung und **Weitere Regierungs- und Flottenbefehle**. Für den Markt brauchst du die passende Zivilisationsstufe und einen Markt; eine Order benötigt einen tatsächlichen Gegenhandel.

**So handelst du:** Wähle im Marktformular **Planet**, **Gut**, **Order** als Kaufen oder Verkaufen, **Menge** und **Preis je Einheit**. Prüfe Gesamtbetrag und Gebühren. Mit **Order einstellen** stellst du ein Angebot ein. Bei Kauf wird Geld, bei Verkauf Ware gebunden. Ein Abschluss braucht einen passenden Partner; die Lieferung folgt anschließend ihren Flug- und Blockaderegeln.

**Verträge mit Kaution:** Wähle **Partner**, **Vertrag** und **Kaution**, dann **Vertrag anbieten**. Der andere Spieler muss das Angebot bearbeiten. Persönliche Korrespondenz und einfache diplomatische Anfragen liegen im **Briefkasten**.

**Weitere Befehle verwenden:** Klappe **Weitere Regierungs- und Flottenbefehle** auf, wähle **Befehl**, fülle die dazu angezeigten Felder aus und benutze **Befehl ausführen**. Hier liegen etwa Stufenaufstieg, Steuern, Doktrin, Raketen, Bombardieren und besonderer Flottennachschub. Lies die Bedeutung, bevor du bestätigst: Ein angenommener Regierungs- oder Militärbefehl verändert die Welt.

### Briefkasten

**Hier findest du:** Persönlichen Posteingang, gesendete Briefe, Betreff, Lesestatus und Antworten. Deine Verbündeten stehen oben als direkt auswählbare Kontakte. **Brief schreiben** öffnet ein großes Schreibfenster. Gib den bekannten Spielernamen an oder wähle einen Verbündeten. Klicke einen Brief an und benutze **Antworten**. Ältere Post lässt sich nachladen und geladene Post durchsuchen. Allianz und aktuelle Beziehung stehen neben dem Namen.

**Diplomatische Anfrage:** Wähle Empfänger und Art: Allianzbeitritt, NAP, Verteidigungsbündnis, Handelsabkommen, Krieg oder Frieden. Ein Brieftext allein schließt keinen Vertrag. Angebote benötigen eine ausdrückliche Entscheidung; die Kriegserklärung gilt unmittelbar. Die ausführlichen Regeln stehen in Kapitel 17.

### Allianz

**Gründen und beitreten:** Ohne Mitgliedschaft kannst du eine Allianz mit eigenem Namen gründen. Die Leitung lädt bekannte Spieler ein; diese nehmen ihre Einladung an. Der geschützte Allianzbereich erscheint nach dem Beitritt.

**Gemeinsam handeln:** Mitglieder sehen die Mitgliederliste mit Rollen und eigenen Allianz-Koordinaten, den gemeinsamen Chat, interne Warenangebote und Hilferufe. Angebote binden echte Ware. Ein Kauf kostet Credits und Gebühren, danach folgt eine Lieferung. Hilfszusagen starten keine Flotte automatisch.

### Allianzpost · Führung

**Hier findest du:** Den Briefwechsel zwischen Allianzen und gemeinsame Vertragsangebote. Nur die obersten vier Mitglieder beider beteiligten Allianzen sind berechtigt. Adressiert wird der Allianzname. Normale Mitglieder haben keinen Zugriff; nach Austritt oder Verlust des Leitungsrangs entfällt er ebenfalls.

### Berichte

**Hier findest du:** Reichsereignisse und bekannte Spionage- oder Kampfergebnisse. Hier prüfst du, was tatsächlich angekommen, geschehen oder beobachtet worden ist. Persönliche Post hat ihren eigenen Briefkasten.

**So liest du einen Bericht:** Vergleiche Berichtstyp, Zielkoordinate, Beobachtungszeit und Informationsumfang. Trenne Ressourcen, Schiffe, Verteidigung, Gebäude und Technik. Ein Sondenbericht ist eine Momentaufnahme. Bei einem alten oder unvollständigen Bericht brauchst du vor einer wichtigen Entscheidung gegebenenfalls eine neue Sonde.

**So reagierst du:** Nach einer Aufklärung bewertest du ein Ziel. Nach einem Gefecht prüfst du Verluste, Beute und Trümmer. Bei einer Versorgungskrise suchst du den betroffenen Planeten auf und kontrollierst die verbleibende Frist. Nachrichten an andere Reiche verfasst du im **Briefkasten**; prüfe den Namen des Empfängers.

**Wenn etwas fehlt:** Prüfe, ob die Flotte schon am Ziel angekommen ist. Ein gestarteter Flug liefert nicht sofort einen Bericht. Nicht aufgeklärte militärische Daten bleiben unbekannt. Private Nachrichten und Spionageberichte eines anderen Reichs werden durch die Rangliste oder bloßes Zuschauen nicht freigegeben.

### Imperiumsvergleich

**Hier findest du:** Rangliste und öffentlich sichtbare Reichswerte. Damit beurteilst du die Entwicklung im Wettbewerb, ohne Zugriff auf die privaten Vorräte oder vollständige Flotten anderer Reiche zu erhalten.

**So benutzt du ihn:** Suche dein Reich, vergleiche Punktestand und Entwicklungsstand und prüfe über mehrere Zeitpunkte, ob deine Investitionen wirken. Ein einzelner Rang ist kein Kampfbericht. Für einen Angriff brauchst du eigene Aufklärung; für einen wirtschaftlichen Engpass deine Versorgungstabelle.

**Was die Punkte bedeuten:** Abgeschlossene Gebäude und Forschung, vorhandene Streitkräfte, Bevölkerung und Zivilisationsstufe tragen bei. Ein großer unverbauter Rohstoffvorrat bringt keine entsprechenden Investitionspunkte. Verluste können Punkte senken. Besiegte Reiche werden hinter aktiven eingeordnet; ihr Zustand ist endgültig für diese Epoche.

**Beispiel:** Du sparst zwei Tage Material und dein Rang steigt kaum. Nach einem fertigen Ausbau ändern sich Investitionspunkte. Eine verlorene Flotte ist anschließend kein dauerhaft behaltenes Punktekonto. Am Epochenende zählt die festgeschriebene Abschlussrangliste.

### Technologiebaum

**Hier findest du:** Die Abhängigkeiten zwischen Zivilisationsstufe, Gebäuden, Forschung, Schiffen und Verteidigung. Er beantwortet vor allem die Frage: Was fehlt mir vor meinem nächsten Ziel?

**So benutzt du ihn:** Suche zuerst das gewünschte Ziel, beispielsweise eine Spionagesonde oder ein Kolonieschiff. Lies seine direkten Voraussetzungen. Prüfe anschließend auch deren Voraussetzungen. Plane daraus die Bau- und Forschungsreihenfolge. Gehe zum eigentlichen Bestellen zu **Gebäude**, **Forschung** oder **Schiffswerft**.

**Was eine Freischaltung bedeutet:** Die Technik wird grundsätzlich nutzbar. Das betreffende Gebäude, Schiff oder Forschungsniveau entsteht dadurch noch nicht. Örtliche Werft- oder Laboranforderungen, Kosten, Personal und Bauteile bleiben nötig. Ein gutes Labor auf einem Planeten baut auch keine Werft auf einem anderen.

**Beispiel:** Astrophysik erlaubt eine weitere Kolonie. Du brauchst trotzdem Stufe IV, eine geeignete Werft und Orbitalwerft, die Kolonieschiff-Bauteile, Siedler, Eskorte und Startfracht. Nutze die Kolonisationscheckliste zusätzlich zum Technologiebaum.

### Modelle & Team

Hier verwaltest du bis zu vier benannte Modelle aus Ollama und OpenRouter, testest jeden Zugang und ordnest Modelle den Regierungsrollen zu. Ein Modell kann mehrere Rollen übernehmen. Die Texte **soul.md** und **rollen.md** sind pro Rolle editierbar, importierbar und als Markdown herunterladbar.

Öffne **＋ Modell hinzufügen**, wähle Anbieter und genaue Modell-ID, teste die Verbindung, besetze die gewünschten Rollen und speichere. Mit **Team starten / fortsetzen** beginnt die Ausführung. **Team stoppen** gibt die Steuerung zurück. Die bebilderte Anleitung in Kapitel 20 erklärt jeden Klick einschließlich Browserfreigabe und Fehlerbehebung.

### Pinwände

Für alle Spielweisen: Erstelle mehrere Wände, ergänze Karten mit Wissen, Zielen und Empfängern, ändere ihre Reihenfolge mit ↑/↓ und bearbeite oder lösche alte Inhalte. **Pinwände speichern** schreibt die Änderungen dauerhaft ins Konto. Alle Modelle lesen sie vor jeder neuen Entscheidung zusammen mit den bestätigten Übergaben. Ausführliche Beispiele stehen in Kapitel 20.

### Spielerprofil

**Hier findest du:** Deine Spielweise und die persönliche Fortschrittsdarstellung. Die Volkswahl ist nach der Reichsgründung für die laufende Epoche festgelegt.

**So wechselst du die Steuerung:** Wähle **Mensch**, **Agent** oder **Gemischt** und bestätige **Platz belegen / Spielweise ändern**. Reich und Teilnehmerplatz bleiben dabei erhalten. Ein Spielweisenwechsel stoppt eine laufende Browser-Agentensteuerung. Richte Rollen und Modelle unter **Modelle & Team** anschließend bewusst neu ein.

**So stellst du Timer ein:** Bei der Fortschrittsdarstellung wählst du Systemvorgabe, **Uhrzeiger · im Uhrzeigersinn einfärben** oder **Ruhige Darstellung · ohne Uhrzeiger**. Die Wahl verändert die optische Anzeige, keine Kosten oder Bauzeiten. Restzeiten und Serverabschlüsse bleiben entscheidend.

**Beispiel:** Du möchtest die Flotten selbst führen und nur die Wirtschaft abgeben. Wähle **Gemischt**, lasse Feldherr und Diplomat frei und aktiviere zunächst den Verwalter. Während dieser Rolle dürfen eigene gleichzeitige Wirtschaftsbefehle gesperrt sein. Stoppe den Agenten, bevor du seine Aufgaben manuell übernimmst.

### Spielanleitung

**Hier findest du:** Hilfe im Browser und **Vollständiges Regelwerk dieser Welt**. Diese Regelreferenz stammt vom verbundenen Server und beschreibt das dort aktive Profil.

**So benutzt du sie:** Lies bei einer unbekannten Mechanik zuerst ihre Erklärung im Spielguide und bei einer konkreten Kosten- oder Freischaltungsfrage zusätzlich die Regelreferenz und Aktionsdetails. Vergleiche den laufenden Weltstatus: Tempo, Pause und Epochenende betreffen alle Spieler. Ein normaler Spielzugang besitzt keinen privaten Verwaltungszugang.

**Wenn du feststeckst:** Notiere Bereich, Planet, Auftrag und genaue Ablehnungsnachricht. Prüfe Heimat- oder Ortsbestand, Voraussetzungen, Warteschlange, Flottenplätze und Steuerungsrolle. Bei einem Fehlerbericht sind ein Zeitpunkt und die konkrete Aktion hilfreicher als die Aussage, ein Knopf funktioniere nicht.

**Beispiel:** Ein angekündigter Ausbau braucht zwei Spielstunden. Bei 1× sind das zwei reale Stunden, bei 60× zwei reale Minuten. Die aktuell im Spiel angezeigte Geschwindigkeit erklärt die tatsächliche Wartezeit. Eine pausierte Planetenanimation ist dagegen keine Weltpause.

## 9 Spielzeit Bauzeit und Pausen

Der aktuelle Server steht auf **1× Tempo**. Eine reale Stunde entspricht dann einer Spielstunde. Der Betreiber kann das Tempo verändern.

| Tempo | Zwei Spielstunden dauern real |
|---|---|
| 1× | Zwei Stunden |
| 60× | Zwei Minuten |
| 3.600× | Zwei Sekunden |

Befehle werden direkt geprüft. Ein angenommener Bauauftrag beginnt, sobald seine Bauposition frei ist und die Voraussetzungen erfüllt sind.

Produktion läuft mit der Spielzeit. Bevölkerung, Versorgung, Steuern, Stabilität und Forschung werden an Spielstunden verarbeitet; der Flottenunterhalt wird täglich verrechnet. Bauabschlüsse und Flottenankünfte haben eigene Ereigniszeitpunkte.

Bei Forschung ist die angezeigte Restzeit eine Prognose mit der aktuellen Forschungsleistung. Wenn Energie, Personal oder Labore ausfallen, stimmt eine ältere Prognose nicht mehr.

Eine **Weltpause** des Betreibers hält die Spielzeit und Aufträge an. Eine ruhige Kartenansicht hält nur die optische Bewegung an. Bei einer unterbrochenen Browserverbindung ist ein lokal sichtbarer Timer kein Beweis für einen fertigen Serverauftrag.

**Beispiel:** Eine Rettungsfrist von 72 Spielstunden dauert bei 1× drei reale Tage. Bei 60× bleiben dafür nur 72 reale Minuten. Ein schneller Testserver verlangt deshalb eine völlig andere Reaktionsgeschwindigkeit.

## 10 Wirtschaft Versorgung und Bevölkerung

### Die Güter und ihre Aufgaben

| Wert oder Gut | Hauptaufgabe |
|---|---|
| **Erz** | Grundstoff für Gebäude, Schiffe und Weiterverarbeitung |
| **Kristall** | Grundstoff für Technik, Gebäude, Forschung und Elektronik |
| **Deuterium** | Flugtreibstoff, Fusion und verschiedene Produktions- und Forschungskosten |
| **Nahrung** | Lebensversorgung der Bevölkerung außer bei Syntheten |
| **Legierung** | Weiterverarbeiteter Werkstoff, aus vier Erz je Einheit |
| **Elektronik** | Weiterverarbeitung, aus einem Erz und zwei Kristall je Einheit |
| **Konsumgut** | Versorgung und Entwicklung, aus einem Erz, einem Kristall und einem halben Deuterium je Einheit |
| **Xenokristall** | Späterer Spezialrohstoff aus geeigneten Nebelsystemen mit passender Anlage |
| **Energie** | Laufende Leistung für Bevölkerung und Anlagen; kein verladenes Lagergut |
| **Credits** | Reichsweite Währung für Unterhalt, Handel und Verträge |
| **Bauteile** | Beispielsweise Antriebskerne und Habitatmodule für anspruchsvolle Fertigung |

Die Rohstoffraten in der Oberfläche berücksichtigen bereits laufende Produktion und Verbrauch. Subtrahiere die Nahrung der Bevölkerung oder den Erzbedarf einer Gießerei nicht noch einmal von einer bereits angezeigten **Nettorate**.

**Beispiel:** Eine Gießerei liefert eine Legierung und verbraucht dafür vier Erz. Deine Mine fördert zehn Erz im selben Zeitraum. Ohne andere Verbraucher bleiben sechs Erz übrig. Mehr verarbeitende Betriebe können deshalb einen vorher positiven Erzsaldo negativ machen.

### Nahrung und Energie

Im Grundprofil verbrauchen 1.000 Einwohner **40 Nahrung je Spielstunde** und drei Konsumgüter. Nahrung wird aus Produktion und Vorräten gedeckt. Ein negativer Saldo ist eine Warnung, aber noch keine sofortige Niederlage.

**Beispiel:** Bei 500 Nahrung und einer Nettorate von −25 je Stunde reicht der Vorrat unter unveränderten Bedingungen ungefähr 20 Spielstunden. Bevölkerungswachstum oder weitere Verbraucher können diese Zeit verkürzen.

Energie ist eine laufende Bilanz. Wenn Anlagen und Bevölkerung mehr benötigen, als du erzeugst, sinkt die mögliche Wirtschaftsleistung. Syntheten benötigen zusätzlich Energie für ihre Bevölkerung.

**Beispiel:** 80 erzeugte Energie bei 100 benötigter Energie bedeuten grundsätzlich nur 80 % Deckung. Eine weitere Mine kann den Mangel verschärfen. Prüfe zunächst Kraftwerke, Verbraucher und Arbeitsprioritäten.

Fusion braucht Deuterium. Ein stärkeres Fusionskraftwerk kann dir deshalb Treibstoff für Flotten wegnehmen. Behalte die gemeinsame Deuteriumbilanz im Blick.

### Arbeiter Spezialisten Wohnraum und Stabilität

Grundsätzlich stehen 60 % der Bevölkerung als Arbeitskräfte zur Verfügung. Anspruchsvolle Anlagen und Forschung brauchen außerdem Spezialisten; die Akademie hilft beim Ausbau. Zu wenig Personal oder Energie kann erklären, warum eine fertige Anlage weniger leistet als erwartet.

Wohnraum begrenzt weiteres Wachstum. Nahrung allein reicht nicht, wenn die Wohnkapazität ausgeschöpft ist. Konsumgüter und Stabilität werden für spätere Entwicklungsschritte wichtig.

Steuern können zwischen 0 und 50 % liegen; der Ausgangswert ist 10 %. Höhere Steuern bringen Einnahmen, belasten aber die angestrebte Stabilität. Besonders oberhalb von 15 % steigt dieser Druck.

Unter **30 Stabilität** herrschen Unruhen: Neue Bauaufträge werden gesperrt und laufende Bauarbeiten unterbrochen. Unter 40 verschwindet das reguläre Bevölkerungswachstum. Ein kurzer hoher Steuerertrag kann dadurch langfristig teuer werden.

### Lager Bunker und laufende Kosten

Ein Lager begrenzt örtliche Vorräte. Ein Bunker schützt einen Teil bestimmter Rohstoffe vor Plünderung. Er erzeugt keine Güter und schützt nicht vor Gebäudebombardement.

Schiffe kosten laufend Credits. Im Profil beträgt der tägliche Unterhalt 0,5 % ihres gewichteten Bauwerts; jede Kolonie kostet zusätzlich 300 Credits je Spieltag. Ortsfeste Verteidigung hat keinen solchen Schiffsunterhalt.

Nach drei nicht bezahlten Tagen droht Flottendesertion: Pro weiterem unbezahltem Tag können 10 % der Schiffe verloren gehen. Leere Credits bedeuten daher nicht sofort eine Niederlage, sind aber eine ernsthafte Warnung.

**Beispiel:** Eine neue Kolonie erweitert Produktion und Möglichkeiten. Wenn sie unversorgt bleibt und gleichzeitig die Credits für Kolonie und Flotte fehlen, kann sie dein Reich zunächst schwächen. Plane ihre Versorgung vor dem Start.

## 11 Bauen Forschen und Zivilisationsstufen

### Gebäude und Warteschlangen

Ein Planet bearbeitet einen Bauauftrag gleichzeitig. Seine Bauschlange umfasst maximal **fünf Aufträge einschließlich des gerade laufenden**.

Die Baukosten werden bezahlt, wenn der jeweilige Auftrag wirklich beginnt. Ein vorgemerkter Auftrag reserviert nicht automatisch alle später benötigten Rohstoffe. Kann der erste wartende Auftrag nicht bezahlt werden, bleiben auch die dahinterstehenden Aufträge liegen.

**Beispiel:** Die erste Mine wird fertig. Als Nächstes steht eine teure Lagerstufe an, für die Kristall fehlt. Die günstige Farm dahinter beginnt dadurch nicht von selbst. Du musst die fehlenden Güter beschaffen oder die Reihenfolge bewusst korrigieren.

Gebäudestufen brauchen zunehmend Material und Zeit. Außerdem gelten Zivilisationsstufe, Technologie, Bauplätze und örtliche Voraussetzungen. Fertige Gebäude können bei Unruhen oder Beschädigung eingeschränkt wirken.

### Forschung und Fertigung

Ein Reich hat **eine aktive Forschung und bis zu drei wartende Forschungsaufträge**. Die Forschungsleistung kommt aus seinen Laboren. Der Forschungsstart über die Browserkacheln benutzt die **Heimatwelt**: Dort müssen die verlangte Laborstufe und die Materialien beim tatsächlichen Projektstart vorhanden sein. Ein rechts ausgewählter anderer Planet ändert diesen Startort nicht.

Eine freigeschaltete Forschung ist noch nicht erforscht. Ebenso ist ein freigeschaltetes Schiff noch nicht gebaut. Die Werft benötigt ihre Voraussetzungen, Material und Besatzung. Fertigung bindet reale Rohstoffe; die Besatzung kommt aus deiner Bevölkerung.

**Beispiel:** Zivilisationsstufe IV macht fortgeschrittene Technik grundsätzlich erreichbar. Ein Kolonieschiff erscheint trotzdem erst als baubar, wenn Werft, Orbitalwerft, Bauteile und weitere Voraussetzungen vorhanden sind.

### Die fünf Zivilisationsstufen

Du beginnst auf Stufe I. Für einen Aufstieg musst du die Bedingungen **48 Spielstunden durchgehend** erfüllen und anschließend den Aufstieg auslösen und bezahlen. Sinkt eine Bedingung zwischendurch darunter, beginnt die Haltezeit erneut.

Die Einwohnerzahl zählt reichsweit. Bei einer verlangten Gebäudestufe genügt der höchste passende Ausbau auf einem eigenen Planeten. Vorgaben zu Stabilität und Konsumdeckung beziehen sich auf den Heimatplaneten.

| Zielstufe | Wesentliche Voraussetzungen | Aufstiegskosten |
|---|---|---|
| **II** | 2.000 Einwohner; Erzmine 5; Kristallmine 5; auf allen eigenen Planeten Nahrung und Energie im Plus, bei Syntheten ohne Nahrungspflicht | 2.000 Erz, 1.000 Kristall |
| **III** | 8.000 Einwohner; Gießerei 3; Elektronikfabrik 2; Energietechnik 3; Heimatstabilität mindestens 55 | 10.000 Erz, 6.000 Kristall, 500 Legierung, 200 Elektronik |
| **IV** | 32.000 Einwohner; Werft 4; Impulsantrieb 3; Astrophysik 1; Heimatstabilität mindestens 60; Konsumdeckung mindestens 80 % | 40.000 Erz, 25.000 Kristall, 10.000 Deuterium, 2.000 Legierung, 1.000 Elektronik |
| **V** | 700.000 Einwohner; Hyperraumantrieb 1; mindestens drei Kolonien | 200.000 Erz, 150.000 Kristall, 50.000 Deuterium, 10.000 Legierung, 5.000 Elektronik, 5.000 Xenokristall |

Die Aufstiegskosten müssen auf dem Heimatplaneten bereitliegen. Die Oberfläche zeigt den Stand der Voraussetzungen; der Befehl **Stufenaufstieg** ist auch bei den weiteren Regierungsbefehlen zu finden.

Auf der höchsten Stufe kommen große Projekte hinzu: beispielsweise ein Orbitalring mit zusätzlichen Bauplätzen und Wohnraum, ein Archiv mit stärkerer Forschung oder ein Versorgungsnetz mit höherer Energieausbeute. Auch diese Projekte kosten Material und Bauzeit.

**Beispiel:** Du hast 2.000 Einwohner und die nötigen Minen. Nach 30 Stunden sinkt auf einer Kolonie die Energie ins Minus. Die Haltezeit für Stufe II zählt danach nicht einfach bei Stunde 31 weiter. Erst wenn alle Bedingungen wieder erfüllt sind, beginnt sie neu.

## 12 Die Galaxie erkunden und Planeten ausspähen

### Die räumliche Karte

Die Onlinegalaxie hat zwei Sektoren mit jeweils 60 Sonnensystemen und zwölf Planetenpositionen. Eine Koordinate wie **1:27:6** bedeutet Sektor 1, System 27, Position 6.

Unter **Galaxie** gehst du von der Sternwolke des Sektors zum Sonnensystem und dann zum Planeten. In der Systemansicht bewegen sich die Planeten optisch auf ihren Bahnen um die Sonne. Ziehen dreht die Kamera, Scrollen zoomt, **Kamera zentrieren** zeigt das System vollständig.

Ein Klick auf einen Planeten, seine Nummer oder seinen Listeneintrag öffnet seine **Infokarte** über der Ansicht. Du schließt sie mit ×, Escape oder einem Klick daneben. Die Liste bietet dieselben Ziele auch ohne genaues Treffen eines bewegten Planeten.

Die sichtbare Umlaufbewegung ist eine Darstellung. Sie verändert nicht spontan die im Flugplan berechnete Reisezeit.

### Ein Systemscan ist kein Scan aller Planeten

Dein Heimatsystem ist bereits kartiert. Die anderen Planeten darin bleiben trotzdem unbekannt, solange du sie nicht einzeln untersucht hast.

Für ein unbekanntes System brauchst du zuerst die Mission **System erkunden** mit einer Sonde. Sie kartiert das System, etwa Nebel und Asteroidengürtel. Danach untersuchst du mit **Spionage** jeden interessanten Planeten einzeln.

**Beispiel:** Du schickst eine Sonde zu System 1:27. Danach weißt du, dass dieses System existiert und welche systemweiten Merkmale erkannt wurden. Ob Position 4 bewohnt ist und welche Eigenschaften Position 8 hat, erfährst du erst durch deren eigene Sondenmissionen.

Eine zurückgekehrte Sonde kann erneut eingesetzt werden. Du brauchst also einen echten Sondeneinsatz pro zu untersuchendem Ziel, nicht zwingend für jeden Scan ein fabrikneues Schiff. Sonden brauchen Treibstoff und einen freien Flottenplatz und können verloren gehen.

### Einen Sondenflug starten

1. Öffne System oder Planet auf der Karte.
2. Wähle **Systemscan** oder **Sonde planen**.
3. Prüfe in der Flottenplanung Startplanet, Ziel, Mission, Sonde und Treibstoff.
4. Wähle **Flotte starten**. Das bloße Öffnen des Plans sendet nichts ab.
5. Warte auf die Ankunft und lies den Bericht unter **Berichte**.
6. Aktualisiere bei Bedarf die Berichte.

Ein unaufgeklärter grauer Planet ist nicht automatisch frei. Planetenpositionen verraten im Onlineprofil auch nicht zuverlässig ihren Typ. Eigenschaften und Belegung werden erst durch erlaubte Aufklärung sichtbar.

Berichte sind **Momentaufnahmen mit Zeitstempel**. Ein neuer Browseraufruf liefert keinen frischen Sondenbericht. Ein Gegner kann seit deinem letzten Bericht Schiffe verlegt, Güter verbraucht oder Gebäude verändert haben.

### Wie detailliert ein Planetenbericht wird

Bei bewohnten Planeten hängt die militärische Tiefe von der Sondenzahl und dem Unterschied der Spionagetechnik ab:

**Informationswert = Sondenzahl + eigene Spionagetechnik − gegnerische Spionagetechnik**

Ab Informationswert 2 werden Schiffe, ab 3 Verteidigung, ab 5 Gebäude und ab 7 Forschung gemeldet. Planeteneigenschaften und Rohstoffbeobachtung ersetzen diese militärische Tiefe nicht.

**Beispiel:** Zwei Sonden, eigene Spionagetechnik 3 und gegnerische Spionagetechnik 2 ergeben Informationswert 3. Der Bericht kann Schiffe und Verteidigung enthalten, aber noch keine vollständige Gebäude- oder Forschungsliste.

Mehr Sonden können mehr Informationen liefern, erhöhen aber auch das Risiko, dass die Sondengruppe entdeckt und abgeschossen wird.

Die Ressourcenangaben beschreiben Vorräte oder Produktionsbedingungen. Es gibt hier keine gesonderte endliche Erzader, die du durch einen Scan als unbegrenzt verfügbaren Lagerbestand erhältst.

## 13 Angriffe erkennen und anfliegende Flotten untersuchen

### Die Grundwarnung

Ohne Sensorenausbau wird ein feindlicher Anflug grundsätzlich **zwei Spielstunden vor seiner Ankunft** angezeigt. Dauert der ganze Flug kürzer, liegt die Warnung entsprechend frühestens ab dem Start vor.

Sobald deine Sensoren die Flotte erfassen, sind Ziel und Ankunftszeit klar. Die Grundwarnung verrät noch nicht automatisch Besitzer, Schiffstypen, genaue Zahl, Fracht oder Herkunft.

### Vier zusammenwirkende Bestandteile

Für eine bessere passive Erkennung brauchst du:

1. **Geheimdienst** am bedrohten Planeten.
2. **Sensorphalanx** am bedrohten Planeten.
3. Die reichsweite Forschung **Überwachungstechnik**.
4. Die reichsweite **Spionagetechnik**.

Die effektive Sensorstufe ist die **kleinste Stufe dieser vier Bestandteile**. Beschädigte örtliche Gebäude werden entsprechend ihrer Integrität schwächer berücksichtigt. Fehlt ein Bestandteil vollständig, bleibt die effektive Stufe bei null.

Syntheten erhalten eine zusätzliche Sensorstufe, wenn bereits alle vier Bestandteile wirksam vorhanden sind. Sie bekommen den Ausbau nicht geschenkt.

**Warnzeit = 120 Minuten + 30 Minuten × effektive Sensorstufe**

Die gegnerische Abschirmtechnik verdeckt Einzelheiten. Sie entfernt die grundsätzliche Ankunftswarnung nicht. Für die passive Berichtstiefe zählt die effektive Sensorstufe abzüglich der beim Abflug vorhandenen Abschirmstufe:

| Vorsprung deiner Sensoren | Was zusätzlich erkennbar ist |
|---|---|
| 0 oder weniger | Ankunft und Ziel, keine Flottendetails |
| 1 bis 2 | Besitzer und eine ungefähre Anzahl |
| 3 bis 5 | Genaue Gesamtzahl |
| Ab 6 | Genaue Schiffstypen und Mission |

**Beispiel:** Alle vier Bestandteile stehen auf Stufe 3. Du erkennst den Anflug 210 Spielminuten, also 3 Stunden 30 Minuten, vor Ankunft. Hat der Angreifer Abschirmung 1, bleibt ein Detailvorsprung von 2: Besitzer und ungefähre Zahl werden sichtbar. Mit Geheimdienst 10, aber Sensorphalanx 0, hättest du weiterhin nur die Grundwarnung.

### Eine erkannte Flotte aktiv sondieren

Du kannst eine bereits erkannte anfliegende Flotte mit einer eigenen Sondenmission untersuchen. Benutze den angezeigten Flottenkontakt. Eine erratene Kennung einer noch unsichtbaren Flotte ist kein erlaubtes Aufklärungsziel.

Die Sonde muss die Flotte **vor ihrer Ankunft** erreichen können. Startplanet, Treibstoff, Flottenplatz und verbleibende Flugzeit werden geprüft.

Die aktive Aufklärung vergleicht:

**Eigene Spionagetechnik + wirksamer Geheimdienst am Startplaneten + gegebenenfalls Veyari-Bonus**

Ohne gegnerische Abschirmung gelingt die Detailaufklärung grundsätzlich. Mit Abschirmung muss dein Wert **echt höher** sein. Gleichstand genügt nicht. Für Frachtinformationen brauchst du zusätzlich einen Vorsprung von mindestens fünf.

**Beispiel:** Spionagetechnik 2 und Geheimdienst 3 ergeben Aufklärungswert 5. Gegen Abschirmung 4 reicht das; gegen 5 nicht. Ein Veyari mit erforschter Spionagetechnik erhält hier den zusätzlichen Punkt.

Auch eine erfolgreiche Untersuchung hat ein Abschussrisiko: grundsätzlich 10 %, bei gescheiterter Aufklärung gegen Abschirmung 50 %. Eine größere Sondengruppe ersetzt den nötigen Technikvorsprung nicht.

Wird die Zielflotte vorher zurückgerufen oder ist schon angekommen, kann der Kontakt verloren gehen. Der Bericht ist außerdem eine Beobachtung dieses Zeitpunkts; ein später veränderter Flugplan bleibt möglich.

## 14 Flotten verschicken und richtig saven

### Vor jedem Flug

Öffne **Flotten & Saven** und **Neue Flotte planen**. Wähle Startplanet, Schiffe, Ziel, Mission, Geschwindigkeit und gegebenenfalls Wartezeit. Benutze **Flugzeit & Treibstoff prüfen**, bevor du startest.

Die Flotte fliegt nach ihrem langsamsten Schiff. Eine langsamere Geschwindigkeit kann Treibstoff sparen, verlängert aber die Reise. Der Flugplan berechnet die tatsächlich nötige Deuteriummenge für die gewählte Mission.

Du hast grundsätzlich **einen Flottenplatz plus einen weiteren je Stufe Computertechnik**. Aufklärung, Transport und Saven belegen ebenfalls Plätze.

**Beispiel:** Dein einziger Flottenplatz ist von einer Sonde belegt. In dieser Zeit kannst du nicht zusätzlich deine Kriegsflotte saven. Computertechnik und eine bewusste Reserve sind deshalb auch Verteidigung.

### Die wichtigsten Missionen

| Mission | Zweck |
|---|---|
| **System erkunden** | Unbekanntes System mit einer Sonde kartieren |
| **Spionage** | Einen bestimmten Planeten untersuchen |
| **Transport** | Güter liefern und danach zurückfliegen |
| **Stationieren** | Schiffe auf einen anderen eigenen Planeten verlegen |
| **Halten** | Zeitweise bei einem berechtigten Partner verteidigen |
| **Saven** | Schiffe mit Fracht auf einen Hinflug mit Wartephase und Rückkehr schicken |
| **Angriff** | Kämpfen und bei Erfolg begrenzt plündern |
| **Kolonisieren** | Einen bekannten freien Planeten besiedeln |
| **Abbau** | Mit geeigneten Bergbauschiffen am Asteroidengürtel Rohstoffe gewinnen |
| **Recyceln** | Mit Recyclern Trümmer bergen |
| **Blockade** | Den feindlichen Orbit kontrollieren und Verkehr behindern |
| **Bombardieren** | Nach ausgeschalteter bewaffneter Verteidigung Gebäude beschädigen |
| **Kampfkolonisieren** | Eine feindliche Kolonie unter den Übernahmeregeln erobern |

Für den Asteroidengürtel ist Position **0** des Systems vorgesehen. Das ist kein normaler Planetenplatz. Abbau und Haltemissionen haben zusätzliche Zeit- und Einheitenvoraussetzungen; prüfe die jeweilige Missionsmeldung.

### Was Saven bedeutet

Beim Saven sind deine Schiffe und ihre mitgenommene Fracht während der Abwesenheit nicht auf dem bedrohten Planeten. Die Mission führt an einem anderen gültigen Ziel eine Wartephase aus und kehrt zurück. Dort erfolgt keine normale Warenlieferung und kein Angriff.

Während dieses Save-Flugs sind Schiffe und Fracht vor dem Angriff auf den Planeten geschützt. Gefährlich sind insbesondere der zu späte Start und die Rückkehr auf einen wieder bedrohten oder blockierten Planeten.

Du kannst bis zu **72 Spielstunden Wartezeit** einstellen. Hin- und Rückflug kommen hinzu.

### Ein Save Schritt für Schritt

1. Prüfe die Ankunft des Angreifers und deinen freien Flottenplatz.
2. Wähle **Saven** und ein anderes gültiges Ziel.
3. Wähle die zu rettenden Schiffe.
4. Lass bei Bedarf die Fracht automatisch füllen.
5. Prüfe, dass genug Deuterium für den geplanten Flug reserviert bleibt.
6. Wähle eine Rückkehrzeit, zu der du dich wieder kümmern kannst.
7. Starte rechtzeitig und kontrolliere, dass der Server den Abflug bestätigt hat.
8. Prüfe später Rückkehr, neue Angriffe und den Zustand des Rückkehrplaneten.

**Zeitbeispiel bei 1×:** Um 20:30 erscheint ein Angriff für 22:30. Du savest um 20:40. Der Hinflug dauert laut Vorschau eine Stunde, die Wartephase sechs Stunden und der Rückflug eine Stunde. Deine Flotte ist acht Stunden abwesend und kehrt am nächsten Tag um 04:40 zurück. Bei höherem Welttempo verkürzt sich diese reale Abwesenheit.

**Frachtbeispiel:** Der freie Laderaum beträgt 5.000. Du besitzt 3.000 Erz, 1.500 Kristall und 400 Deuterium. Wenn die konkrete Vorschau 100 Deuterium als Treibstoff verlangt, kannst du beispielsweise 3.000 Erz, 1.500 Kristall und 300 Deuterium als Fracht mitnehmen. Das sind 4.800 Frachteinheiten. Treibstoff und verladenes Deuterium sind getrennt zu beachten.

Gebäude, Verteidigung und Bevölkerung bleiben auf dem Planeten. Ein Save schützt sie nicht. Credits sind reichsweit und keine normale Beutefracht.

Bei einer aktiven feindlichen Orbitblockade kann ein Abflug bereits verhindert werden. Deshalb darfst du Saven nicht erst nach der gegnerischen Ankunft planen. Auch exakt zeitgleiche Rückkehr und Angriff sind riskant: Plane Abstand ein.

Kann die Flotte nicht normal zurückkehren, gelten die tatsächlichen Rückkehr- und Blockaderegeln. Ein Besitzwechsel kann einen zusätzlichen Weg zum Heimatplaneten nötig machen; die Schiffe teleportieren nicht dorthin.

## 15 Kampf Verteidigung und gemeinsame Angriffe

### Wie eine Schlacht abläuft

Eine Schlacht dauert höchstens **sechs Kampfrunden**. Die Einheiten schießen auf einzelne gegnerische Ziele. Zu Beginn einer Runde werden Schilde erneuert; bereits verlorene Struktur bleibt beschädigt.

Waffen-, Schild- und Panzertechnik verbessern ihren jeweiligen Wert grundsätzlich um 10 % je Forschungsstufe. Hinzu kommen passende Volksboni oder Nachteile.

Ein einzelner Treffer unter 1 % des vollen gegnerischen Schildwerts verpufft. Wenn ein Schiff unter 70 % seiner ursprünglichen Struktur fällt, besteht zusätzlich eine vom Schaden abhängige Explosionswahrscheinlichkeit.

**Beispiel:** Bei einem vollen Schildwert von 1.000 verpufft ein Treffer mit Stärke 5. Ein Treffer mit Stärke 1.500 verbraucht dagegen die 1.000 Schildpunkte und verursacht zusätzlich 500 Strukturschaden.

Einige Einheiten haben **Schnellfeuer** gegen bestimmte Ziele. Schnellfeuer 6 bedeutet eine Chance von 5 zu 6 auf einen weiteren Schuss gegen den betreffenden Zieltyp. Es bedeutet nicht garantiert sechs Treffer pro Runde.

Der Kampf ist gewonnen, wenn keine gegnerischen Kampfeinheiten mehr übrig sind. Überleben nach sechs Runden auf beiden Seiten Einheiten, endet er unentschieden. Deshalb reichen Schiffszahl oder ein einziger summierter Angriffswert nicht, um den Ausgang sicher vorherzusagen.

### Simulator und Berichte

Der Simulator hilft dir, eine Flottenzusammenstellung einzuschätzen. Für eine belastbare Zielbelegung brauchst du einen Spionagebericht mit Schiffen und Verteidigung. Fehlen Technikdaten, muss die Simulation mit Annahmen arbeiten.

**Beispiel:** Dein Bericht ist zehn Stunden alt. Seitdem kann der Gegner eine Flotte stationiert haben. Eine gute simulierte Siegchance auf Grundlage des Berichts garantiert deshalb keinen Sieg bei der echten Ankunft.

Lies nach einer Schlacht Verluste, Trümmer, Beute und den verbliebenen Zustand. Ein Kampfgewinn macht dich weder automatisch zum Besitzer des Planeten noch zum Sieger über das ganze Reich.

### Beute Trümmer und Verteidigung

Ein erfolgreicher normaler Angriff plündert grundsätzlich **40 % der ungeschützten Güter**, begrenzt durch freien Frachtraum. Krath haben eine Quote von 50 %. Credits sind keine normale Beute.

Ein Bunker schützt je Stufe 20.000 Einheiten der betreffenden Grundrohstoffe beziehungsweise 4.000 der weiterverarbeiteten Güter.

**Beispiel nur für Erz:** Auf dem Ziel liegen 50.000 Erz. Bunker 1 schützt davon 20.000. Von den verbleibenden 30.000 sind grundsätzlich 12.000 plünderbar, bei Krath 15.000. Mit nur 8.000 freiem Frachtraum kannst du höchstens 8.000 davon mitnehmen.

Aus zerstörten Schiffen entstehen Trümmer in Höhe von 30 % ihres Erz- und Kristallbauwerts. Recycler können diese bergen. Bei normalem Kampf werden grundsätzlich 70 % zerstörter ortsfester Verteidigung wiederhergestellt.

Verteidigung kann nicht gesavt werden. Sie kostet keinen laufenden Schiffsunterhalt, hilft aber auch nicht beim Transport oder bei einem entfernten Angriff.

### Raketen

Ein Raketensilo lagert Abfang- und Interplanetarraketen. Es hat zehn Lagerplätze je Stufe. Jede Rakete belegt einen Platz; laufende Fertigungsaufträge zählen bereits mit. Die Fertigung braucht fünf Spielminuten je Rakete.

Eine Abfangrakete fängt eine Interplanetarrakete ab und wird dabei verbraucht. Interplanetarraketen treffen einen ausgewählten Typ ortsfester Verteidigung. Die Reichweite beträgt fünf Systeme je Silostufe innerhalb desselben Sektors. Der Flug dauert 20 Spielminuten plus eine Minute je System Abstand.

Raketenangriffe sind echte feindliche Handlungen mit Materialkosten und Flugzeit. Für sie gibt es eine eigene Warnmeldung ab dem Start. Sie bringen keine Beute und übernehmen keinen Planeten. Durch Raketen zerstörte Verteidigung wird nicht nach der normalen 70-%-Kampfregel wiederhergestellt. Die Bau- und Startbefehle stehen bei den weiteren Regierungs- und Flottenbefehlen.

### Verbände

Für einen gemeinsamen Angriff eröffnet ein Spieler einen Verband zu seiner bereits fliegenden Angriffsflotte. Eigene oder passende verbündete Angriffsflotten können beitreten, wenn sie dasselbe Ziel haben und die Verbandsbedingungen erfüllen. Ein Verteidigungsbündnis allein ersetzt die gemeinsame Allianz dafür nicht.

Die Ankunft wird auf die späteste nötige Ankunft abgestimmt. Eine frühere Flotte wird dadurch verzögert; eine spätere wird nicht beschleunigt. Kontrolliere nach jedem Beitritt die neue gemeinsame Zeit.

**Beispiel:** Deine Flotte käme um 18:00 an, die eines Allianzpartners erst um 18:20. Im gültigen Verband kann die gemeinsame Ankunft auf 18:20 verschoben werden. Das gibt auch dem Gegner mehr Reaktionszeit.

### Anfängerschutz

Ein neues Reich hat grundsätzlich zehn Spieltage Anfängerschutz. Er endet früher, wenn es Zivilisationsstufe III erreicht oder selbst einen feindlichen Angriff startet. Er wird ab dem eigenen Beitritt gerechnet.

Eine kleine Punktzahl allein schützt nach Ablauf dieses Schutzes nicht automatisch vor stärkeren Reichen. Diplomatie, Sensoren, eine tragfähige Wirtschaft und Saven bleiben wichtig.

## 16 Kolonisieren Erobern und Reparieren

### Eine freie Kolonie gründen

Astrophysik bestimmt die Zahl möglicher Kolonien: Stufe 1 erlaubt eine, Stufe 3 zwei, Stufe 5 drei; höchstens acht. Der ursprüngliche Heimatplanet zählt dabei nicht als Kolonie. Laufende Kolonisationsvorhaben belegen ebenfalls geplante Kapazität.

Für eine tatsächliche Gründung brauchst du mehr als Astrophysik:

- Das System und der konkrete Planet müssen aufgeklärt sein.
- Das Ziel muss bei der Ankunft noch frei sein.
- Dein Reich muss die erforderliche freie Koloniekapazität haben.
- Ein **Kolonieschiff** muss verfügbar sein. Seine Fertigung verlangt Zivilisationsstufe IV, Werft 4, Orbitalwerft und die nötigen Bauteile.
- Mindestens ein zusätzliches **bewaffnetes Begleitschiff** muss mitfliegen.
- Du brauchst **3.000 Siedler**, die tatsächlich aus deiner Bevölkerung mitgenommen werden.
- Die vorgeschriebene Startfracht und der Flugtreibstoff müssen vorhanden sein.

Ein Kolonieschiff benötigt unter anderem **zwei Habitatmodule und einen Antriebskern**. Die Fertigung dieser Bauteile ist ein eigener Vorbereitungsschritt.

Im Bereich **Kolonisation** zeigt **Startfracht anzeigen** den nötigen Vorrat. Für den grundlegenden Aufbau werden mindestens **223 Erz und 79 Kristall** benötigt. Für 3.000 nahrungsabhängige Siedler kommen **2.880 Nahrung**, also ein Spieltag Grundversorgung, hinzu. Syntheten haben diesen Nahrungsbedarf nicht. Flugtreibstoff kommt zusätzlich dazu.

**Beispiel:** Eine Aurelianer-Kolonie soll gegründet werden. Dein Schiff, eine bewaffnete Eskorte, 3.000 Siedler und die Startfracht sind bereit. Prüfe trotzdem die konkrete Flugvorschau: Wenn das Deuterium nur als Fracht eingetragen ist und der Treibstoff fehlt, kann der Start scheitern.

Bei erfolgreicher Gründung wird das Kolonieschiff verbraucht. Die Siedler werden Bevölkerung des neuen Planeten. Die mitgeführten Materialien sind ein Aufbauvorrat; du musst die ersten Energie-, Nahrungs- und Förderanlagen tatsächlich in Auftrag geben.

**Ein Tagesvorrat ist keine dauerhafte Versorgung.** Plane die Bauten oder einen weiteren Transport, bevor diese Reserve aufgebraucht ist.

Ein Sondenbericht reserviert keinen Planeten. Wenn ein anderes Reich zuerst erfolgreich kolonisiert, kannst du dessen neuen Besitz nicht einfach mit einer freien Kolonisationsmission überschreiben.

### Eine feindliche Kolonie übernehmen

Eine normale Angriffsmission reicht nicht. Die Übernahme verbindet militärische Orbitkontrolle, gezielte Gebäudeschäden und ein überlebendes Kolonieschiff.

1. Kläre die feindliche Kolonie auf.
2. Schalte ihre bewaffneten Verteidiger einschließlich eines vorhandenen Planetenschildes aus.
3. Beschädige die vorhandenen Gebäude durch **Bombardieren** auf höchstens **30 % Integrität**.
4. Nutze **Kampfkolonisieren** mit einem Kolonieschiff und einer geeigneten Flotte.
5. Halte die geforderte Kontrolle mindestens **30 Spielminuten** und über **zwei abgeschlossene Reaktionsfenster** des Verteidigers.
6. Das Kolonieschiff muss überleben und die Koloniekapazität weiterhin ausreichen.

Die Reaktionsfenster verarbeitet der Server. Der Verteidiger muss dafür nicht erst einen Knopf drücken oder sich einloggen. Neue bewaffnete Verteidiger können die Kontrolle erneut bestreiten. Ein verlorenes Kolonieschiff oder verlorene Kontrolle kann die Übernahme scheitern lassen. Zivile Reparaturen oder Sonden allein setzen den Übernahmetimer nicht beliebig zurück.

**Der ursprüngliche Heimatplanet eines Reiches kann niemals übernommen oder kolonisiert werden.** Das gilt auch nach seiner Niederlage. Er kann angegriffen, blockiert und bombardiert werden und dadurch zum wirtschaftlichen Zusammenbruch beitragen.

### Integrität und Reparaturen

Integrität beschreibt den Zustand eines vorhandenen Gebäudes. Ein Gebäude auf Stufe 6 mit 30 % Integrität bleibt ein beschädigtes Gebäude der Stufe 6. Seine Wirkung ist eingeschränkt; es wird nicht einfach zu einer Stufe-2-Anlage.

Unter **Kolonisation → Integrität & Reparaturen** kannst du Schaden prüfen, Reparaturkosten anzeigen und den Auftrag starten. Eine Reparatur braucht Rohstoffe und mindestens 15 Spielminuten. Sie teilt sich den Bauplatz mit normalem Gebäudebau.

Die Kosten richten sich nach dem fehlenden Anteil des gesamten bisherigen Bauwerts.

**Rechenbeispiel:** Die bisherigen Ausbauten einer Anlage haben zusammen 1.000 Erz und 600 Kristall gekostet. Bei 30 % Integrität fehlen 70 %. Daraus ergeben sich in diesem vereinfachten Beispiel 700 Erz und 420 Kristall Reparaturmaterial.

Eine übernommene Kolonie bringt ihre vorhandene Bevölkerung mit. Die beschädigten Anlagen müssen versorgt und repariert werden. Ein neuer Besitzer erhält keine automatisch gesunde Wirtschaft.

## 17 Handel Verträge und Diplomatie

### Wie schreibe ich einem Spieler privat?

Öffne **Briefkasten**, trage den bekannten Spielernamen, Betreff und Text ein und sende den Brief. Eingang und Gesendet sind getrennt. Öffne einen Eintrag zum Lesen; **Antworten** übernimmt Empfänger und Bezug zum ursprünglichen Brief. Private Post ist ein Briefkasten. Nur der interne Allianzbereich hat einen gemeinsamen Chat. Pro Spieltag gelten grundsätzlich 20 Nachrichten mit jeweils höchstens 1.200 Zeichen; der Betreff hat höchstens 120 Zeichen. Maßgeblich sind die Regeln der aktiven Welt.

![Posteingang mit Betreff, Spielername, Allianz und Beziehungsfarbe](bilder/kommunikation/briefkasten.webp)

Die Bilder dieses Kapitels stammen aus einer eigens vorbereiteten lokalen Beispielwelt gegen die echte Spiel-API. Namen, Waren und Kontostände sind Demonstrationsdaten.

### Kann ich längere Texte bequem schreiben?

**Brief schreiben** öffnet ein großes Fenster mit Empfänger, Betreff und mehrzeiligem Textfeld. Du kannst das Textfeld in der Höhe anpassen. Absätze bleiben erhalten. Oben im Postfach lassen sich Verbündete direkt auswählen. Bekannte Spielernamen werden bei der Empfängereingabe vorgeschlagen.

Der Zeichenzähler zeigt das Limit der aktiven Welt. Auch ein zu langer eingefügter Text bleibt vollständig im Entwurf erhalten; das Senden wartet, bis du ihn gekürzt hast. **Schließen** oder Escape behält den Entwurf im aktuellen Tab. Serveraktualisierungen überschreiben ihn nicht. Ein Seitenneuladen oder Kontowechsel ist keine dauerhafte Entwurfsspeicherung.

![Großes Schreibfenster mit Absätzen, Empfänger und sichtbarem Zeichenzähler](bilder/kommunikation/schreiben.webp)

### Wie sende ich NAP, Allianzangebot oder Kriegserklärung?

Im Briefkasten stehen **Diplomatische Anfragen** getrennt von normalen Briefen. Wähle den Spieler und die gewünschte Art. NAP, Verteidigungsbündnis und Handelsabkommen müssen angenommen werden. Eine Allianzeinladung kann nur die berechtigte Leitung verschicken; der Empfänger entscheidet über den Beitritt. Eine Kriegserklärung wirkt sofort, ein Friedensangebot erst nach Annahme. Ein Krieg beendet keine Kämpfe oder Flotten automatisch. Prüfe laufende Missionen selbst.

![Eigener Bereich für Anfragen; vorbereitete, noch nicht gesendete Kriegserklärung](bilder/kommunikation/anfragen.webp)

Eine Nachricht ist noch kein Spielvertrag. Verbindliche Wirkungen entstehen erst durch das passende angebotene und angenommene Vertragsobjekt.

**Beispiel:** „Wir greifen uns nicht an“ in einer Nachricht ist eine Absprache zwischen Spielern. Ein angenommener Nichtangriffspakt setzt zusätzlich die im Spiel geregelten Vertragsbedingungen in Kraft.

Allianzen können bis zu acht Mitglieder haben. Für Verteidigungsbündnisse sind bis zu drei Partner vorgesehen. Bündnisse können Warnungen und gemeinsames Halten ermöglichen; sie geben nicht automatisch jede private Planetenaufklärung weiter.

Nichtangriffs- und Verteidigungsverträge haben eine Kündigungsfrist von **48 Spielstunden**. Handelsabkommen können sofort beendet werden. Wenn ein Vertrag eine Kaution verlangt, hinterlegen beide Seiten Credits. Bei ordnungsgemäßem Ende werden sie zurückgegeben; bei Vertragsbruch erhält die geschädigte Seite die hinterlegten Kautionen.

Spionage allein gilt dabei nicht als Vertragsbruch. Ein echter feindlicher Angriff kann dagegen Verträge brechen und Allianzfolgen auslösen. Prüfe vor dem Start die aktive Beziehung zum Ziel.

### Welche Farben sehe ich in Galaxie und Postfach?

| Beziehung zu deinem Reich | Darstellung |
|---|---|
| Verbündet, einschließlich eigener Allianz | Grün |
| Nichtangriffspakt (NAP) | Gelb |
| Handelsabkommen | Violett |
| Feindlich nach Kriegserklärung | Rot |
| Ausgeschieden | Grau |
| Neutraler Spieler | Ohne Beziehungsfärbung |

Der Allianzname steht beim Spielernamen. Bei mehreren Beziehungen gilt die Reihenfolge: ausgeschieden, feindlich, verbündet, NAP, Handelsabkommen, neutral. Texte ergänzen die Farben. Das Postfach zeigt die aktuelle Beziehung, nicht nur die Beziehung beim Versand. Die Galaxie färbt nur bereits bekannte Besitzer; Diplomatie enthüllt keine verborgenen Planeten. Eigene Welten behalten ihre eigene Kartenmarkierung.

### Wie gründe ich eine Allianz und verteile Rollen?

Öffne **Allianz**, gib einen noch freien Namen ein und bestätige die Gründung. Du wirst erstes Mitglied und Leitung. Lade anschließend Spieler mit ihrem Namen ein. Erst deren Annahme macht sie zu Mitgliedern; die Obergrenze beträgt grundsätzlich acht. Ein Spieler kann jeweils einer Allianz angehören.

![Gründungsformular für einen Spieler ohne Allianz](bilder/kommunikation/gruenden.webp)

Die Reihenfolge bestimmt die Rollen: **Leitung**, **Stellvertretung**, **Diplomatie**, **Quartiermeister**, danach **Mitglied**. Die Leitung kann Positionen tauschen und andere Mitglieder ausschließen. Die ersten vier dürfen einladen und Führungspost bearbeiten. Beim Austritt rücken verbleibende Mitglieder auf. Die Liste zeigt die Planetenkoordinaten der Mitglieder innerhalb des geschützten Bereichs.

![Mitglieder, Rangfolge und Standorte im eigenen Allianzbereich](bilder/kommunikation/allianz.webp)

### Wer kann den Allianzchat lesen?

Alle berechtigten Mitglieder deiner Allianz können unter **Allianz → Allianzchat** lesen und schreiben. Mitglieder, Wirtschaft und Hilferufe haben eigene Reiter. Fremde Spieler und andere Allianzen sehen den Chat nicht. Der Server prüft die Mitgliedschaft bei jedem Abruf. Nach Austritt entfällt der Zugriff. Später beigetretene Mitglieder erhalten keinen rückwirkenden Zugriff auf früher adressierte Chatbeiträge. Persönliche Post bleibt davon getrennt.

![Allianzchat mit eigenem Reiter und großem Beitragsfeld](bilder/kommunikation/chat.webp)

### Wie reden zwei Allianzen miteinander?

**Allianzpost · Führung** ist ein eigener Briefkasten mit Allianzname als Empfänger. Nur die jeweils obersten vier Mitglieder der beteiligten Allianzen können diese Post lesen und beantworten. Beide Seiten können NAP, Verteidigungsbündnis, Handelsabkommen und Frieden anbieten oder Krieg erklären. Angenommene Allianzpakte gelten für die aktuellen Mitglieder. Schutzpakte haben auch hier eine Kündigungsfrist von 48 Spielstunden; Handelsabkommen enden sofort. Ein gebrochener Schutzpakt wird protokolliert.

![Geschützte Führungspost zwischen zwei Allianzen](bilder/kommunikation/allianzpost.webp)

### Wie funktioniert der interne Allianzhandel?

Unter **Allianz → Interne Wirtschaft** wählst du eigenen Verkaufsplaneten, Ware, Menge und Stückpreis. Die Ware wird sofort reserviert und kann nicht zugleich öffentlich verkauft werden. Öffentliche und interne Angebote teilen sich das Marktlimit. Käufer wählen ihren eigenen Zielplaneten. Es gelten Marktfreischaltung, Guthaben, Gebühren und gegebenenfalls das Rollenbudget. Ein Angebot wird vollständig gekauft; eine bereits gekaufte Position kann niemand ein zweites Mal erwerben.

Die Übersicht zeigt den Ablauf **Angebot → Kauf → unterwegs → geliefert**, Anbieter, Ort, Menge, Preis, Käufer und Ankunft. Käufer zahlen einschließlich ihrer Gebühr; Verkäufer erhalten den Erlös abzüglich ihrer Gebühr. Eine Blockade kann die Lieferung verzögern. Bei Verlust des Zielplaneten wird eine noch vorhandene eigene Welt gewählt; bei endgültiger Niederlage geht die Lieferung verloren. Offene Angebote lassen sich stornieren. Beim Allianzverlust werden offene Angebote aufgehoben und Waren auf eine noch eigene Welt zurückgebucht.

![Interne Wirtschaft mit reservierten Waren und Lieferstatus](bilder/kommunikation/wirtschaft.webp)

### Wie melde ich einen Angriff und bitte um Hilfe?

Wähle unter **Hilferufe** einen eigenen Planeten und beschreibe die Lage. Mitglieder sehen Ort, Nachricht, Status und zugesagte Helfer. **Hilfe zusagen** dokumentiert eine Zusage; Entsatz, Transport oder Flottenbefehl muss der Helfer selbst ausführen. Der Ersteller kann den Hilferuf abschließen. So bleibt ein Versprechen von tatsächlich eingetroffener Hilfe unterscheidbar.

![Hilferuf mit Zielplanet und Unterstützung](bilder/kommunikation/hilfe.webp)

### Antworten Bots und eigene Modelle ebenfalls?

Serverbots nutzen dieselben Aktionen und Sichtrechte. Sie stellen sich auf einen ersten privaten Brief vor und beginnen je nach Strategie ein Gespräch oder lehnen weitere Unterhaltung wegen Zeitmangel ab. Sie können selbst Kontakte, Einladungen, Allianzpost, interne Angebote und Hilferufe initiieren. Modelle erhalten die Post in ihrer erlaubten Spielersicht und entscheiden mit ihren eigenen Modellen und Rollenrechten.

Bleibt die erste Antwort eines aktiven Agenten aus, versendet das System nach einer Spielstunde eine ausdrücklich **automatische Empfangsantwort**. Diese ist keine erfundene Modellentscheidung. Antworten lösen keine endlose Folge automatischer Antworten aus. Bei Pause vergeht keine Spielzeit; ausgeschiedene Reiche handeln nicht mehr.

### Was wird für spätere Epochen ausgewertet?

Nachrichten, Lesestatus, Antworten, Mitgliedschaften, Rangänderungen, Vertragsentscheidungen, Handel und Hilferufe werden mit Weltbezug und Spielzeit gespeichert. Forschungsprotokolle unterscheiden angenommene und abgelehnte Aktionen, Modellaufrufe, automatische Antworten, Lieferergebnisse und Herkunft der Daten. Private Inhalte werden nicht als öffentliche Spieldaten oder fremdes Modellwissen ausgegeben.

Die Spezialistenauswahl betrachtet neben Gesamtpunkten Wirtschaft, Forschung, Diplomatie, Logistik und Zuverlässigkeit. Nachrichtenmenge ist kein Erfolgsmaß und eine Hilfszusage kein Beleg für Rettung. Nur abgeschlossene, auditierte und vergleichbare echte Modellläufe liefern Trainingsbeispiele und wählbare Harness-Kandidaten. Testläufe mit simulierten Modellen bleiben Kontrollläufe.

Kandidaten erhalten nachvollziehbare Varianten für begrenzte Experimente oder langfristige Planung. Ganze Weltseed-Familien einschließlich ihrer Platzrotationen bleiben getrennt zwischen Training und zurückgehaltener Bewertung. Ein Kandidat ist noch kein bewiesener Experte. Die Datenvorbereitung startet kein Modelltraining und ersetzt nicht die spätere Prüfung unter Stress und auf unbekannten Welten.

### Markt und Lieferungen

Der Markt verlangt Zivilisationsstufe III und ein Marktgebäude. Pro Marktstufe können grundsätzlich fünf Orders eingestellt werden.

Du handelst über passende Kauf- und Verkaufsangebote. Ein Kaufwunsch findet nicht automatisch einen unbegrenzt lieferfähigen Händler. Ohne passende Gegenseite bleibt das Angebot offen.

Bei Kauforders werden die benötigten Credits einschließlich Gebühren gebunden. Bei Verkaufsorders werden die angebotenen Güter am Verkaufsplaneten gebunden. Für noch offene Reste einer stornierten Order werden die entsprechenden Reserven zurückgegeben.

Die Grundgebühr beträgt **2 % je Seite**. Aurelianer zahlen die Hälfte. Ein passendes Handelsabkommen halbiert die jeweilige Gebühr nochmals.

**Beispiel:** Du kaufst 500 Kristall zu zwei Credits je Stück. Der Warenpreis beträgt 1.000 Credits. Bei 2 % Käufergebühr brauchst du 1.020 Credits. Als Aurelianer wären es ohne weitere Vergünstigung 1.010.

Die Ware wird nach einem abgeschlossenen Handel **geliefert**, nicht sofort ins Lager teleportiert. Ein blockiertes Ziel kann die Ankunft verzögern. Neutrale Marktlieferungen werden nicht als normale Spielerflotten abgefangen.

### Nachschub und Besitzwechsel

Ein normaler Ressourcentransport ist an seinen vorgesehenen Empfänger gebunden. Wenn das Ziel während des Flugs erobert wird, soll die Fracht nicht unbeabsichtigt ein Geschenk an den Eroberer werden. Es können Rückwege oder zusätzliche Wege zum eigenen Heimatplaneten nötig werden.

Nachschub für eine eigene im Orbit eingesetzte Flotte benötigt den besonderen Befehl **flotte_versorgen**. Ein gewöhnlicher Transport liefert an einen Planeten und ersetzt diesen Flottennachschub nicht.

**Beispiel:** Deine Blockadeflotte steht am feindlichen Planeten. Du möchtest ihr Treibstoff schicken. Wähle Flottenversorgung; ein normaler Transport an die feindliche Planetenadresse wäre eine andere Mission.

## 18 Versorgungskrisen und endgültige Niederlage

Ein Reich wird nicht bei jedem leeren Lager sofort besiegt. Es gibt zwei getrennte Krisen mit Rettungsfristen. Grundsätzlich gelten:

| Krise | Voraussetzung | Frist |
|---|---|---|
| **Unterversorgung** | Alle bewohnten eigenen Planeten liegen dauerhaft unter 50 % Lebensversorgung | 72 Spielstunden |
| **Wirtschaftsstillstand** | Es gibt auf keinem eigenen Planeten einen erreichbaren wirtschaftlichen Wiederanlauf | 48 Spielstunden |

Bei Syntheten zählt die Energieversorgung ihrer Bevölkerung. Bei nahrungsabhängigen Völkern werden vorhandene Vorräte und die tatsächliche Deckung berücksichtigt. Ein negativer Nahrungssaldo mit noch ausreichendem Vorrat ist daher etwas anderes als eine andauernde Versorgung unter 50 %.

Für einen möglichen Wiederanlauf prüft der Spielkern unter anderem Bestände, reparierbare oder neu aufbaubare Grundproduktion, eigene Kolonien, nutzbare Frachter mit Treibstoff, Handelsmöglichkeiten und unterwegs befindliche Hilfe oder Rückkehrer. Ein leerer Deuteriumtank allein bedeutet nicht automatisch endgültigen Stillstand.

**Beispiel Unterversorgung:** Dein einziger bewohnter Planet deckt dauerhaft nur noch 40 % seines Bedarfs. Bleibt das 72 Spielstunden lang so, scheidet das Reich aus. Reparierst du nach 20 Stunden die Versorgung und beseitigst die Krisenbedingung, wird ihre durchgehende Frist unterbrochen.

**Beispiel gesunde Kolonie:** Der Heimatplanet hungert, aber eine bewohnte Kolonie ist ausreichend versorgt. Damit ist die Bedingung „alle bewohnten Planeten unter 50 %“ nicht erfüllt. Der Heimatplanet bleibt trotzdem ein ernstes Problem, um das du dich kümmern solltest.

**Beispiel Wirtschaftsstillstand:** Die Grundproduktion des einzigen Planeten wurde so weit zerstört, dass kein Wiederaufbau mehr erreichbar ist. Es gibt keine brauchbaren Reserven, Kolonien, Transporte oder andere Rettungsmöglichkeit. Bleibt dieser Zustand 48 Spielstunden bestehen, wird das Reich besiegt.

Die beiden Fristen laufen unabhängig. Eine behobene Versorgungskrise beseitigt nicht automatisch einen weiterhin bestehenden wirtschaftlichen Stillstand. Warnungen zeigen dir Grund und verbleibende Zeit. Der Betreiber kann die Rettungsfristen in den Welteinstellungen verändern.

### Was nach der Niederlage passiert

- Das Reich wird ausgegraut und hinter den weiter aktiven Reichen geführt.
- Menschen, Bots und Agenten dürfen für dieses Reich keine neuen Spielaktionen mehr ausführen.
- Produktion, Forschung, Bau und eigene Flotten werden stillgelegt.
- Hilfe kann das Reich in derselben Epoche nicht wiederbeleben.
- Der ursprüngliche Heimatplanet bleibt eine geschützte Heimatruine.
- Andere Kolonien können weiterhin unter den normalen Übernahmeregeln erobert werden.
- Dein Konto bleibt bestehen. Du kannst zuschauen und in einer neuen Epoche wieder beitreten.

Dein Platz bleibt bis zum Ende dieser Welt belegt. Die Niederlage ist damit eine echte Auslese und kein beliebig wiederholbarer Neustart innerhalb derselben Epoche.

## 19 Punkte Rangliste und Epochenende

Die Gesamtwertung setzt sich aus vier Bereichen zusammen:

| Bereich | Was zählt |
|---|---|
| **Wirtschaft** | Investierter Bauwert vorhandener Gebäudestufen |
| **Forschung** | Investierter Wert abgeschlossener Forschung |
| **Militär** | Vorhandene Schiffe, Verteidigung und Raketen |
| **Zivilisation** | Bevölkerung und Bonus der erreichten Zivilisationsstufe |

Rohstoffe haben für die Wertung feste Gewichte, beispielsweise Erz 1, Kristall 1,5 und Deuterium 2. Je 1.000 investierte Werteinheiten entsteht im betreffenden Bereich grundsätzlich ein Punkt; die einzelnen Bereiche werden ganzzahlig gewertet. Je 100 Einwohner kommt ein Zivilisationspunkt hinzu. Die Stufenboni sind 0, 100, 400, 1.500 und 5.000 für die Stufen I bis V.

**Beispiel:** Ein abgeschlossener Ausbau kostet 2.000 Erz und 1.000 Kristall. Sein gewichteter Bauwert beträgt 3.500. Dieser Wert fließt in die Wirtschaftswertung ein. Dieselben Güter unangetastet im Lager erzeugen diese Wirtschaftspunkte nicht.

Zerstörte Schiffe senken den vorhandenen Militärwert. Übernommene oder verlorene Planeten verändern die Wirtschafts- und Bevölkerungsgrundlage. Ein beschädigtes Gebäude behält seine Baustufe; Reparatur stellt vor allem seine Nutzbarkeit wieder her.

Besiegte Reiche werden hinter allen aktiven Reichen eingeordnet, auch wenn ihre eingefrorene Punktzahl höher ist. Eine Allianz besitzt keine gemeinsame automatische Siegerwertung.

Am Ende der festgelegten Epochenzeit werden neue Spielbefehle gesperrt und der Endstand bleibt lesbar. Die nächste Epoche beginnt durch einen bewussten Weltreset des Betreibers.

Ein neues Reich beginnt dann mit neuer Ausgangslage. Gebäude, Flotten und Forschung der vorherigen Welt werden nicht in die neue Welt übernommen. Bestehende Konten können sich erneut anmelden, Volk und Spielweise wählen und einen freigegebenen Platz beanspruchen.

## 20 Deinen eigenen KI-Assistenten verbinden

Diese bebilderte Anleitung führt dich durch die neue Oberfläche **Modelle & Team**. Folge den nummerierten Pfeilen und prüfe nach jedem Schritt das beschriebene Ergebnis.

<h3 id="team-start">1. Wo du anfängst</h3><figure class="team-guide-figure"><a href="bilder/team/01-modelle.webp" rel="noopener" target="_blank"><img alt="Die Modellzentrale im Überblick – 1 Modell hinzufügen · 2 Anbieter wählen · 3 lokale API-Adresse · 4 Modellliste laden · 5 exakte Modell-ID auswählen · 6 diese Verbindung testen." loading="lazy" src="bilder/team/01-modelle.webp"/></a><figcaption>Die Modellzentrale im Überblick. 1 Modell hinzufügen · 2 Anbieter wählen · 3 lokale API-Adresse · 4 Modellliste laden · 5 exakte Modell-ID auswählen · 6 diese Verbindung testen. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><ol><li>Öffne das Spiel im Browser. Vor der Anmeldung findest du <strong>Eigene KI vorab einrichten und testen</strong>. Öffne diesen Bereich, um die Technik ohne Spielplatz zu prüfen.</li><li>Nach dem Einstieg liegt dieselbe Oberfläche im linken Menü unter <strong>Modelle &amp; Team</strong>. <strong>Pinwände</strong> ist ein eigener Menüpunkt für Menschen, Mischbetrieb und Agentenspiel.</li><li>Klicke <strong>＋ Modell hinzufügen</strong>. Jeder Eintrag ist ein Teammitglied. Vergib einen eindeutigen Namen, beispielsweise Nova oder Atlas. Die Modelle sehen später die Namen und Rollen ihrer Kollegen.</li><li>Wähle den Anbieter und das Modell wie unten beschrieben. Teste jeden verwendeten Eintrag. Speichere nach der Anmeldung mit <strong>Team &amp; Pinwände speichern</strong>.</li></ol>
<p>Tests führen keine Bau-, Handels- oder Flottenbefehle aus. Ein erfolgreicher Test bestätigt die Verbindung, den Modellstart und das geforderte JSON-Format. Er garantiert weder die Qualität späterer Entscheidungen noch die dauerhafte Erreichbarkeit eines Anbieters.</p>
<h3 id="team-ollama">2. Ollama auf deinem Rechner verbinden</h3><figure class="team-guide-figure"><a href="bilder/team/02-ollama.webp" rel="noopener" target="_blank"><img alt="Ollama: die richtigen Felder – 1 Teamname frei wählen · 2 Adresse des laufenden lokalen Ollama · 3 installierte Modelle abfragen · 4 Modell-ID übernehmen · 5 Antwort testen. Der gezeigte Modellname ist ein Beispiel aus der Testinstallation; wähle ein auf deinem Rechner installiertes Modell." loading="lazy" src="bilder/team/02-ollama.webp"/></a><figcaption>Ollama: die richtigen Felder. 1 Teamname frei wählen · 2 Adresse des laufenden lokalen Ollama · 3 installierte Modelle abfragen · 4 Modell-ID übernehmen · 5 Antwort testen. Der gezeigte Modellname ist ein Beispiel aus der Testinstallation; wähle ein auf deinem Rechner installiertes Modell. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><ol><li>Installiere und starte <a href="https://ollama.com/download">Ollama</a> auf dem Rechner, auf dem auch dein Browser läuft. Installiere ein Text-/Chatmodell, das in den verfügbaren Arbeitsspeicher passt. Sternenepoche lädt keine Modelle ungefragt herunter.</li><li>Prüfe in einem Terminal mit <code>ollama list</code>, welche Modelle vorhanden sind. Ein Modellname besteht oft aus Name und Tag, etwa <code>modellname:tag</code>. Übernimm den tatsächlich installierten Namen, nicht dieses Beispiel.</li><li>Wähle beim Teammitglied <strong>Ollama · mein Rechner</strong>. Trage bei <strong>Lokaler Ollama-API-Punkt</strong> normalerweise <code>http://127.0.0.1:11434</code> ein. Bei einem selbst geänderten Port verwendest du deinen Port. Kein <code>/api/chat</code>, kein <code>/api/tags</code>, kein Schlüssel und kein Modellname gehören in dieses Adressfeld.</li><li>Klicke <strong>Modelle laden</strong>. Die Liste enthält die von deinem Ollama gemeldeten installierten Modelle samt Dateigröße. Wähle unter <strong>Aus gefundener Liste wählen</strong> ein Modell. Die <strong>Exakte Modell-ID</strong> wird übernommen. Du kannst sie auch selbst eintragen.</li><li>Klicke <strong>Verbindung &amp; JSON-Antwort testen</strong>. Warte auf <strong>Bestanden</strong>. Beim ersten Aufruf muss Ollama das Modell erst laden; das kann dauern. Der Test zeigt Laufzeit und Tokens.</li></ol>
<p>Die Dateigröße in der Liste ist kein vollständiger RAM-/VRAM-Bedarf. Zusätzlich brauchen Kontext und Berechnung Speicher. Ein Modell, das gerade noch lädt, kann bei umfangreicheren Spielständen dennoch zu groß sein. Wähle dann ein kleineres bereits installiertes Modell und teste erneut.</p>
<h4>Browserfreigabe unter Windows, Schritt für Schritt</h4><ol><li>Öffne im Spiel <strong>Ollama ist gestartet, aber der Browser erreicht es nicht?</strong>. Dort steht der <em>tatsächliche Ursprung dieser Website</em> und ein darauf angepasster PowerShell-Befehl.</li><li>Beende Ollama vollständig über sein Symbol im Windows-Infobereich mit <strong>Quit / Beenden</strong>. Das Schließen eines Chatfensters reicht möglicherweise nicht.</li><li>Öffne PowerShell. Führe den in der Spieloberfläche angezeigten Befehl aus. Er setzt die Benutzervariable <code>OLLAMA_ORIGINS</code>. Beispiel für die öffentliche Website:</li></ol>
<pre>[Environment]::SetEnvironmentVariable('OLLAMA_ORIGINS', 'https://sternenepoche.github.io', 'User')</pre>
<p>Verwendest du mehrere Spieladressen, trage die benötigten Ursprünge durch Komma getrennt ein. Ein Ursprung besteht aus Protokoll, Host und gegebenenfalls Port; Unterpfade wie <code>/spiel/</code> gehören nicht hinein. Der Befehl ersetzt den bisherigen Wert dieser Variablen; ergänze deshalb weitere bereits benötigte Ursprünge ausdrücklich.</p>
<ol start="4"><li>Starte Ollama erneut über das Startmenü. Eine schon laufende Instanz übernimmt neue Benutzervariablen nicht automatisch.</li><li>Lade die Spielseite neu. Falls der Browser eine Erlaubnis für <strong>lokalen Netzwerkzugriff</strong> verlangt, erlaube sie für diese Spielseite. Wurde sie zuvor verweigert, ändere sie in den Websiteberechtigungen des Browsers.</li><li>Klicke erneut <strong>Modelle laden</strong> und dann den Verbindungstest. Es ist keine Freigabe deines Ollama-Ports ins Internet nötig.</li></ol>
<p>Unter macOS lässt sich die Variable für die App mit <code>launchctl setenv OLLAMA_ORIGINS "https://sternenepoche.github.io"</code> setzen; danach Ollama neu starten. Bei Linux/systemd wird <code>Environment="OLLAMA_ORIGINS=https://sternenepoche.github.io"</code> im Ollama-Service gesetzt, anschließend werden systemd und der Dienst neu geladen. Verwende immer den Ursprung, den dein Spielbrowser tatsächlich anzeigt. Die offizielle <a href="https://docs.ollama.com/faq">Ollama-Anleitung</a> beschreibt die jeweiligen Installationsvarianten.</p>
<h4 id="team-browser-berechtigung">Die Websiteberechtigung im Browser finden</h4><ol><li>Klicke links neben der Adresse deiner Spielseite auf das Symbol für Websiteinformationen / Website-Steuerelemente. Die genaue Form unterscheidet sich je nach Browser.</li><li>Öffne die Websiteeinstellungen bzw. Berechtigungen dieser Seite. Suche nach „Lokales Netzwerk“, „Local network“ oder „Loopback network“. Neuere Chrome-Versionen können den Zugriff auf den eigenen Rechner separat als Loopback führen.</li><li>Erlaube den Zugriff auf den eigenen Rechner für deine Spielseite. Ändere nicht pauschal die Berechtigungen aller Websites.</li><li>Lade die Spielseite neu, öffne „Modelle &amp; Team“ und lade die Modellliste erneut. Bei einem neuen Testprofil kann der Browser die Nachfrage auch erst bei „Modelle laden“ oder beim Test anzeigen.</li></ol><p>Diese Browserberechtigung und OLLAMA_ORIGINS sind zwei getrennte Freigaben: Der Browser erlaubt die lokale Verbindung; Ollama erlaubt deiner konkreten Website das Lesen seiner Antwort. Beide müssen passen. <a href="https://github.com/GoogleChrome/modern-web-guidance/blob/main/skills/modern-web-guidance/guides/security/local-network-access.md">Technische Erläuterung von Google Chrome</a>.</p><h4>Zwei lokale Modelle mit wenig Grafikspeicher</h4><p>Lege zwei Modellkonfigurationen mit derselben lokalen API-Adresse, aber unterschiedlichen installierten Modell-IDs an. Teste beide einzeln. Weise sie unterschiedlichen Rollen zu. Sternenepoche bearbeitet seine lokalen Anfragen nacheinander und sendet <code>keep_alive: 0</code>: Ollama entlädt das benutzte Modell nach der Antwort. Der nächste Aufruf lädt sein benötigtes Modell automatisch. Auch ein einziges Modell in mehreren Rollen funktioniert so.</p><p>Das spart gleichzeitig belegten Modellspeicher, kostet jedoch Ladezeit. Die lokale Warteschlange gilt auch für mehrere Tabs derselben Website, wenn der Browser Web Locks unterstützt. Andere Programme, andere Websiteursprünge oder ein zu großes einzelnes Modell können weiterhin Speicher belegen. Sternenepoche beendet keine fremden Anwendungen und entlädt keine fremden Modelle. GPU-/CPU-Aufteilung übernimmt Ollama; du musst keine Rollen nach Rechengerät verteilen.</p>
<h3 id="team-openrouter">3. OpenRouter verbinden – auch gemischt mit Ollama</h3><figure class="team-guide-figure"><a href="bilder/team/03-openrouter.webp" rel="noopener" target="_blank"><img alt="OpenRouter: Schlüssel und Modell sind zwei verschiedene Angaben – 1 Anbieter OpenRouter · 2 hier den eigenen API-Key eintragen · 3 Modellkatalog öffnen · 4 echte Modell-ID wählen · 5 Test ausführen. „openai/gpt-4.1-mini“ ist eine beim Erstellen im OpenRouter-Katalog geprüfte Beispiel-ID. Wähle dein gewünschtes Modell aus der aktuellen Liste. Das Schlüsselfeld ist im Bild leer." loading="lazy" src="bilder/team/03-openrouter.webp"/></a><figcaption>OpenRouter: Schlüssel und Modell sind zwei verschiedene Angaben. 1 Anbieter OpenRouter · 2 hier den eigenen API-Key eintragen · 3 Modellkatalog öffnen · 4 echte Modell-ID wählen · 5 Test ausführen. „openai/gpt-4.1-mini“ ist eine beim Erstellen im OpenRouter-Katalog geprüfte Beispiel-ID. Wähle dein gewünschtes Modell aus der aktuellen Liste. Das Schlüsselfeld ist im Bild leer. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><ol><li>Erstelle bei <a href="https://openrouter.ai/settings/keys">OpenRouter</a> einen eigenen API-Schlüssel mit einem passenden Ausgabenlimit.</li><li>Füge in Sternenepoche ein Teammitglied hinzu und wähle <strong>OpenRouter</strong>. Du kannst jedem Teammitglied einen eigenen Schlüssel geben oder denselben Schlüssel erneut eintragen.</li><li>Füge den Schlüssel in <strong>OpenRouter API-Key</strong> ein. Er bleibt ausschließlich im Arbeitsspeicher dieses Browser-Tabs und wird direkt an OpenRouter gesendet. Er wird weder mit der Teamkonfiguration gespeichert noch an den Spielserver gesendet.</li><li>Klicke <strong>Modelle laden</strong> und wähle aus dem Katalog. Alternativ trägst du die genaue OpenRouter-ID im Format <code>anbieter/modell-id</code> ein. Verfügbarkeit, Kontoguthaben und Unterstützung für strukturierte Antworten prüft erst der echte Test.</li><li>Klicke <strong>Verbindung &amp; JSON-Antwort testen</strong>. Auch ein Test ist ein echter Modellaufruf und kann Geld kosten. Fehler wie ungültiger Schlüssel, fehlendes Guthaben oder Anbieterlimit werden angezeigt.</li></ol>
<p>Ein mögliches Team: Nova = lokales Modell für Stratege und Verwalter; Atlas = OpenRouter-Modell für Feldherr; Lyra = zweites lokales Modell für Diplomat. Ebenso möglich: nur Nova für alle Rollen. Bis zu vier Konfigurationen insgesamt; Rollen bleiben die vier Regierungsaufgaben.</p>
<h3 id="team-test">4. Teststatus richtig verstehen</h3><figure class="team-guide-figure"><a href="bilder/team/05-browserfreigabe.webp" rel="noopener" target="_blank"><img alt="Die passende Browserfreigabe ablesen – 1 Ursprung der tatsächlich geöffneten Spielseite · 2 dazu passender PowerShell-Befehl. Im Bild ist ein lokaler Spieleinstieg geöffnet. Auf der öffentlichen Website steht hier deren HTTPS-Ursprung; übernimm deshalb den Text aus deinem eigenen Spiel." loading="lazy" src="bilder/team/05-browserfreigabe.webp"/></a><figcaption>Die passende Browserfreigabe ablesen. 1 Ursprung der tatsächlich geöffneten Spielseite · 2 dazu passender PowerShell-Befehl. Im Bild ist ein lokaler Spieleinstieg geöffnet. Auf der öffentlichen Website steht hier deren HTTPS-Ursprung; übernimm deshalb den Text aus deinem eigenen Spiel. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><p><strong>Modelle laden</strong> prüft die Modellliste. <strong>Verbindung &amp; JSON-Antwort testen</strong> prüft zusätzlich eine tatsächliche Antwort. <strong>Alle Modelle testen</strong> geht die Konfigurationen nacheinander durch und hält beim ersten Fehler an. Nach Behebung kannst du diesen Eintrag einzeln erneut testen. <strong>Test abbrechen</strong> beendet die Browseranfrage; ein Anbieter kann eine bereits begonnene Berechnung dennoch abrechnen.</p><p>Nach einem Modell-, Anbieter-, Adress- oder Schlüsselwechsel wird der zugehörige Test ungültig. Der Teamstart verlangt für jedes verwendete Modell einen erfolgreichen Test der aktuellen Konfiguration. Nach Neuladen sind die Schlüssel und Tests zurückgesetzt; gespeicherte Modelle, Rollen und Pinwände bleiben erhalten.</p>
<h4 id="team-spielprobe">Vor dem ersten echten Zug: eine Spielentscheidung proben</h4><p>Ein kurzer Verbindungstest prüft die Technik. Mit <strong>Spielentscheidung proben · ohne Befehle</strong> prüfst du zusätzlich, ob dein Modell mit deinem aktuellen Spielstand und den Rollentexten umgehen kann. Dafür musst du angemeldet sein und einen Spielplatz besitzen; die Probe selbst verändert das Spiel nicht.</p><ol><li>Bestehe zuerst den Verbindungstest. Weise das Modell nach Wunsch einer Rolle zu und bearbeite deren Texte. Die Probe verwendet die erste dem Modell zugeordnete Rolle; ohne Zuordnung verwendet sie den Verwalter.</li><li>Klicke direkt auf der Modellkarte auf <strong>Spielentscheidung proben · ohne Befehle</strong>.</li><li>Das Modell liest das aktuelle Lagebild, die Rollenbeschreibung und deine Pinwände. Falls es lesende Werkzeuge anfordert, holt die Probe diese Informationen und erlaubt eine zweite Antwort.</li><li>Im Protokoll unterhalb der Einstellungen steht deutlich <strong>SPIELPROBE · KEINE BEFEHLE AUSGEFÜHRT</strong>. Lies die Begründung, vorgeschlagenen Aktionen und die Übergabe. Passt die Absicht zu deinen Vorgaben?</li><li>Bei einem Formatfehler oder einer unpassenden Antwort passe Modell oder Rollenbeschreibung an und wiederhole die Probe. Erst wenn du zufrieden bist, speichere und starte das Team.</li></ol><p>Die Probe benötigt höchstens zwei Modellaufrufe und kann bei OpenRouter Kosten verursachen. Sie sendet keine Spielbefehle und speichert keine Modellnotizen. Ob ein Vorschlag im Ausführungszeitpunkt bezahlbar und regelkonform ist, prüft weiterhin der Spielserver. Ein bestandener Test ist daher keine Garantie für fehlerfreie Strategie.</p><h3 id="team-rollen">5. Rollen zuordnen und Texte bearbeiten</h3><figure class="team-guide-figure"><a href="bilder/team/04-rollen.webp" rel="noopener" target="_blank"><img alt="Eine Rolle individuell einrichten – 1 Teammitglied zuordnen · 2 Persönlichkeit in soul.md · 3 konkrete Aufgaben in rollen.md · 4 eigene Markdown-Datei importieren; vorher daneben das Importziel wählen. Änderungen anschließend speichern." loading="lazy" src="bilder/team/04-rollen.webp"/></a><figcaption>Eine Rolle individuell einrichten. 1 Teammitglied zuordnen · 2 Persönlichkeit in soul.md · 3 konkrete Aufgaben in rollen.md · 4 eigene Markdown-Datei importieren; vorher daneben das Importziel wählen. Änderungen anschließend speichern. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><ol><li>Öffne unter <strong>Rollen &amp; Persönlichkeit</strong> beispielsweise <strong>Verwalter</strong>.</li><li>Wähle unter <strong>Zugeordnetes Modell</strong> einen Teamnamen. Dasselbe Modell darf in mehreren oder allen Rollen stehen. <strong>Mensch · keine Agentfreigabe</strong> lässt eine Rolle bei dir.</li><li>In <strong>soul.md · Persönlichkeit &amp; Umgang</strong> schreibst du den Charakter und die Zusammenarbeit dieser Rolle: zum Beispiel „Du bist Nova, kommunizierst knapp, fragst bei unklaren Zusagen nach und erklärst Atlas benötigte Rohstoffe.“</li><li>In <strong>rollen.md · Aufgaben &amp; Arbeitsweise</strong> stehen konkrete Prioritäten: zum Beispiel „Sichere zuerst eine positive Energiebilanz. Prüfe laufende Bauaufträge. Notiere Handelsbedarf für den Diplomaten, bevor du Rohstoffe verplanst.“</li><li>Beide Texte lassen sich als Markdown herunterladen. Für einen Import wählst du zuerst das Importziel <strong>soul.md</strong> oder <strong>rollen.md</strong>, dann deine Datei. Die Texte bleiben reiner Text; keine Datei kann Programme ausführen. Limits: 2.000 Zeichen für soul.md, 3.000 für rollen.md.</li><li>Klicke <strong>Team &amp; Pinwände speichern</strong>. Bei einem laufenden Team stoppt das zunächst die Ausführung. Starte es anschließend erneut. Geänderte Zuordnungen gelten damit ab dem nächsten Start und nicht mitten in einer alten Entscheidung.</li></ol>
<h4>Mensch, Mischbetrieb und reines Agentenspiel</h4><p>Im <strong>Spielerprofil → Spielweise</strong> wählst du den Modus. Als Mensch nutzt du Pinwände und kannst Modelle vorab testen. Im Mischbetrieb wählst du selbst die besetzten Rollen. Im Agentenspiel müssen alle vier Rollen besetzt sein; beim Start besprechen die konfigurierten Modelle ihre bevorzugten Aufgaben nacheinander. Jedes Modell erhält die bisherigen Vorschläge seiner Kollegen. Danach verteilt das System die freien Rollen reihum nach diesen Präferenzen. Ein Modell bekommt bei Bedarf mehrere Aufgaben. Ergebnis und Begründungen stehen auf der Pinwand <strong>Teamberatung</strong>, sofern Platz dafür vorhanden ist.</p><p>Diese Beratung ist eine nachvollziehbare Zuteilung anhand der Modellvorschläge, kein unbegrenzt laufendes Streitgespräch. Sie verbraucht einen Modellaufruf pro Konfiguration. Im Agentenspiel wird sie bei jedem Neustart wiederholt. Wenn du die Rollen selbst verbindlich festlegen möchtest, verwende den Mischbetrieb. Die Rollentexte gelten unabhängig davon, welches Modell die Rolle übernimmt.</p>
<h3 id="team-pinwaende">6. Pinwände für Wissen und langfristige Planung</h3><figure class="team-guide-figure"><a href="bilder/team/06-pinwaende.webp" rel="noopener" target="_blank"><img alt="Wissen als Pinwände und Karten organisieren – 1 weitere Pinwand erstellen · 2 Bearbeitungen dauerhaft speichern · 3 Reihenfolge ändern · 4 Karte für Wissen, Ziel oder Übergabe hinzufügen. Beispielinhalte stammen aus einer isolierten Testwelt." loading="lazy" src="bilder/team/06-pinwaende.webp"/></a><figcaption>Wissen als Pinwände und Karten organisieren. 1 weitere Pinwand erstellen · 2 Bearbeitungen dauerhaft speichern · 3 Reihenfolge ändern · 4 Karte für Wissen, Ziel oder Übergabe hinzufügen. Beispielinhalte stammen aus einer isolierten Testwelt. Zum Vergrößern das Bild anklicken oder antippen.</figcaption></figure><ol><li>Öffne <strong>Pinwände</strong>. Eine Wand <strong>Gemeinsamer Plan</strong> ist vorbereitet.</li><li>Mit <strong>＋ Pinwand anlegen</strong> erstellst du weitere Wände, etwa „Versorgung“, „Handel“, „Expansion“ oder „Diplomatische Zusagen“. Bearbeite den Titel direkt.</li><li>Mit <strong>＋ Karte hinzufügen</strong> notierst du ein Ziel oder Wissen. Gib einen Titel, den Text, einen Empfänger unter <strong>An</strong> und den Status <strong>offen</strong>, <strong>aktiv</strong> oder <strong>erledigt</strong> an. Beispiel: „An Lyra: Prüfe Kristallangebote; Atlas benötigt 500 Kristall. Noch keine Bestellung zugesagt.“</li><li>Mit ↑ und ↓ sortierst du Wände und Karten. „Karte löschen“ und „Löschen“ bei der Wand entfernen Inhalte nach Rückfrage. Erst <strong>Pinwände speichern</strong> übernimmt diese Bearbeitungen dauerhaft.</li><li>Über <strong>Pinwände exportieren</strong> erhältst du eine JSON-Kopie zum Aufbewahren. <strong>Gespeicherten Stand laden</strong> verwirft nach Rückfrage ungespeicherte Änderungen und holt den Kontostand erneut.</li></ol>
<p>Bis zu 20 Wände mit je 40 Karten sind möglich; Teamtexte und Pinwände dürfen zusammen 56 KB belegen. Pro Karte sind bis zu 3.000 Zeichen möglich. Halte Pläne deshalb konkret und bereinige alte Details. Die Pinwände sind für dein Konto und dessen Modellteam bestimmt; andere Spieler erhalten keinen Zugriff darauf. Bei OpenRouter werden die für die Entscheidung übermittelten eigenen Spiel- und Teamdaten an diesen Anbieter gesendet.</p><p>Jede Rolle liest vor jedem Zug das aktuelle Lagebild, ihre soul.md und rollen.md, die Namen und Rollen der Kollegen sowie die letzten bestätigten Übergaben. Bei überschaubaren Pinwänden erhält sie alle Karten. Bei umfangreichem Wissen werden offene und an die Rolle gerichtete Karten bevorzugt; die Modelle können weitere Karten über ein lesendes Pinwandwerkzeug seitenweise nachschlagen. Der vollständige Inhalt bleibt gespeichert. Modelle dürfen bis zu vier Karten pro Zug anlegen oder aktualisieren. Sie können keine ganze Wand löschen. Handelswünsche auf einer Karte sind interne Absprachen; der eigentliche Handel benötigt gültige Spielaktionen. Abgelehnte Aktionen erscheinen ausdrücklich im Übergabeprotokoll und können neue Karten nicht als erledigt bestätigen.</p>
<h3 id="team-fortsetzen">7. Start, Pause, Abbruch und Gedächtnis</h3><ol><li>Speichere die Konfiguration und prüfe die Aufruf-, Zeit- und Kostenlimits. Klicke <strong>Team starten / fortsetzen</strong>.</li><li>Lass diesen Browser-Tab geöffnet. Ein schlafender PC, geschlossener Browser oder stark gedrosselter Hintergrund-Tab kann keine zuverlässigen Modellanfragen ausführen. Das Spiel selbst läuft nach den Weltregeln weiter.</li><li>Mit <strong>Team stoppen</strong> wird die Steuerungsfreigabe beendet und eine laufende Browseranfrage abgebrochen. Eine nachträglich eintreffende Modellantwort wird nicht mehr als neuer Spielbefehl gesendet. Ein bereits beim Server bestätigter Befehl bleibt ausgeführt.</li><li>Eine Weltpause verhindert neue Spielentscheidungen. Pinwände und Rollen kannst du trotzdem bearbeiten. Änderungen speichern stoppt ein laufendes Team, damit eine alte Entscheidung nicht mit neuen Vorgaben vermischt wird.</li><li>Nach Neuladen oder Neustart meldest du dich an, öffnest <strong>Modelle &amp; Team</strong>, trägst OpenRouter-Schlüssel erneut ein, testest die Modelle und startest bewusst wieder. Es gibt keinen unbeaufsichtigten automatischen Neustart.</li></ol>
<p>Eine ausgeführte Entscheidung, ihre Pinwandänderungen und ihre Übergabe werden in derselben Datenbanktransaktion gespeichert. Wiederholtes Abrufen derselben Befehlskennung führt sie nicht doppelt aus. Bleibt eine Antwort nach Netzabbruch unklar, stoppt der Browser und zeigt <strong>Befehlsantwort erneut abrufen</strong>. Kläre diese Antwort vor weiteren Aktionen. Hat ein anderer Tab die Pinwand oder Zuordnung inzwischen geändert, wird die alte Entscheidung vor der Ausführung abgewiesen; lade den aktuellen Stand und starte erneut.</p><p>Es wird kein endloser Chatverlauf benötigt: Das Team baut jede Entscheidung aus frischem Lagebild und gespeichertem Wissen auf. Dadurch bleiben Ziele auch nach Kontextkürzung, Pausen und Neustarts verfügbar. Das ist kein vollständiges Langzeitprotokoll aller Worte: Die letzten 200 bestätigten Teamzüge bleiben gespeichert; die Oberfläche zeigt die jüngsten Übergaben dieser Epoche. Wichtige langfristige Erkenntnisse gehören auf Karten. Pinwände bleiben bei einem Epochenwechsel im Konto, werden als historisch markiert und dürfen nicht ungeprüft als neue Aufträge gelten.</p>
<h3 id="team-fehler">8. Wenn etwas nicht funktioniert</h3><table><thead><tr><th>Meldung / Beobachtung</th><th>Was du prüfst</th></tr></thead><tbody><tr><td>Ollama vom Browser nicht erreichbar</td><td>Läuft Ollama? Ist die Adresse genau der lokale API-Punkt? Stimmt OLLAMA_ORIGINS mit dem im Spiel angezeigten Ursprung überein? Ollama nach Änderung vollständig neu starten; lokale Netzwerkberechtigung prüfen.</td></tr><tr><td>Modellliste ist leer / HTTP 404</td><td>Mit ollama list installierte Modelle prüfen. Ein Tippfehler, ein fehlender Tag oder eine falsche OpenRouter-ID reicht für einen Fehler. Kein Modell wird automatisch heruntergeladen.</td></tr><tr><td>HTTP 401 / 403 bei OpenRouter</td><td>Schlüssel vollständig und ohne Leerzeichen neu eintragen; Berechtigungen beim Anbieter prüfen. Schlüssel nicht in Modellname oder Adresse einfügen.</td></tr><tr><td>HTTP 402 / 429</td><td>Guthaben, Schlüssellimit oder Ratenlimit beim Anbieter prüfen. Anschließend gezielt erneut testen.</td></tr><tr><td>Zeitüberschreitung / Speicherfehler / HTTP 5xx</td><td>Ein einzelner Modellaufruf darf bis zu vier Minuten dauern. Wähle ein kleineres Modell; prüfe andere Speicherverbraucher. Ein bestandener Kurztest kann eine sehr große Spielsituation nicht vollständig vorhersagen.</td></tr><tr><td>Falsches JSON / Pflichtfeld fehlt</td><td>Die Modellantwort wird nicht ausgeführt. Ein anderes Textmodell probieren. Rollentexte dürfen dem geforderten Ausgabeformat nicht widersprechen.</td></tr><tr><td>Team oder Pinwand inzwischen geändert</td><td>Keine alte Entscheidung wurde ausgeführt. Sichere bei Bedarf eigene Texte, lade den gespeicherten Stand und starte mit den aktuellen Daten neu.</td></tr><tr><td>Speichern nach Verbindungsabbruch unklar</td><td>Gespeicherten Stand laden und Inhalte prüfen. Eine neue Speicherung mit alter Revision wird absichtlich abgewiesen, statt Änderungen eines anderen Tabs zu überschreiben.</td></tr><tr><td>Teamstart verlangt andere Spielweise</td><td>Im Spielerprofil „Gemischt“ oder „Agent“ wählen. Pinwände und Tests sind im Menschenmodus erlaubt.</td></tr><tr><td>Der Browser läuft auf einem anderen Gerät</td><td>127.0.0.1 bezeichnet immer das Browsergerät. Diese Oberfläche unterstützt lokale Loopback-Endpunkte auf diesem Gerät oder OpenRouter. Ein Ollama auf einem anderen PC wird nicht über dessen 127.0.0.1 erreicht.</td></tr></tbody></table>
<p>Technische Grundlagen: <a href="https://docs.ollama.com/api/chat">Ollama Chat-API</a>, <a href="https://docs.ollama.com/faq">Ollama Browserfreigaben und Speichersteuerung</a>, <a href="https://openrouter.ai/docs/api_reference/overview">OpenRouter API</a>. Stand der beschriebenen Spieloberfläche: 8. Oktober 2026.</p>


## 21 Häufige Probleme und ihre Lösung

### Ich bin angemeldet aber habe keinen Heimatplaneten

Du hast möglicherweise erst ein Konto angelegt. Wähle Volk und Spielweise und beanspruche einen freigegebenen Platz. Sind alle freigegebenen Plätze belegt, brauchst du einen Wartelistenplatz.

### Ich sehe im Browser nur einen Verbindungsfehler

Prüfe deine Internetverbindung und lade den öffentlichen Spielzugang erneut. Beachte einen angezeigten Wartungshinweis. Bleibt die Spielwelt nicht erreichbar, versuche es später erneut. Dein Konto und dein bestätigter Fortschritt bleiben erhalten.

### Ein Bau oder Forschungsknopf bleibt grau

Lies die angezeigte Voraussetzung. Mögliche Gründe sind fehlende Güter, Gebäudestufen, Technologie, Zivilisationsstufe oder Bauplätze. Auch Pause, Unruhen, Niederlage, Epochenende oder eine aktive Agentenzuständigkeit können Befehle sperren.

### Ein Auftrag wartet obwohl ich genug Erz habe

Prüfe sämtliche Güter am richtigen Planeten, den ersten wartenden Auftrag und den aktiven Bauplatz. Ein Auftrag kann beispielsweise Kristall, Elektronik oder eine beendete Reparatur benötigen. Erz auf einem anderen Planeten hilft erst nach einem Transport.

### Meine Forschung kommt kaum voran

Prüfe aktive Forschung, Laborleistung, Energie, Arbeitskräfte und Spezialisten. Die Forschung wird an Spielstunden verarbeitet. Eine alte Restzeitprognose gilt nicht unverändert nach einem Versorgungsausfall.

### Der Systemscan ist fertig aber die Planeten bleiben unbekannt

Der Systemscan kartiert das System. Für Eigenschaften, Belegung und Güter eines einzelnen Planeten braucht dieser eine eigene Spionagemission. Prüfe auch den Zeitstempel deines Berichts.

### Ich sehe einen Angriff aber keine Schiffstypen

Die Grundwarnung zeigt die Ankunft. Für Details brauchst du genügend Vorsprung deiner Sensoren vor der Abschirmtechnik. Prüfe alle vier Sensorbestandteile oder schicke rechtzeitig eine Flottensonde.

### Warum wurde mein Flottenstart abgelehnt

Prüfe Flottenplatz, vorhandene Schiffe, Besatzung, Ziel, Missionsvoraussetzungen, Frachtraum und echten Flugtreibstoff. Verladenes Deuterium allein ist keine automatische Treibstoffreserve. Eine Blockade oder veränderte Zielbelegung kann den Plan ebenfalls ungültig machen.

### Das Kolonieschiff ist noch nicht baubar

Astrophysik allein reicht nicht. Prüfe Zivilisationsstufe IV, Werft, Orbitalwerft, Bauteile und die übrigen angezeigten Voraussetzungen. Für die spätere Mission brauchst du zusätzlich Siedler, Eskorte und Startfracht.

### Mein Agent hat Rohstoffe aber seine Aktion wird abgelehnt

Lies den konkreten Ablehnungsgrund. Neben örtlichen Gütern und Technik gelten Rollenfreigabe, Agentenbudget und der aktuelle Weltzustand. Ein wiederholter identischer Befehl behebt keine fehlende Voraussetzung.

### Ollama wird nicht gefunden

Prüfe, ob Ollama auf deinem PC läuft und ein Modell installiert ist. Vergleiche die genaue Modellkennung, den erlaubten Ursprung der Spielseite und eine mögliche lokale Netzwerkberechtigung des Browsers. Benutze danach erneut **Modelle laden** und den Modelltest.

### Der OpenRouter-Test schlägt fehl

Prüfe Schlüssel, Modellkennung, Anbieterlimit, Guthaben und die angezeigte Fehlermeldung. Ein erfolgreicher Test beweist die Verbindung; er beweist noch keine gute Spielstrategie.

### Mein Agent hört auf zu handeln

Prüfe Aufruflimit, Fehlerprotokoll, Browser-Tab, PC-Energiesparmodus, Modellverbindung und Steuerungsfreigabe. Auch Weltpause, Niederlage und Epochenende stoppen Aktionen. Nach drei aufeinanderfolgenden Fehlern muss die Ursache behoben und der Agent bewusst neu gestartet werden.

### Meine Vorräte steigen aber meine Punkte nicht

Vorräte im Lager sind keine investierten Wirtschaftspunkte. Punkte ändern sich durch abgeschlossene Bauten, Forschung, vorhandene Streitkräfte, Bevölkerung und Zivilisationsstufe.

### Mein Reich ist grau und ich kann nichts bauen

Prüfe den Status. Ein besiegtes Reich ist für diese Epoche ausgeschieden. Es kann nicht durch neue Lieferungen wieder aktiviert werden. Du kannst zuschauen und in einer neuen Epoche erneut mitspielen.

## 22 Die wichtigsten Begriffe

| Begriff | Bedeutung |
|---|---|
| **Epoche** | Eine gemeinsame Welt mit festgelegter Laufzeit und Endwertung |
| **Reich** | Die Zivilisation eines menschlichen Teilnehmers, Bots oder Agenten |
| **Heimatplanet** | Der ursprüngliche Startplanet; dauerhaft gegen Übernahme geschützt |
| **Kolonie** | Ein zusätzlich besiedelter oder eroberter Planet |
| **Volk** | Deine für die Epoche feste Auswahl mit bestimmten Vor- und Nachteilen |
| **Zivilisationsstufe** | Entwicklungsstufe I bis V, die weitere Möglichkeiten freischaltet |
| **Gebäudestufe** | Ausbaugrad eines bestimmten Gebäudes |
| **Integrität** | Zustand eines Gebäudes; beeinflusst seine tatsächliche Wirkung |
| **FP** | Forschungspunkte, die deine Labore erzeugen |
| **Nettorate** | Veränderung eines Vorrats nach laufender Produktion und laufendem Verbrauch |
| **Spielzeit** | Zeit der Welt, abhängig von Tempo und Weltpause |
| **Teilnehmerplatz** | Platz für ein Konto und sein Reich |
| **Flottenplatz** | Kapazität für eine gleichzeitig laufende Flottenmission |
| **Aufklärung** | Eigene, durch erlaubte Beobachtung gewonnene Informationen |
| **Saven** | Schiffe und transportierbare Güter rechtzeitig auf eine geplante Abwesenheit schicken |
| **Blockade** | Feindliche Kontrolle des Orbits, die Verkehr behindern kann |
| **Verbandsangriff** | Mehrere gültig zusammengeführte Angriffsflotten mit gemeinsamer Ankunft |
| **Doktrin** | Ziele und Budgetverteilung für die Agentenrollen |
| **Agentenrolle** | Ein Aufgabenbereich wie Verwalter oder Feldherr |
| **Steuerungsfreigabe** | Befristetes exklusives Recht des Browsers, Agenten für ein Reich zu führen |
| **Reset** | Bewusster Beginn einer neuen Welt, mit Verlust des bisherigen Spielfortschritts |

## 23 Weiterlesen und im Spiel nachschlagen

Der [bebilderte Spielguide](../Sternenepoche-Start.html) verbindet erste Schritte, alle 20 Spielbereiche und Regeln mit aktuellen Ansichten. Die genaue Einrichtung deiner eigenen Modelle steht in den Kapiteln 6 und 20.

Kosten, Voraussetzungen, Flugpläne und Berichte findest du unmittelbar bei der jeweiligen Aktion im Spiel. Diese aktuellen Angaben sind maßgeblich, wenn die laufende Epoche abweichende Einstellungen verwendet.
