"""Real chapter/control/glyph checks; numerical oracle is exact and independent."""
from pathlib import Path
import argparse,json,hashlib,math,os,shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot,settle
from engineering_oracle import verify,glyphs,close

def require(x,message):
    if not x:raise ValueError(message)

def check(page):
    state=snapshot(page);q=state['reference'];lab=state['lab'];verify(q,lab)
    expected=[]
    for origin,vector,label in glyphs(q,lab):
        length=math.sqrt(sum(v*v for v in vector))
        if length>=1e-8:expected.append((origin,vector,label,length))
    require(len(state['arrows'])==len(expected),'Rendered glyph count')
    for actual,(origin,vector,label,length) in zip(state['arrows'],expected):
        require(actual['method']==label,'Glyph physical quantity')
        close(actual['length'],length)
        for a,b in zip(actual['origin'],origin):close(a,b)
        for a,b in zip(actual['direction'],vector):close(a,b/length)
    boxes=state['boxes']
    if lab=='faceflux':
        wanted=[([-.4,0,0],[.8,.8,.8]),([.4,0,0],[.8,.8,.8]),([0,0,0],[.012,.8*math.sqrt(q['area']),.8*math.sqrt(q['area'])])]
    elif lab=='checker':wanted=[([(i-1.5)*.4,0,0],[.4,.4,.4]) for i in range(4)]
    else:wanted=[([0,0,0],[1,1,1])]
    require(len(boxes)==len(wanted),'Rendered cell/opening count')
    for i,(actual,(origin,size)) in enumerate(zip(boxes,wanted)):
        for a,b in zip(actual['origin'],origin):close(a,b)
        for a,b in zip(actual['size'],size):close(a,b)
        if lab=='checker':require(actual['color']==(0x067d91 if q['pressure'][i]>=0 else 0xaa5735),'Pressure sign colour')
    require(state['webgl'],'WebGL renderer missing')
    require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Horizontal overflow')
    return q

def qualify(site):
    site=Path(site);out=site/'engineering-evidence';out.mkdir(exist_ok=True)
    errors=[];records=[]
    cases=[('08-boundary-geometry','two-cells-and-boundary-work','faceflux'),('08-boundary-geometry','balanced-flow-is-not-zero-boundary-work','faceflux'),('04-pressure-projection','a-collocated-checkerboard','checker'),('04-pressure-projection','constant-pressure-control','checker'),('22-viscosity-and-stress','affine-rotation-and-shear','stressprobe'),('22-viscosity-and-stress','boundary-work-supplies-shear','stressprobe'),('22-viscosity-and-stress','extension-control','stressprobe')]
    with serve_site(site) as base,sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
        for width,height in [(1440,1000),(390,844),(320,740)]:
            page=browser.new_page(viewport={'width':width,'height':height})
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.on('response',lambda r:errors.append(str(r.status)+' '+r.url) if r.status>=400 else None)
            for chapter,anchor,lab in cases:
                page.goto(base+'/chapters/'+chapter+'.html#'+anchor,wait_until='networkidle')
                page.wait_for_function('(lab)=>document.querySelector("#lab").value===lab && document.querySelector("#controls select")',arg=lab)
                if width<760:page.locator('[data-reading-mode="example"]').click()
                settle(page);q=check(page)
                if anchor=='two-cells-and-boundary-work':close(q['pressureWork'],-2)
                if anchor=='balanced-flow-is-not-zero-boundary-work':close(q['pressureWork'],0);close(q['boundaryWork'],1)
                if anchor=='a-collocated-checkerboard':close(q['divergence'][0],4)
                if anchor=='constant-pressure-control':require(q['pattern']=='constant','Constant preset')
                if lab=='stressprobe':require(q['kind']=={'affine-rotation-and-shear':'rotation','boundary-work-supplies-shear':'shear','extension-control':'extension'}[anchor],'Stress preset')
                require(page.locator('.proof-disclosure[open]').count()==0,'Proof must start collapsed')
                page.screenshot(path=str(out/f'{anchor}-{width}.png'))
                # Every control is operated; the scene must remain bound to its readout.
                for control in page.locator('#controls select').all():
                    control.select_option('0');settle(page);check(page)
                    control.select_option(str(control.locator('option').count()-1));settle(page);check(page)
                before=snapshot(page);page.locator('#scene').press('ArrowRight');settle(page)
                require(snapshot(page)['camera']!=before['camera'],'Camera did not rotate')
                require(check(page)==before['reference'],'Camera changed physical state')
                if width<760:
                    for mode in ['text','split']:
                        page.locator(f'[data-reading-mode="{mode}"]').click();settle(page);require(check(page)==before['reference'],'Reading mode changed state')
                records.append({'viewport':[width,height],'chapter':chapter,'anchor':anchor,'initial':q})
            page.close()
        version=browser.version;browser.close()
    require(not errors,'Browser/network errors '+str(errors))
    receipt={'source_head':json.loads((site/'build-receipt.json').read_text())['source_base'],'chromium':version,'states':records,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['engineering_reference.js','engineering_labs.js','labs.js','reader.js']},'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.png')},'scope':'Prescribed finite examples; no production/PDE/IEEE or visual-review claim.'}
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print('PASS engineering: three viewports, seven heading states, every control and actual glyph geometry')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path,required=True);qualify(parser.parse_args().output_dir)
