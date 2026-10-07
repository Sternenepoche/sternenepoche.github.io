// Load both real deferred scripts separately, matching the browser's execution order.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const elements=new Map(),requests=[];
const html=fs.readFileSync('web-client/admin/index.html','utf8');
function element(id){assert(html.includes('id="'+id+'"'),'Real HTML is missing '+id);return {id,value:'',textContent:'',innerHTML:'',hidden:false,disabled:id==='reset-submit',classList:{toggle(){}},addEventListener(){},querySelectorAll(){return []}};}
const state={lobby:{world_id:'world-one',sekunden:0,paused:false,freie_plaetze:20,freie_zugaenge:3,freigegebene_plaetze:3,wartende:0,tempo:1},storage_error:null,flotten:0,ereignisse:3,uptime_s:5,db_bytes:1000,invariant_errors:[],epoche_tage:365,bot_period_secs:7200,bots_enabled:true,world_hash:'hash',accounts:[],bots:[],agents:[],bot_actions:0,bot_rejected:0,monitor:{bots:[],letztes_backup:null,audit_eintraege:0}};
const ctx=vm.createContext({console,AbortSignal,setInterval(){},setTimeout(){return 1},clearTimeout(){},
 document:{getElementById(id){if(!elements.has(id))elements.set(id,element(id));return elements.get(id)},addEventListener(){},activeElement:null},
 fetch:async(path,options)=>{const body=options.body?JSON.parse(options.body):null;requests.push({path,body});let value=state;
 if(path.endsWith('/bootstrap'))value={key:'fake-test-key',game_url:'http://game.test'};
 if(body?.action==='rules_profile')value={text:'test profile'};
 if(body?.action==='reset_preview')value={valid:true,profile_hash:'test-hash'};
 return {ok:true,json:async()=>value};}});
(async()=>{
 for(const file of ['admin.js','monitoring.js'])vm.runInContext(fs.readFileSync('web-client/admin/'+file,'utf8'),ctx,{filename:file});
 await new Promise(resolve=>setImmediate(resolve));
 assert.equal(elements.get('admission-limit').value,3);assert.equal(elements.get('waitlist').innerHTML,'Niemand wartet derzeit.');
 assert.equal(elements.get('status').textContent,'Server läuft','dashboard bootstrap did not render status');
 const before=requests.length;await elements.get('refresh').onclick();assert(requests.length>before,'refresh control is disconnected');
 await elements.get('profile-load').onclick();assert.equal(elements.get('rule-profile').value,'test profile');
 await elements.get('profile-validate').onclick();assert.equal(elements.get('reset-submit').disabled,false);
 elements.get('rule-profile').value='edited profile';elements.get('rule-profile').oninput();assert.equal(elements.get('reset-submit').disabled,true,'edited profile remained approved');
 assert(requests.filter(r=>r.body?.action).every(r=>r.body.world_id==='world-one'));
 console.log('PASS: actual admin script load order, bootstrap, refresh control, rule preview and invalidation after editing.');
})().catch(e=>{console.error(e);process.exitCode=1});
