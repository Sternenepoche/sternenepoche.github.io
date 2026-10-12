"""Export only public rule data for reproducible website builds without production masters."""
from pathlib import Path
import json,re,shutil
ROOT=Path(__file__).resolve().parents[1]
def main():
 data=json.loads((ROOT/'regeln/flottenausbau.json').read_text(encoding='utf-8'))
 factions=json.loads((ROOT/'content/catalog.json').read_text(encoding='utf-8'))['factions']
 source=(ROOT/'regeln/online-v1.ron').read_text(encoding='utf-8').split('voelker:',1)[1].split('wirtschaft:',1)[0]
 for key,f in factions.items():
  body=(re.search(r'\b'+key+r':\s*\((.*?)\)',source,re.S) or [None,''])[1]
  f['values']={k:float(v) for k,v in re.findall(r'\b(ladung|waffen|panzerung|kolonieschiff):\s*([0-9.]+)',body)}
 data['factions']=factions
 (ROOT/'docs/flottenkodex.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 shutil.copyfile(ROOT/'content/public-game-catalog.json',ROOT/'docs/technologie-regeln.json')
 print('Public catalogue sources exported: 35 unit types, four factions, 87 technology goals')
if __name__=='__main__':main()
