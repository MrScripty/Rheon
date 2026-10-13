"""Actual chapter geometry and controls for finite completion mechanisms."""
from pathlib import Path
import argparse,json,hashlib,math,os,shutil
from playwright.sync_api import sync_playwright
from verify_browser import serve_site
from teaching_browser import snapshot,settle
from engineering_browser import require
from practical_oracle import DOMAINS,verify,scene,close
CASES=[('01-purpose-and-model','gating-a-pressure-proposal','proposal',[.25,0,.125,'finite']),('02-continuum-equations','wall-forces-and-body-work','wallload',[1,1,2]),('05-linear-solvers','two-unknowns-and-a-real-iteration','jacobi',[1,4,'mixed']),('11-particle-methods','one-grid-sample-two-particles','particletransfer',[1,0,1,0,0]),('13-memory-and-parallelism','counting-faces-and-sparse-arrays','memory',[8,4,4]),('15-floating-point','a-unit-lost-in-a-reduction','roundoff',[24,'left','binary32']),('16-formal-contracts','inspect-the-premise-before-the-result','proposal',[.25,0,.125,'finite']),('18-validation-program','midpoint-refinement-with-a-known-answer','refinement',[4,0]),('18-validation-program','refining-a-biased-model','refinement',[4,.0625]),('22-viscosity-and-stress','a-specified-sst-2003-limiter','sstprobe',[1,1,4,1]),('A-algorithm-reference','execute-the-iteration-contract','jacobi',[1,4,'mixed']),('B-reproduction','reproduce-an-arithmetic-counterexample','roundoff',[24,'left','binary32']),('C-bibliography','read-a-model-variant-before-using-it','sstprobe',[1,1,4,1]),('D-notation-and-provenance','units-in-a-force-and-power-ledger','wallload',[1,1,2]),('E-worked-matrix-laboratory','a-dirichlet-matrix-and-its-error-modes','jacobi',[1.5,8,'alternating']),('F-expansion-contracts-and-sources','proof-assumptions-and-runtime-admission','proposal',[0,0,.125,'nonfinite'])]
def check(page):
 s=snapshot(page);q=s['reference'];verify(q);name=q['kind'];require(s['lab']==name,'Wrong active model')
 controls=page.locator('#controls select');require(controls.count()==len(DOMAINS[name]),'Control count')
 require(q['args']==[values[int(controls.nth(i).input_value())] for i,values in enumerate(DOMAINS[name])],'Control ignored')
 expected_boxes,expected_arrows,expected_spheres=scene(q)
 require(len(s['arrows'])==len(expected_arrows),'Arrow count')
 for actual,(o,v,label) in zip(s['arrows'],expected_arrows):
  length=math.hypot(*v);require(actual['method']==label,'Arrow quantity');close(actual['length'],length);close(actual['tipLength'],length);close(actual['shaftLength']+actual['headLength'],length)
  for x,y in zip(actual['origin'],o):close(x,y)
  for x,y in zip(actual['direction'],v):close(x,y/length)
 require(len(s['boxes'])==len(expected_boxes),'Box count')
 for actual,(o,d) in zip(s['boxes'],expected_boxes):
  for x,y in zip(actual['origin'],o):close(x,y)
  for x,y in zip(actual['size'],d):close(x,y)
 require(len(s['spheres'])==len(expected_spheres),'Sphere count')
 for actual,(o,r) in zip(s['spheres'],expected_spheres):
  close(actual['radius'],r)
  for x,y in zip(actual['origin'],o):close(x,y)
 require(page.locator('#lab-expected').text_content().strip() not in ['', 'undefined'],'Expected behavior absent')
 require(page.locator('#metrics dd').count()>=4,'Readouts absent')
 for value in page.locator('#metrics dd').all_text_contents():require(not any(token in value for token in ['NaN','Infinity']),'Nonfinite displayed metric')
 require(s['webgl'],'WebGL missing');require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Horizontal overflow');return q

def qualify(site):
 site=Path(site);out=site/'practical-evidence';out.mkdir(exist_ok=True);records=[];errors=[]
 with serve_site(site) as base,sync_playwright() as p:
  browser=p.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'),headless=True,chromium_sandbox=True,args=['--enable-unsafe-swiftshader'])
  for width,height in [(1440,1000),(390,844),(320,740)]:
   page=browser.new_page(viewport={'width':width,'height':height});page.on('pageerror',lambda e:errors.append(str(e)));page.on('response',lambda r:errors.append(str(r.status)+' '+r.url) if r.status>=400 else None)
   for chapter,anchor,name,args in CASES:
    page.goto(base+'/chapters/'+chapter+'.html#'+anchor,wait_until='networkidle');page.wait_for_function('(name)=>document.querySelector("#lab").value===name',arg=name)
    if width<760:page.locator('[data-reading-mode="example"]').click()
    settle(page);q=check(page);require(q['args']==args,'Heading preset')
    proof=page.locator('details').filter(has=page.locator('summary',has_text='Practical study proof scope'));require(proof.count()==1 and proof.get_attribute('open') is None,'Proof must start collapsed')
    page.screenshot(path=str(out/f'{anchor}-{width}.png'))
    notes=page.locator('details.example-notes');require(notes.get_attribute('open') is None,'Expected-result notes start collapsed');notes.locator('summary').click();settle(page);require(page.locator('#lab-expected').is_visible() and page.locator('#lab-expected').inner_text().strip() not in ['', 'undefined'],'Expanded expected result unavailable');require(check(page)==q,'Disclosure changes data');notes.locator('summary').click();settle(page)
    for control in page.locator('#controls select').all():
     control.select_option('0');settle(page);check(page);control.select_option(str(control.locator('option').count()-1));settle(page);check(page)
    before=snapshot(page);pixels=page.locator('#scene canvas').screenshot();page.locator('#scene').press('ArrowRight');settle(page);require(snapshot(page)['camera']!=before['camera'],'Camera fixed');require(check(page)==before['reference'],'Camera changes data');require(page.locator('#scene canvas').screenshot()!=pixels,'Camera must change actual rendered canvas')
    if width<760:
     for mode in ['text','split']:page.locator(f'[data-reading-mode="{mode}"]').click();settle(page);require(check(page)==before['reference'],'Layout changes data')
    records.append({'viewport':[width,height],'anchor':anchor,'initial':q})
   page.goto(base+'/chapters/18-validation-program.html#midpoint-refinement-with-a-known-answer',wait_until='networkidle');page.wait_for_function('document.querySelector("#lab").value==="refinement"');settle(page)
   page.evaluate("""() => {const t=document.getElementById('refining-a-biased-model');scrollTo(0,scrollY+t.getBoundingClientRect().top-document.querySelector('header').getBoundingClientRect().bottom-31);} """)
   page.wait_for_function('document.querySelector(".current-lesson").id==="refining-a-biased-model"');settle(page);require(check(page)['args'][-1]==.0625,'Scroll preset');page.close()
  version=browser.version;browser.close()
 require(not errors,'Browser/network errors '+str(errors))
 receipt={'source_head':json.loads((site/'build-receipt.json').read_text())['source_base'],'chromium':version,'states':records,'sources':{n:hashlib.sha256((site/n).read_bytes()).hexdigest() for n in ['practical_reference.js','practical_labs.js','labs.js','reader.js']},'screenshots':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.png')},'scope':'Finite educational mechanisms; no production/PDE/IEEE, performance or human visual-review claim.'}
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS practical lessons: real controls, geometry, camera, scroll and three viewport sizes')
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path,required=True);qualify(parser.parse_args().output_dir)
