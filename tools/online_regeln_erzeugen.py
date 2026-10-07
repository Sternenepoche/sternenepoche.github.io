"""Generate the separate online rules without touching historical laboratory rules."""
from pathlib import Path
root = Path(__file__).resolve().parents[1]
text = (root / 'regeln/regelwerk.ron').read_text(encoding='utf-8')
text = text.replace('version: "0.1.0"', 'version: "0.2.0-online.1"', 1)
text = text.replace('warnzeit_basis_minuten: 30', 'warnzeit_basis_minuten: 120', 1)
text = text.replace('        kaserne: (kosten:', '''        geheimdienst: (kosten: {erz: 1500, kristall: 2000, elektronik: 50}, faktor: 1.8, arbeiter: 3.0, fachkraefte: 2.0, energie: 15.0, ab_stufe: 2,
            braucht: {labor: 1}, wirkung: "wertet Sensoren und Flottensonden aus; gemeinsam mit Überwachungs- und Spionagetechnik"),
        kaserne: (kosten:''', 1)
text = text.replace('        spionagetechnik: (kosten:', '''        ueberwachungstechnik: (kosten: {erz: 300, kristall: 600, deuterium: 200}, faktor: 2.0, fp: 300.0, ab_stufe: 2, labor: 2,
            braucht: {spionagetechnik: 1}, wirkung: "verbessert mit Geheimdienst und Sensorphalanx die Vorwarnung und passive Flottenanalyse"),
        abschirmtechnik: (kosten: {erz: 400, kristall: 800, deuterium: 300}, faktor: 2.0, fp: 400.0, ab_stufe: 2, labor: 2,
            braucht: {spionagetechnik: 1}, wirkung: "verhindert gegnerische Flottendetails; Stand bei Abflug gilt für diese Flotte, Basiswarnung bleibt"),
        spionagetechnik: (kosten:''', 1)
text = text.replace('verlängert je Stufe die Vorwarnzeit vor anfliegenden Flotten', 'verlängert zusammen mit Geheimdienst, Überwachungs- und Spionagetechnik die Vorwarnzeit', 1)
# The stage descriptions are used as agent-readable unlock lists.
for line in text.splitlines():
    if 'schaltet_frei:' in line and 'Elektronikwerk' in line:
        text = text.replace(line, line.replace('schaltet_frei: "', 'schaltet_frei: "Geheimdienst, Überwachungstechnik, Abschirmtechnik, ', 1))
(root / 'regeln/online-v1.ron').write_text(text, encoding='utf-8', newline='\n')
