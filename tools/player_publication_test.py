"""Public player documentation must stay useful without exposing operator instructions."""
from pathlib import Path
from urllib.parse import urlsplit, unquote
import argparse, json, re, tempfile
from build_pages import verify_player_publication

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'.pages-artifact/source'

def main():
    verify_player_publication(OUT)
    for name in ['docs/SERVER-BETRIEB.md','docs/ONLINE-KONZEPT.md','docs/ANMELDUNG-UND-KONTEN.md',
                 'docs/README.md','Bestandsaufnahme.html','betrachter/index.html','awarre.html']:
        assert not (OUT/name).exists(), name
    guide=(OUT/'Sternenepoche-Start.html').read_text(encoding='utf-8')
    assert 'id="agent-setup"' in guide and 'id="server-start"' not in guide
    for term in ['OpenRouter','OLLAMA_ORIGINS','Modelle laden','Pinwände','soul.md','rollen.md','127.0.0.1:11434']:
        assert term in guide, 'Player model setup lost: '+term
    structured=json.loads((OUT/'docs/spielguide.json').read_text(encoding='utf-8'))
    assert next(c for c in structured['chapters'] if c['number']==20)['title']=='Deinen eigenen KI-Assistenten verbinden'
    for name in ['index.html','Sternenepoche-Start.html','Sternenepoche-Handbuch.html','KI-Hinweis.html']:
        document=(OUT/name).read_text(encoding='utf-8')
        for href in re.findall(r'\bhref="([^"]+)"',document):
            url=urlsplit(href)
            if url.scheme or url.netloc or not url.path: continue
            target=(OUT/unquote(url.path.lstrip('/'))).resolve()
            assert target.is_relative_to(OUT.resolve()),href
            assert target.is_file() or (target/'index.html').is_file() or (target.suffix=='.html' and target.with_suffix('.md').is_file()),(name,href)
    # Guard machine-readable exports as well as HTML. Removing navigation alone isn't enough.
    with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
        target=Path(temporary)/'guide.json'
        target.write_text(json.dumps({'text':'Privates Dashboard: http://127.0.0.1:8891'}),encoding='utf-8')
        try: verify_player_publication(Path(temporary))
        except ValueError: pass
        else: raise AssertionError('Operator content was accepted in a JSON export')
    print('PASS: operator documents excluded, all player model instructions retained, public links valid, JSON leak rejected.')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--source',type=Path,default=OUT)
    OUT=parser.parse_args().source.resolve()
    main()
