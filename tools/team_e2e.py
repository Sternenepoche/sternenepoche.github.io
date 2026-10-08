"""Exercise the real team UI and Rust persistence; provider mocks are explicit.
Optional --real-ollama probes one installed local model via the browser, no downloads.
"""
from pathlib import Path
import json,subprocess,time,urllib.request,uuid,sys,shutil
from playwright.sync_api import sync_playwright

ROOT=Path(__file__).resolve().parents[1]
DATA=ROOT/'laeufe/team-2026-10-08'/uuid.uuid4().hex[:8]
OUT=ROOT/'docs/bilder/team'
API='http://127.0.0.1:18998';ADMIN='http://127.0.0.1:18999'
# Same browser origin as a normally configured local installation; all its page
# resources are served from the workspace via routes, never changing the live world.
WEB='http://127.0.0.1:8890'
def request(path,body=None,token=None,admin=False):
    h={'Content-Type':'application/json'}
    if token:h['X-Sternenepoche-Admin' if admin else 'Authorization']=token if admin else 'Bearer '+token
    req=urllib.request.Request((ADMIN if admin else API)+path,data=json.dumps(body).encode() if body is not None else None,headers=h)
    with urllib.request.urlopen(req,timeout=30) as r:return json.load(r)
def main():
    DATA.mkdir(parents=True);OUT.mkdir(parents=True,exist_ok=True)
    shutil.copy2(ROOT/'target/debug/sternenepoche-server.exe',DATA/'test-server.exe')
    server=subprocess.Popen([str(DATA/'test-server.exe'),'--data',str(DATA),'--bind','127.0.0.1:18998','--admin-bind','127.0.0.1:18999','--origin',WEB],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,creationflags=subprocess.CREATE_NO_WINDOW)
    errors=[];model_requests=[];commands=[];api_requests=[]
    try:
        for _ in range(100):
            try:lobby=request('/api/lobby');break
            except OSError:time.sleep(.1)
        key=request('/api/admin/bootstrap',admin=True)['key'];wid=lobby['world_id']
        request('/api/admin/action',{'world_id':wid,'action':'settings','paused':False,'bots_enabled':False,'tempo':1},key,True)
        request('/api/admin/action',{'world_id':wid,'action':'test_account','name':'TeamProbe','password':'isolierter-team-ui-test'},key,True)
        auth=request('/api/login',{'name':'TeamProbe','password':'isolierter-team-ui-test'})
        request('/api/claim',{'mode':'gemischt','volk':'aurelianer'},auth['token'])
        with sync_playwright() as p:
            browser=p.chromium.launch(channel='chrome',headless=True)
            page=browser.new_page(viewport={'width':1440,'height':1100})
            cdp=browser.new_browser_cdp_session();contexts=cdp.send('Target.getBrowserContexts')['browserContextIds'];cdp.send('Browser.setPermission',{'permission':{'name':'local-network-access'},'setting':'granted','origin':WEB,'browserContextId':contexts[0]})
            for permission in ['loopback-network','local-network']:
                cdp.send('Browser.setPermission',{'permission':{'name':permission},'setting':'granted','origin':WEB,'browserContextId':contexts[0]})
            page.on('pageerror',lambda e:(errors.append(str(e)),print('PAGE ERROR',str(e),flush=True)))
            page.on('console',lambda m:print('BROWSER',m.text,flush=True) if m.type=='error' else None)
            def until(expression,timeout=30000):
                deadline=time.monotonic()+timeout/1000
                while time.monotonic()<deadline:
                    if page.evaluate(expression):return
                    page.wait_for_timeout(80)
                raise AssertionError('Timed out: '+expression+'; status: '+page.locator('#team-feedback').inner_text()+'; agent: '+page.locator('#agent-status').inner_text()+'; log: '+page.locator('#agent-log').inner_text()+'; probes: '+str(page.locator('.team-probe').all_inner_texts()))
            def static(route):
                rel=route.request.url[len(WEB):].split('?')[0].lstrip('/') or 'index.html'
                if rel=='config.js':return route.fulfill(content_type='application/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};')
                f=ROOT/'web-client'/rel
                if f.is_file():route.fulfill(path=str(f))
                else:route.fulfill(status=404,body='not found')
            page.route(WEB+'/**',static)
            def observed(req):
                if req.url.startswith(API):
                    body=req.post_data or '';api_requests.append(body)
                    if req.url.endswith('/api/command'):commands.append(json.loads(body))
            page.on('request',observed)
            page.add_init_script('sessionStorage.setItem("sternenepoche-session",'+json.dumps(json.dumps({'server':API,**auth}))+');')
            page.goto(WEB+'/',wait_until='networkidle');print('INITIAL',page.locator('#connection').inner_text(),page.locator('#message').text_content(),flush=True);page.locator('#game').wait_for(state='visible');page.locator('#sternen-loader').wait_for(state='hidden');page.locator('[data-tab="agent"]').click();page.locator('#team-workbench h2').wait_for()
            page.locator('[data-team="add-model"]').click()
            card=page.locator('.team-model').first
            card.locator('[data-model="model"]').fill('qwen2.5-coder:1.5b')
            if '--real-ollama' in sys.argv:
                card.locator('[data-team="list-models"]').click();until("document.querySelector('[data-catalog]')!==null")
                card.locator('[data-catalog]').select_option('qwen2.5-coder:1.5b')
                card.locator('[data-team="test"]').click()
                until("!document.querySelector('.team-probe').textContent.startsWith('Prüfe')",timeout=250000)
                assert card.locator('.team-probe').inner_text().startswith('Bestanden'),card.locator('.team-probe').inner_text()
                (DATA/'real-ollama.txt').write_text(card.locator('.team-probe').inner_text(),encoding='utf-8')
                if '--real-rehearsal' in sys.argv:
                    card.locator('[data-team="rehearse"]').click();until("document.getElementById('agent-log').textContent.includes('SPIELPROBE')",timeout=250000)
                    assert not commands
                    (DATA/'real-rehearsal.txt').write_text(page.locator('#agent-log').inner_text(),encoding='utf-8')
            def mock(route):
                url=route.request.url
                if url.endswith('/api/tags'):return route.fulfill(json={'models':[{'name':'qwen2.5-coder:1.5b','size':900000000},{'name':'zweites-testmodell:latest','size':1800000000}]})
                if url.endswith('/models'):return route.fulfill(json={'data':[{'id':'openai/gpt-4.1-mini','name':'Beispielmodell','architecture':{'output_modalities':['text']}}]})
                body=route.request.post_data_json;model_requests.append(body)
                answer={'aktionen':[],'abfragen':[],'begruendung':'Verbindung funktioniert. Versorgung prüfen.','notiz':'Atlas: Handelsbedarf gemeinsam prüfen; noch keinen Handel zugesagt.','wecker_stunden':1,'team_pins':[]}
                schema=body.get('format',{});message=' '.join(x['content'] for x in body.get('messages',[]))
                if '"prognose"' in message:answer['prognose']='Versorgung bleibt unter Beobachtung.'
                if 'rollenwahl' in schema.get('properties',{}) or 'rollenwahl' in message:answer['rollenwahl']=['verwalter','diplomat','stratege','feldherr']
                if 'Deine feste Teamidentität' in message:
                    memory=json.loads(body['messages'][-1]['content'].split('Gedächtnis: ',1)[1].split('\nLies zuerst',1)[0])
                    answer['team_pins']=[{'board_id':memory['pinwaende'][0]['id'],'note':{'id':'handel-plan','title':'Handelsbedarf','text':'Atlas: Kristallbedarf mit Nova abstimmen. Keine Zusage ohne Prüfung.','to':'Atlas','status':'offen'}}]
                result={'message':{'content':json.dumps(answer)},'choices':[{'message':{'content':json.dumps(answer)}}],'usage':{'total_tokens':30,'cost':0.00001}}
                route.fulfill(json=result)
            page.route('http://127.0.0.1:11434/**',mock);page.route('https://openrouter.ai/api/v1/**',mock)
            if '--real-ollama' not in sys.argv:
                card.locator('[data-team="test"]').click();until("document.querySelector('.team-probe').textContent.startsWith('Bestanden')")
            page.locator('[data-team="add-model"]').click();second=page.locator('.team-model').nth(1)
            second.locator('[data-model="provider"]').select_option('openrouter');second.locator('[data-key]').fill('TEST-SECRET-NEVER-PUBLISH');second.locator('[data-model="model"]').fill('openai/gpt-4.1-mini');second.locator('[data-team="test"]').click();until("[...document.querySelectorAll('.team-probe')].every(x=>x.textContent.startsWith('Bestanden'))")
            page.locator('[data-team="add-model"]').click();third=page.locator('.team-model').nth(2);third.locator('[data-model="model"]').fill('zweites-testmodell:latest');third.locator('[data-team="test"]').click();until("[...document.querySelectorAll('.team-probe')].every(x=>x.textContent.startsWith('Bestanden'))")
            page.locator('[data-team="add-model"]').click();assert page.locator('[data-team="add-model"]').is_disabled();page.locator('.team-model').last.locator('[data-team="remove-model"]').click();assert page.locator('.team-model').count()==3
            ids=page.locator('.team-model').evaluate_all('(es)=>es.map(e=>e.dataset.modelId)')
            for role,mid in [('stratege',ids[0]),('verwalter',ids[0]),('diplomat',ids[1]),('feldherr',ids[2])]:
                detail=page.locator('[data-role-id="'+role+'"]');detail.locator('summary').click();detail.locator('[data-role="model"]').select_option(mid)
            page.locator('.team-model').first.locator('[data-team="rehearse"]').click();until("document.getElementById('team-feedback').textContent.startsWith('Spielprobe bestanden')");assert not commands,'rehearsal executed a command'
            page.locator('[data-limit="limit"]').fill('4');page.locator('[data-limit="delay"]').fill('15')
            page.locator('#team-workbench [data-team="save"]').click();until("document.getElementById('team-feedback').textContent.startsWith('Gespeichert')")
            page.locator('[data-tab="pinwaende"]').click();page.locator('[data-team="add-note"]').first.click();note=page.locator('.pin-note').first;note.locator('[data-note="title"]').fill('Unser nächstes Ziel');note.locator('[data-note="text"]').fill('Nova prüft Energie. Atlas klärt Handelsmöglichkeiten. Ergebnisse zuerst bestätigen.');note.locator('[data-note="to"]').fill('Nova und Atlas')
            page.locator('[data-team="add-board"]').click();page.locator('[data-board="title"]').last.fill('Langfristiger Ausbau');page.locator('[data-team="board-up"]').last.click();page.locator('#pinboard-content [data-team="save"]').click();until("document.getElementById('pinboard-feedback').textContent.startsWith('Dauerhaft gespeichert')")
            saved=request('/api/workspace',token=auth['token']);assert saved['state']['boards'][0]['title']=='Langfristiger Ausbau';assert len(saved['state']['boards'])==2
            page.locator('[data-tab="agent"]').click();page.locator('[data-team="start"]').click();until("document.getElementById('agent-status').textContent.includes('Aufruflimit erreicht')",timeout=60000)
            saved=request('/api/workspace',token=auth['token']);assert len(saved['journal'])==4,saved['journal'];assert len(commands)==4;assert all(x.get('team') for x in commands)
            assert {x['model'] for x in model_requests}>={'qwen2.5-coder:1.5b','openai/gpt-4.1-mini'}
            assert all(x.get('keep_alive')==0 for x in model_requests if 'format' in x)
            assert all('TEST-SECRET' not in x for x in api_requests),'Provider key sent to game API'
            # A second tab's save must invalidate the local optimistic revision.
            external=saved['state'];external['boards'][0]['title']='Extern bearbeitet';request('/api/workspace',{'world_id':wid,'revision':saved['revision'],'state':external},auth['token'])
            page.locator('#team-workbench [data-team="save"]').click();until("document.getElementById('team-feedback').textContent.includes('inzwischen geändert')")
            page.locator('#team-workbench [data-team="reload"]').click();until("document.getElementById('team-feedback').textContent.includes('Konfiguration im Konto')")
            # Fully agent-controlled start asks both models for preferences, then plays.
            request('/api/claim',{'mode':'agent','volk':'aurelianer'},auth['token']);page.evaluate('refresh()');until("session.mode==='agent'")
            page.locator('[data-limit="limit"]').fill('7');page.locator('#team-workbench [data-team="save"]').click();until("document.getElementById('team-feedback').textContent.startsWith('Gespeichert')")
            page.locator('[data-team="start"]').click();until("document.getElementById('agent-status').textContent.includes('Aufruflimit erreicht')",timeout=60000)
            saved=request('/api/workspace',token=auth['token']);assert any(b['title']=='Teamberatung' for b in saved['state']['boards']);assert len(saved['journal'])==8
            # Persisted pinboards survive reload; credentials and probe approval do not.
            page.reload(wait_until='networkidle');page.locator('#sternen-loader').wait_for(state='hidden');page.locator('[data-tab="agent"]').click();page.locator('.team-model').first.wait_for();assert page.locator('[data-key]').input_value()=='';assert 'Noch nicht getestet' in page.locator('.team-probe').first.inner_text()
            page.locator('[data-tab="pinwaende"]').click();assert 'Handelsbedarf' in page.locator('#pinboard-content').inner_text()
            page.set_viewport_size({'width':390,'height':844});page.screenshot(path=str(DATA/'mobile-pinboards.png'),full_page=True);assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+2')
            page.locator('[data-tab="agent"]').click();page.screenshot(path=str(DATA/'mobile-team.png'),full_page=True);assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+2')
            page.set_viewport_size({'width':1440,'height':1100})
            def shot(name,selector,marks):
                page.locator(selector).scroll_into_view_if_needed();page.evaluate('''({selector,marks,name})=>{const root=document.querySelector(selector),w=root.getBoundingClientRect().width,frame=document.createElement('div');frame.id='guide-capture-frame';frame.style.cssText='position:relative;background:#0b1724;padding:24px 24px 24px 72px;box-sizing:content-box;width:'+w+'px';root.dataset.guideOldStyle=root.getAttribute('style')||'';root.replaceWith(frame);frame.append(root);root.style.width=w+'px';if(name==='01-modelle'){frame.style.maxHeight='1140px';frame.style.overflow='hidden';}for(const [sel,num] of marks){const e=root.querySelector(sel);if(!e)continue;const a=document.createElement('span');const r=e.getBoundingClientRect(),b=frame.getBoundingClientRect();a.textContent=num+' ➜';a.style.cssText='position:absolute;z-index:100;pointer-events:none;background:#ffdc80;color:#172230;font:bold 18px system-ui;border-radius:6px;padding:3px 7px;left:8px;top:'+(r.top-b.top+4)+'px';frame.append(a);}}''',{'selector':selector,'marks':marks,'name':name})
                page.locator('#guide-capture-frame').screenshot(path=str(OUT/(name+'.png')))
                page.evaluate("()=>{const f=document.getElementById('guide-capture-frame'),r=f.firstElementChild;r.setAttribute('style',r.dataset.guideOldStyle);delete r.dataset.guideOldStyle;f.replaceWith(r);}")
            shot('01-modelle','#team-workbench',[('[data-team="add-model"]','1'),('[data-model="provider"]','2'),('[data-model="url"]','3'),('[data-team="list-models"]','4'),('[data-model="model"]','5'),('[data-team="test"]','6')])
            shot('02-ollama','.team-model:first-child',[('[data-model="name"]','1'),('[data-model="url"]','2'),('[data-team="list-models"]','3'),('[data-model="model"]','4'),('[data-team="test"]','5')])
            shot('03-openrouter','.team-model:nth-child(2)',[('[data-model="provider"]','1'),('[data-key]','2'),('[data-team="list-models"]','3'),('[data-model="model"]','4'),('[data-team="test"]','5')])
            page.locator('[data-role-id="verwalter"] summary').click();shot('04-rollen','[data-role-id="verwalter"]',[('[data-role="model"]','1'),('[data-role="soul"]','2'),('[data-role="instructions"]','3'),('[data-import-role]','4')])
            page.locator('.team-origins summary').click();shot('05-browserfreigabe','.team-origins',[('code','1'),('pre','2')])
            page.locator('[data-tab="pinwaende"]').click();shot('06-pinwaende','#pinwaende',[('[data-team="add-board"]','1'),('[data-team="save"]','2'),('[data-team="board-up"]','3'),('[data-team="add-note"]','4')])
            page.goto(WEB+'/agenten-hilfe.html',wait_until='networkidle');assert page.locator('h1').count()==1
            assert not errors,errors
            browser.close()
        (DATA/'report.json').write_text(json.dumps({'passed':True,'page_errors':errors,'provider_mock_calls':len(model_requests),'confirmed_game_commands':len(commands),'real_ollama':'--real-ollama' in sys.argv},indent=2),encoding='utf-8')
        print('PASS team UI, mixed providers, role council, atomic memory, conflict, reload, mobile, keys. Artifacts:',DATA)
    finally:server.terminate();server.wait(timeout=10)
if __name__=='__main__':main()
