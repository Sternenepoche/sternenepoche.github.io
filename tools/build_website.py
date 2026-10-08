"""Rebuild every public player-facing page and its machine-readable sources."""
import json
import re
from website_content import ROOT
from build_landingpage import build as landing
from build_startpage import build as guide
from build_handbook import build as handbook

def build():
    landing(); guide(); handbook()
    source = (ROOT/'docs/HANDBUCH.md').read_text(encoding='utf-8')
    chapters = [{'number':int(m[1]),'title':m[2],'markdown':m[3].strip()}
        for m in re.finditer(r'^## (\d+) ([^\n]+)\n(.*?)(?=^## |\Z)',source,re.M|re.S)]
    data = {'title':'Sternenepoche Spielguide','updated':'2026-10-08','language':'de',
        'scope':'Gemeinsame Onlinewelt, Online V1','website':'https://sternenepoche.github.io/',
        'guide':'https://sternenepoche.github.io/Sternenepoche-Start.html',
        'rules_note':'Konkrete Kosten, Voraussetzungen und Flugpläne kommen aus dem aktiven Regelprofil im Spiel. Angaben zu Freigaben beschreiben die Startphase, keine Livebelegung.',
        'chapters':chapters}
    (ROOT/'docs/spielguide.json').write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
    (ROOT/'docs/spielguide.txt').write_text(source,encoding='utf-8')
    print('Structured handbook:',len(chapters),'chapters')

if __name__=='__main__': build()
