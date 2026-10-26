"""Handbuch für die Website: nur der heutige Stand (Karl 21.10.2026: „nur das neueste … ohne Verweise auf Altes“).

docs/HANDBUCH.md bleibt unverändert (Quelle der Regel-Sitzungen, mit Vermerken wie „(bis 20.10.2026: …)“ oder „Gilt für neue
Welten“). Für die Website entfallen Versionsvermerke, Hinweise auf ältere Welten und die alte Übernahmeregel; Sätze werden so
umgestellt, dass sie direkt beschreiben, was gilt. Jede gezielte Ersetzung muss greifen – sonst bricht der Bau ab, damit eine
geänderte Quelle auffällt. Am Ende darf kein Alt-Bezug übrig sein.
"""
from __future__ import annotations

import re

# Gezielte Umformulierungen (Text muss genau so vorkommen).
GENAU = [
    (", wie sie in neu erzeugten Welten gelten", ""),
    ("**Gilt für neue Welten.** Monde, Mondgebäude, Hyperraumtor-Generatoren und Weltenkrümmer gibt es nur in Welten, die mit der Mondgeneration erzeugt wurden. Ältere Welten bleiben ohne Monde.\n", ""),
    ("**Gilt für neue Welten.** Ältere Welten behalten ihre bisherigen Wachstums- und Sonnentabellen; ihre Werte zeigt die Regelreferenz im Spiel.\n", ""),
    ("**Gilt für neue Welten.** In älteren Welten bleibt es bei acht Mitgliedern und ohne Dysonsphäre.\n", ""),
    ("**Gilt für neue Welten mit Monden.** ", ""),
    ("**Gilt für neue Welten mit Monden.**\n", ""),
    ("**In neuen Welten gilt die Invasion aus Kapitel 30:**", "**Die Übernahme ist eine Invasion (Kapitel 30):**"),
    ("In neuen Welten ruhen Verarbeitungswerke", "Verarbeitungswerke ruhen"),
    ("; in neuen Welten ist er gesperrt", "; er ist gesperrt"),
    ("In neuen Welten ruht ein Verarbeitungswerk", "Ein Verarbeitungswerk ruht"),
    ("In neuen Welten gilt außerdem:", "Außerdem gilt:"),
    ("In neuen Welten ist er gesperrt", "Er ist gesperrt"),
    ("In neuen Welten wechseln drei Schiffe", "Drei Schiffe wechseln"),
    ("Allianzen können in neuen Welten bis zu **50 Mitglieder** haben, in älteren Welten bis zu acht.", "Allianzen können bis zu **50 Mitglieder** haben."),
    ("; die Obergrenze beträgt in neuen Welten 50, in älteren acht.", "; die Obergrenze beträgt 50."),
    # 0.4.7/0.4.8 (Handbuch-Stellen zu Kolonieplätzen, Mehrfachfeuer, Reparatur, Konter-Übersicht, Invasion v2, gestrandeten Flotten)
    ('(in neuen Welten 2 frei ab Start, ab der dritten Kolonie je Stufe Terraforming einer mehr)', '(2 frei ab Start, ab der dritten Kolonie je Stufe Terraforming einer mehr)'),
    ('**Kolonieplätze (neue Welten ab 0.4.7):**', '**Kolonieplätze:**'),
    (' In älteren, schon laufenden Welten erlaubt weiter jede Astrophysikstufe eine Kolonie.', ''),
    (' In älteren, schon laufenden Welten belegen laufende Kolonisationsvorhaben weiter geplante Kapazität.', ''),
    ('In neuen Welten (ab 0.4.7) ist Terraforming zugleich die **Kolonieforschung**', 'Terraforming ist zugleich die **Kolonieforschung**'),
    ('**In neuen Welten gilt Mehrfachfeuer:**', '**Mehrfachfeuer:**'),
    (' In älteren Welten endet Schnellfeuer mit dem Abschuss.', ''),
    ('**In neuen Welten** kostet eine Reparatur 75 % der Baukosten der aktuellen Stufe', 'Eine Reparatur kostet 75 % der Baukosten der aktuellen Stufe'),
    (' **In älteren Welten** richten sich die Kosten nach dem fehlenden Anteil des gesamten bisherigen Bauwerts.', ''),
    ('**Rechenbeispiel:** Die bisherigen Ausbauten einer Anlage haben zusammen 1.000 Erz und 600 Kristall gekostet. Bei 30 % Integrität fehlen 70 %. Daraus ergeben sich in diesem vereinfachten Beispiel 700 Erz und 420 Kristall Reparaturmaterial.', '**Rechenbeispiel:** Die aktuelle Stufe einer Anlage kostet 1.000 Erz und 600 Kristall. Bei 30 % Integrität zahlst du 75 % davon: 750 Erz und 450 Kristall. Bei 65 % Integrität ist der Schaden halb so groß: 375 Erz und 225 Kristall.'),
    (' (in älteren Welten Stufe IV und Orbitalwerft)', ''),
    ('Die Übersicht unten nennt für neue Welten Werte,', 'Die Übersicht unten nennt Werte,'),
    ('**Konter-Übersicht neuer Welten**', '**Konter-Übersicht**'),
    ('**Gilt für neue Welten.** Ältere, schon laufende Welten behalten die Belagerungsregeln ihrer Version. ', ''),
    ('**Gilt für neue Welten.** Eine Flotte ist **gestrandet**', 'Eine Flotte ist **gestrandet**'),
    ("; in neuen Welten wirken nur Bomber", "; dabei wirken nur Bomber"),
    ("mit Kolonieschiff und, in neuen Welten, Bodentruppen", "mit Kolonieschiff und Bodentruppen"),
]
# Muster: Versionsvermerke in Klammern, Satz über die alte Miliz, „in neuen Welten“ mitten im Satz.
MUSTER = [
    (r"\s*\([^()]*\b(?:bis|seit) \d{1,2}\.\d{1,2}\.\d{4}[^()]*\)", ""),
    (r"^\*\*Stand des [^*]+:\*\*[^\n]*Android 0\.\d[^\n]*\n", ""),
    (r" ?Die folgende kurze Frist und die frühere Miliz gelten dort nicht mehr\.", ""),
    (r" ?Die folgende kurze Frist gilt dort nicht\.", ""),
    (r",? in neuen Welten,?(?= )", ""),
]
# Abschnitt mit der Übernahmeregel älterer Welten (Kapitel 16) bis zum Hinweis auf den Heimatplaneten.
ABSCHNITT = (r"\*\*In älteren Welten\*\* läuft die Übernahme so:.*?(?=\*\*Der ursprüngliche Heimatplanet)", "")
REST = re.compile(r"neue[nr]? Welten|älter(?:e|en|er) Welten|\b(?:bis|seit) \d{1,2}\.\d{1,2}\.\d{4}|Android 0\.\d|frühere Miliz|In älteren")


def heute(text: str) -> str:
    # Seit 22.10. bereinigen die Regel-Sitzungen die Quelle selbst: fehlt eine Stelle, ist sie dort schon entfernt.
    # Die Restprüfung unten bleibt streng – übrig gebliebene Alt-Bezüge brechen den Bau weiter ab.
    for alt, neu in GENAU:
        if alt not in text:
            raise ValueError(f"Handbuch geändert – Stelle fehlt: {alt[:70]!r}")
        text = text.replace(alt, neu)
    text, n = re.subn(ABSCHNITT[0], ABSCHNITT[1], text, flags=re.S)
    if n != 1:
        raise ValueError("Handbuch geändert – Übernahmeregel älterer Welten nicht gefunden")
    for muster, neu in MUSTER:
        text = re.sub(muster, neu, text, flags=re.M)
    rest = sorted({m.group() for m in REST.finditer(text)})
    if rest:
        raise ValueError(f"Handbuch: Alt-Bezüge übrig: {rest}")
    return text
