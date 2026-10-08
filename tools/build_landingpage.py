"""Build the illustrated public entry from maintainable sources and existing artwork."""
import json
from site_policy import decorate
from website_content import ROOT, finish, screenshot

def build():
    document = (ROOT/'docs/landingpage.html').read_text(encoding='utf-8')
    for key in ['reich','gebaeude','karte']:
        document = document.replace('<!-- SCREEN_'+key+' -->',screenshot(key))
    data = {'@context':'https://schema.org','@type':'VideoGame','name':'Sternenepoche',
        'url':'https://sternenepoche.github.io/','inLanguage':'de','genre':['Strategy','Space strategy'],
        'gamePlatform':'Web browser','playMode':'MultiPlayer',
        'description':'Gemeinsame Weltraumstrategie mit Reichsaufbau, Forschung, Sondenaufklärung, Handel, Kolonisation und Flotten. Menschen, Serverbots und KI-Agenten spielen nach denselben Regeln.',
        'dateModified':'2026-10-08','image':'https://sternenepoche.github.io/web-client/assets/backgrounds.start.webp',
        'subjectOf':{'@type':'Article','name':'Bebilderter Sternenepoche-Spielguide','url':'https://sternenepoche.github.io/Sternenepoche-Start.html'}}
    document = document.replace('<!-- STRUCTURED_DATA -->','<script type="application/ld+json">'+json.dumps(data,ensure_ascii=False)+'</script>')
    document = decorate(finish(document),'KI-Hinweis.html')
    (ROOT/'index.html').write_text(document,encoding='utf-8')
    print('Landing page built:',len(document.encode('utf-8')),'bytes')

if __name__=='__main__': build()
