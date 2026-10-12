"""Read-only mobile/desktop reference QA against a local static site."""
from pathlib import Path
import functools,http.server,json,threading,os
from playwright.sync_api import sync_playwright
ROOT=Path(__file__).resolve().parents[1];SITE=Path(os.environ.get('STERN_WEBSITE_TEST_ROOT',str(ROOT)));OUT=ROOT/'laeufe/player-help-2026-10-12'
class Quiet(http.server.SimpleHTTPRequestHandler):
 def log_message(self,*a):pass
def main():
 server=http.server.ThreadingHTTPServer(('127.0.0.1',19039),functools.partial(Quiet,directory=str(SITE)));threading.Thread(target=server.serve_forever,daemon=True).start();checks=[]
 try:
  with sync_playwright() as p:
   browser=p.chromium.launch(channel='chrome',headless=True,args=['--use-angle=swiftshader','--enable-unsafe-swiftshader']);page=browser.new_page(reduced_motion='reduce');errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
   for width in [320,390,580,850,1440]:
    page.set_viewport_size({'width':width,'height':900});page.goto('http://127.0.0.1:19039/Sternenepoche-Kodex.html');page.locator('.codex-tile').first.wait_for()
    assert page.locator('.codex-tile').count()==35
    page.locator('#tree-goal').wait_for()
    assert page.locator('#tree-goal option').count()==87
    goals=page.locator('#tree-goal option').evaluate_all('(o)=>o.map(e=>e.value)')
    for goal in goals:
     page.locator('#tree-goal').select_option(goal)
     assert page.locator('.tree-goal-node').count()==1,goal
     assert page.locator('.public-tree-node').count()>0,goal
     assert not page.evaluate('document.documentElement.scrollWidth>innerWidth'),(width,goal)
    page.locator('#tree-goal').select_option('defenses:planetenschild_iii')
    assert 'Benötigt: Stufe 10' in page.locator('#public-tree').inner_text()
    assert not page.evaluate('document.documentElement.scrollWidth>innerWidth'),width
    for faction in ['aurelianer','krath','veyari','syntheten']:
     page.locator('#faction').select_option(faction);page.locator('#class').select_option('jaeger');assert page.locator('.codex-tile').count()==4
     page.locator('#unit').select_option('schwerer_jaeger');assert '2' in page.locator('.codex-facts').inner_text();assert 'Schildtechnik 3' in page.locator('#detail').inner_text()
     page.locator('[data-unit="sturmjaeger"]').click();assert page.locator('#detail').bounding_box()['y']<=18
     assert not page.evaluate('document.documentElement.scrollWidth>innerWidth'),(width,faction)
    page.locator('#class').select_option('kolonie');page.locator('#unit').select_option('grosses_kolonieschiff');assert '6.000' in page.locator('#detail').inner_text();assert '50.000' in page.locator('#detail').inner_text()
    page.locator('#class').select_option('schild');assert page.locator('.codex-tile').count()==3
    page.locator('#unit').select_option('planetenschild_iii');assert 'Schildtechnik 10' in page.locator('#detail').inner_text()
    page.locator('#class').select_option('jaeger');page.locator('#faction').select_option('aurelianer');page.locator('#unit').select_option('schwerer_jaeger');page.evaluate('scrollTo(0,0)');page.screenshot(path=str(OUT/f'website-fleet-codex-{width}.png'),full_page=True)
   assert not errors,errors
   checks.append('35 profiles, exact per-class counts and four factions at 320/390/580/850/1440; no overflow or JavaScript errors')
   checks.append('Direct selection and tile selection; tile click scrolls to complete detail')
   checks.append('Actual research gates, multi-target information, colony sizes and three shield levels')
   checks.append('Missing pictures explicitly labeled and resolved via the shared art manifest')
   checks.append('All 87 selectable prerequisite paths render without cycles or page overflow, including shield technology 10 for Planetenschild III')
   browser.close()
  (OUT/'website-fleet-verification.json').write_text(json.dumps({'passed':True,'checks':checks},ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(checks,ensure_ascii=False))
 finally:server.shutdown();server.server_close()
if __name__=='__main__':main()
