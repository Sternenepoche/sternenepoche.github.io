// Verify timing semantics and production HTML, without a model call or live-world mutation.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const code=fs.readFileSync('web-client/game-ui.js','utf8');
const ctx=vm.createContext({performance:{now:()=>0}});
vm.runInContext(code.split('// DOM bindings below.')[0]+';globalThis.clock=GameTime;',ctx);
const c=ctx.clock;
c.sync({sekunden:100,tempo:3,paused:false},1000);assert.equal(c.now(2000),103);
assert.equal(c.now(61000),145,'a stale snapshot must not continue indefinitely');
c.sync({sekunden:150,tempo:3,paused:true},1000);assert.equal(c.now(9000),150);
c.sync({sekunden:100,tempo:2,paused:false},1000);c.disconnect(3000);assert.equal(c.now(9000),104);
let j={fertig_sekunden:200,dauer_sekunden:100};assert.equal(c.job(j,150,100).progress,.5);
assert.equal(c.job({...j,pausiert:true},180,150).progress,.5,'unrest must freeze the build');
assert.equal(c.job({...j,wartet:true},180,150).progress,0);
assert.equal(c.job(j,220,100).remaining,0);
const research={fertig_sekunden:7200,punkte_gesamt:100,punkte_rest:100,rate:50,naechster_tick_sekunden:3600,begonnen_sekunden:1800};
assert.equal(c.job(research,1800,1800).progress,0,'research started mid-hour must begin at zero');
assert.equal(c.job(research,2700,1800).progress,.25);
assert.equal(c.duration(3601),'01:00:01');
let html=fs.readFileSync('web-client/index.html','utf8');let ids=[...html.matchAll(/\bid="([^"]+)"/g)].map(m=>m[1]);
assert.equal(ids.length,new Set(ids).size,'duplicate IDs make live forms ambiguous');
for(const tab of [...html.matchAll(/data-tab="([^"]+)"/g)])assert(ids.includes(tab[1]),'navigation target missing: '+tab[1]);
assert.equal([...html.matchAll(/data-tab="/g)].length,17);
assert(html.includes('minlength="12"'),'registration password policy regressed');
const art=vm.createContext({window:{}});vm.runInContext(fs.readFileSync('web-client/art.js','utf8'),art);
assert.equal(Object.keys(art.window.STERNEN_ART.images).length,402);
for(const image of Object.values(art.window.STERNEN_ART.images))assert(fs.existsSync('web-client/'+image.url.split('?')[0]),image.url);
assert(fs.readFileSync('web-client/game.css','utf8').includes('prefers-reduced-motion'));
const atlas=vm.createContext({});vm.runInContext(fs.readFileSync('web-client/galaxy.js','utf8').split('// Browser scene.')[0]+';globalThis.model=GalaxyModel;',atlas);
const m=atlas.model,positions=new Set();
for(let sector=1;sector<=2;sector++)for(let system=1;system<=60;system++){const p=m.position(sector,system,60);assert(Number.isFinite(p.x+p.y+p.z));positions.add(JSON.stringify(p));}
assert.equal(positions.size,120,'each system must have its own public display position');
for(let n=1;n<=12;n++){const a=m.orbit(n,0),b=m.orbit(n,30);assert(Math.abs(Math.hypot(b.x,b.z)-a.radius)<1e-8);assert.notDeepEqual(a,b,'planet does not move along its orbit');}
const redacted=m.planets({system:2,plaetze:[{position:1,bekannt:false,zone:'leben',spieler:'SECRET',bestand:{erz:999}}]},1,12);
assert.equal(redacted.length,12);assert(!JSON.stringify(redacted).includes('SECRET'));assert(!JSON.stringify(redacted).includes('erz'));assert(!redacted[0].zone);
const known=m.planets({system:2,plaetze:[{position:1,bekannt:true,zone:'frost',status:'frei'}]},1,12);assert.equal(known[0].zone,'frost');assert.equal(known[0].koord,'1:2:1');
const owners=new Map();for(const file of ['app.js','presentation.js','game-ui.js'])for(const match of fs.readFileSync('web-client/'+file,'utf8').matchAll(/^function (\w+)\(/gm)){assert(!owners.has(match[1]),'ambiguous renderer owner '+match[1]);owners.set(match[1],file);}
for(const file of ['three.min.js','galaxy.js','galaxy.css'])assert(html.includes(file));
console.log('PASS: authoritative timer interpolation, pause, disconnect, stale cap, unrest, queue, 17 navigation targets, unique HTML IDs, 402 packaged assets and reduced motion.');
console.log('PASS: 120 distinct 3D system positions, 12 moving circular orbits, unknown-planet redaction and single renderer ownership.');
