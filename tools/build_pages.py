"""Prepare only deliberately public Jekyll sources. Never package the checkout."""
from pathlib import Path
import argparse, shutil, json, re, subprocess, base64, hashlib
from io import BytesIO
from PIL import Image
from site_policy import decorate
ROOT=Path(__file__).resolve().parents[1]
EXT={'.html','.md','.css','.js','.svg','.png','.jpg','.jpeg','.webp','.ico','.gif'}

def externalize_art(document:str,out:Path)->str:
    """Keep local pages standalone; publish each repeated bitmap once for browser caching."""
    def image(match):
        mime,encoded=match.groups()
        data=base64.b64decode(encoded,validate=True)
        stem=hashlib.sha256(data).hexdigest()[:32]
        name=stem+'.'+mime
        target=out/'site-art'/name
        target.parent.mkdir(exist_ok=True)
        if not target.exists():
            target.write_bytes(data)
            with Image.open(BytesIO(data)) as im:
                if mime=='webp':
                    if 'A' in im.getbands() and im.getchannel('A').getextrema()[0]<255:
                        # Preserve cutout alpha; JPEG would restore a rectangular matte.
                        im.save(target.with_suffix('.png'),'PNG',optimize=True)
                    else:
                        im.convert('RGB').save(target.with_suffix('.jpg'),'JPEG',quality=82,optimize=True,progressive=True)
                else:
                    im.save(target.with_suffix('.webp'),'WEBP',quality=86,method=6)
        return '/site-art/'+name
    document=re.sub(r'data:image/(webp|png);base64,([A-Za-z0-9+/]+={0,2})',image,document)
    # CSS backgrounds and SVG image elements use small, universally decoded JPEGs.
    document=re.sub(r'(url\(["\']?)(/site-art/[a-f0-9]+)\.webp',r'\1\2.jpg',document)
    document=re.sub(r'(<image\b[^>]*\bhref=")(/site-art/[a-f0-9]+)\.webp',r'\1\2.jpg',document)
    document=re.sub(r'(image: ")(/site-art/[a-f0-9]+)\.webp',r'\1\2.jpg',document)
    def picture(match):
        tag=match[0]
        src=re.search(r'\bsrc="(/site-art/[a-f0-9]+\.(webp|png))"',tag)
        if not src:return tag
        url,kind=src.groups()
        if 'data-animated-logo' in tag:return tag
        if 'id="planet-image"' in tag:return tag.replace(url,url.removesuffix('.webp')+'.jpg')
        modern=url.removesuffix('.png')+'.webp' if kind=='png' else url
        fallback=(url.removesuffix('.webp')+('.png' if (out/url.removeprefix('/')).with_suffix('.png').exists() else '.jpg')) if kind=='webp' else url
        return '<picture><source type="image/webp" srcset="'+modern+'">'+tag.replace(url,fallback)+'</picture>'
    return re.sub(r'<img\b[^>]*>',picture,document)
def prepare(out:Path):
    out=out.resolve()
    if not out.is_relative_to(ROOT/'.pages-artifact'):
        raise ValueError('Output must stay inside .pages-artifact')
    out.mkdir(parents=True,exist_ok=False)
    approved=[]
    tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode('utf-8').split('\0'))
    for name in ['index.html','KI-Hinweis.html','Sternenepoche-Start.html','Sternenepoche-Handbuch.html','_config.yml','llms.txt','robots.txt','sitemap.xml','docs/HANDBUCH.md','docs/spielguide.json','docs/spielguide.txt','docs/branding/sternenepoche-neuralstern-512.gif']:
        file=ROOT/name
        if file.is_file(): approved.append(file)
    # Deliberate file list: the sibling admin directory must never be packaged.
    for name in ['index.html','app.js','identity-ui.js','identity-ui.css','presentation.js','style.css','config.js','game-ui.js','game.css','art.js','three.min.js','galaxy.js','galaxy.css','ai-labels.js','loading-screen.css','loading-screen.js','brand-animation.js','loading-no-js.css','brand/neuralstern-panels.webp','brand/neuralstern-panels.png','brand/neuralstern-poster.jpg']:
        file=ROOT/'web-client'/name
        if file.is_file(): approved.append(file)
    # Exact reviewed image manifest, never the whole web-client directory.
    manifest=ROOT/'web-client/art.js'
    if manifest.exists():
        raw=manifest.read_text(encoding='utf-8').removeprefix('window.STERNEN_ART=').strip().removesuffix(';')
        art=json.loads(raw)
        for item in art['images'].values():
            name=item['url'].split('?')[0]
            if not re.fullmatch(r'assets/[a-z0-9_.-]+\.webp',name): raise ValueError('Unsafe image manifest')
            image=ROOT/'web-client'/name
            if image.relative_to(ROOT).as_posix() not in tracked: raise ValueError('Untracked public image')
            approved.append(image)
    # Only player-facing documentation and explicitly public artwork are shipped.
    # In particular: no recursive docs export, historical viewer, inventories or awarre.html.
    for folder in ['_layouts','docs/bilder/online','docs/branding','docs/artwork']:
        approved.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix.lower() in EXT
                        and (folder=='_layouts' or p.suffix.lower() in {'.svg','.png','.jpg','.jpeg','.webp','.ico','.gif'})
                        and p.relative_to(ROOT).as_posix() in tracked
                        and not p.is_symlink() and not any(s in {'data','saves','keys','geheimnisse'} for s in p.parts))
    for file in approved:
        # Resolve before reading to reject links/junctions into runtime data.
        if not file.resolve().is_relative_to(ROOT): raise ValueError('Public source leaves checkout')
        target=out/file.relative_to(ROOT);target.parent.mkdir(parents=True,exist_ok=True)
        if file.suffix.lower()=='.md':
            body=file.read_text(encoding='utf-8-sig')
            if not body.startswith('---\n'):
                title=next((line[2:].strip() for line in body.splitlines() if line.startswith('# ')),file.stem)
                body='---\nlayout: default\ntitle: '+json.dumps(title,ensure_ascii=False)+'\n---\n\n'+body
            # Repository Markdown keeps .md links; the built website uses .html.
            body=re.sub(r'\]\((?!https?://)([^)\s]+)\.md(#[^)]*)?\)',lambda m:']('+m[1]+'.html'+(m[2] or '')+')',body)
            target.write_text(body,encoding='utf-8')
        elif file.suffix.lower()=='.html':
            name=file.relative_to(ROOT).as_posix()
            body=decorate(externalize_art(file.read_text(encoding='utf-8-sig'),out),
                          models=name=='web-client/index.html',
                          handlers=name=='betrachter/index.html')
            body=re.sub(r'href="(?!https?://)([^"#]+)\.md(#[^"]*)?"',
                        lambda m:'href="'+m[1]+'.html'+(m[2] or '')+'"',body)
            target.write_text(body,encoding='utf-8')
        else:
            shutil.copyfile(file,target)
    loading_files=['brand-animation.js','loading-screen.js','loading-screen.css','brand/neuralstern-panels.webp','brand/neuralstern-poster.jpg']
    if sum((ROOT/'web-client'/name).stat().st_size for name in loading_files)>110000:
        raise ValueError('Default loader exceeds its 110 KB transfer budget')
    # Preserve old shared links, but never publish the 27 MB production master.
    compact=out/'docs/branding/sternenepoche-neuralstern-512.gif'
    if compact.is_file() and compact.stat().st_size>1100000: raise ValueError('Public GIF exceeds 1.1 MB')
    if compact.is_file():
        shutil.copyfile(compact,compact.with_name('sternenepoche-neuralstern-1280.gif'))
    verify_player_publication(out)
    return len(approved)

def verify_player_publication(out:Path):
    """Fail closed if operator instructions leak into any public text representation."""
    forbidden=re.compile(r'8891|Dashboard-oeffnen|Server-starten\.cmd|Server-stoppen\.cmd|Server-status\.cmd|SERVER-BETRIEB|ONLINE-KONZEPT|ANMELDUNG-UND-KONTEN|awarre\.html|Verwaltungsdashboard|privates? Dashboard|Karls PC|Für Karl|data/online|[A-Za-z]:[\\/]projekte_ki',re.I)
    for file in out.rglob('*'):
        if file.is_file() and file.suffix.lower() in {'.html','.md','.txt','.json','.xml'}:
            match=forbidden.search(file.read_text(encoding='utf-8-sig'))
            if match: raise ValueError(f'Operator content in public file {file.relative_to(out)}: {match.group()}')
    for name in ['awarre.html','web-client/admin','docs/SERVER-BETRIEB.md','docs/ONLINE-KONZEPT.md','betrachter','Bestandsaufnahme.html']:
        if (out/name).exists(): raise ValueError('Private/retired document packaged: '+name)
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--out',type=Path,default=ROOT/'.pages-artifact/source')
    args=parser.parse_args();print(f'{prepare(args.out)} public files prepared; no runtime data or admin UI.')
