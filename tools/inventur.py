"""Metadata-only project inventory: never read account files or model keys."""
from pathlib import Path
import hashlib, json, subprocess
root=Path(__file__).resolve().parents[1]
github=root.parent/'sternenepoche.github.io'
before=root/'laeufe/inventur-2026-10-07/vorher/kern'
def files(path, extensions=None):
    return [p for p in path.rglob('*') if p.is_file() and not any(part in {'target','target-windows','__pycache__','.git','node_modules'} for part in p.parts) and (extensions is None or p.suffix in extensions)]
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
source=[]
for p in files(before,{'.rs','.toml','.md'}):
    other=github/'crates/kern'/p.relative_to(before)
    if other.exists() and digest(p)!=digest(other): source.append(str(p.relative_to(before)))
catalog=json.loads((root/'content/catalog.json').read_text(encoding='utf-8'))
assets=catalog.get('assets',[])
night=json.loads((root/'content/night-status.json').read_text(encoding='utf-8'))
report={
    'date':'2026-10-07','local_root':str(root),'github_root':str(github),
    'github_status':subprocess.run(['git','status','--short'],cwd=github,capture_output=True,text=True).stdout.splitlines(),
    'core_differences_before_this_task':source,
    'rust_source_files':{str(d.relative_to(root)):len(files(d,{'.rs'})) for d in (root/'crates').iterdir() if d.is_dir()},
    'web_python_files':len(files(github/'web',{'.py'})),
    'web_static_files':len(files(github/'web/static')),
    'catalog_assets':len(assets),
    'catalog_outputs_present':sum((root/'content'/a.get('file','MISSING')).is_file() for a in assets),
    'night_status':{k:night.get(k) for k in ('state','total','completed')},
    'content_pngs':len(files(root/'content/assets',{'.png'})),
    'core_files_changed_in_this_task':[str(p.relative_to(root/'crates/kern')) for p in files(root/'crates/kern',{'.rs'}) if not (before/p.relative_to(root/'crates/kern')).exists() or digest(p)!=digest(before/p.relative_to(root/'crates/kern'))],
}
out=root/'laeufe/inventur-2026-10-07/inventur.json';out.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
