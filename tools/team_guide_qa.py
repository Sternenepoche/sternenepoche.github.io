from pathlib import Path
import functools,http.server,threading,json,sys
from playwright.sync_api import sync_playwright
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'laeufe/website-2026-10-08/publish-screenshots/.pages-artifact/team-ready'
OUT=ROOT/'laeufe/team-2026-10-08/guide-review';OUT.mkdir(exist_ok=True,parents=True)
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self,*args):pass
def main():
    server=http.server.ThreadingHTTPServer(('127.0.0.1',18892),functools.partial(Quiet,directory=str(SOURCE)))
    threading.Thread(target=server.serve_forever,daemon=True).start()
    root=sys.argv[1] if len(sys.argv)>1 else 'http://127.0.0.1:18892'
    errors=[];failed=[]
    try:
        with sync_playwright() as p:
            browser=p.chromium.launch(channel='chrome',headless=True)
            page=browser.new_page(viewport={'width':1440,'height':1100})
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.on('response',lambda r:failed.append([r.status,r.url]) if r.status>=400 else None)
            page.goto(root+'/Sternenepoche-Start.html#agent-setup',wait_until='networkidle');page.locator('#sternen-loader').wait_for(state='hidden');page.locator('#agent-setup').scroll_into_view_if_needed();page.screenshot(path=str(OUT/'desktop-setup.png'))
            assert page.locator('#agent-setup .team-guide-figure').count()==2
            page.locator('#agent-setup a.button').click()
            chapter=page.locator('.guide-chapter[aria-labelledby="guide-20-deinen-eigenen-ki-assistenten-verbinden"]')
            assert chapter.locator('.team-guide-figure').count()==6
            chapter.locator('.team-guide-figure img').evaluate_all("async es=>{await Promise.all(es.map(e=>{e.loading='eager';return e.decode()}));}")
            assert 'Spielentscheidung proben' in chapter.inner_text();assert 'soul.md' in chapter.inner_text();assert 'Loopback' in chapter.inner_text()
            chapter.locator('#team-ollama').scroll_into_view_if_needed();page.screenshot(path=str(OUT/'desktop-ollama.png'))
            page.set_viewport_size({'width':390,'height':844});chapter.locator('#team-openrouter').scroll_into_view_if_needed();page.screenshot(path=str(OUT/'mobile-openrouter.png'));assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+2')
            page.goto(root+'/Sternenepoche-Handbuch.html#team-rollen',wait_until='networkidle');page.locator('#sternen-loader').wait_for(state='hidden');assert page.locator('.team-guide-figure').count()==6;page.locator('#team-rollen').scroll_into_view_if_needed();page.screenshot(path=str(OUT/'mobile-handbook.png'));assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+2')
            page.goto(root+'/web-client/agenten-hilfe.html',wait_until='networkidle');assert page.locator('.team-guide-figure').count()==6;page.locator('.team-guide-figure img').evaluate_all("async es=>{await Promise.all(es.map(e=>{e.loading='eager';return e.decode()}));}");page.screenshot(path=str(OUT/'mobile-game-help.png'));assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+2'),page.evaluate("[...document.querySelectorAll('body *')].filter(e=>e.getBoundingClientRect().right>innerWidth+2).slice(0,12).map(e=>[e.tagName,e.className,e.getBoundingClientRect().width])")
            assert not errors,errors;assert not failed,failed
            browser.close()
        (OUT/'report.json').write_text(json.dumps({'root':root,'passed':True,'page_errors':errors,'failed_requests':failed},indent=2),encoding='utf-8')
        print('PASS six numbered guides, image decode, source links, desktop/mobile pages and standalone game help:',root)
    finally:server.shutdown()
if __name__=='__main__':main()
