"""Check real startup, delayed world data, fallback and error handling in Chrome."""
from pathlib import Path
import argparse,json,subprocess,time,urllib.request
from playwright.sync_api import sync_playwright,expect
ROOT=Path(__file__).resolve().parents[1]
RUN=ROOT/'laeufe/website-2026-10-08'
API='http://127.0.0.1:18994'

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--server-exe',type=Path,default=ROOT/'target/release/sternenepoche-server.exe');args=parser.parse_args()
    process=subprocess.Popen([str(args.server_exe),'--data',str(RUN/'loader-test-world'),'--bind','127.0.0.1:18994','--admin-bind','127.0.0.1:18995'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW)
    report=[]
    try:
        for _ in range(100):
            try:urllib.request.urlopen(API+'/api/lobby',timeout=1).close();break
            except OSError:time.sleep(.1)
        else:raise RuntimeError('Isolated test server did not start')
        for path,mime in [('loading-screen.js','text/javascript'),('loading-screen.css','text/css'),('brand/neuralstern-768.webp','image/webp'),('brand/neuralstern-768.gif','image/gif'),('brand/neuralstern-poster.jpg','image/jpeg')]:
            with urllib.request.urlopen(API+'/'+path) as response:assert response.status==200 and mime in response.headers['Content-Type']
        with sync_playwright() as p:
            browser=p.chromium.launch(channel='chrome',headless=True)
            for width,height in [(1440,1000),(390,844)]:
                page=browser.new_page(viewport={'width':width,'height':height});errors=[];held=[]
                page.on('pageerror',lambda e:errors.append(str(e)))
                page.route('**/config.js',lambda r:r.fulfill(content_type='text/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};'))
                page.route('**/api/rules',lambda r:held.append(r))
                page.goto(API,wait_until='domcontentloaded')
                loader=page.locator('#sternen-loader');expect(loader).to_be_visible()
                expect(page.locator('[data-loader-status]')).to_have_text('Regeln der Welt laden …')
                assert page.locator('main').evaluate('(node)=>node.inert')
                expect(loader.locator('img')).to_have_js_property('naturalWidth',768)
                assert loader.locator('img').bounding_box()['width']>250
                page.wait_for_timeout(4600)
                page.screenshot(path=str(RUN/f'loading-game-{width}.png'))
                response=held[0].fetch();held[0].fulfill(response=response)
                expect(loader).to_be_hidden();expect(page.locator('#ranking table')).to_be_visible()
                assert not page.locator('main').evaluate('(node)=>node.inert')
                assert not errors,errors
                report.append({'width':width,'delayed_data':True,'assets_served_by_rust':True,'ready_screen_dismissed':True,'errors':errors});page.close()
            page=browser.new_page(reduced_motion='reduce');requests=[]
            page.on('request',lambda r:requests.append(r.url))
            page.route('**/config.js',lambda r:r.fulfill(content_type='text/javascript',body='window.STERNENEPOCHE={api:'+json.dumps(API)+'};'))
            page.route('**/api/rules',lambda r:r.abort())
            page.goto(API,wait_until='domcontentloaded');expect(page.locator('#sternen-loader')).to_be_hidden()
            assert not any('768.webp' in u or '768.gif' in u for u in requests)
            expect(page.locator('#server')).to_be_visible();expect(page.locator('#message')).to_be_visible();page.close()
            page=browser.new_page(java_script_enabled=False);page.goto(API);expect(page.locator('#sternen-loader')).to_be_hidden();page.close()
            browser.close()
        (RUN/'loading-verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
        print('PASS: large desktop/mobile loader, real delayed world data, Rust asset routes, reduced motion, server error, accessible page after readiness and no-JS fallback.')
    finally:process.terminate();process.wait(timeout=15)

if __name__=='__main__':main()
