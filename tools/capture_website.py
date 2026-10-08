"""Capture the real current client in an isolated local demonstration world on D:."""
from pathlib import Path
import functools, http.server, json, subprocess, threading, time, urllib.request
from PIL import Image
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/bilder/online'
DATA = ROOT / 'laeufe/website-2026-10-08/demo-world'
API = 'http://127.0.0.1:18994'
ADMIN = 'http://127.0.0.1:18995'
WEB = 'http://127.0.0.1:18888'
SCREENS = ['reich','gebaeude','kolonie','forschen','werft','verteidigung','flotten',
           'karte','kolonien','kampf','aktionen','berichte','imperium','freischaltungen',
           'agent','profil','regeln']

def request(path, body=None, token=None, admin=False):
    headers = {'Content-Type':'application/json'}
    if token: headers['X-Sternenepoche-Admin' if admin else 'Authorization'] = token if admin else 'Bearer '+token
    req = urllib.request.Request((ADMIN if admin else API)+path,
        data=json.dumps(body).encode() if body else None, headers=headers)
    with urllib.request.urlopen(req, timeout=15) as response: return json.load(response)

class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self,*args): pass

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    DATA.mkdir(parents=True,exist_ok=True)
    server = subprocess.Popen([str(ROOT/'target/release/sternenepoche-server.exe'),
        '--data',str(DATA),'--bind','127.0.0.1:18994','--admin-bind','127.0.0.1:18995',
        '--origin',WEB],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NO_WINDOW)
    web = http.server.ThreadingHTTPServer(('127.0.0.1',18888),functools.partial(Quiet,directory=str(ROOT)))
    threading.Thread(target=web.serve_forever,daemon=True).start()
    try:
        for _ in range(100):
            try: lobby = request('/api/lobby'); break
            except OSError: time.sleep(.1)
        else: raise RuntimeError('Demo server did not start')
        key = request('/api/admin/bootstrap',admin=True)['key']
        request('/api/admin/action',{'world_id':lobby['world_id'],'action':'settings','paused':True,'bots_enabled':False},key,True)
        request('/api/admin/action',{'world_id':lobby['world_id'],'action':'test_account','name':'Sternenreisende','password':'lokale-demo-fuer-den-spielguide'},key,True)
        auth = request('/api/login',{'name':'Sternenreisende','password':'lokale-demo-fuer-den-spielguide'})
        request('/api/claim',{'world_id':lobby['world_id'],'mode':'mensch','volk':'aurelianer'},auth['token'])
        with sync_playwright() as p:
            browser = p.chromium.launch(channel='chrome',headless=True)
            page = browser.new_page(viewport={'width':1440,'height':1000},device_scale_factor=1)
            errors=[]
            page.on('pageerror',lambda error: errors.append(str(error)))
            page.route('**/config.js',lambda route: route.fulfill(content_type='application/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};'))
            page.add_init_script('sessionStorage.setItem("sternenepoche-session",'+json.dumps(json.dumps({'server':API,**auth}))+');')
            page.goto(WEB+'/web-client/',wait_until='networkidle')
            page.locator('#game').wait_for(state='visible')
            for screen in SCREENS:
                page.locator('[data-tab="'+screen+'"]').click()
                page.wait_for_timeout(400)
                page.evaluate('window.scrollTo(0,0)')
                page.screenshot(path=str(OUT/(screen+'.png')))
                with Image.open(OUT/(screen+'.png')) as im: im.save(OUT/(screen+'.webp'),'WEBP',quality=85,method=6)
                (OUT/(screen+'.png')).unlink()
            browser.close()
        provenance={'captured':'2026-10-08','source':'web-client in isolated real Rust test world',
            'viewport':[1440,1000],'state':'Paused initial Aurelianer world; no live accounts or secrets',
            'screens':SCREENS,'page_errors':errors}
        (OUT/'capture.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2),encoding='utf-8')
        print(json.dumps({'screenshots':len(SCREENS),'errors':errors,'bytes':sum(x.stat().st_size for x in OUT.glob('*.webp'))}))
        if errors: raise RuntimeError('Client errors during screenshot capture')
    finally:
        web.shutdown()
        server.terminate();server.wait(timeout=15)

if __name__=='__main__': main()
