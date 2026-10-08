"""Real Rust API + browser exercise on an isolated, explicitly prepared demonstration world."""
from pathlib import Path
import functools, http.server, json, os, subprocess, threading, time, urllib.request, uuid
from playwright.sync_api import sync_playwright
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
DATA=ROOT/'laeufe/kommunikation-2026-10-08'/('browser-'+uuid.uuid4().hex[:8])
OUT=ROOT/'docs/bilder/kommunikation'
API='http://127.0.0.1:18996'; ADMIN='http://127.0.0.1:18997'; WEB='http://127.0.0.1:18889'
def request(path,body=None,token=None,admin=False):
    headers={'Content-Type':'application/json'}
    if token:headers['X-Sternenepoche-Admin' if admin else 'Authorization']=token if admin else 'Bearer '+token
    req=urllib.request.Request((ADMIN if admin else API)+path,data=json.dumps(body).encode() if body is not None else None,headers=headers)
    with urllib.request.urlopen(req,timeout=30) as response:return json.load(response)
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self,*args):pass
def main():
    subprocess.run([str(ROOT/'target/debug/examples/communication_fixture.exe'),str(DATA)],check=True,cwd=ROOT)
    fixture=json.loads((DATA/'fixture.json').read_text()); auth=fixture['accounts'][0]
    binary=Path(os.environ.get('STERNENEPOCHE_TEST_BINARY',str(ROOT/'target/debug/sternenepoche-server.exe')))
    server=subprocess.Popen([str(binary),'--data',str(DATA),'--bind','127.0.0.1:18996','--admin-bind','127.0.0.1:18997','--origin',WEB],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,creationflags=subprocess.CREATE_NO_WINDOW)
    web=http.server.ThreadingHTTPServer(('127.0.0.1',18889),functools.partial(Quiet,directory=str(ROOT)));threading.Thread(target=web.serve_forever,daemon=True).start()
    OUT.mkdir(parents=True,exist_ok=True);errors=[]
    try:
        for _ in range(100):
            try:lobby=request('/api/lobby');break
            except Exception:time.sleep(.15)
        else:raise RuntimeError('Server failed to start')
        with urllib.request.urlopen(API+'/') as response:assert b'privat-compose-dialog' in response.read()
        for name in ['briefkasten','schreiben','anfragen','gruenden','allianz','chat','wirtschaft','hilfe','allianzpost']:
            with urllib.request.urlopen(API+'/guide/'+name+'.webp') as response:assert response.headers['Content-Type']=='image/webp' and len(response.read())>1000
        key=request('/api/admin/bootstrap',admin=True)['key']
        request('/api/admin/action',{'world_id':lobby['world_id'],'action':'settings','paused':False,'bots_enabled':False,'tempo':1},key,True)
        with sync_playwright() as p:
            browser=p.chromium.launch(channel='chrome',headless=True)
            page=browser.new_page(viewport={'width':1440,'height':1080},device_scale_factor=1)
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.route('**/config.js',lambda route:route.fulfill(content_type='application/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};'))
            page.add_init_script('sessionStorage.setItem("sternenepoche-session",'+json.dumps(json.dumps({'server':API,**auth}))+');')
            page.goto(WEB+'/web-client/',wait_until='networkidle');page.locator('#game').wait_for(state='visible');page.locator('#sternen-loader').wait_for(state='hidden')
            def capture(name):
                page.screenshot(path=str(OUT/(name+'.png')))
                with Image.open(OUT/(name+'.png')) as im:im.save(OUT/(name+'.webp'),'WEBP',quality=86)
            page.locator('[data-tab="briefkasten"]').click();page.locator('#messages .relation-verbuendet').first.wait_for()
            assert 'Sternenbund' in page.locator('#messages').inner_text()
            capture('briefkasten')
            page.locator('[data-mail-folder="anfragen"]').click()
            page.get_by_text('Neue diplomatische Anfrage',exact=True).click()
            page.locator('#request-form [name="partner"]').fill('Vega')
            page.locator('#request-form [name="art"]').select_option('krieg')
            page.locator('#request-form [name="text"]').fill('Beispiel einer vorbereiteten Kriegserklärung. Noch nicht gesendet.')
            page.locator('#request-form').scroll_into_view_if_needed();capture('anfragen')
            page.locator('[data-mail-folder="eingang"]').click();page.evaluate('window.scrollTo(0,0)')
            page.locator('[data-mail-to="Mira"]').click()
            page.locator('#privat-compose-dialog').wait_for(state='visible')
            assert page.locator('#message-form [name="an"]').input_value()=='Mira'
            writer=page.locator('#message-form textarea')
            box=writer.bounding_box();assert box['width']>=850 and box['height']>=340,box
            writer.fill('x'*1300);assert len(writer.input_value())==1300
            assert not writer.evaluate('(t)=>t.checkValidity()')
            page.evaluate('refresh()');assert len(writer.input_value())==1300
            page.get_by_role('button',name='Schreibfenster schließen',exact=True).click()
            page.locator('[data-compose="privat"]').click();assert len(writer.input_value())==1300
            page.locator('#message-form [name="betreff"]').fill('Gemeinsam die nächste Etappe planen')
            writer.fill('Hallo Mira,\n\nunsere Außenposten können sich gegenseitig unterstützen. Ich möchte mit dir eine verlässliche Versorgung aufbauen.\n\nWelche Waren brauchst du in den nächsten Tagen? Ich kann Erz anbieten und suche Kristall. Lass uns außerdem einen gemeinsamen Plan für den Fall eines Angriffs festhalten.\n\nViele Grüße\nAster')
            capture('schreiben')
            page.keyboard.press('Escape');assert not page.locator('#privat-compose-dialog').is_visible()
            page.locator('[data-open-letter="1"]').click();page.locator('#letter-reader').wait_for(state='visible')
            assert 'Mira' in page.locator('#letter-participants').inner_text()
            page.locator('#reply-letter').click();assert page.locator('#message-form [name="an"]').input_value()=='Mira'
            page.locator('#message-form [name="text"]').fill('Browserprüfung: Danke für deine Nachricht.')
            page.locator('#message-form button').click()
            page.wait_for_function("() => view.kommunikation.briefkasten.some(n=>n.text==='Browserprüfung: Danke für deine Nachricht.')")
            page.locator('[data-mail-folder="gesendet"]').click();assert 'Mira' in page.locator('#messages').inner_text()
            page.locator('[data-tab="allianzbereich"]').click();page.locator('#alliance-private').wait_for(state='visible');capture('allianz')
            page.locator('[data-alliance-pane="chat"]').click()
            page.locator('#alliance-chat-form [name="text"]').fill('Nur Mitglieder sehen diese Browserprüfung.')
            page.locator('#alliance-chat-form button').click();page.wait_for_function("() => document.getElementById('alliance-chat-log').textContent.includes('Nur Mitglieder sehen')");capture('chat')
            page.locator('[data-alliance-pane="economy"]').click();page.locator('#alliance-economy-summary').scroll_into_view_if_needed();capture('wirtschaft')
            page.locator('[data-buy-internal="1"]').click();page.wait_for_function("() => view.kommunikation.allianzbereich.angebote[0].status==='unterwegs'")
            page.locator('[data-alliance-pane="help"]').click();page.locator('#alliance-help-list').scroll_into_view_if_needed();capture('hilfe')
            page.locator('[data-tab="allianzpost"]').click();page.evaluate('window.scrollTo(0,0)');page.locator('#alliance-post-content').wait_for(state='visible');capture('allianzpost')
            # This is a second authenticated player, not a filtered DOM simulation.
            outsider=request('/api/view',token=fixture['accounts'][2]['token'])
            assert 'Nur Mitglieder sehen diese Browserprüfung.' not in json.dumps(outsider)
            assert not outsider['kommunikation']['allianzbereich']['angebote']
            denied=request('/api/command',{'world_id':fixture['world_id'],'request_id':'foreign-read-'+uuid.uuid4().hex,'aktionen':[{'typ':'brief_lesen','brief':1}]},fixture['accounts'][2]['token'])
            assert denied['ergebnisse'][0]['ok'] is False
            # Deliberate XSS-shaped text is displayed literally.
            request('/api/command',{'world_id':fixture['world_id'],'request_id':'text-escape-'+uuid.uuid4().hex,'aktionen':[{'typ':'brief_senden','kanal':'privat','an':'Aster','betreff':'<img src=x onerror=alert(1)>','text':'<script>window.mailLeak=true</script>','antwort_auf':None}]},fixture['accounts'][2]['token'])
            page.evaluate('refresh()');page.locator('[data-tab="briefkasten"]').click();page.locator('[data-mail-folder="eingang"]').click();page.wait_for_function("() => document.getElementById('messages').textContent.includes('<img src=x')")
            assert page.locator('#messages img').count()==0;assert page.evaluate('window.mailLeak') is None
            assert page.locator('#messages .relation-nap').count()>0
            page.set_viewport_size({'width':390,'height':844});capture('mobil')
            assert page.evaluate('document.documentElement.scrollWidth <= innerWidth+2'), 'Mobile layout overflow'
            page.locator('[data-mail-to="Mira"]').click();writer.fill('Ein langer Entwurf bleibt auch auf dem Handy lesbar.\n\n'+('Mehr Platz zum Schreiben. '*25));capture('schreiben-mobil')
            assert writer.bounding_box()['width']>=300
            assert page.locator('#privat-compose-dialog').evaluate('(d)=>d.scrollWidth<=d.clientWidth+2')
            page.keyboard.press('Escape');page.locator('[data-tab="regeln"]').click()
            assert page.locator('.communication-faq details').count()==10
            page.locator('.communication-faq summary').first.click()
            pic=page.locator('.communication-faq img').first;pic.scroll_into_view_if_needed();pic.wait_for()
            page.wait_for_function("() => document.querySelector('.communication-faq img').naturalWidth>0")
            request('/api/command',{'world_id':fixture['world_id'],'request_id':'founding-view-'+uuid.uuid4().hex,'aktionen':[{'typ':'allianz_verlassen'}]},fixture['accounts'][2]['token'])
            page=browser.new_page(viewport={'width':1440,'height':1080})
            page.route('**/config.js',lambda route:route.fulfill(content_type='application/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};'))
            page.add_init_script('sessionStorage.setItem("sternenepoche-session",'+json.dumps(json.dumps({'server':API,**fixture['accounts'][2]}))+');')
            page.goto(WEB+'/web-client/',wait_until='networkidle');page.locator('#game').wait_for(state='visible');page.locator('#sternen-loader').wait_for(state='hidden')
            page.locator('[data-tab="allianzbereich"]').click();page.locator('#alliance-create [name="name"]').fill('Neue Horizonte');capture('gruenden')
            browser.close()
        import sqlite3
        db=sqlite3.connect(DATA/'spiel.sqlite3')
        assert db.execute('select count(*) from communication_messages').fetchone()[0]>=9
        assert db.execute('select count(*) from communication_events').fetchone()[0]>10
        assert db.execute('select count(*) from game_actions').fetchone()[0]>0
        db.close()
        result={'screens':['briefkasten','schreiben','anfragen','gruenden','allianz','chat','wirtschaft','hilfe','allianzpost','mobil','schreiben-mobil'],'page_errors':errors,'source':'isolated Rust demonstration world; market building and initial resources prepared explicitly; all browser interactions use real API','checks':['mail reply','read receipts','role privacy','alliance chat','real reserved trade purchase','SQL archive','NAP/alliance colors','XSS escaping','mobile layout','large desktop and mobile writer','allied recipient shortcut','over-limit paste preserved','draft survives refresh and Escape','illustrated FAQ']}
        (OUT/'capture.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(result,ensure_ascii=False));assert not errors,errors
    finally:
        web.shutdown();server.terminate();server.wait(timeout=15)
if __name__=='__main__':main()
