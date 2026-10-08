"""Real HTTP / SQLite / process-restart test, with isolated disposable test data on D:."""
from pathlib import Path
import concurrent.futures, json, os, subprocess, sys, time, urllib.request, urllib.error, uuid
ROOT=Path(__file__).resolve().parents[1]
DATA=ROOT/'laeufe'/'inventur-2026-10-07'/('http-test-'+uuid.uuid4().hex[:8])
PUBLIC='http://127.0.0.1:18990'; ADMIN='http://127.0.0.1:18991'
process=None
def request(path,body=None,token=None,admin=False,headers=None):
    h=dict(headers or {})
    if body is not None:h['Content-Type']='application/json'
    if token:h['X-Sternenepoche-Admin' if admin else 'Authorization']=token if admin else 'Bearer '+token
    req=urllib.request.Request((ADMIN if admin else PUBLIC)+path,data=None if body is None else json.dumps(body).encode(),headers=h)
    try:
        with urllib.request.urlopen(req,timeout=15) as r:return r.status,json.loads(r.read())
    except urllib.error.HTTPError as e:return e.code,json.loads(e.read())
def start():
    global process
    DATA.mkdir(parents=True,exist_ok=True)
    binary='sternenepoche-server'+('.exe' if sys.platform=='win32' else '')
    executable=Path(os.environ.get('STERNENEPOCHE_SERVER_EXE',str(ROOT/'target/release'/binary)))
    if not executable.is_file():raise AssertionError('server binary not found: '+str(executable))
    process=subprocess.Popen([str(executable),'--data',str(DATA),'--bind','127.0.0.1:18990','--admin-bind','127.0.0.1:18991'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    for _ in range(100):
        if process.poll() is not None:raise AssertionError('server failed to start')
        try:
            if request('/api/lobby')[0]==200:return
        except OSError:pass
        time.sleep(.1)
    raise AssertionError('server did not become ready')
def stop():
    global process
    if process is not None:process.terminate();process.wait(timeout=15);process=None
def check(status,expected):assert status==expected,(status,expected)
try:
    start()
    _,bootstrap=request('/api/admin/bootstrap',admin=True);key=bootstrap['key']
    check(request('/api/admin/bootstrap')[0],404)
    check(request('/monitoring.js')[0],401)
    check(request('/api/admin/status',admin=True)[0],401)
    check(request('/api/lobby',headers={'Origin':'https://unknown.invalid'})[0],403)
    lobby=request('/api/lobby')[1];assert lobby['freie_plaetze']==20 and lobby['freie_zugaenge']==3 and lobby['freigegebene_plaetze']==3 and len(lobby['rangliste'])==30
    assert set(lobby['verlauf'][0])<= {'sekunden','reiche','kolonien','punkte','stufen'} if lobby['verlauf'] else True
    auth={}
    for name in ['SpielerA','SpielerB','Zuschauer']:
        s,v=request('/api/register',{'name':name,'password':'nur-ein-test-passwort'});check(s,200);auth[name]=v['token']
    check(request('/api/view',token=auth['Zuschauer'])[0],403)
    check(request('/monitoring.js',token=auth['Zuschauer'])[0],404)
    check(request('/api/command',{'world_id':lobby['world_id'],'request_id':'watch','aktionen':[{'typ':'steuersatz','prozent':20}]},auth['Zuschauer'])[0],403)
    for name in ['SpielerA','SpielerB']:
        check(request('/api/claim',{'mode':'gemischt','volk':'veyari'},auth[name])[0],200)
    racers=['Dritter','Vierter']
    for name in racers:
        s,v=request('/api/register',{'name':name,'password':'nur-ein-test-passwort'});check(s,200);auth[name]=v['token']
    claim={'world_id':lobby['world_id'],'mode':'mensch','volk':'krath'}
    with concurrent.futures.ThreadPoolExecutor(2) as pool:
        results=list(pool.map(lambda name:request('/api/claim',claim,auth[name]),racers))
    assert all(s==200 for s,v in results)
    assert sum(v['spieler'] is not None for s,v in results)==1,results
    waiter=next(name for name,(s,v) in zip(racers,results) if v['spieler'] is None)
    assert request('/api/me',token=auth[waiter])[1]['warteliste']['position']==1
    assert request('/api/claim',claim,auth[waiter])[1]['warteliste']['position']==1
    check(request('/api/view',token=auth[waiter])[0],403)
    public=request('/api/lobby')[1];assert public['freie_zugaenge']==0
    assert public['freigegebene_plaetze']==3 and public['freie_plaetze']==17 and public['wartende']==1 and 'warteliste' not in public
    assert request('/api/claim',claim,auth['Zuschauer'])[1]['warteliste']['position']==2
    check(request('/api/waitlist/leave',{'world_id':lobby['world_id']},auth['Zuschauer'])[0],200)
    assert request('/api/me',token=auth['Zuschauer'])[1]['warteliste'] is None
    views={name:request('/api/view',token=auth[name])[1] for name in ['SpielerA','SpielerB']}
    a,b=views.values();assert a['planeten'][0]['koord']!=b['planeten'][0]['koord']
    s,context=request('/api/context',{'rolle':'verwalter'},auth['SpielerA']);check(s,200)
    assert [p['koord'] for p in context['view']['planeten']]==[a['planeten'][0]['koord']] and context['text']
    cmd={'world_id':lobby['world_id'],'request_id':'duplicate','aktionen':[{'typ':'bauen','planet':a['planeten'][0]['koord'],'gebaeude':'erzmine'}]}
    with concurrent.futures.ThreadPoolExecutor(2) as pool:
        results=list(pool.map(lambda _:request('/api/command',cmd,auth['SpielerA']),range(2)))
    assert results[0]==results[1];check(results[0][0],200);assert results[0][1]['ergebnisse'][0]['ok']
    own=request('/api/view',token=auth['SpielerA'])[1];assert len(own['planeten'][0]['bauschleife'])==1
    bad={**cmd,'request_id':'foreign','aktionen':[{'typ':'bauen','planet':b['planeten'][0]['koord'],'gebaeude':'farm'}]}
    s,v=request('/api/command',bad,auth['SpielerA']);check(s,200);assert not v['ergebnisse'][0]['ok']
    s,lease=request('/api/lease',{'world_id':lobby['world_id'],'action':'start','roles':['verwalter']},auth['SpielerA']);check(s,200)
    check(request('/api/lease',{'world_id':lobby['world_id'],'action':'start','roles':['verwalter']},auth['SpielerA'])[0],409)
    check(request('/api/command',{**cmd,'request_id':'manual-conflict'},auth['SpielerA'])[0],409)
    check(request('/api/agent-report',{'world_id':lobby['world_id'],'lease':lease['lease'],'status':'aktiv','calls':1,'tokens':50,'provider':'ollama','model':'test-model','decision':'Testentscheidung','accepted':2,'rejected':1},auth['SpielerA'])[0],200)
    check(request('/api/lease',{'world_id':lobby['world_id'],'action':'stop'},auth['SpielerA'])[0],200)
    check(request('/api/command',{**cmd,'request_id':'late','lease':lease['lease'],'rolle':'verwalter'},auth['SpielerA'])[0],409)
    check(request('/api/admin/action',{'world_id':lobby['world_id'],'action':'settings','paused':True},key,True)[0],200)
    before=request('/api/admin/status',token=key,admin=True)[1]
    assert before['agents'][0]['stats']['calls']==1 and before['agents'][0]['stats']['model']=='test-model'
    assert before['agents'][0]['stats']['accepted']==2 and len(before['monitor']['bots'])==30
    def admin_action(action,**extra):return request('/api/admin/action',{'world_id':lobby['world_id'],'action':action,**extra},key,True)
    for action in ['events','audit'] :check(admin_action(action)[0],200)
    s,private_map=admin_action('map',sektor=1,system=13);check(s,200);assert len(private_map['planeten'])==12
    s,profile=admin_action('rules_profile');check(s,200)
    s,preview=admin_action('reset_preview',rules=profile['text']);check(s,200)
    assert preview['valid'] and len(preview['profile_hash'])==64 and preview['betroffene_konten']==5
    check(admin_action('rules_validate',rules='(')[0],400)
    bad_reset={'world_id':lobby['world_id'],'action':'reset','confirm':'RESET '+lobby['world_id'],'rules':'('}
    check(request('/api/admin/action',bad_reset,key,True)[0],400)
    assert not list((DATA/'backups').glob('*')) if (DATA/'backups').exists() else True
    assert request('/api/admin/status',token=key,admin=True)[1]['world_hash']==before['world_hash']
    check(request('/api/admin/action',{'world_id':lobby['world_id'],'action':'backup'},key,True)[0],200)
    stop();start()
    after=request('/api/admin/status',token=key,admin=True)[1];assert after['world_hash']==before['world_hash']
    assert after['lobby']['freigegebene_plaetze']==3 and after['warteliste'][0]['name']==waiter
    assert request('/api/me',token=auth[waiter])[1]['warteliste']['position']==1
    check(admin_action('settings',admission_limit=4)[0],200)
    assert request('/api/me',token=auth[waiter])[1]['spieler'] is not None
    assert request('/api/me',token=auth[waiter])[1]['warteliste'] is None
    check(request('/api/view',token=auth['SpielerA'])[0],200)
    check(request('/api/login',{'name':'Zuschauer','password':'nur-ein-test-passwort'})[0],200)
    check(request('/api/login',{'name':'Zuschauer','password':'falsches-test-passwort'})[0],401)
    reset={'world_id':lobby['world_id'],'action':'reset','confirm':'RESET '+lobby['world_id'],'rules':profile['text']}
    check(request('/api/admin/action',reset,key,True)[0],200)
    check(request('/api/view',token=auth['SpielerA'])[0],401)
    assert request('/api/lobby')[1]['freie_plaetze']==20 and request('/api/lobby')[1]['freie_zugaenge']==4 and request('/api/lobby')[1]['wartende']==0
    new_lobby=request('/api/lobby')[1]
    s,new_profile=request('/api/admin/action',{'world_id':new_lobby['world_id'],'action':'rules_profile'},key,True);check(s,200)
    assert new_profile['hash']==preview['profile_hash']
    print('PASS: three-seat concurrent admission, private durable FIFO waitlist, cancellation, automatic promotion, reset limit persistence, HTTP login, private role contexts, two players, concurrent duplicate, ownership, exclusive roles, stop, admin isolation, CORS, bot monitoring, model reports, rule preview without mutation, invalid reset without backup, backup, durable restart and exact-profile reset.')
    print('Evidence directory:',DATA)
finally:stop()
