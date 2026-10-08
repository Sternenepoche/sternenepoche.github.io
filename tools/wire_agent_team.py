"""One-time, idempotent wiring of the team workspace into the existing client."""
from pathlib import Path
root=Path(__file__).resolve().parents[1]
p=root/'web-client/index.html'
s=p.read_text(encoding='utf-8')
if 'src="agent-team.js"' not in s:
    s=s.replace('<script defer src="communication.js"></script>','<script defer src="communication.js"></script><script defer src="agent-team.js"></script><link rel="stylesheet" href="agent-team.css">')
if 'id="team-lobby"' not in s:
    s=s.replace('<section class="panel" hidden="" id="claim-panel">','<details class="panel" id="team-lobby"><summary>Eigene KI vorab einrichten und testen</summary><div id="team-lobby-content"><div id="team-workbench"></div></div></details>\n<section class="panel" hidden="" id="claim-panel">')
if 'data-tab="pinwaende"' not in s:
    s=s.replace('<button aria-selected="false" data-tab="agent">Agentensteuerung</button>','<button aria-selected="false" data-tab="agent">Modelle &amp; Team</button><button aria-selected="false" data-tab="pinwaende">Pinwände</button>')
    s=s.replace('<section class="panel" hidden="" id="agent">','<section class="panel" hidden id="pinwaende"><h2>Deine Pinwände</h2><p>Wissen behalten. Ziele planen. Das Team aufeinander abstimmen.</p><div id="pinboard-content"></div></section>\n<section class="panel" hidden="" id="agent">')
s=s.replace('http://localhost:* http://[::1]:*;', 'http://localhost:*;')
p.write_text(s,encoding='utf-8')
p=root/'tools/build_pages.py';s=p.read_text(encoding='utf-8');s=s.replace("'communication.js','guide/", "'communication.js','agent-team.js','agent-team.css','agenten-hilfe.html','guide/");p.write_text(s,encoding='utf-8')
p=root/'crates/server/src/main.rs';s=p.read_text(encoding='utf-8')
if '(false,"/agent-team.js")' not in s:
    marker='            (false,"/communication.js")'
    i=s.index(marker)
    s=s[:i]+''.join(f'            (false,"/{name}")=>Some((include_bytes!("../../../web-client/{name}").as_slice(),"{mime}; charset=utf-8")),\n' for name,mime in [('agent-team.js','text/javascript'),('agent-team.css','text/css'),('agenten-hilfe.html','text/html')])+s[i:]
s=s.replace('http://localhost:* http://[::1]:*;', 'http://localhost:*;')
p.write_text(s,encoding='utf-8')
