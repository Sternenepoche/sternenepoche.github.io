"""Prepare only deliberately public Jekyll sources. Never package the checkout."""
from pathlib import Path
import argparse, shutil, json, re, subprocess
ROOT=Path(__file__).resolve().parents[1]
EXT={'.html','.md','.css','.js','.svg','.png','.jpg','.jpeg','.webp','.ico'}
def prepare(out:Path):
    out=out.resolve()
    if not out.is_relative_to(ROOT/'.pages-artifact'):
        raise ValueError('Output must stay inside .pages-artifact')
    out.mkdir(parents=True,exist_ok=False)
    approved=[]
    tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode('utf-8').split('\0'))
    for name in ['index.html','Bestandsaufnahme.html','_config.yml']:
        file=ROOT/name
        if file.is_file(): approved.append(file)
    # Deliberate file list: the sibling admin directory must never be packaged.
    for name in ['index.html','app.js','presentation.js','style.css','config.js','game-ui.js','game.css','art.js','three.min.js','galaxy.js','galaxy.css']:
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
    for folder in ['_layouts','docs','betrachter']:
        approved.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix.lower() in EXT
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
        else:
            shutil.copyfile(file,target)
    return len(approved)
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--out',type=Path,default=ROOT/'.pages-artifact/source')
    args=parser.parse_args();print(f'{prepare(args.out)} public files prepared; no runtime data or admin UI.')
