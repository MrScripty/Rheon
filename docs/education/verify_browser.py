from pathlib import Path
import json
from playwright.sync_api import sync_playwright
HERE=Path(__file__).resolve().parent
with sync_playwright() as p:
    browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--enable-unsafe-swiftshader'])
    page=browser.new_page(viewport={'width':1440,'height':1050},device_scale_factor=1)
    errors=[];failed=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.on('response',lambda r:failed.append(r.url) if r.status>=400 else None)
    page.goto('http://127.0.0.1:8765/index.html'); page.wait_for_load_state('networkidle')
    page.screenshot(path='/tmp/rheon-education-home.png',full_page=True)
    page.goto('http://127.0.0.1:8765/labs.html'); page.wait_for_selector('#metrics dd'); page.wait_for_timeout(500)
    assert page.locator('canvas').count()==1,'WebGL did not initialize'
    checks=[]
    for lab in ['projection','collision','hydrostatic','viscous','slip','cap']:
        page.select_option('#lab',lab);page.wait_for_timeout(80)
        before=page.locator('#metrics').inner_text()
        controls=page.locator('#controls select')
        if controls.count():controls.last.select_option(index=controls.last.locator('option').count()-1)
        ranges=page.locator('#controls input[type=range]')
        if ranges.count():ranges.first.fill(ranges.first.get_attribute('max'));ranges.first.dispatch_event('input')
        after=page.locator('#metrics').inner_text();assert before!=after,lab
        page.screenshot(path=f'/tmp/rheon-education-{lab}.png',full_page=True)
        checks.append({'lab':lab,'control_changes_metrics':True,'readout':after})
    page.select_option('#lab','cap'); page.locator('input[type=range]').fill('12');page.locator('input[type=range]').dispatch_event('input')
    page.screenshot(path='/tmp/rheon-education-cap90.png',full_page=True)
    page.mouse.move(500,500);page.mouse.down();page.mouse.move(610,540,steps=5);page.mouse.up()
    page.locator('#reset-view').click()
    page.goto('http://127.0.0.1:8765/chapters/23-wetting-and-adhesion.html');page.wait_for_load_state('networkidle')
    assert page.locator('.katex-error').count()==0
    assert page.locator('.katex').count()>5
    page.screenshot(path='/tmp/rheon-education-chapter.png',full_page=True)
    mobile=browser.new_page(viewport={'width':390,'height':844},is_mobile=True,has_touch=True)
    mobile.goto('http://127.0.0.1:8765/index.html');mobile.locator('#menu').click();assert mobile.locator('#navigation').is_visible()
    mobile.locator('#search').fill('wetting');assert mobile.locator('.chapter-link:visible').count()==1
    mobile.screenshot(path='/tmp/rheon-education-mobile.png',full_page=True)
    mobile.goto('http://127.0.0.1:8765/labs.html#cap');mobile.wait_for_selector('#metrics dd');assert mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1')
    page.goto('http://127.0.0.1:8765/print.html');page.wait_for_load_state('networkidle');page.evaluate('document.fonts.ready')
    (HERE/'downloads').mkdir(exist_ok=True)
    page.pdf(path=str(HERE/'downloads/Rheon-expanded-book.pdf'),print_background=True,prefer_css_page_size=True,display_header_footer=True,header_template='<div></div>',footer_template='<div style="font-size:9px;width:100%;text-align:center;color:#52676d">Rheon · Puma · <span class="pageNumber"></span> / <span class="totalPages"></span></div>')
    assert not errors,errors
    # PDF is created after the site build; its download is copied on the next build.
    assert not failed,failed
    receipt={'schema':'rheon-education-browser-v1','browser':browser.version,'webgl':True,'labs':checks,'mobile_navigation_search':True,'mobile_no_horizontal_overflow':True,'katex_no_errors':True,'page_errors':errors,'http_failures':failed}
    (HERE/'browser-qualification.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:v for k,v in receipt.items() if k!='labs'},indent=2));browser.close()
