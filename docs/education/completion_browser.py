"""Actual chapter geometry and controls for finite completion mechanisms."""
from pathlib import Path
import argparse,json,hashlib,math,os,shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot,settle
from engineering_browser import require
from completion_oracle import DOMAINS,verify,glyphs,boxes,close
CASES=[('08-boundary-geometry','skew-samples-and-normal-gradients','reconstruction',[.5,1,1,1,'two-point']),('08-boundary-geometry','an-affine-correction-with-a-third-sample','reconstruction',[.5,1,1,1,'corrected']),('04-pressure-projection','continuity-is-not-momentum-convergence','coupling',[1,4]),('04-pressure-projection','an-overrelaxed-two-path-counterexample','coupling',[1.5,8])]
def check(page):
 s=snapshot(page);q=s['reference'];verify(q);name=q['kind'];require(s['lab']==name,'Wrong active model')
 controls=page.locator('#controls select');require(controls.count()==len(DOMAINS[name]),'Control count')
 require(q['args']==[values[int(controls.nth(i).input_value())] for i,values in enumerate(DOMAINS[name])],'Control ignored')
 expected=[(o,v,label) for o,v,label in glyphs(q) if math.hypot(*v)>=1e-8];require(len(s['arrows'])==len(expected),'Arrow count')
 for a,(o,v,label) in zip(s['arrows'],expected):
  length=math.hypot(*v);require(a['method']==label,'Arrow quantity');close(a['length'],length)
  for x,y in zip(a['origin'],o):close(x,y)
  for x,y in zip(a['direction'],v):close(x,y/length)
 expected=boxes(q);require(len(s['boxes'])==len(expected),'Box count')
 for a,(o,d) in zip(s['boxes'],expected):
  for x,y in zip(a['origin'],o):close(x,y)
  for x,y in zip(a['size'],d):close(x,y)
 if name=='reconstruction':
  require(len(s['spheres'])==3,'Sample count')
  for a,p in zip(s['spheres'],q['points']):
   close(a['radius'],.03)
   for x,y in zip(a['origin'],p):close(x,.55*y)
 require(s['webgl'],'WebGL missing');require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Horizontal overflow');return q

def qualify(site):
 site=Path(site);out=site/'completion-evidence';out.mkdir(exist_ok=True);records=[];errors=[]
 with serve_site(site) as base,sync_playwright() as p:
  browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
  for width,height in [(1440,1000),(390,844),(320,740)]:
   page=browser.new_page(viewport={'width':width,'height':height});page.on('pageerror',lambda e:errors.append(str(e)));page.on('response',lambda r:errors.append(str(r.status)+' '+r.url) if r.status>=400 else None)
   for chapter,anchor,name,args in CASES:
    page.goto(base+'/chapters/'+chapter+'.html#'+anchor,wait_until='networkidle');page.wait_for_function('(name)=>document.querySelector("#lab").value===name',arg=name)
    if width<760:page.locator('[data-reading-mode="example"]').click()
    settle(page);q=check(page);require(q['args']==args,'Heading preset')
    proof=page.locator('details').filter(has=page.locator('summary',has_text='Reconstruction and coupling proof scope'));require(proof.count()==1 and proof.get_attribute('open') is None,'Proof must start collapsed')
    page.screenshot(path=str(out/f'{anchor}-{width}.png'))
    for control in page.locator('#controls select').all():
     control.select_option('0');settle(page);check(page);control.select_option(str(control.locator('option').count()-1));settle(page);check(page)
    before=snapshot(page);page.locator('#scene').press('ArrowRight');settle(page);require(snapshot(page)['camera']!=before['camera'],'Camera fixed');require(check(page)==before['reference'],'Camera changes data')
    if width<760:
     for mode in ['text','split']:page.locator(f'[data-reading-mode="{mode}"]').click();settle(page);require(check(page)==before['reference'],'Layout changes data')
    records.append({'viewport':[width,height],'anchor':anchor,'initial':q})
   page.goto(base+'/chapters/08-boundary-geometry.html#skew-samples-and-normal-gradients',wait_until='networkidle');page.wait_for_function('document.querySelector("#lab").value==="reconstruction"');settle(page)
   page.evaluate("""() => {const t=document.getElementById('an-affine-correction-with-a-third-sample');scrollTo(0,scrollY+t.getBoundingClientRect().top-document.querySelector('header').getBoundingClientRect().bottom-31);} """)
   page.wait_for_function('document.querySelector(".current-lesson").id==="an-affine-correction-with-a-third-sample"');settle(page);require(check(page)['args'][-1]=='corrected','Scroll preset');page.close()
  version=browser.version;browser.close()
 require(not errors,'Browser/network errors '+str(errors))
 receipt={'source_head':json.loads((site/'build-receipt.json').read_text())['source_base'],'chromium':version,'states':records,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['completion_reference.js','completion_labs.js','labs.js','reader.js']},'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.png')},'scope':'Finite educational mechanisms; no production/PDE/IEEE, performance or human visual-review claim.'}
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS completion lessons: real controls, geometry, camera, scroll and three viewport sizes')
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path,required=True);qualify(parser.parse_args().output_dir)
