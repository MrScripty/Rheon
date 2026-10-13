"""Actual chapter/control/render checks against independent amplification oracles."""
from decimal import Decimal, localcontext
from fractions import Fraction as F
from pathlib import Path
import argparse
import hashlib
import json
import math
import os
import shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot, settle


def require(condition,message):
    if not condition: raise ValueError(message)


def close(actual,want,message):
    require(abs(actual-float(want))<=2e-12,message+f': {actual} != {want}')


def check_field(page):
    state=snapshot(page);q=state['reference'];a=F(q['duration'],q['steps']);j=q['index']
    with localcontext() as ctx:
        ctx.prec=60
        exact=float((-Decimal(q['duration']*j)/Decimal(q['steps'])).exp())
    expected={'explicit':float((1-a)**j),'implicit':float((1/(1+a))**j),'exact':exact}
    for method,value in expected.items():
        close(q['amplitudes'][method],value,method+' amplitude')
        close(q['energy'][method],value*value/4,method+' energy')
    # Inspect real Three.js ArrowHelpers, not only the lesson's readout object.
    methods=[q['method']] if q['method']=='exact' else [q['method'],'exact']
    actual=state['arrows'];offset=0
    for method in methods:
        for x in [.25,.75]:
            for z in [.2,.5,.8]:
                for yindex in range(8):
                    y=(yindex+.5)/8;speed=expected[method]*math.sin(2*math.pi*y)
                    if abs(speed*.18)<1e-8:continue
                    require(offset<len(actual),'Missing rendered arrow')
                    glyph=actual[offset];offset+=1
                    require(glyph['method']==method,'Rendered integrator label')
                    for observed,want in zip(glyph['origin'],[x-.5,y-.5,z-.5]):close(observed,want,'Glyph position')
                    close(glyph['length'],abs(speed)*.18,'Glyph fixed velocity scale')
                    for observed,want in zip(glyph['direction'],[1 if speed>0 else -1,0,0]):close(observed,want,'Glyph direction')
    require(offset==len(actual),'Unexpected rendered arrows')
    require(state['webgl'],'No WebGL renderer')
    return {k:q[k] for k in ['duration','steps','index','method','amplitudes','energy','error']}


def qualify(site):
    site=Path(site);evidence=site/'decay-evidence';evidence.mkdir(exist_ok=True)
    errors=[];states=[]
    with serve_site(site) as base, sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
        version=browser.version
        for width,height in [(1440,1000),(390,844),(320,740)]:
            page=browser.new_page(viewport={'width':width,'height':height})
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.on('response',lambda r:errors.append(f'{r.status} {r.url}') if r.status>=400 else None)
            path=base+'/chapters/12-time-and-error.html'
            page.goto(path+'#restrictions',wait_until='networkidle')
            page.wait_for_function("document.querySelector('#lab').value==='decay' && document.querySelector('#controls input')")
            if width<760:page.locator('[data-reading-mode="example"]').click()
            settle(page);cases=[check_field(page)]
            require(page.locator('.proof-disclosure').get_attribute('open') is None,'Proof must start collapsed')
            require(cases[-1]['amplitudes']['explicit']==-1,'Threshold preset')
            page.screenshot(path=str(evidence/f'decay-threshold-{width}.png'))
            page.get_by_label('Displayed integrator',exact=True).select_option('1');settle(page);cases.append(check_field(page))
            require(page.locator('#follow-reading').get_attribute('aria-pressed')=='false','Manual selection did not pause follow')
            page.get_by_label('Steps across fixed duration',exact=True).select_option('3');settle(page);cases.append(check_field(page))
            close(cases[-1]['amplitudes']['implicit'],F(4,5)**8,'Refined implicit')
            refined=cases[-1]
            page.get_by_label('Displayed integrator',exact=True).select_option('2');settle(page);cases.append(check_field(page))
            page.get_by_label('Displayed integrator',exact=True).select_option('1');settle(page)
            page.get_by_label('Displayed step',exact=True).press('Home');settle(page);cases.append(check_field(page))
            require(cases[-1]['amplitudes']=={'explicit':1,'implicit':1,'exact':1},'Initial state')
            page.get_by_label('Displayed step',exact=True).press('End');settle(page)
            before=snapshot(page);page.locator('#scene').press('ArrowRight');settle(page)
            require(snapshot(page)['camera']!=before['camera'],'Camera did not rotate')
            require(check_field(page)==refined,'Camera changed physical state')
            page.screenshot(path=str(evidence/f'decay-refined-{width}.png'))
            if width<760:
                for mode in ['text','split']:
                    page.locator(f'[data-reading-mode="{mode}"]').click();settle(page)
                    require(check_field(page)==refined,mode+' changed state')
                    require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),mode+' overflow')
            else:require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Desktop overflow')
            for anchor,duration,steps,method,amplitude in [
                ('error-accounting',2,8,'explicit',float(F(3,4)**8)),
                ('long-horizons-and-outcomes',3,1,'explicit',-2),
                ('one-mode-three-time-updates',2,1,'explicit',-1),
                ('implicit-decay-and-fixed-time-refinement',2,1,'implicit',float(F(1,3))),
                ('an-energy-growing-step',3,1,'explicit',-2)]:
                page.goto(path+'#'+anchor,wait_until='networkidle')
                page.wait_for_function("document.querySelector('#lab').value==='decay' && document.querySelector('#controls input')")
                settle(page);q=check_field(page);cases.append(q)
                require(q['steps']==steps and q['duration']==duration and q['method']==method,'Deep-link controls');close(q['amplitudes'][method],amplitude,'Deep-link amplitude')
            require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Final overflow')
            states.append({'viewport':[width,height],'cases':cases});page.close()
        browser.close()
    require(not errors,'Browser/network errors '+str(errors))
    receipt={'schema':'rheon-modal-decay-browser-v1','chromium':version,'source_head':json.loads((site/'build-receipt.json').read_text())['source_base'],'states':states,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['decay_reference.js','decay_labs.js','labs.js','reader.js','chapters/12-time-and-error.html']},'verifier_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in evidence.glob('*.png')},'scope':'Modal time integration and actual glyph geometry; not native solver/PDE/IEEE qualification.'}
    (evidence/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print('PASS modal decay: three viewports, actual glyphs, three integrators, controls and deep links')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output-dir',required=True,type=Path)
    qualify(parser.parse_args().output_dir)
