'use strict';
(() => {
const catalog=JSON.parse(document.getElementById('public-rules').textContent),root=document.getElementById('public-tree'),select=document.getElementById('tree-goal');
const escape=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const title=(kind,key)=>kind==='civilization'?catalog.stufen[Number(key)-2]?.name||'Zivilisation '+key:window.STERNEN_ART?.names?.[key]||key.replaceAll('_',' ');
const typeLabel={research:'Forschung',buildings:'Gebäude',ships:'Schiff',defenses:'Verteidigung',civilization:'Zivilisation'};
select.innerHTML=[['Zivilisationsstufen','civilization',Object.fromEntries(catalog.stufen.map((_,i)=>[String(i+2),{}]))],['Forschungen','research',catalog.forschung],['Schiffe','ships',Object.fromEntries(Object.entries(catalog.einheiten).filter(([,e])=>e.schiff))],['Verteidigung','defenses',Object.fromEntries(Object.entries(catalog.einheiten).filter(([,e])=>!e.schiff))],['Gebäude','buildings',catalog.gebaeude]].map(([label,kind,entries])=>`<optgroup label="${label}">${Object.keys(entries).map(k=>`<option value="${kind}:${k}">${escape(title(kind,k))}</option>`).join('')}</optgroup>`).join('');
function render(){
 const[kind,key]=select.value.split(':'),g=TechnologyRules.graph(catalog,{kind,key,level:kind==='civilization'?Number(key):1});
 if(g.cycles.length){root.textContent='Dieser Regelpfad muss geprüft werden: '+g.cycles.map(k=>title(...k.split(':'))).join(', ');return;}
 const columns=new Map();for(const n of g.nodes){const row=columns.get(n.depth)||[];row.push(n);columns.set(n.depth,row);}
 const width=(columns.size)*274+24,height=Math.max(...[...columns.values()].map(c=>c.length))*184+30;
 const positions=new Map();for(const[depth,nodes]of columns){nodes.sort((a,b)=>a.kind.localeCompare(b.kind)||a.key.localeCompare(b.key));nodes.forEach((n,i)=>positions.set(n.id,{x:12+depth*274,y:15+i*184+(height-nodes.length*184)/2}));}
 const paths=g.edges.map(e=>{const a=positions.get(e.from),b=positions.get(e.to),x=a.x+230,y=a.y+54,bx=b.x,by=b.y+54;return `<path d="M ${x} ${y} C ${x+18} ${y}, ${bx-18} ${by}, ${bx} ${by}"/><text x="${(x+bx)/2}" y="${(y+by)/2-5}">${e.level}</text>`;}).join('');
 root.innerHTML=`<div class="public-tree-canvas" style="width:${width}px;height:${height}px"><svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" aria-hidden="true">${paths}</svg>${g.nodes.map(n=>{const p=positions.get(n.id),r=n.rule||{},cost=r.kosten||{},cat=n.kind==='civilization'?'portraits':n.kind,images=window.STERNEN_ART?.images||{},f=document.getElementById('faction').value,image=n.kind==='civilization'?images[`portraits|${f}|${f}`]:images[`${cat}|${n.key}|${f}`]||images[`${cat}|${n.key}|`];let facts=[];
  if(n.kind==='civilization'){facts.push(`${Number(r.einwohner||0).toLocaleString('de-DE')} Einwohner`);if(r.kolonien)facts.push(r.kolonien+' Kolonien');if(r.versorgung_plus)facts.push('Positive Grundversorgung');if(r.stabilitaet)facts.push('Stabilität ≥ '+r.stabilitaet+' %');if(r.konsum_deckung)facts.push('Konsumdeckung ≥ '+Math.round(r.konsum_deckung*100)+' %');facts.push(catalog.stufen_haltezeit_stunden+' Spielstunden Bedingungen halten');}
  if(n.kind==='research')facts.push(r.wirkung,Number(r.fp*Math.pow(r.faktor,n.level-1)).toLocaleString('de-DE',{maximumFractionDigits:0})+' Forschungspunkte für Stufe '+n.level);
  if(n.kind==='buildings'){facts.push(r.wirkung);if(r.energie)facts.push('Grundstrombedarf '+Number(r.energie).toLocaleString('de-DE')+'; wächst mit dem Ausbau');}
  if(n.kind==='buildings'&&n.scope==='realm')facts.push('Auf mindestens einem eigenen Planeten');
  if(n.kind==='buildings'&&n.scope==='home')facts.push('Auf deiner Heimatwelt erforderlich');
  if(n.kind==='ships')facts.push('Freie Besatzung, Ressourcen und Fertigungsplatz erforderlich');
  if(n.kind==='buildings')facts.push('Freie Baufelder und Rohstoffe erforderlich');
  const factor=['research','buildings'].includes(n.kind)?Math.pow(r.faktor||1,n.level-1):1;
  return `<article class="public-tree-node ${n.id===g.targetId?'tree-goal-node':''}" data-kind="${n.kind}" style="left:${p.x}px;top:${p.y}px"><button type="button" data-tree-choice="${n.kind}:${n.key}">${image?`<img src="web-client/${escape(image.url)}" alt="">`:'<span class="tree-placeholder" aria-hidden="true">✧</span>'}<span><small>${typeLabel[n.kind]}</small><strong>${escape(title(n.kind,n.key))}</strong><b>${n.kind==='civilization'?'Stufe '+n.key:['ships','defenses'].includes(n.kind)?'Bauziel':'Benötigt: Stufe '+n.level}${n.levels.length>1?' · Etappen '+n.levels.join(' → '):''}</b></span></button><details><summary>Alle Bedingungen</summary><ul>${facts.filter(Boolean).map(f=>`<li>${escape(f)}</li>`).join('')}${Object.entries(cost).map(([k,v])=>`<li>${escape(title('resource',k))}: ${Math.floor(Number(v)*factor).toLocaleString('de-DE')}${['research','buildings'].includes(n.kind)?' für Stufe '+n.level+' (einzelner Ausbau)':''}</li>`).join('')}</ul></details></article>`;
 }).join('')}</div>`;
 root.dataset.nodeCount=g.nodes.length;root.dataset.pathCount=g.edges.length;
 requestAnimationFrame(()=>{const p=positions.get(g.targetId);root.scrollLeft=Math.max(0,p.x-root.clientWidth+245);root.scrollTop=Math.max(0,p.y-root.clientHeight/2);});
}
select.addEventListener('change',render);document.getElementById('faction').addEventListener('change',render);
root.addEventListener('click',e=>{const b=e.target.closest('[data-tree-choice]');if(!b)return;const value=b.dataset.treeChoice;if([...select.options].some(o=>o.value===value)){select.value=value;render();}});
window.PublicTechnologyTree={render,select(value){if([...select.options].some(o=>o.value===value)){select.value=value;render();}}};select.value='ships:schwerer_jaeger';render();
})();
