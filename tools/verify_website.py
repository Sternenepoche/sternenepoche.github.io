"""Browser checks for actual images, responsive layout and guide interactions."""
from pathlib import Path
import json
from playwright.sync_api import sync_playwright
from playwright.sync_api import expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'laeufe/website-2026-10-08'
BASE = 'http://127.0.0.1:18888'

def main():
    report = []
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome',headless=True)
        for path in ['index.html','Sternenepoche-Start.html','Sternenepoche-Handbuch.html']:
            for width,height in [(1440,1000),(390,844)]:
                page = browser.new_page(viewport={'width':width,'height':height})
                errors=[];bad_responses=[]
                page.on('pageerror',lambda e:errors.append(str(e)))
                page.on('response',lambda r:bad_responses.append(r.url) if r.status>=400 else None)
                page.goto(BASE+'/'+path,wait_until='networkidle')
                page.evaluate('document.querySelectorAll("img").forEach(i => i.loading="eager")')
                page.wait_for_function('Array.from(document.images).filter(i=>i.hasAttribute("src")).every(i=>i.complete)')
                issues=page.evaluate('''() => ({
                  overflow:document.documentElement.scrollWidth>innerWidth,
                  brokenImages:[...document.images].filter(i=>i.hasAttribute('src')&&!i.naturalWidth).map(i=>i.alt),
                  brokenAnchors:[...document.querySelectorAll('a[href^="#"]')].filter(a=>a.hash.length>1&&!document.getElementById(decodeURIComponent(a.hash.slice(1)))).map(a=>a.hash),
                  images:document.images.length,
                  h1:document.querySelectorAll('h1').length,
                  structured:[...document.querySelectorAll('script[type="application/ld+json"]')].map(s=>JSON.parse(s.textContent)['@type'])
                })''')
                assert not issues['overflow'],(path,width,'horizontal overflow')
                assert not issues['brokenImages'],(path,issues['brokenImages'])
                assert not issues['brokenAnchors'],(path,issues['brokenAnchors'])
                assert issues['h1']==1
                page.screenshot(path=str(OUT/(Path(path).stem+'-'+str(width)+'.png')))
                if path=='Sternenepoche-Start.html':
                    page.locator('#guide-query').fill('Saven')
                    assert page.locator('#guide-search-results li').count()>0
                    page.locator('#guide-query').fill('')
                    if width>800:
                        page.locator('[data-screen="flotten"]').click()
                    else:
                        page.locator('#screen-select').select_option('flotten')
                    assert page.locator('#bedienweg-flotten').is_visible()
                    assert page.locator('#bedienweg-reich').is_hidden()
                    page.locator('#guide-title').scroll_into_view_if_needed()
                    page.screenshot(path=str(OUT/('guide-detail-'+str(width)+'.png')))
                    page.locator('label[for="race-krath"]').click()
                    assert page.locator('#krath-panel').is_visible()
                page.locator('.zoom').first.click()
                assert page.locator('#image-dialog').is_visible()
                page.keyboard.press('Escape')
                assert not page.locator('#image-dialog').is_visible()
                page.emulate_media(reduced_motion='reduce')
                if path!='Sternenepoche-Handbuch.html':
                    expect(page.locator('body')).to_have_attribute('data-motion','paused')
                    page.locator('#space-toggle').click()
                    assert page.locator('body').get_attribute('data-motion')=='running'
                assert not errors,(path,errors)
                assert not bad_responses,(path,bad_responses)
                report.append({'page':path,'width':width,**issues,'errors':errors})
                page.close()
        # Content and navigation survive when scripting is disabled.
        page = browser.new_page(java_script_enabled=False,viewport={'width':390,'height':844})
        page.goto(BASE+'/Sternenepoche-Start.html')
        assert page.locator('.screen-panel:visible').count()==17
        assert page.locator('.guide-chapter').count()==22
        browser.close()
    (OUT/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print('PASS: 3 pages x desktop/mobile; images, anchors, layout, search, area selection, faction selection, screenshot zoom, reduced motion, no-JS reading and CSP/script errors.')

if __name__=='__main__': main()
