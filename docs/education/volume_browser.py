"""Check the generated chapter in sandboxed Chromium, retaining source-bound evidence."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot, settle


def require(value,message):
    if not value: raise ValueError(message)


def qualify(site):
    site=Path(site)
    evidence=site/'volume-evidence';evidence.mkdir(exist_ok=True)
    errors=[];states=[]
    with serve_site(site) as url, sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
        version=browser.version
        for width,height in [(1440,1000),(390,844),(320,740)]:
            page=browser.new_page(viewport={'width':width,'height':height})
            page.on('pageerror',lambda error:errors.append(str(error)))
            page.on('response',lambda response:errors.append(str(response.status)+' '+response.url) if response.status>=400 else None)
            page.goto(url+'/chapters/09-liquid-surfaces.html#volume-fractions',wait_until='networkidle')
            page.wait_for_function("document.querySelector('#lab').value==='volume' && document.querySelector('#controls input')")
            if width<760:
                page.locator('[data-reading-mode="example"]').click()
            settle(page)
            initial=snapshot(page)
            require(initial['webgl'],'WebGL unavailable')
            require(abs(initial['reference']['q'])<1e-14,'Left preset differs')
            require(not page.locator('.proof-disclosure').get_attribute('open'),'Proof should start collapsed')
            page.get_by_label('Liquid location',exact=True).select_option('1')
            settle(page)
            q=snapshot(page)['reference']
            require(abs(q['q']-.3)<1e-14 and abs(q['clampDrift']-.2)<1e-14,'Right preset/ledger mismatch')
            require(page.locator('#follow-reading').get_attribute('aria-pressed')=='false','Manual control did not pause follow')
            page.get_by_label('Initial liquid fraction (%)',exact=True).fill('80')
            page.get_by_label('Swept width (% of cell)',exact=True).fill('20')
            settle(page)
            q=snapshot(page)['reference']
            require(abs(q['donor']-.6)<1e-14 and abs(q['receiver']-.2)<1e-14,'Sliders did not recompute')
            page.locator('#scene').press('ArrowRight');settle(page)
            require(snapshot(page)['camera']!=initial['camera'],'Camera controls did not rotate')
            require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Horizontal overflow')
            page.screenshot(path=str(evidence/f'volume-{width}.png'))
            if width<760:
                for mode in ['text','split']:
                    page.locator(f'[data-reading-mode="{mode}"]').click();settle(page)
                    require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),mode+' overflow')
                    require(abs(snapshot(page)['reference']['q']-.2)<1e-14,mode+' changed mechanism')
            # A real heading deep link must select the right-filled comparison.
            page.goto(url+'/chapters/09-liquid-surfaces.html#piecewise-linear-interface-reconstruction',wait_until='networkidle')
            page.wait_for_function("document.querySelector('#lab').value==='volume' && document.querySelector('#controls select')?.value==='1'")
            states.append({'viewport':[width,height],'left':initial['reference'],'manual':q,'deep_link':snapshot(page)['reference']})
            page.close()
        browser.close()
    require(not errors,'Browser errors: '+str(errors))
    receipt={'schema':'rheon-volume-browser-v1','chromium':version,'states':states,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['volume_reference.js','volume_labs.js','labs.js','reader.js','chapters/09-liquid-surfaces.html']},'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in evidence.glob('*.png')},'scope':'Generated chapter controls, WebGL, camera, mobile modes and deep links; not native liquid qualification.'}
    (evidence/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print('PASS volume chapter desktop, 390px/320px phone, controls, geometry ledger and deep links')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output-dir',required=True,type=Path)
    qualify(parser.parse_args().output_dir)
