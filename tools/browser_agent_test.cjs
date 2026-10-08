// Execute the actual browser agent loop with mock model/network endpoints, no billable calls.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
function element(){return {hidden:false,textContent:'',value:'',classList:{toggle(){}},addEventListener(){},setAttribute(){},querySelector(){return element();},querySelectorAll(){return []}};}
const html=fs.readFileSync('web-client/index.html','utf8');
const elements=new Map(),requests=[];
const world={world_id:'world-one',api_version:1,sekunden:0,paused:false,tempo:1,bots:30,freie_plaetze:20,rangliste:[]};
let mode='normal',releaseModel;
async function fetchMock(url,opts={}){
 const body=opts.body?JSON.parse(opts.body):null;requests.push({url,body,headers:opts.headers});
 if(url.endsWith('/api/chat')||url.includes('openrouter.ai')){
   if(mode==='invalid')return {ok:true,json:async()=>({message:{content:'not json'}})};
   if(mode==='error')return {ok:false,status:503};
   if(mode==='slow')await new Promise(resolve=>{releaseModel=resolve;});
   const response={aktionen:[],abfragen:[],begruendung:'Warten',notiz:'private Notiz',wecker_stunden:1};
   return {ok:true,json:async()=>({message:{content:JSON.stringify(response)},choices:[{message:{content:JSON.stringify(response)}}],usage:{total_tokens:12}})};
 }
 let v={};if(url.endsWith('/api/context'))v={view:{world_id:'world-one',name:'Eigenes Reich',paused:mode==='paused',beendet:mode==='ended',reich_status:{status:mode==='defeated'?'besiegt':'aktiv'}},schema:{type:'object',properties:{aktionen:{type:'array'},abfragen:{type:'array'}}}};
 if(url.endsWith('/api/lobby'))v=world;
 if(url.endsWith('/api/me'))v={name:'Konto',mode:'agent',spieler:null,world_id:world.world_id};
 if(url.endsWith('/api/command')){if(mode==='lost'){mode='normal';throw Error('lost HTTP response');}v={ergebnisse:[]};}
 return {ok:true,json:async()=>v};
}
const values=new Map();const storage={getItem(k){return values.get(k)||null},setItem(k,v){values.set(k,v)},removeItem(k){values.delete(k)}};
const ctx=vm.createContext({console,URL,AbortSignal,AbortController,performance,crypto:require('node:crypto').webcrypto,
 fetch:fetchMock,setTimeout:(fn)=>setTimeout(fn,0),clearTimeout,setInterval:()=>1,clearInterval(){},
 document:{getElementById(id){assert(html.includes('id="'+id+'"'),'Real HTML is missing '+id);if(!elements.has(id))elements.set(id,element());return elements.get(id);},addEventListener(){},querySelectorAll(){return []}},
 window:{addEventListener(){},STERNENEPOCHE:{api:''}},location:{hostname:'offline.test',origin:'https://offline.test',protocol:'https:'},localStorage:storage,sessionStorage:storage});
vm.runInContext(fs.readFileSync('web-client/presentation.js','utf8')+fs.readFileSync('web-client/app.js','utf8')+`;globalThis.harness={fuelReserve,modelCall,agentLoop,stopAgent,gameReadOnly,prepare(a){base='http://game.test';session={token:'game-session'};world=${JSON.stringify(world)};rules={text:'public rules'};agent=a;}};`,ctx);
const harness=ctx.harness;
function agent(overrides={}){return {provider:'ollama',url:'http://127.0.0.1:11434',key:'PRIVATE-KEY',model:'mock',limit:1,delay:15,roles:['verwalter'],controller:new AbortController(),world_id:world.world_id,lease:'lease-one',calls:0,errors:0,tokens:0,status:'start',latency:0,...overrides};}
(async()=>{
 assert.equal(harness.gameReadOnly({paused:false,beendet:false},{reich_status:{status:'aktiv'}}),'');
 assert.match(harness.gameReadOnly({paused:true},{}),/pausiert/);
 assert.match(harness.gameReadOnly({beendet:true},{}),/beendet/);
 assert.match(harness.gameReadOnly({}, {beendet:true}),/beendet/);
 assert.match(harness.gameReadOnly({}, {reich_status:{status:'besiegt'}}),/besiegt/);
 assert.equal(harness.fuelReserve({treibstoff_je_strecke:1,treibstoff_je_strecke_milli:1999},2),4,'save cargo must leave enough fractional fuel for both legs');
 assert.equal(harness.fuelReserve({treibstoff_je_strecke:1,treibstoff_je_strecke_milli:1500},2),3,'reserve must round the complete trip, not each leg separately');
 assert.equal(harness.fuelReserve({treibstoff_je_strecke:12},2),24,'legacy quote remains supported');
 let a=agent();await harness.modelCall(a,[{role:'user',content:'own view'}],{});assert.equal(a.calls,1);assert.equal(a.tokens,12);
 await assert.rejects(()=>harness.modelCall(a,[],{}),/Aufruflimit/);
 a=agent({provider:'openrouter'});await harness.modelCall(a,[],{});assert.equal(requests.at(-1).headers.Authorization,'Bearer PRIVATE-KEY');assert(!JSON.stringify(requests.at(-1).body).includes(a.key));
 a=agent({currentRole:'diplomat',roleSettings:{diplomat:{provider:'openrouter',model:'diplomacy-model',key:'PRIVATE-KEY'}}});await harness.modelCall(a,[],{});assert.equal(requests.at(-1).body.model,'diplomacy-model');assert(requests.at(-1).url.includes('openrouter.ai'));
 a=agent({costLimit:1,cost:1});const callsBeforeCost=requests.length;await assert.rejects(()=>harness.modelCall(a,[],{}),/Kostenlimit/);assert.equal(requests.length,callsBeforeCost,'budget limit still made a provider request');
 mode='invalid';await assert.rejects(()=>harness.modelCall(agent(),[],{}));
 mode='normal';a=agent();harness.prepare(a);await harness.agentLoop(a);assert.equal(a.calls,1);assert(requests.some(r=>r.url.endsWith('/api/command')));
 mode='lost';a=agent();harness.prepare(a);const retriesBefore=requests.filter(r=>r.url.endsWith('/api/command')).length;await harness.agentLoop(a);const replay=requests.filter(r=>r.url.endsWith('/api/command')).slice(retriesBefore);assert.equal(replay.length,2);assert.deepEqual(replay[0].body,replay[1].body,'retry did not preserve request ID');
 mode='paused';a=agent({limit:10});harness.prepare(a);const pausedLoop=harness.agentLoop(a);await new Promise(resolve=>setTimeout(resolve,10));await harness.stopAgent();await pausedLoop;assert.equal(a.calls,0,'paused world consumed model calls');
 for(const state of ['ended','defeated']){mode=state;a=agent({limit:10});harness.prepare(a);const before=requests.length;await harness.agentLoop(a);assert.equal(a.calls,0,state+' world consumed model calls');assert(!requests.slice(before).some(r=>r.url.endsWith('/api/command')),state+' agent sent a command');}
 const commandsBefore=requests.filter(r=>r.url.endsWith('/api/command')).length;
 mode='slow';a=agent({limit:5});harness.prepare(a);const running=harness.agentLoop(a);await new Promise(resolve=>setTimeout(resolve,10));assert(releaseModel);await harness.stopAgent('stopped');releaseModel();await running;
 assert.equal(requests.filter(r=>r.url.endsWith('/api/command')).length,commandsBefore,'late model response executed after stop');
 mode='error';a=agent({limit:10});harness.prepare(a);await harness.agentLoop(a);assert.equal(a.errors,3);assert.equal(a.calls,3);
 for(const r of requests.filter(r=>r.url.startsWith('http://game.test')))assert(!JSON.stringify(r).includes('PRIVATE-KEY'),'provider key sent to game server');
 vm.runInContext(fs.readFileSync('web-client/agent-team.js','utf8'),ctx);
 const team=ctx.window.AgentTeam;
 assert.equal(team.localURL('http://127.0.0.1:11434'),'http://127.0.0.1:11434');
 for(const url of ['https://foreign.example','http://user:password@localhost:11434','http://localhost:11434/api/chat'])assert.throws(()=>team.localURL(url));
 const contract={type:'object',additionalProperties:false,required:['pins'],properties:{pins:{type:'array',maxItems:1,items:{type:'object',required:['status'],properties:{status:{enum:['offen','erledigt']}}}}}};
 team.validateAnswer({pins:[{status:'offen'}]},contract);
 assert.throws(()=>team.validateAnswer({pins:[{status:'invented'}]},contract));
 assert.throws(()=>team.validateAnswer({pins:[],key:'must-not-be-accepted'},contract));
 assert.throws(()=>team.validateAnswer({pins:[{status:'offen'},{status:'offen'}]},contract));
 const memory={memorySnapshot:[{id:'own-board',title:'Plan',notes:[1,2,3,4,5]}]};
 assert.equal(team.readTool(memory,{typ:'pinwand_lesen',board_id:'own-board',ab:0}).notes.length,4);
 assert.equal(team.readTool(memory,{typ:'pinwand_lesen',board_id:'own-board',ab:4}).notes[0],5);
 assert.throws(()=>team.readTool(memory,{typ:'pinwand_lesen',board_id:'foreign-board',ab:0}));
 console.log('PASS: actual browser model loop, local and OpenRouter transport, call budget, pause without model calls, malformed response, three-error stop, late-result cancellation and private key boundary.');
 console.log('PASS: team endpoint validation, strict model JSON contract and account-local paged pinboard memory.');
})().catch(e=>{console.error(e);process.exitCode=1;});
