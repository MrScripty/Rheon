"""Sandboxed actual-render checks for the finite pressure-gauge lesson."""
from pathlib import Path
import argparse,json,hashlib,math,os,shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot,settle,metrics,number
from engineering_browser import require
from gauge_oracle import verify,glyphs,boxes,close

def check(page):
    s=snapshot(page);q=s['reference'];require(s['lab']=='gauges','Wrong lesson');verify(q)
    controls=[('input',['compatible','global-only']),('geometry',['uniform','unequal']),('mode',['fixed','mean','volume','projected']),('shiftA',[-2,0,2]),('shiftB',[-2,0,2]),('error',[0,.25])]
    selects=page.locator('#controls select');require(selects.count()==len(controls),'Control count')
    for i,(key,values) in enumerate(controls):require(q[key]==values[int(selects.nth(i).input_value())],'Ignored control '+key)
    expected=[(o,v,label) for o,v,label in glyphs(q) if math.hypot(*v)>=1e-8]
    require(len(s['arrows'])==len(expected),'Flux glyph count')
    for a,(o,v,label) in zip(s['arrows'],expected):
        length=math.hypot(*v);require(a['method']==label,'Glyph quantity');close(a['length'],length)
        for actual,want in zip(a['origin'],o):close(actual,want)
        for actual,want in zip(a['direction'],v):close(actual,want/length)
    wanted=boxes(q);require(len(s['boxes'])==len(wanted),'Cell/scalar bar count')
    for a,(o,d,color) in zip(s['boxes'],wanted):
        require(a['color']==color,'Scalar sign colour')
        for actual,want in zip(a['origin'],o):close(actual,want)
        for actual,want in zip(a['size'],d):close(actual,want)
    m=metrics(page)
    require(m['Original-system status'].startswith('Accepted')==q['acceptedOriginal'],'False displayed acceptance')
    for title,key in [('Full residual maximum','fullResidual'),('Retained-row residual maximum','reducedResidual'),('Maximum absolute divergence (1/s)','maxDivergence'),('Boundary-work change from offsets (W)','boundaryShift')]:
        require(abs(number(m,title)-q[key])<=.000051,'Rounded readout mismatch '+key)
    require(s['webgl'],'WebGL missing');require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Horizontal overflow')
    return q

def qualify(site):
    site=Path(site);out=site/'gauge-evidence';out.mkdir(exist_ok=True);records=[];errors=[]
    anchors=['two-disconnected-pressure-regions','global-balance-can-hide-component-errors','fixed-cell-and-mean-zero-gauges','volume-weighted-pressure-gauges','projecting-a-source-changes-the-problem','gauge-shifts-and-boundary-work','a-pressure-error-is-not-a-gauge']
    with serve_site(site) as base,sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
        for width,height in [(1440,1000),(390,844),(320,740)]:
            page=browser.new_page(viewport={'width':width,'height':height})
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.on('response',lambda r:errors.append(str(r.status)+' '+r.url) if r.status>=400 else None)
            for index,anchor in enumerate(anchors):
                page.goto(base+'/chapters/05-linear-solvers.html#'+anchor,wait_until='networkidle')
                page.wait_for_function('document.querySelector("#lab").value==="gauges"')
                if width<760:page.locator('[data-reading-mode="example"]').click()
                settle(page);q=check(page)
                require(q['mode']==['fixed','fixed','mean','volume','projected','fixed','fixed'][index],'Heading gauge preset')
                require(q['input']==('global-only' if index in [1,4,5] else 'compatible'),'Heading data preset')
                if index==1:close(q['reducedResidual'],0);close(q['fullResidual'],1)
                if index==4:close(max(map(abs,q['usedResidual'])),0);close(q['fullResidual'],.5)
                if index==5:close(q['boundaryShift'],-4)
                if index==6:close(q['fullResidual'],.125)
                proof=page.locator('details').filter(has=page.locator('summary',has_text='Exact gauge contracts and their limits'))
                require(proof.count()==1 and proof.get_attribute('open') is None,'Proof must start collapsed')
                page.screenshot(path=str(out/f'{anchor}-{width}.png'))
                for control in page.locator('#controls select').all():
                    control.select_option('0');settle(page);check(page)
                    control.select_option(str(control.locator('option').count()-1));settle(page);check(page)
                before=snapshot(page);page.locator('#scene').press('ArrowRight');settle(page)
                require(snapshot(page)['camera']!=before['camera'],'Camera did not rotate');require(check(page)==before['reference'],'Camera changed data')
                if width<760:
                    for mode in ['text','split']:
                        page.locator(f'[data-reading-mode="{mode}"]').click();settle(page);require(check(page)==before['reference'],'Layout changed data')
                records.append({'viewport':[width,height],'anchor':anchor,'initial':q})
            page.goto(base+'/chapters/05-linear-solvers.html#'+anchors[0],wait_until='networkidle');page.wait_for_function('document.querySelector("#lab").value==="gauges"');settle(page)
            page.evaluate("""() => {const target=document.getElementById('global-balance-can-hide-component-errors');scrollTo(0,scrollY+target.getBoundingClientRect().top-document.querySelector('header').getBoundingClientRect().bottom-31);} """)
            page.wait_for_function('document.querySelector(".current-lesson").id==="global-balance-can-hide-component-errors"');settle(page)
            require(check(page)['input']=='global-only','Scroll did not change source');require(page.evaluate('location.hash')=='#'+anchors[0],'Scroll navigated')
            page.close()
        version=browser.version;browser.close()
    require(not errors,'Browser/network errors '+str(errors))
    receipt={'source_head':json.loads((site/'build-receipt.json').read_text())['source_base'],'chromium':version,'states':records,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['gauge_reference.js','gauge_lab.js','labs.js','reader.js']},'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.png')},'scope':'Finite exact model checks; no production/PDE/IEEE/performance or human visual-review claim.'}
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS gauges: 288-state oracle; three viewports, seven headings, controls, scalar bars, face arrows and scroll')
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path,required=True);qualify(parser.parse_args().output_dir)
