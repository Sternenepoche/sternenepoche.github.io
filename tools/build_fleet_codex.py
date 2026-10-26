"""Öffentlicher Kodex der Sternenepoche: Schiffe & Verteidigung, Bodentruppen mit Kontertabelle, Raketen, Mondanlagen
und Allianzwunder. Werte kommen aus dem Kern (docs/flottenkodex.json und docs/technologie-regeln.json, erzeugt von
crates/server/examples/catalog_export.rs + tools/sync_website_catalog.py), Texte und Archiv aus content/beschreibungen.json,
auch die Dysonsphäre (regeln/dyson.ron über den Kern). Bilder löst das gemeinsame Bildverzeichnis (web-client/art.js) zur Laufzeit auf.
"""
from pathlib import Path
import hashlib, json
from website_content import ROOT, finish
from site_policy import decorate

VOELKER = ("aurelianer", "krath", "syntheten", "veyari")


def wunder(public_rules: dict) -> dict:
    """Allianzwunder aus dem Kernkatalog (catalog_export: Dysonregeln, Anteil je Epochenlänge aus `anteil_fuer`).
    Absolute Wirkungen skaliert der Kern mit s = Anteil / voller Anteil (`dyson_skala_ppm`), relative Boni nicht."""
    d = public_rules.get("dyson")
    if not d:
        return {}
    r = d["regeln"]
    tabelle = [{"tage": z["tage"], "anteil": z["anteil"], "einspeisung": r["einspeisung"] * z["anteil"] / r["anteil_voll"],
                "deuterium": r["strahlsynthese_deuterium"] * z["anteil"] / r["anteil_voll"],
                "xenokristall": r["strahlsynthese_xenokristall"] * z["anteil"] / r["anteil_voll"]} for z in d["anteile"]]
    return {"dysonsphaere": {"regeln": r, "tabelle": tabelle, "allianz_max": public_rules.get("allianz_max")}}


CSS = r"""html{scroll-padding-top:0}
body{background:#050b13;color:#edf5fc}.codex-wrap{max-width:1280px;margin:auto;padding:20px clamp(12px,3vw,36px) 60px}.codex-head{display:flex;align-items:center;justify-content:space-between;gap:14px}.codex-head a{color:#dcf5ff}.codex-intro{max-width:850px;line-height:1.65;color:#c6d7e4}.codex-intro h1{color:#f7e5b6;font-size:clamp(1.55rem,4vw,2.6rem);line-height:1.15}
.codex-bereiche{display:flex;flex-wrap:wrap;gap:8px;margin:18px 0 0;padding:0;list-style:none}.codex-bereiche button{min-height:44px;padding:6px 15px;border:1px solid #476278;border-radius:22px;background:#102033;color:#dcf5ff;font:inherit;cursor:pointer}.codex-bereiche button[aria-pressed=true]{border-color:#f0cf8a;color:#fff1c9;box-shadow:inset 0 0 0 1px #f0cf8a}.codex-bereiche small{color:#9fb4c6;margin-left:6px}
.codex-filter{display:grid;grid-template-columns:1fr 1fr 1.4fr 1.2fr;gap:12px;margin:18px 0 22px}.codex-filter label{display:grid;gap:7px;font-size:.82rem;color:#cee4f5}.codex-filter label[hidden]{display:none}.codex-filter select,.codex-filter input{background:#102033;color:white;border:1px solid #4c637b;min-height:44px;border-radius:9px;padding:8px;width:100%;box-sizing:border-box}
.codex-layout{display:grid;grid-template-columns:minmax(0,1.4fr) minmax(280px,.9fr);gap:24px;align-items:start}.codex-detail{border:1px solid #476278;border-radius:18px;overflow:hidden;background:linear-gradient(125deg,#102336ec,#091724ed);scroll-margin-top:16px}.codex-art{width:100%;height:clamp(180px,27vw,360px);object-fit:contain;background:radial-gradient(ellipse at center,#20354b55,#030811)}.codex-copy{padding:clamp(14px,2vw,26px)}.codex-copy h2{margin:0 0 9px;color:#fff0ca}.codex-copy p{color:#d4e2ed;line-height:1.55}.codex-facts{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px}.codex-facts div{background:#0b1727a6;border:1px solid #334e68;border-radius:10px;padding:10px}.codex-facts dt{font-size:.74rem;color:#bacee0}.codex-facts dd{margin:5px 0 0;color:#fff1c9;font-weight:700}.codex-copy h3{font-size:.95rem;color:#fff1c9}.codex-copy ul{padding-left:20px;color:#d4e2ed;line-height:1.75}
.codex-gallery{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}.codex-tile{min-width:0;border:1px solid #34506a;border-radius:12px;overflow:hidden;background:#0e2134c9;color:#f1f7fe;text-align:left;padding:0;cursor:pointer;min-height:44px}.codex-tile[aria-pressed=true]{border-color:#92dfff;box-shadow:inset 0 0 0 1px #92dfff}.codex-tile img{width:100%;height:100px;object-fit:contain;background:#050c16}.codex-tile span{display:block;padding:9px;font-size:.85rem;line-height:1.35}.codex-pending{min-height:120px;display:grid;place-content:center;background:linear-gradient(135deg,#172c4050,#061321);color:#c9e8fb;font-size:.82rem;padding:12px}.codex-preview-note{border-left:3px solid #99dbf7;padding:10px 14px;color:#d2e8f6;background:#17324b55;border-radius:6px}.codex-tile:focus-visible,a:focus-visible,.codex-bereiche button:focus-visible{outline:3px solid #adeaff;outline-offset:3px}
.codex-texte{display:grid;gap:8px;margin:14px 0}.codex-texte div{border-left:3px solid #476278;padding:2px 0 2px 12px}.codex-texte dt{font-size:.74rem;letter-spacing:.06em;text-transform:uppercase;color:#9fdcff}.codex-texte dd{margin:3px 0 0;color:#d4e2ed;line-height:1.55}
.codex-konter{list-style:none;padding:0!important;display:grid;gap:6px}.codex-konter li{display:grid;grid-template-columns:3.6em minmax(0,1fr);gap:10px;align-items:baseline}.codex-konter b{min-width:3.2em;text-align:center;border-radius:12px;padding:1px 8px;font-size:.84rem}.k-stark{background:#3e6b3a;color:#eaffd9}.k-gut{background:#4b5f2d;color:#f3ffd6}.k-schwach{background:#6a2f2a;color:#ffe2dc}.k-normal{background:#1d3147;color:#bacee0}
.codex-info{margin-top:28px;border:1px solid #34506a;border-radius:16px;background:linear-gradient(160deg,#0f2234f0,#08131ef0);padding:clamp(14px,2.4vw,26px)}.codex-info[hidden]{display:none}.codex-info h2{margin:0 0 8px;color:#fff0ca;font-size:clamp(1.15rem,2.6vw,1.5rem)}.codex-info p,.codex-info li{color:#d4e2ed;line-height:1.6;max-width:80ch}.codex-info ol{padding-left:22px}.codex-info strong{color:#fff1c9}
.matrix-scroll{overflow:auto;max-width:100%;border:1px solid #34506a;border-radius:12px;margin:14px 0}.codex-matrix{border-collapse:separate;border-spacing:0;font-size:.8rem;min-width:max-content}.codex-matrix th,.codex-matrix td{border-bottom:1px solid #1f3349;border-right:1px solid #1f3349;padding:0;text-align:center}.codex-matrix thead th{background:#0b1727;color:#bacee0;font-weight:600;vertical-align:bottom;height:9.5em;position:sticky;top:0;z-index:1}.codex-matrix thead th span{writing-mode:vertical-rl;transform:rotate(180deg);display:inline-block;padding:8px 4px;white-space:nowrap}.codex-matrix tbody th{position:sticky;left:0;background:#0b1727;z-index:2;text-align:left}.codex-matrix thead th:first-child{left:0;z-index:3;text-align:left;vertical-align:bottom;padding:8px;writing-mode:horizontal-tb}.codex-matrix tbody th button{all:unset;display:block;box-sizing:border-box;min-height:36px;padding:8px 10px;cursor:pointer;color:#dcf5ff;white-space:nowrap}.codex-matrix tbody th button:focus-visible{outline:3px solid #adeaff;outline-offset:-3px}.codex-matrix td{min-width:2.9em;height:36px;font-weight:700}.codex-matrix td.k-normal{background:transparent;color:#5d748a;font-weight:400}.codex-matrix tr.aktiv th,.codex-matrix tr.aktiv td{box-shadow:inset 0 2px 0 #92dfff,inset 0 -2px 0 #92dfff}.codex-matrix th.aktiv{color:#fff1c9;background:#16314a}.codex-legende{display:flex;flex-wrap:wrap;gap:8px 14px;padding:0;list-style:none;font-size:.85rem;color:#bacee0}.codex-legende b{display:inline-block;min-width:2.6em;text-align:center;border-radius:10px;padding:1px 6px;margin-right:6px}
.codex-tabelle{border-collapse:collapse;width:100%;font-size:.88rem;margin:8px 0 4px}.codex-tabelle th,.codex-tabelle td{border-bottom:1px solid #2a4258;padding:7px 6px;text-align:right}.codex-tabelle th:first-child,.codex-tabelle td:first-child{text-align:left}.codex-tabelle th{color:#bacee0;font-weight:600}.codex-tabelle td{color:#fff1c9}.tabelle-scroll{overflow-x:auto}
.codex-figur{margin:16px 0 0}.codex-figur img{width:100%;border-radius:12px;border:1px solid #34506a;background:#030811}.codex-figur figcaption{font-size:.8rem;color:#9fb4c6;margin-top:6px}
.codex-archiv{margin-top:24px;padding-top:4px;border-top:1px solid #34506a}.codex-archiv>h3{margin:16px 0 10px;font-size:1.15rem;letter-spacing:.04em;color:#f7e5b6}.codex-archiv h4{margin:20px 0 6px;font-size:.76rem;letter-spacing:.09em;text-transform:uppercase;color:#f0cf8a}.codex-archiv p{max-width:68ch;line-height:1.65;text-wrap:pretty}.codex-archiv .codex-archiv-lead{border-left:3px solid #c9a961;padding:2px 0 2px 14px;font-style:italic;color:#e6eef5}.codex-akte{max-width:70ch;margin:0 0 8px;border:1px solid #34506a;border-left:3px solid var(--akte,#8aa);border-radius:8px;background:#0b1727a6}.codex-akte[data-volk=aurelianer]{--akte:#d7ab65}.codex-akte[data-volk=krath]{--akte:#d46e59}.codex-akte[data-volk=syntheten]{--akte:#87b9de}.codex-akte[data-volk=veyari]{--akte:#79bc9e}.codex-akte summary{display:flex;align-items:center;min-height:44px;box-sizing:border-box;padding:8px 12px;cursor:pointer;font-weight:700;color:var(--akte)}.codex-akte summary small{margin-left:8px;font-weight:400;color:#bacee0}.codex-akte p{margin:0 0 .8em;padding:0 14px}.codex-copy ul.codex-verweise{list-style:none;padding:0;margin:0;display:flex;flex-wrap:wrap;gap:8px;line-height:1.3}.codex-verweise button,.codex-verweise span{display:inline-flex;align-items:center;min-height:36px;box-sizing:border-box;padding:4px 13px;border:1px solid #476278;border-radius:18px;background:#102033;color:#9fdcff;font:inherit;font-size:.86rem}.codex-verweise button{cursor:pointer}.codex-verweise span{color:#bacee0;border-style:dashed}.codex-verweise button:focus-visible,.codex-akte summary:focus-visible{outline:3px solid #adeaff;outline-offset:2px}
@media(max-width:700px){.codex-verweise button{min-height:44px}.codex-akte p{padding:0 11px}.codex-layout{grid-template-columns:minmax(0,1fr)}.codex-filter{grid-template-columns:1fr 1fr}.codex-filter label:nth-child(n+3){grid-column:1/-1}.codex-gallery{grid-template-columns:repeat(3,minmax(0,1fr))}.codex-tile img{height:80px}.codex-tile span{font-size:.72rem;padding:8px}.codex-pending{min-height:80px}.codex-wrap{padding-top:14px}.codex-facts{grid-template-columns:repeat(2,minmax(0,1fr))}.codex-bereiche button{padding:6px 12px}}
@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}@media(forced-colors:active){.codex-detail,.codex-facts div,.codex-tile,.codex-info{border:1px solid CanvasText}}
"""

JS = r"""
const data=JSON.parse(document.getElementById('fleet-data').textContent),el=id=>document.getElementById(id),esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])),fmt=n=>Number(n||0).toLocaleString('de-DE',{maximumFractionDigits:1}),ART=window.STERNEN_ART||{},names=ART.names||{};
const VOLK={aurelianer:'Aurelianer',krath:'Krath',syntheten:'Syntheten',veyari:'Veyari'};
const BEREICHE={flotte:'Schiffe & Verteidigung',boden:'Bodentruppen & Miliz',raketen:'Raketen',mond:'Mondanlagen',wunder:'Allianzwunder'};
const classes={erkundung:'Erkundung & Bergung',jaeger:'Jäger',transporter:'Transporter',kreuzer:'Kreuzer',schlachtkreuzer:'Schlachtkreuzer',schlachter:'Schlachter',bomber:'Bomber',zerstoerer:'Zerstörer',megaschiff:'Megaschiffe',kolonie:'Kolonieschiffe',sonde:'Sonden',bergbau:'Bergbau',bergung:'Recycler',truppen:'Truppentransport',satellit:'Satelliten',verteidigung:'Verteidigungswaffen',schild:'Planetenschilde'};
const TRUPPEN=data.boden.typen,TRUPPE=Object.fromEntries(TRUPPEN.map((t,i)=>[t.id,{...t,i}])),RAKETE=Object.fromEntries(data.raketen.map(r=>[r.key,r]));
const ziel=j=>j<TRUPPEN.length?TRUPPEN[j].name:'Miliz';
const name=k=>data.units[k]?(names[k]||data.units[k].label):(TRUPPE[k]?.name||RAKETE[k]?.name||data.texte[k]?.name||names[k]||data.archivnamen?.[k]||k.replaceAll('_',' '));
const LISTE={flotte:()=>Object.keys(data.units).filter(k=>!el('class').value||data.units[k].klasse===el('class').value),boden:()=>[...TRUPPEN.map(t=>t.id),'miliz','garnison'],raketen:()=>data.raketen.map(r=>r.key),mond:()=>Object.keys(data.mond?.gebaeude||{}),wunder:()=>Object.keys(data.wunder||{})};
const wo=k=>data.units[k]?'flotte':TRUPPE[k]||k==='miliz'||k==='garnison'?'boden':RAKETE[k]?'raketen':data.mond?.gebaeude?.[k]?'mond':data.wunder?.[k]?'wunder':'';
let bereich='flotte';const chosen={flotte:'leichter_jaeger',boden:'infanterie',raketen:'interplanetar',mond:'mondbasis',wunder:'dysonsphaere'};
el('faction').innerHTML=Object.entries(data.factions).map(([k,f])=>`<option value="${k}">${esc(f.label)}</option>`).join('');el('class').insertAdjacentHTML('beforeend',Object.entries(classes).filter(([k])=>Object.values(data.units).some(u=>u.klasse===k)).map(([k,v])=>`<option value="${k}">${v}</option>`).join(''));
el('bereiche').innerHTML=Object.entries(BEREICHE).filter(([b])=>LISTE[b]().length||b==='flotte').map(([b,t])=>`<li><button type="button" data-bereich="${b}" aria-pressed="${b===bereich}">${t}<small>${b==='flotte'?Object.keys(data.units).length:LISTE[b]().length}</small></button></li>`).join('');
// Archiv wie im Spiel (web-client/bauseite.js): lange Felder in Absätze zu je etwa drei Sätzen, gewähltes Volk zuerst offen.
const absaetze=t=>{const s=String(t).split(/(?<=[.!?“])\s+(?=[A-ZÄÖÜ„])/),n=s.length>4?3:s.length,out=[];for(let i=0;i<s.length;i+=n)out.push(`<p>${esc(s.slice(i,i+n).join(' '))}</p>`);return out.join('');};
function archiv(key,faction){const a=data.archiv?.[key];if(!a)return '';const reihe=[faction,...Object.keys(VOLK).filter(v=>v!==faction)].filter(v=>a[v]),verweise=Array.isArray(a.verweise)?a.verweise:[];
 const akte=(v,i)=>`<details class="codex-akte" data-volk="${v}"${i===0?' open':''}><summary>${VOLK[v]}-Akte${i===0?' <small>gewähltes Volk</small>':''}</summary>${absaetze(a[v])}</details>`;
 const verweis=k=>`<li>${wo(k)?`<button type="button" data-archiv-unit="${esc(k)}">${esc(name(k))}</button>`:`<span>${esc(name(k))}</span>`}</li>`;
 return `<section class="codex-archiv" aria-labelledby="archiv-titel"><h3 id="archiv-titel">Archiv</h3>${a.lore?`<p class="codex-archiv-lead">${esc(a.lore)}</p>`:''}${a.technik?`<h4>Technisches Dossier</h4>${absaetze(a.technik)}`:''}${a.geschichte?`<h4>Chronik</h4>${absaetze(a.geschichte)}`:''}${reihe.length?`<h4>Volksakten</h4>${reihe.map(akte).join('')}`:''}${verweise.length?`<h4>Siehe auch</h4><ul class="codex-verweise">${verweise.map(verweis).join('')}</ul>`:''}</section>`;}
function texte(key){const t=data.texte[key];if(!t)return '';const z=[['Wozu',t.zweck],['Stärke',t.staerke],['Grenzen',t.nische]].filter(([,v])=>v);return z.length?`<dl class="codex-texte">${z.map(([k,v])=>`<div><dt>${k}</dt><dd>${esc(v)}</dd></div>`).join('')}</dl>`:'';}
function bildUrl(cat,key,f){const im=ART.images||{},al=ART.aliases||{};for(const z of [`${cat}|${key}`,al[`${cat}|${key}`]].filter(Boolean))for(const s of [f,''])if(im[`${z}|${s}`])return 'web-client/'+im[`${z}|${s}`].url;return '';}
function quelle(key){const b=wo(key),f=el('faction').value;if(b==='wunder')return bildUrl('buildings',key+'_bau',f)||bildUrl('buildings',key,f);if(b==='flotte'){const k=data.units[key].klasse;return bildUrl(k==='satellit'?'satellites':k==='megaschiff'?'megaships':['verteidigung','schild'].includes(k)?'defenses':'ships',key,f);}if(b==='boden')return bildUrl('ground_units',TRUPPE[key]?.bild||key,f);if(b==='raketen')return bildUrl('missiles',key,f);if(b==='mond')return bildUrl('buildings',key,f);return '';}
function picture(key,large=false){const u=wo(key)==='wunder'&&large?bildUrl('buildings',key,''):quelle(key),f=data.factions[el('faction').value].label;return u?`<img class="${large?'codex-art':''}" src="${esc(u)}" alt="${esc(name(key)+' · '+f)}" ${large?'':'loading="lazy"'}>`:`<div class="codex-pending${large?' codex-art':''}"><span>✧ ${esc(name(key))}<br>Volksmotiv wird ergänzt</span></div>`;}
const fact=(k,n)=>`<div><dt>${k}</dt><dd>${typeof n==='string'?esc(n):fmt(n)}</dd></div>`,kosten=c=>Object.entries(c||{}).filter(([,n])=>n).map(([k,n])=>`${esc(names[k]||data.texte[k]?.name||k)} ${fmt(n)}`).join(' · ')||'keine',dauer=s=>s>=3600?fmt(s/3600)+' h':fmt(s/60)+' min';
const faktor=p=>({3000:'×3',2500:'×2,5',2000:'×2',1500:'×1,5',1000:'×1',750:'×¾',500:'×½',250:'×¼'}[p]||'×'+fmt(p/1000)),stufe=p=>p>=2000?'k-stark':p>1000?'k-gut':p<1000?'k-schwach':'k-normal';
function kopf(key,unter,beschreibung){return `<small>${esc(data.factions[el('faction').value].label)} · ${esc(unter)}</small><h2>${esc(name(key))}</h2>${beschreibung?`<p>${esc(beschreibung)}</p>`:''}`;}
function flotte(key,faction){const f=data.factions[faction],u=data.units[key],p=data.profiles[key],v=f.values||{},cost=Object.fromEntries(Object.entries(u.kosten).map(([k,n])=>[k,p.klasse==='kolonie'?Math.floor(n*(v.kolonieschiff||1)):n]));
 const werft=u.klasse==='megaschiff'?`Werftdock ${u.werft} eines Mondes`:u.werft?`Werft ${u.werft}`:'';
 return kopf(key,classes[u.klasse]||u.klasse,p.rolle)+`<dl class="codex-facts">${fact('Angriff',Math.floor(u.angriff*(v.waffen||1)))}${fact('Schild',u.schild)}${fact('Struktur',Math.floor(u.struktur*(v.panzerung||1)))}${fact('Tempo',u.tempo||'stationär')}${fact('Fracht',Math.floor(u.ladung*(v.ladung||1)))}${fact('Treibstoff',u.verbrauch)}${fact('Besatzung',u.besatzung)}${fact('Getrennte Ziele',p.ziele)}${fact('Ziv.-Stufe',u.ab_stufe)}</dl><p>Grundwerte mit Volksbonus; deine Forschung verbessert sie zusätzlich. Treibstoffkosten ergeben sich aus Strecke, Tempo und Raumhafen.</p>${texte(key)}<h3>Voraussetzungen</h3><ul><li>Zivilisationsstufe ${u.ab_stufe}</li>${werft?`<li>${werft}</li>`:''}${Object.entries(u.braucht).map(([k,n])=>`<li>${esc(name(k))} ${n}</li>`).join('')}${Object.entries(p.voraussetzungen).map(([k,n])=>`<li>${esc(name(k))} ${n}</li>`).join('')}</ul><h3>Kosten</h3><p>${Object.entries(cost).map(([k,n])=>`${esc(name(k))} ${fmt(n)}`).join(' · ')}</p><h3>Konter und Nachteile</h3><ul>${Object.entries(p.schadensfaktoren).map(([k,n])=>`<li>${esc(name(k))}: Schaden ×${fmt(n)}</li>`).join('')||'<li>Grundschaden; Eskorte für zivile Schiffe einplanen.</li>'}</ul>${p.schildfaktor!==1?`<p>Schildschaden ×${fmt(p.schildfaktor)}.</p>`:''}${p.siedler?`<h3>Koloniestart</h3><p>${fmt(p.siedler)} eingeschiffte Siedler · ${p.kolonie_minen} fertige Minen · Grundversorgung und mindestens ${fmt(u.ladung)} Rohstoff-Lagerkapazität. Ein Kolonieschiff wird bei Gründung verbraucht.</p>`:''}${p.klasse==='schild'?'<p>Pro Planet eine aktive Schildstufe. Eine höhere Stufe ersetzt das kleinere System.</p>':''}<h3>Schnellfeuer</h3><p>${Object.entries(u.schnellfeuer).map(([k,n])=>`${esc(name(k))} ×${n}`).join(' · ')||'Keines'} · Folgeschüsse auf dasselbe Ziel; maximal 64. Mehrzielwaffen wählen bis zu vier verschiedene Ziele je Runde.</p>`;}
function konterListe(zeilen){return zeilen.length?`<ul class="codex-konter">${zeilen.map(([n,p,zusatz])=>`<li><b class="${stufe(p)}">${faktor(p)}</b><span>${esc(n)}${zusatz?` · ${esc(zusatz)}`:''}</span></li>`).join('')}</ul>`:'<p>Keine.</p>';}
// Freischaltung der Miliz: Kaserne und (Invasion v2) die Forschung mit ihrem Anzeigenamen
const milizFrei=M=>[String(M.freischaltung||'').replace('kaserne','Kaserne'),...Object.entries(M.forschung||{}).map(([k,n])=>name(k)+' '+n)].filter(Boolean).join(' + ');
function boden(key){const I=data.invasion||{},M=I.miliz||{};
 if(key==='garnison')return kopf(key,'Planetare Bodenverteidigung',data.texte[key]?.rolle)+texte(key)+`<p>Die Garnison ist keine eigene Einheit, sondern alles, was am Boden verteidigt: stationierte Bodentruppen, verbündete Hilfstruppen und die ausgerufene Miliz. ${I.regeln==='invasion-v2'?'Sie steht auf den Verteidigungslinien und im Rückhalt; fällt der Rückhalt, ist der Planet verwüstet.':'Eine Übernahme gelingt erst, wenn keiner dieser Verteidiger mehr lebt.'}</p>`;
 const sp=TRUPPEN.length;
 if(key==='miliz'){const gegen=TRUPPEN.map(t=>[t.name,t.konter_promille[sp],`${fmt(t.angriff*t.konter_promille[sp]/1000)} Schaden je Schuss`]).filter(([,p])=>p>1000).sort((a,b)=>b[1]-a[1]);
  return kopf(key,'Volksaufgebot',data.texte[key]?.rolle)+`<dl class="codex-facts">${fact('Waffenwert je Bewohner',M.waffenwert)}${fact('Rüstwert je Bewohner',M.ruestwert)}${fact('Legierung je Bewohner',M.legierung_je_bewohner)}${fact('Freischaltung',milizFrei(M))}${M.ausbildung_sekunden_je_1000?fact('Ausbildung je 1.000',fmt(Math.round(M.ausbildung_sekunden_je_1000/60))+' Spielminuten'):''}${fact('Arbeitsausfall',(M.arbeitsausfall_prozent||0)+' %')}${fact('Kapitulation ab',(M.kapitulation_prozent||0)+' % Gefallene')}</dl>${texte(key)}<h3>Wer die Miliz besonders hart trifft</h3>${konterListe(gegen)}<p>${I.regeln==='invasion-v2'?`Jederzeit ausrufbar und wieder entlassbar; es kämpft ein Anteil der Bewohner je Volk: ${Object.entries(M.anteil_promille||{}).sort((a,b)=>b[1]-a[1]).map(([v,p])=>`${esc(VOLK[v]||v)} ${p>=1000?'alle':fmt(p/10)+'&nbsp;%'}`).join(', ')}. Ohne ausgerufene Miliz leisten die Bewohner keinen Widerstand.`:'Ohne ausgerufene Miliz leisten die Bewohner keinen Widerstand. Gilt in Welten mit Invasion v1.'}</p>`;}
 const t=TRUPPE[key],i=t.i,c={};data.gueter.forEach((g,j)=>{if(t.kosten_milli[j])c[g]=t.kosten_milli[j]/1000;});
 const stark=t.konter_promille.map((p,j)=>[ziel(j),p,`${fmt(t.angriff*p/1000)} Schaden je Schuss`]).filter(([,p])=>p>1000).sort((a,b)=>b[1]-a[1]);
 const schwach=t.konter_promille.map((p,j)=>[ziel(j),p,`${fmt(t.angriff*p/1000)} Schaden je Schuss`]).filter(([,p])=>p<1000).sort((a,b)=>a[1]-b[1]);
 const gefahr=TRUPPEN.map(a=>[a.name,a.konter_promille[i],`${fmt(a.angriff*a.konter_promille[i]/1000)} Schaden je Schuss`]).filter(([,p])=>p>1000).sort((a,b)=>b[1]-a[1]);
 return kopf(key,'Bodentruppe',t.rolle)+`<dl class="codex-facts">${fact('Angriff',t.angriff)}${fact('Struktur',t.struktur)}${fact('Kaserne',t.kaserne)}${fact('Ziv.-Stufe',t.ab_stufe)}${fact('Transportplätze',t.transportplaetze)}${fact('Fertigungsplätze',t.fertigungsplaetze)}${fact('Personal',t.personal)}${fact('Ausbildung',dauer(t.dauer_sekunden))}${fact('Schaden je Platz',t.angriff/t.transportplaetze)}</dl>${texte(key)}<h3>Kosten</h3><p>${kosten(c)}</p><h3>Stark gegen</h3>${konterListe(stark)}<h3>Schwach gegen</h3>${konterListe(schwach)}<h3>Gefährlich für diese Einheit</h3>${konterListe(gefahr)}<p>Alle übrigen Paarungen treffen mit Grundschaden (×1). Die ganze Tabelle steht unter der Auswahl.</p>`;}
function rakete(key){const r=RAKETE[key],z=r.zielprofil||{},sf=Object.entries(z.schadensfaktoren||{});
 return kopf(key,'Rakete aus dem Silo',r.besonderheit)+`<dl class="codex-facts">${fact('Schaden',r.schaden)}${fact('Ziele je Rakete',z.ziele_je_rakete)}${fact('Silo-Stufe',r.silo)}${fact('Ziv.-Stufe',r.zivilisation)}${fact('Siloplätze',r.plaetze)}${fact('Bauzeit',dauer(r.bauzeit_sekunden))}${fact('Flugzeit',r.flug_min_sekunden?dauer(r.flug_min_sekunden)+' + '+dauer(r.flug_je_system_sekunden)+'/System':'sofort')}${fact('Reichweite',r.reichweite_je_silo?r.reichweite_je_silo+' Systeme je Silostufe':'eigener Planet')}${fact('Abfangraketen nötig',r.abfangbedarf||'–')}</dl>${texte(key)}<h3>Kosten</h3><p>${kosten(r.kosten)}</p><h3>Voraussetzungen</h3><ul><li>Zivilisationsstufe ${r.zivilisation}</li><li>Raketensilo ${r.silo}</li>${Object.entries(r.braucht||{}).map(([k,n])=>`<li>${esc(name(k))} ${n}</li>`).join('')}</ul><h3>Ziel</h3><ul><li>${esc(z.primaer)}</li><li>${esc(z.sekundaer)}</li></ul><p>${esc(z.verteilung)}</p>${sf.length?`<h3>Schaden je Zielklasse</h3><p>${sf.map(([k,n])=>`${esc(name(k))} ×${fmt(n)}`).join(' · ')}</p>`:''}`;}
function mond(key){const g=data.mond.gebaeude[key],ertrag=Object.entries(g).filter(([k])=>k.startsWith('ertrag_')).map(([k,n])=>`${esc(names[k.slice(7)]||k.slice(7))} ${fmt(n)}`).join(' · ');
 return kopf(key,'Mondanlage',g.wirkung)+`<dl class="codex-facts">${fact('Ziv.-Stufe',g.ab_stufe)}${fact('Energie',g.energie||0)}${fact('Kostenfaktor je Stufe',g.faktor)}${fact('Höchststufe',g.max_stufe||'offen')}${ertrag?fact('Grundertrag',ertrag):''}</dl>${texte(key)}<h3>Kosten (Stufe 1)</h3><p>${kosten(g.kosten)}</p><h3>Voraussetzungen</h3><ul><li>Zivilisationsstufe ${g.ab_stufe}</li>${Object.entries(g.braucht||{}).map(([k,n])=>`<li>${esc(name(k))} ${n}</li>`).join('')}${Object.entries(g.forschung||{}).map(([k,n])=>`<li>${esc(name(k))} ${n}</li>`).join('')}</ul>`;}
function wunder(key,faction){const W=data.wunder[key],r=W.regeln,mio=n=>n>=1e9?fmt(n/1e9)+' Mrd.':n>=1e6?fmt(n/1e6)+' Mio.':fmt(n),bau=bildUrl('buildings',key+'_bau',faction);
 return kopf(key,'Allianzwunder · Großprojekt einer Allianz','Milliarden Solarsatelliten auf eigenen Bahnen um die Sonne eines Systems. Eine Allianz baut sie gemeinsam, und bis zu vier ihrer Planeten im System hängen danach direkt am Strom der Sonne.')+`<dl class="codex-facts">${fact('Projektmitglieder',r.mitglieder_min+' oder mehr')}${fact('Ziv.-Stufe',r.stufe_min)}${fact('Bauzeit',fmt(r.bau_sekunden/86400)+' Spieltage')}${fact('Anschlüsse',r.anschluesse)}${fact('Solarenergie',`×${fmt(r.sonnenfaktor)}`)}${fact('Strombad',`×${fmt(r.strombad)}`)}</dl>
 <h3>Voraussetzungen</h3><ul><li>Mindestens ${r.mitglieder_min} Mitglieder derselben Allianz treten dem Projekt bei, jedes mit einem Planeten im System.</li><li>Jedes Projektmitglied hat Zivilisationsstufe ${r.stufe_min} und ist beim Baubeginn seit mindestens ${r.mitgliedschaft_min_tage} Tagen in der Allianz.</li><li>Eine offene Baustelle je Allianz und eine Sphäre je System; weitere Sphären nacheinander.</li>${W.allianz_max?`<li>Allianzen haben in neuen Welten bis zu ${W.allianz_max} Mitglieder.</li>`:''}<li>Die Wirkung hält, solange die Eigner-Allianz mindestens ${r.halten_min_plaetze} Plätze im System besetzt.</li></ul>
 <h3>Kosten je Mitglied</h3><p>Jedes Projektmitglied zahlt genau einmal seinen Anteil, in Teilbeträgen über die Projektkasse der Allianz, und zwar je Gut gleich viel Erz, Kristall und Deuterium. Die Höhe hängt von der Länge der Epoche ab; in kürzeren Welten wird die Sphäre billiger, wirkt aber im selben Verhältnis schwächer.</p><div class="tabelle-scroll"><table class="codex-tabelle"><thead><tr><th>Epoche</th><th>Anteil je Gut</th><th>Einspeisung/h</th><th>Deuterium/h</th><th>Xenokristall/h</th></tr></thead><tbody>${W.tabelle.map(z=>`<tr><td>${fmt(z.tage)} Tage</td><td>${mio(z.anteil)}</td><td>${mio(z.einspeisung)}</td><td>${mio(z.deuterium)}</td><td>${fmt(z.xenokristall)}</td></tr>`).join('')}</tbody></table></div><p>Unter ${fmt(r.min_epoche_tage)} Tagen Epochendauer gibt es keine Dysonsphäre. Die Werte je Stunde gelten für jeden Anschluss.</p>
 <h3>Wirkung nach Fertigstellung</h3><ul><li>Solarkraftwerke und Solarsatelliten der Eigner-Allianz im System liefern ×${fmt(r.sonnenfaktor)}.</li><li>Bis zu ${r.anschluesse} Planeten im System werden direkt angeschlossen, höchstens ${r.anschluesse_je_spieler} je Spieler; ein neuer Anschluss braucht ${fmt(r.anschluss_vorlauf_stunden)} Stunden Vorlauf.</li><li>Jeder Anschluss erhält eine feste Einspeisung und Strahlsynthese (Tabelle) sowie das Strombad: Deuteriumsynthesizer und Xenoextraktor arbeiten ×${fmt(r.strombad)}.</li>${r.xenosynthese_ohne_malus?`<li>Xenosynthese auf Anschlüssen ohne Malus, Rezeptbedarf ×${fmt(r.xenosynthese_rezept_faktor)}.</li>`:''}</ul>
 <h3>Austritt, Abbruch, Übernahme</h3><ul><li>Austritt vor Baubeginn: ${fmt(r.rueckzahlung_austritt*100)}&nbsp;% zurück. Abbruch vor Baubeginn: ${fmt(r.rueckzahlung_abbruch_vor_bau*100)}&nbsp;%, im Bau ${fmt(r.rueckzahlung_abbruch_im_bau*100)}&nbsp;% mal dem noch nicht gebauten Anteil; nach Fertigstellung nichts.</li><li>Eine verwaiste Sphäre geht nach ${fmt(r.uebernahme_tage)} Spieltagen Vollbesetzung an die besetzende Allianz.</li></ul>${bau?`<figure class="codex-figur"><img src="${esc(bau)}" alt="Bau der Dysonsphäre · ${esc(VOLK[faction])}" loading="lazy"><figcaption>Baustelle der Sphäre im Stil der ${esc(VOLK[faction])}.</figcaption></figure>`:''}<p>Gilt in neuen Welten. Laufende Welten behalten ihre bisherigen Regeln.</p>`;}
function info(){const I=data.invasion||{},B=data.boden,box=el('bereich-info');
 if(bereich==='boden'){const sp=TRUPPEN.length,kopfzeile=[...TRUPPEN.map(t=>t.name),'Miliz'],akt=TRUPPE[chosen.boden]?.i??(chosen.boden==='miliz'?sp:-1);
  box.hidden=false;box.innerHTML=`<h2>Wer schlägt wen am Boden</h2><p>Zeile greift an, Spalte wird getroffen. <strong>×2</strong> heißt doppelter Schaden, <strong>×½</strong> halber. Bodenrunden laufen alle ${fmt((B.rundenabstand_sekunden||900)/60)} Spielminuten${B.gleichzeitiges_feuer?', alle Seiten feuern gleichzeitig':''}. Tippe auf einen Namen, um die Einheit zu öffnen.</p>
  <div class="matrix-scroll" role="region" aria-label="Kontertabelle der Bodentruppen" tabindex="0"><table class="codex-matrix"><thead><tr><th scope="col">Angreifer ↓ · Ziel →</th>${kopfzeile.map((n,j)=>`<th scope="col" class="${j===akt?'aktiv':''}"><span>${esc(n)}</span></th>`).join('')}</tr></thead><tbody>${TRUPPEN.map((t,i)=>`<tr class="${i===akt?'aktiv':''}"><th scope="row"><button type="button" data-matrix="${t.id}">${esc(t.name)}</button></th>${t.konter_promille.map((p,j)=>`<td class="${stufe(p)}" title="${esc(t.name)} gegen ${esc(kopfzeile[j])}: ${faktor(p)}, ${fmt(t.angriff*p/1000)} Schaden je Schuss">${p===1000?'·':faktor(p)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>
  <ul class="codex-legende"><li><b class="k-stark">×2–3</b>stark</li><li><b class="k-gut">×1,5</b>überlegen</li><li><b class="k-normal">·</b>Grundschaden</li><li><b class="k-schwach">×¼–¾</b>schwach</li></ul>
  ${I.regeln==='invasion-v2'?`<h2>So läuft eine Invasion</h2><ol><li><strong>Verteidigung brechen:</strong> Eine Flotte mit Auftrag Bombardieren (mindestens ein Bomber) oder Kampfkolonisieren kämpft gegen Schiffe, Verteidigung und Schilde im Orbit. Nur der Sieger hält den Orbit.</li><li><strong>Bombardieren:</strong> die erste Bombenrunde sofort, dann alle ${fmt(I.runde_sekunden/60)} Minuten. Bombenlast: Bomber ${fmt(I.bombenlast?.bomber)}, Strategischer Bomber ${fmt(I.bombenlast?.strategischer_bomber)}. Alle Gebäude verlieren gleich viel Integrität, nie unter ${fmt(I.integritaet_grenze_prozent)}&nbsp;%.</li><li><strong>Landen:</strong> sobald alle Gebäude höchstens ${fmt(I.integritaet_grenze_prozent)}&nbsp;% haben, mit Bodentruppen in Truppentransportern (${fmt(I.transporter_plaetze)} Plätze je Transporter). Ein Kolonieschiff braucht es erst für die Übernahme.</li><li><strong>Linien und Wellen:</strong> Der Planet hat ${fmt(I.linien)} Verteidigungslinien, die letzte ist der Rückhalt. Alle ${fmt(I.wellen_abstand_sekunden/3600)} Spielstunden greift eine Welle die aktuelle Linie an, höchstens ${fmt(I.wellen_runden_max)} Bodenrunden nach der Tabelle oben. Der Verteidiger verteilt Garnison und Miliz frei auf die Linien.</li><li><strong>Verwüstet und übernommen:</strong> Fällt der Rückhalt, ist der Planet verwüstet; ein Kolonieschiff der Besatzer übernimmt ihn, sobald ein Kolonieplatz frei ist. Der ursprüngliche Heimatplanet ist nie übernehmbar.</li></ol><p>Die Verteidiger können Nachschub landen, Allianzmitglieder um Bodentruppen bitten und die <strong>Miliz</strong> ausrufen (${esc(milizFrei(I.miliz||{}))}, ${fmt(Math.round((I.miliz?.ausbildung_sekunden_je_1000||0)/60))} Spielminuten Ausbildung je 1.000 Bewohner). Die Kaserne bildet während einer Invasion mit ${fmt(I.ausbildung_waehrend_invasion_prozent)}&nbsp;% Tempo aus. Beschädigte Gebäude lassen sich danach reparieren.</p>`:I.belagerung_sekunden?`<h2>So läuft eine Invasion</h2><ol><li><strong>Verteidigung brechen:</strong> Eine Flotte mit Auftrag Bombardieren (mindestens ein Bomber) oder Kampfkolonisieren kämpft gegen Schiffe, Verteidigung und Schilde im Orbit. Nur der Sieger hält den Orbit.</li><li><strong>Bombardieren:</strong> alle ${fmt(I.runde_sekunden/60)} Minuten eine Bombenrunde. Bombenlast: Bomber ${fmt(I.bombenlast?.bomber)}, Strategischer Bomber ${fmt(I.bombenlast?.strategischer_bomber)}. Der Planet hat ${fmt(I.strukturpunkte_je_gebaeudestufe)} Strukturpunkte je Gebäudestufe; die Gebäude sinken bis auf ${fmt(I.integritaet_grenze_prozent)}&nbsp;% Integrität.</li><li><strong>Landen:</strong> Kolonieschiff und Bodentruppen in Truppentransportern. Ohne Bodentruppen keine Landung.</li><li><strong>Belagern:</strong> ${fmt(I.belagerung_sekunden/3600)} Spielstunden Bodenkampf nach der Tabelle oben.</li><li><strong>Übernehmen:</strong> erst, wenn kein Verteidiger mehr lebt. Der ursprüngliche Heimatplanet ist nie übernehmbar.</li></ol><p>Die Verteidiger können in dieser Zeit Nachschub landen (er kommt trotz Blockade durch), Allianzmitglieder um Bodentruppen bitten und die <strong>Miliz</strong> ausrufen (ab ${esc(String(I.miliz?.freischaltung||'').replace('kaserne','Kaserne'))}). Gilt in neuen Welten.</p>`:''}`;return;}
 if(bereich==='mond'&&data.mond){const m=data.mond;box.hidden=false;box.innerHTML=`<h2>Wie ein Mond entsteht</h2><ul><li>Nach einer Schlacht entsteht mit ${fmt(1)}&nbsp;% Chance je ${fmt(m.truemmer_je_prozent)} Trümmern ein Mond, höchstens mit ${fmt(m.chance_max_prozent)}&nbsp;%. Jeder Mond ist verdichteter Schlachtschutt.</li><li>Ein Mond hat ${fmt(m.grundfelder)} Grundfeld; jede Stufe Mondbasis bringt ${fmt(m.felder_je_mondbasis)} Felder dazu.</li><li>Unterhalt: ${fmt(m.unterhalt_basis_tag)} Credits am Tag plus ${fmt(m.unterhalt_je_stufe_tag)} je Gebäudestufe des Mondes.</li><li>Ab Werftdock ${fmt(m.weltentruemmer_werftdock)} kann der Mond Weltenkrümmer bauen – die einzigen Schiffe, die Monde zerstören.</li></ul><p>Mondanlagen gelten in neuen Welten (Mondgeneration).</p>`;return;}
 box.hidden=true;box.innerHTML='';}
// Silbentrennung wie im Spiel (stable-dom.js) – Kachelnamen brachen am Handy mitten im Wort (26.10.2026).
// Weiche Trennstellen für lange deutsche Wörter (Karl 25./26.10.2026: nie mitten im Wort umbrechen; Edge am Desktop trennt mit
// hyphens:auto nicht). Erst Wortfugen bekannter Grundwörter (…schiff, …kraftwerk), dann Silben je Teil: Vokalgruppen bleiben
// zusammen, von mehreren Konsonanten wandert ein gültiger Silbenanfang (bl, br, schl …) oder der letzte Konsonant in die nächste
// Silbe; ch, ck, sch, ph, th, qu bleiben ganz. Mindestens zwei Buchstaben vor und nach jeder Trennstelle. Ergebnis mit U+00AD.
const SILBEN_FUGEN = ['synthesizer', 'kolonisieren', 'wissenschaft', 'transporter', 'verbrauch', 'kapazität', 'infanterie', 'generator',
  'zerstörer', 'kraftwerk', 'förderung', 'abkommen', 'spionage', 'schlacht', 'frachter', 'satellit', 'extraktor', 'geschütz', 'schleier',
  'krümmer', 'kreuzer', 'antrieb', 'technik', 'bündnis', 'phalanx', 'zentrum', 'pionier', 'kaution', 'brecher', 'gebäude', 'archiv',
  'punkte', 'gürtel', 'schlucht', 'rakete', 'drohne', 'panzer', 'kräfte', 'werfer', 'schiff', 'stoff', 'lager', 'kunde', 'sonde', 'frost',
  'werft', 'werk', 'welt', 'mond', 'pakt', 'netz', 'dock'];
const SILBEN_VOKAL = /[aeiouäöüy]/i, SILBEN_DOPPEL = ['au', 'äu', 'eu', 'ei', 'ai', 'ey', 'ie', 'aa', 'ee', 'oo'];
const SILBEN_ANFANG = ['schl', 'schm', 'schn', 'schr', 'schw', 'sch', 'bl', 'br', 'dr', 'fl', 'fr', 'gl', 'gr', 'kl', 'kn', 'kr', 'pf',
  'pl', 'pr', 'tr', 'zw', 'ch', 'ck', 'ph', 'th', 'qu'];
function silbenTeil(w) {
  if (w.length < 6) return w;
  const k = w.toLocaleLowerCase('de'), stellen = [];
  let i = 0;
  while (i < k.length && !SILBEN_VOKAL.test(k[i])) i++;
  while (i < k.length) {
    let e = i + 1;                                           // Vokalgruppe [i, e)
    while (e < k.length && SILBEN_VOKAL.test(k[e]) && SILBEN_DOPPEL.includes(k.slice(e - 1, e + 1))) e++;
    let n = e;                                               // Konsonanten [e, n)
    while (n < k.length && !SILBEN_VOKAL.test(k[n])) n++;
    if (n >= k.length) break;                                // kein Vokal mehr: Wortende
    let s;
    if (n === e) s = e;                                      // Vokal trifft Vokal (De·u? nein: Doppellaut oben) → zwischen beide
    else {
      const c = k.slice(e, n), a = SILBEN_ANFANG.find(x => c.endsWith(x) && (c.length > x.length || x !== 'ck'));
      s = n - (a ? a.length : 1);
      if (a === 'ck') s = n - 1;                             // Zu·cker: k beginnt die Silbe
    }
    if (s >= 2 && k.length - s >= 2) stellen.push(s);
    i = n;
  }
  let r = '', z = 0;
  for (const s of stellen) { r += w.slice(z, s) + '­'; z = s; }
  return r + w.slice(z);
}
function silbenWort(w) {
  if (w.length < 10) return w;
  const k = w.toLocaleLowerCase('de');
  const f = SILBEN_FUGEN.find(x => k.endsWith(x) && w.length - x.length >= 3);
  if (f) return silbenWort(w.slice(0, w.length - f.length)) + '­' + silbenTeil(w.slice(w.length - f.length));
  return silbenTeil(w);
}
const silbenTrennen = s => String(s ?? '').replace(/[A-Za-zÄÖÜäöüß]{10,}/g, silbenWort);
function render(){const faction=el('faction').value,f=data.factions[faction],query=el('search').value.toLocaleLowerCase('de');
 el('class').closest('label').hidden=bereich!=='flotte';
 const suche=k=>(name(k)+' '+(data.profiles[k]?.rolle||'')+' '+(data.texte[k]?.rolle||'')+' '+(TRUPPE[k]?.rolle||'')+' '+Object.keys(data.profiles[k]?.schadensfaktoren||{}).map(name).join(' ')).toLocaleLowerCase('de');
 const entries=LISTE[bereich]().filter(k=>!query||suche(k).includes(query));
 let k=chosen[bereich];if(!entries.includes(k))k=chosen[bereich]=entries[0];
 el('unit').innerHTML=entries.map(x=>`<option value="${x}" ${k===x?'selected':''}>${esc(name(x))}</option>`).join('');el('result-count').textContent=entries.length+' Einträge · '+BEREICHE[bereich]+' · '+f.label;
 el('gallery').innerHTML=entries.map(x=>`<button class="codex-tile" type="button" data-unit="${x}" aria-pressed="${k===x}">${picture(x)}<span>${(typeof silbenTrennen==='function'?silbenTrennen(esc(name(x))).replace(/\u00ad/g,'&shy;'):esc(name(x)))}</span></button>`).join('');
 for(const b of el('bereiche').querySelectorAll('[data-bereich]'))b.setAttribute('aria-pressed',b.dataset.bereich===bereich);
 info();
 if(!k){el('detail').innerHTML='<div class="codex-copy">Keine passenden Einträge.</div>';return;}
 const inhalt={flotte:()=>flotte(k,faction),boden:()=>boden(k),raketen:()=>rakete(k),mond:()=>mond(k),wunder:()=>wunder(k,faction)}[bereich]();
 el('detail').innerHTML=picture(k,true)+`<div class="codex-copy">${inhalt}${archiv(k,faction)}</div>`;}
const baum=()=>{const k=chosen.flotte;if(bereich==='flotte'&&k)window.PublicTechnologyTree?.select((['verteidigung','schild'].includes(data.units[k].klasse)?'defenses:':'ships:')+k);};
const zeige=()=>el('detail').scrollIntoView({block:'start',behavior:matchMedia('(prefers-reduced-motion:reduce)').matches?'instant':'smooth'});
function waehle(k){const b=wo(k);if(!b)return;if(b!==bereich){bereich=b;el('search').value='';}if(b==='flotte'&&el('class').value&&data.units[k].klasse!==el('class').value)el('class').value='';chosen[b]=k;render();baum();zeige();}
el('gallery').addEventListener('click',e=>{const b=e.target.closest('[data-unit]');if(!b)return;chosen[bereich]=b.dataset.unit;render();baum();zeige();});
el('detail').addEventListener('click',e=>{const b=e.target.closest('[data-archiv-unit]');if(b)waehle(b.dataset.archivUnit);});
el('bereich-info').addEventListener('click',e=>{const b=e.target.closest('[data-matrix]');if(b)waehle(b.dataset.matrix);});
el('bereiche').addEventListener('click',e=>{const b=e.target.closest('[data-bereich]');if(!b)return;bereich=b.dataset.bereich;el('search').value='';render();});
el('unit').onchange=()=>{chosen[bereich]=el('unit').value;render();baum();};for(const id of ['faction','class','search'])el(id).addEventListener(id==='search'?'input':'change',render);render();
"""


def build():
    data = json.loads((ROOT / "docs/flottenkodex.json").read_text(encoding="utf-8"))
    public_rules = json.loads((ROOT / "docs/technologie-regeln.json").read_text(encoding="utf-8"))
    texte = json.loads((ROOT / "content/beschreibungen.json").read_text(encoding="utf-8"))["eintraege"]
    data["boden"] = public_rules["bodentruppen"]
    data["invasion"] = public_rules.get("invasion_werte") or {}
    data["raketen"] = public_rules["raketen"]
    data["mond"] = public_rules.get("monde") or {}
    data["gueter"] = public_rules["gueter"]
    data["wunder"] = wunder(public_rules)
    schluessel = (list(data["units"]) + [t["id"] for t in data["boden"]["typen"]] + ["miliz", "garnison"]
                  + [r["key"] for r in data["raketen"]] + list((data["mond"] or {}).get("gebaeude", {})) + list(data["wunder"]))
    fehlend = [k for k in schluessel if k not in texte]
    if fehlend:
        print("Hinweis: ohne Beschreibung:", fehlend)
    # Lore-Archiv (docs/LORE-BIBEL.md §8) aus dem Inhaltskatalog: Dossier, Chronik, vier Volksakten, Verweise je Eintrag.
    data["archiv"] = {k: {"lore": (texte[k].get("lore") or {}).get("allgemein", ""), **texte[k]["archiv"]}
                      for k in schluessel if isinstance(texte.get(k, {}).get("archiv"), dict)}
    data["texte"] = {k: {f: texte[k].get(f, "") for f in ("name", "rolle", "zweck", "staerke", "nische")}
                     for k in schluessel if k in texte}
    data["archivnamen"] = {v: texte[v]["name"] for a in data["archiv"].values() for v in a.get("verweise", []) if v in texte}
    n = {"units": len(data["units"]), "boden": len(data["boden"]["typen"]), "raketen": len(data["raketen"]),
         "mond": len(data["mond"].get("gebaeude", {}))}
    intro = (f"{n['units']} Schiffe, Verteidigungen, Schilde und Satelliten, {n['boden']} Bodentruppen mit Kontertabelle, "
             f"{n['raketen']} Raketen, {n['mond']} Mondanlagen" + (" und die Dysonsphäre als Allianzwunder" if data["wunder"] else "")
             + ". Vergleiche Aufgaben, Werte und den Forschungsweg zu deinem nächsten Ziel. Zu jedem Eintrag führt das Archiv "
             "das technische Dossier, die Chronik und die Akten aller vier Völker.")
    doc = ('<!doctype html><html lang="de"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
           '<title>Sternenepoche · Flottenkodex</title><meta name="description" content="Alle vier Völker: Schiffe, Verteidigung, '
           'Bodentruppen mit Kontertabelle, Raketen, Mondanlagen und Allianzwunder – Werte, Konter, Voraussetzungen und Archiv.">'
           '<!-- SITE_THEME --><style>' + CSS + '</style><script src="web-client/art.js"></script><script src="web-client/technology-rules.js"></script>'
           '<link rel="stylesheet" href="web-client/public-techtree.css"></head><body><main class="codex-wrap"><header class="codex-head">'
           '<a href="index.html">⌂ Sternenepoche</a><a href="Sternenepoche-Start.html">Spielguide</a><a href="web-client/">Zum Spiel</a></header>'
           f'<div class="codex-intro"><h1>Deine Flotte. Dein Plan.</h1><p>{intro}</p><p class="codex-preview-note">Mondgeneration, '
           'Invasion und Allianzwunder gelten in neuen Welten. Laufende Welten behalten ihre Regeln; in Onlinewelten bestimmt der '
           'jeweilige Server die verfügbaren Inhalte.</p></div><nav aria-label="Bereiche"><ul id="bereiche" class="codex-bereiche"></ul></nav>'
           '<div class="codex-filter"><label>Volk<select id="faction"></select></label><label>Klasse<select id="class"><option value="">Alle Klassen</option></select></label>'
           '<label>Direktauswahl<select id="unit"></select></label><label>Suchen<input id="search" type="search" placeholder="Name, Aufgabe oder Konter"></label></div>'
           '<div class="codex-layout"><section id="detail" class="codex-detail" aria-live="polite"></section><section><p id="result-count"></p>'
           '<div id="gallery" class="codex-gallery" aria-label="Einträge auswählen"></div></section></div>'
           '<section id="bereich-info" class="codex-info" hidden></section></main>'
           '<script id="fleet-data" type="application/json">' + json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
           + "</script><script>" + JS + "</script></body></html>")
    tree = ('<section class="public-tech-section" aria-labelledby="tree-title"><h2 id="tree-title">Dein Weg zum nächsten Ziel</h2><p>Wähle eine Forschung, '
            'ein Schiff, eine Verteidigung, ein Gebäude oder eine Zivilisationsstufe. Die Pfade zeigen die tatsächlichen Voraussetzungen von links nach '
            'rechts; die Zahl am Pfad ist die benötigte Stufe. Auf dem Handy kannst du den Baum innerhalb dieses Fensters verschieben. Öffne „Alle '
            'Bedingungen“ für Kosten und weitere Anforderungen.</p><label>Ziel planen<select id="tree-goal"></select></label><div id="public-tree" '
            'class="public-tree" tabindex="0" aria-label="Voraussetzungsbaum, horizontal und vertikal scrollbar"></div></section>')
    doc = doc.replace("</main>", tree + "</main>", 1).replace("</body>", '<script id="public-rules" type="application/json">'
          + json.dumps(public_rules, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
          + '</script><script src="web-client/public-techtree.js"></script></body>', 1)
    for datei in ["art.js", "technology-rules.js", "public-techtree.js", "public-techtree.css"]:
        version = hashlib.sha256((ROOT / "web-client" / datei).read_bytes()).hexdigest()[:12]
        doc = doc.replace("web-client/" + datei + '"', "web-client/" + datei + "?v=" + version + '"')
    target = ROOT / "Sternenepoche-Kodex.html"
    target.write_text(decorate(finish(doc), "KI-Hinweis.html"), encoding="utf-8")
    print("Erstellt: " + str(target))


if __name__ == "__main__":
    build()
