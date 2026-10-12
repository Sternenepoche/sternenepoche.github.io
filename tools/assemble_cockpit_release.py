"""Reassemble the locally verified Windows release from a pinned base and binary delta."""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import urllib.request
import zipfile

parser = argparse.ArgumentParser()
parser.add_argument('--base', type=Path)
parser.add_argument('--output', type=Path, default=Path('release-out'))
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
payload = root / 'release-update' / '0.2.3'
meta = json.loads((payload / 'metadata.json').read_text())
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
base = args.base or out / 'base.zip'
if not base.exists():
    with urllib.request.urlopen(meta['base_url'], timeout=90) as source, base.open('wb') as target:
        while block := source.read(1024 * 1024):
            target.write(block)
assert hashlib.sha256(base.read_bytes()).hexdigest() == meta['base_sha256'], 'Base release differs'
work = out / 'files'
work.mkdir(exist_ok=True)
with zipfile.ZipFile(base) as archive:
    for entry in meta['entries']:
        name = Path(entry['filename']).name
        target = work / name
        replacement = payload / name
        target.write_bytes(replacement.read_bytes() if replacement.exists() else archive.read(entry['filename']))
subprocess.run(['git', 'apply', '--binary', str(payload / 'cockpit.patch')], cwd=work, check=True)
result = out / meta['filename']
with zipfile.ZipFile(result, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for entry in meta['entries']:
        data = (work / Path(entry['filename']).name).read_bytes()
        assert hashlib.sha256(data).hexdigest() == entry['sha256'], entry['filename']
        info = zipfile.ZipInfo(entry['filename'], tuple(entry['date_time']))
        for field in ('create_system', 'create_version', 'extract_version', 'external_attr', 'internal_attr'):
            setattr(info, field, entry[field])
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, data, compresslevel=9)
digest = hashlib.sha256(result.read_bytes()).hexdigest()
assert digest == meta['expected_sha256'], f'Archive differs: {digest}'
(out / 'SHA256SUMS.txt').write_text(f'{digest}  {result.name}\n')
print(f'PASS: byte-identical verified release {result.name} ({digest})')
