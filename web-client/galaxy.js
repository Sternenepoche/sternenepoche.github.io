'use strict';
// Coordinates are public. Display layout and orbit motion never reveal world state
// and never enter the server's distance or flight calculations.
const GalaxyModel={
 position(sector,system,count){
  const angle=system*2.3999632297+sector*1.3,r=23+112*Math.sqrt(system/count);
  return {x:Math.cos(angle)*r,y:Math.sin(system*7.19+sector)*13,z:Math.sin(angle)*r};
 },
 orbit(position,seconds){
  const radius=23+position*10.5,angle=position*2.3999632297+seconds*.095/Math.sqrt(position);
  return {x:radius*Math.cos(angle),y:0,z:radius*Math.sin(angle),radius};
 },
 planets(system,sector,count){
  return Array.from({length:count},(_,i)=>{
   const position=i+1,raw=(system.plaetze||system.planeten||[]).find(p=>Number(p.position||p.koord?.split(':')[2])===position);
   const coord=`${sector}:${system.system}:${position}`;
   if(!raw?.bekannt)return {koord:coord,position,bekannt:false,status:'unbekannt'};
   return {...raw,koord:coord,position};
  });
 }
};
// Browser scene. Loaded once; renderer and resources are disposed on world change.
window.GalaxyUI=(()=>{
 let host,canvas,renderer,scene,camera,group,raycaster,glow,observer,frameId=0,active=false,dirty=true;
 let stage='galaxy',sector=1,system=1,selected=0,clock=0,last=0,stamp='',epoch=0,paused=false,motionOverride=false;
 let cache=new Map(),targets=[],labels=[],planets=[],orbits=[],yaw=.15,pitch=.72,distance=600,drag=null;
 const $g=id=>document.getElementById(id),reduced=matchMedia('(prefers-reduced-motion: reduce)');
 const palette={sun:['#ffb52e','#ffe09a','#fff8dd'],unknown:['#69778b','#8192a8','#a4b4c7'],leben:['#1b4e80','#76a56b','#cad7b4'],glut:['#422d30','#bd613d','#eda758'],frost:['#386081','#b3dce3','#f4f9ff'],nebel:['#443775','#9478ae','#d1b4df']};
 const own=c=>view?.planeten?.some(p=>p.koord===c),profile=()=>rules.catalog.welt;
 function disposeTree(tree){const textures=new Set(),materials=new Set(),geometries=new Set();tree.traverse(o=>{if(o.geometry)geometries.add(o.geometry);for(const m of [].concat(o.material||[])){materials.add(m);if(m.map)textures.add(m.map);}});geometries.forEach(x=>x.dispose());materials.forEach(x=>x.dispose());textures.forEach(x=>{if(x!==glow)x.dispose();});}
 function texture(){const c=document.createElement('canvas');c.width=c.height=128;const x=c.getContext('2d'),g=x.createRadialGradient(64,64,1,64,64,64);g.addColorStop(0,'#fff');g.addColorStop(.12,'rgba(255,255,255,.85)');g.addColorStop(.4,'rgba(255,255,255,.14)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(0,0,128,128);return new THREE.CanvasTexture(c);}
 function sprite(color,size,opacity=1){const m=new THREE.Sprite(new THREE.SpriteMaterial({map:glow,color,transparent:true,opacity,depthWrite:false,blending:THREE.AdditiveBlending}));m.scale.setScalar(size);return m;}
 function stars(){
  const pos=[],colors=[];let seed=41;const random=()=>{seed=(seed*1664525+1013904223)>>>0;return seed/4294967296;};
  for(let i=0;i<2400;i++){const a=random()*Math.PI*2,r=700+random()*750,y=(random()-.5)*1400;pos.push(Math.cos(a)*r,y,Math.sin(a)*r);const c=new THREE.Color(i%3?'#94b9dd':'#ffd2a0');colors.push(c.r,c.g,c.b);}
  const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(pos,3));g.setAttribute('color',new THREE.Float32BufferAttribute(colors,3));scene.add(new THREE.Points(g,new THREE.PointsMaterial({size:1.4,vertexColors:true,transparent:true,opacity:.6})));
 }
 function cloud(cx,cz,scale,color,seed){
  const pos=[];for(let i=0;i<3200;i++){const a=i*2.3999632297+seed,r=Math.sqrt((i+.5)/3200)*145*scale,arm=Math.sin(a*3+r*.026);pos.push(cx+Math.cos(a+r*.005)*r,Math.sin(i*17.3)*18*scale,cz+Math.sin(a+r*.005)*r+arm*6*scale);}
  const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(pos,3));group.add(new THREE.Points(g,new THREE.PointsMaterial({size:2.6*scale,color,transparent:true,opacity:.62,depthWrite:false,blending:THREE.AdditiveBlending})));
  const haze=sprite(color,350*scale,.5);haze.position.set(cx,-20,cz);group.add(haze);const core=sprite('#c4b8ee',80*scale,.65);core.position.set(cx,0,cz);group.add(core);
 }
 function planetTexture(zone,seed){
  const c=document.createElement('canvas');c.width=512;c.height=256;const x=c.getContext('2d'),image=x.createImageData(c.width,c.height),colors=(palette[zone]||palette.frost).map(v=>new THREE.Color(v).convertLinearToSRGB());
  for(let y=0;y<c.height;y++)for(let px=0;px<c.width;px++){
   const lon=px/c.width*Math.PI*2,lat=(y/c.height-.5)*Math.PI,a=Math.cos(lon)*Math.cos(lat),b=Math.sin(lon)*Math.cos(lat),d=Math.sin(lat);
   const n=Math.sin(a*7+seed)*Math.cos(b*8-seed)+Math.sin(b*17+d*13)*.3+Math.cos(a*28-d*23)*.13;
   let color=n>.2?colors[1]:colors[0];if(zone==='frost'&&Math.abs(d)>.65||zone==='leben'&&Math.abs(d)>.87)color=colors[2];
   const detail=.88+.12*Math.sin(n*20+seed),i=(y*c.width+px)*4;image.data[i]=Math.round(color.r*255*detail);image.data[i+1]=Math.round(color.g*255*detail);image.data[i+2]=Math.round(color.b*255*detail);image.data[i+3]=255;
  }
  x.putImageData(image,0,0);const map=new THREE.CanvasTexture(c);map.colorSpace=THREE.SRGBColorSpace;return map;
 }
 function ring(radius,color,opacity=.25){const points=[];for(let i=0;i<=180;i++){const a=i/180*Math.PI*2;points.push(new THREE.Vector3(Math.cos(a)*radius,0,Math.sin(a)*radius));}const m=new THREE.LineLoop(new THREE.BufferGeometry().setFromPoints(points),new THREE.LineBasicMaterial({color,transparent:true,opacity}));group.add(m);return m;}
 function label(text,obj,key,kind){const b=document.createElement('button');b.type='button';b.className='space-label '+kind;b.textContent=text;b.setAttribute('aria-label',kind==='planet'?'Planet '+key:kind==='sector'?'Sektor '+key+' öffnen':'System '+key+' öffnen');b.onclick=()=>pick(kind,key);$g('galaxy-labels').append(b);labels.push({el:b,obj,kind,key});}
 function resetScene(){if(group){scene.remove(group);disposeTree(group);}group=new THREE.Group();scene.add(group);targets=[];labels=[];planets=[];orbits=[];$g('galaxy-labels').replaceChildren();dirty=true;}
 function cameraUpdate(){camera.position.set(Math.sin(yaw)*Math.cos(pitch)*distance,Math.sin(pitch)*distance,Math.cos(yaw)*Math.cos(pitch)*distance);camera.lookAt(0,0,0);dirty=true;}
 function resize(){if(!renderer)return;const rect=canvas.getBoundingClientRect();if(!rect.width||!rect.height)return;renderer.setSize(rect.width,rect.height,false);camera.aspect=rect.width/rect.height;camera.updateProjectionMatrix();dirty=true;}
 function init(){
  host=$g('galaxy-space');canvas=$g('galaxy-canvas');if(!host||renderer)return;
  $g('galaxy-back').onclick=()=>stage==='system'?showSector(sector):showGalaxy();$g('galaxy-home').onclick=showGalaxy;
  $g('galaxy-reset-camera').onclick=()=>{if(!renderer)return;yaw=.15;pitch=.8;distance=stage==='system'?330:stage==='sector'?350:580;cameraUpdate();};
  $g('galaxy-motion').onclick=()=>{if(reduced.matches&&!motionOverride){motionOverride=true;paused=false;}else paused=!paused;motionState();dirty=true;};
  $g('galaxy-reload').onclick=()=>{cache.clear();if(stage==='system')showSystem(sector,system);else if(stage==='sector')showSector(sector);else showGalaxy();};
  try{renderer=new THREE.WebGLRenderer({canvas,antialias:true,alpha:false,powerPreference:'low-power'});}catch(e){$g('galaxy-fallback').hidden=false;$g('galaxy-fallback').textContent='3D ist in diesem Browser nicht verfügbar. Die Systemliste und Aufklärungsberichte bleiben nutzbar.';return;}
  renderer.setPixelRatio(Math.min(devicePixelRatio,1.75));renderer.setClearColor(0x030813);renderer.outputColorSpace=THREE.SRGBColorSpace;renderer.toneMapping=THREE.ACESFilmicToneMapping;renderer.toneMappingExposure=1.15;
  scene=new THREE.Scene();camera=new THREE.PerspectiveCamera(48,1,.1,4000);raycaster=new THREE.Raycaster();glow=texture();stars();scene.add(new THREE.AmbientLight(0x809ec0,.75));observer=new ResizeObserver(resize);observer.observe(host);
  canvas.addEventListener('pointerdown',e=>{if(e.button!==0)return;drag={x:e.clientX,y:e.clientY,startX:e.clientX,startY:e.clientY};canvas.setPointerCapture(e.pointerId);});
  canvas.addEventListener('pointermove',e=>{if(drag){yaw-=(e.clientX-drag.x)*.005;pitch=Math.max(.15,Math.min(1.45,pitch+(e.clientY-drag.y)*.004));drag.x=e.clientX;drag.y=e.clientY;cameraUpdate();}});
  canvas.addEventListener('pointerup',e=>{if(!drag)return;const moved=Math.hypot(e.clientX-drag.startX,e.clientY-drag.startY);drag=null;if(moved<5){const r=canvas.getBoundingClientRect();raycaster.setFromCamera(new THREE.Vector2((e.clientX-r.left)/r.width*2-1,-(e.clientY-r.top)/r.height*2+1),camera);const hit=raycaster.intersectObjects(targets,false)[0];if(hit)pick(hit.object.userData.kind,hit.object.userData.key);}});
  canvas.addEventListener('pointercancel',()=>drag=null);
  canvas.addEventListener('wheel',e=>{e.preventDefault();distance=Math.max(stage==='system'?160:230,Math.min(1100,distance*Math.exp(e.deltaY*.001)));cameraUpdate();},{passive:false});
  canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();paused=true;$g('galaxy-fallback').hidden=false;$g('galaxy-fallback').textContent='3D-Grafik unterbrochen. Systemliste weiter nutzbar; Seite neu laden, um die Grafik wiederherzustellen.';});
  reduced.addEventListener('change',()=>{motionState();dirty=true;});resize();
 }
 function motionState(){if($g('galaxy-motion')){$g('galaxy-motion').textContent=reduced.matches&&!motionOverride?'Umläufe aktivieren':paused?'Umläufe fortsetzen':'Umläufe pausieren';$g('galaxy-motion').title=reduced.matches&&!motionOverride?'Deine Systemeinstellung reduziert Bewegung. Du kannst Umläufe hier ausdrücklich aktivieren.':'';}}
 function header(title,sub){$g('galaxy-title').textContent=title;$g('galaxy-subtitle').textContent=sub;$g('galaxy-back').hidden=stage==='galaxy';$g('galaxy-motion').hidden=stage!=='system';$g('galaxy-detail').hidden=stage!=='system';motionState();}
 function index(items){$g('galaxy-index').innerHTML=items.map(i=>`<button type="button" data-space-kind="${i.kind}" data-space-key="${i.key}" class="${i.own?'space-own':''}" aria-label="${esc(i.text)} öffnen">${esc(i.text)}${i.known?' <small>kartiert</small>':''}</button>`).join('');}
 function showGalaxy(){
  stage='galaxy';selected=0;epoch++;if(renderer)resetScene();header('Die Galaxie','Sektor wählen · Sterne öffnen die Sonnensysteme');
  const w=profile(),items=[];
  for(let s=1;s<=w.sektoren;s++){
   const a=(s-1)/w.sektoren*Math.PI*2,cx=Math.cos(a)*145,cz=Math.sin(a)*145;
   if(renderer){cloud(cx,cz,.85,s%2?'#5f74bf':'#894f9b',s);const marker=new THREE.Mesh(new THREE.SphereGeometry(20,12,10),new THREE.MeshBasicMaterial({visible:false}));marker.position.set(cx,0,cz);marker.userData={kind:'sector',key:s};group.add(marker);targets.push(marker);label('SEKTOR '+s,marker,s,'sector');
    for(let n=1;n<=w.systeme_je_sektor;n++){const p=GalaxyModel.position(s,n,w.systeme_je_sektor),star=sprite('#dcecff',4.5,.8);star.position.set(cx+p.x*.85,p.y,cz+p.z*.85);group.add(star);}
   }
   items.push({kind:'sector',key:s,text:'Sektor '+s,own:view.planeten.some(p=>Number(p.koord.split(':')[0])===s)});
  }
  index(items);$g('galaxy-table').innerHTML='';if(renderer){distance=580;cameraUpdate();}
 }
 async function fetchSector(s,requestEpoch){
  if(cache.has(s))return cache.get(s);const w=profile(),batches=[];
  for(let from=1;from<=w.systeme_je_sektor;from+=20)batches.push(api('/api/tool',{typ:'galaxie',sektor:s,von:from,bis:Math.min(from+19,w.systeme_je_sektor)}));
  const responses=await Promise.all(batches),data=responses.flatMap(v=>v.systeme||[]).map(v=>({...v,sektor:s}));
  if(requestEpoch!==epoch)return null;cache.set(s,data);return data;
 }
 async function showSector(s){
  sector=Number(s);stage='sector';const request=++epoch;header('Sektor '+sector,'Sonnensystem wählen · Unbekannte Welten bleiben verdeckt');$g('galaxy-loading').hidden=false;
  try{const systems=await fetchSector(sector,request);if(!systems||request!==epoch)return;if(renderer)resetScene();
   index(systems.map(s=>({kind:'system',key:s.system,text:'System '+sector+':'+s.system,known:s.bekannt,own:(s.plaetze||[]).some(p=>own(p.koord))})));
   if(renderer){cloud(0,0,1,'#6d62ae',sector);for(const s of systems){const p=GalaxyModel.position(sector,s.system,profile().systeme_je_sektor),isOwn=(s.plaetze||[]).some(p=>own(p.koord)),color=isOwn?'#f2c482':s.bekannt?'#8de0cf':'#b4c6eb';
    const star=new THREE.Mesh(new THREE.SphereGeometry(isOwn?2.9:2,12,10),new THREE.MeshBasicMaterial({color}));star.position.set(p.x,p.y,p.z);star.userData={kind:'system',key:s.system};group.add(star);targets.push(star);const halo=sprite(color,isOwn?18:12,.75);halo.position.copy(star.position);group.add(halo);label(String(s.system),star,s.system,'system');}
    distance=350;cameraUpdate();
   }
   $g('galaxy-table').innerHTML='<p class="muted">Wähle ein System, um seine Umlaufbahnen und die Aufklärungsberichte zu sehen.</p>';
  }catch(e){if(request===epoch)show(e.message,true);}finally{if(request===epoch)$g('galaxy-loading').hidden=true;}
 }
 async function showSystem(s,n){
  sector=Number(s);system=Number(n);stage='system';const request=++epoch;header('System '+sector+':'+system,'Sonne, Umlaufbahnen und deine Sondenberichte');$g('galaxy-loading').hidden=false;
  try{const systems=await fetchSector(sector,request);if(!systems||request!==epoch)return;const data=systems.find(s=>s.system===system);if(!data)throw Error('System nicht vorhanden');const ps=GalaxyModel.planets(data,sector,profile().plaetze_je_system);if(renderer)resetScene();
   index(ps.map(p=>({kind:'planet',key:p.position,text:p.koord,known:p.bekannt,own:own(p.koord)})));
   if(renderer){const sun=new THREE.Mesh(new THREE.SphereGeometry(14,48,32),new THREE.MeshBasicMaterial({map:planetTexture('sun',system*.6),toneMapped:false}));group.add(sun);group.add(sprite('#ffa34b',115,.9));group.add(sprite('#fff5c0',58,.85));const light=new THREE.PointLight(0xffe2ae,3.5,0,0);light.position.set(0,0,0);group.add(light);
    for(const p of ps){const isOwn=own(p.koord),radius=p.bekannt?4.3+p.position%3*.5:3.4,zone=p.bekannt?p.zone:null;
     const mesh=new THREE.Mesh(new THREE.SphereGeometry(radius,32,24),new THREE.MeshPhongMaterial({map:planetTexture(zone||'unknown',p.position+system),color:p.besiegt?'#8e9096':'#ffffff',shininess:zone==='leben'?28:8}));mesh.userData={kind:'planet',key:p.position,data:p};group.add(mesh);targets.push(mesh);planets.push(mesh);orbits.push(ring(GalaxyModel.orbit(p.position,0).radius,isOwn?'#d6b277':p.bekannt?'#769ca4':'#657b98',isOwn?.7:.4));label(String(p.position),mesh,p.position,'planet');
     if(zone==='leben'){const atmosphere=new THREE.Mesh(new THREE.SphereGeometry(radius*1.035,24,16),new THREE.MeshBasicMaterial({color:0x6ea8c9,transparent:true,opacity:.12,side:THREE.BackSide}));mesh.add(atmosphere);}
    }
    distance=330;pitch=.8;cameraUpdate();dirty=true;
   }
   $g('galaxy-table').innerHTML=`<details ${renderer?'':'open'}><summary>Aufklärungsberichte · System ${sector}:${system}</summary>${galaxyReports([data])}</details>`;selected=(ps.find(p=>own(p.koord))||ps[0]).position;planetDetail(ps.find(p=>p.position===selected));
  }catch(e){if(request===epoch)show(e.message,true);}finally{if(request===epoch)$g('galaxy-loading').hidden=true;}
 }
 function planetDetail(p){
  if(!p)return;selected=p.position;dirty=true;$g('galaxy-detail').hidden=false;
  const known=!!p.bekannt,protectedHome=!!p.heimat,owner=known?(p.spieler??(p.status==='frei'?'Unbewohnt':'Unbekannt')):'Unbekannt';
  $g('galaxy-detail').innerHTML=`<span class="eyebrow">PLANET ${p.position}</span><h3>${esc(p.koord)}</h3>${known&&p.zone?`<img class="space-planet-art ${p.besiegt?'defeated-planet':''}" src="${esc(planetArt(p))}" alt="${esc(named(p.zone))}">`:'<div class="space-unknown" aria-label="Unbekannter Planet">?</div>'}<strong>${esc(p.besiegt?'Besiegt':named(p.status))}</strong><dl><dt>Planetentyp</dt><dd>${esc(known?named(p.zone||'unbekannt'):'Unbekannt')}</dd><dt>Bewohner</dt><dd>${esc(owner)}</dd><dt>Heimatwelt</dt><dd>${protectedHome?'Geschützt':'—'}</dd></dl><p>${known?planetResources(p):'Eine Planetensonde enthüllt Typ, Bewohner und Ressourcen.'}</p><div class="section-actions"><button type="button" data-explore="${esc(p.koord)}" data-system="${!(cache.get(sector)||[]).find(s=>s.system===system)?.bekannt}">${(cache.get(sector)||[]).find(s=>s.system===system)?.bekannt?'Sonde planen':'Systemscan planen'}</button>${known&&p.status==='frei'&&!protectedHome?`<button type="button" class="secondary" data-colonize="${esc(p.koord)}">Kolonie planen</button>`:''}${own(p.koord)?`<button type="button" class="secondary" data-space-own="${esc(p.koord)}">Planet verwalten</button>`:''}</div>`;
  labels.forEach(l=>{if(l.kind==='planet')l.el.setAttribute('aria-pressed',String(Number(l.key)===selected));});
 }
 function pick(kind,key){if(kind==='sector')void showSector(key);else if(kind==='system')void showSystem(sector,key);else if(kind==='planet'){const data=(cache.get(sector)||[]).find(s=>s.system===system);planetDetail(GalaxyModel.planets(data,sector,profile().plaetze_je_system).find(p=>p.position===Number(key)));}}
 function frame(time){
  frameId=0;if(!active||document.hidden||!renderer)return;
  const delta=last?Math.min(.1,(time-last)/1000):0;last=time;
  if(stage==='system'&&!paused&&(!reduced.matches||motionOverride)&&!world?.paused&&!world?.beendet&&GameTime.anchor.connected){clock+=delta;dirty=true;}
  if(dirty){for(const p of planets){const q=GalaxyModel.orbit(p.userData.data.position,clock);p.position.set(q.x,q.y,q.z);p.rotation.y=clock*.05;}
   renderer.render(scene,camera);const w=canvas.clientWidth,h=canvas.clientHeight,v=new THREE.Vector3();
   for(const l of labels){l.obj.getWorldPosition(v);v.project(camera);l.el.hidden=Math.abs(v.x)>1.04||Math.abs(v.y)>1.04||v.z>1;l.el.style.transform=`translate(${(v.x+1)*w/2}px,${(-v.y+1)*h/2}px) translate(-50%,-50%)`;}
   dirty=false;
  }
  frameId=requestAnimationFrame(frame);
 }
 function visible(value){
  active=value;if(!value){cancelAnimationFrame(frameId);frameId=0;last=0;return;}
  const key=world?.world_id+':'+view?.name;if(key!==stamp){stamp=key;cache.clear();epoch++;init();showGalaxy();}
  if(!host)init();resize();dirty=true;if(renderer&&!frameId)frameId=requestAnimationFrame(frame);
 }
 function update(data){const systems=Array.isArray(data)?data:data.systeme||[];const s=Number($('map-form').elements.sektor.value);cache.delete(s);if(systems.length===1){cache.set(s,systems.map(x=>({...x,sektor:s})));void showSystem(s,systems[0].system);}else void showSector(s);}
 document.addEventListener('click',e=>{const b=e.target.closest('[data-space-kind],[data-space-own]');if(!b)return;if(b.dataset.spaceOwn){selectedPlanet=b.dataset.spaceOwn;renderView();navigateGame('reich');}else pick(b.dataset.spaceKind,b.dataset.spaceKey);});
 document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(frameId);frameId=0;last=0;}else if(active&&renderer&&!frameId)frameId=requestAnimationFrame(frame);});
 return {visible,update};
})();
