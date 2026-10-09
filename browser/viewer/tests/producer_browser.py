"""Actual HTTP catalog/recording interactions; supplied existing outputs only."""
import argparse,hashlib,json,shutil,subprocess,sys
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from build import external,PIN

def require(ok,message):
    if not ok:raise AssertionError(message)

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('site','flow','output'):parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--rigid',type=Path)
    args=parser.parse_args();site=args.site.resolve();output=external(args.output);require(not output.exists(),'Fresh evidence output required');output.mkdir(parents=True)
    package=json.loads((site/'package-receipt.json').read_text());require(package['kenoma_commit']==PIN,'Wrong packaged pin')
    for name,sha in package['files_sha256'].items():require(digest(site/name)==sha,'Runtime hash differs')
    source_catalog=json.loads((site/'outputs/catalog.json').read_text());catalog=json.loads(json.dumps(source_catalog))
    for token,state in [('a','incomplete'),('b','failed'),('c','unsupported')]:
        catalog['entries'].append({'id':token*64,'label':'structural-'+state,'state':state,'message':'Test fixture: no completed recording'})
    originals={str(args.flow):digest(args.flow)};flow=json.loads(args.flow.read_text())
    if args.rigid:originals[str(args.rigid)]=digest(args.rigid)
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
    server=ThreadingHTTPServer(('127.0.0.1',0),partial(Handler,directory=str(site.parent)));Thread(target=server.serve_forever,daemon=True).start()
    prefix=f'http://127.0.0.1:{server.server_port}/{site.name}/';checks=[];errors=[]
    try:
        with sync_playwright() as p:
            browser=p.chromium.launch(executable_path=shutil.which('chromium'),headless=True,args=['--no-sandbox','--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader'])
            page=browser.new_page(viewport={'width':1440,'height':1100});page.on('pageerror',lambda e:errors.append(str(e)))
            page.on('response',lambda r:errors.append(str(r.status)+' '+r.url) if r.status>=400 else None)
            page.route('**/outputs/catalog.json',lambda route:route.fulfill(json=catalog))
            page.goto(prefix+'index.html');page.locator('#outputs-summary').filter(has_text='completed recordings').wait_for()
            choices=[entry for entry in source_catalog['entries'] if entry.get('record') and entry['state']=='completed']
            require(page.locator('#outputs option').count()==len(choices)+1,'Completed output choices differ')
            states=page.locator('#output-items').text_content();require(all('structural-'+state+' · '+state in states for state in ['incomplete','failed','unsupported']),'Incomplete/failed/unsupported states hidden')
            checks.append('Automatic HTTP discovery lists completed outputs and explains unavailable states without selecting them')
            entry=next(e for e in choices if e['format']=='rheon-obstacle-flow-records-v1' and e['record']['sha256']==originals[str(args.flow)])
            page.locator('#outputs').select_option(entry['id']);page.locator('#title').filter(has_text='Reduced obstacle').wait_for()
            for i,case in enumerate(flow['shear_cases']):
                page.locator('#cases').select_option(str(i))
                for j,frame in enumerate(case['frames']):
                    page.locator('#timeline').fill(str(j));page.locator('#timeline').dispatch_event('input')
                    actual=page.locator('rheon-viewer').evaluate('(e)=>e.frames()[e.index]')
                    require(actual['profile']==[[y,v] for y,v in zip(case['centers'],frame['velocity'])] and actual['time']==frame['step']*case['dt'],'Discovered flow data changed')
            provenance=page.locator('rheon-viewer').evaluate('(e)=>e.data.provenance')
            require(provenance['source_head']==entry['provenance']['source_head'] and provenance['sha256']==entry['record']['sha256'],'Original producer identity lost')
            checks.append('HTTP-selected original flow retains all supplied samples, times and original source/byte identity')
            # Return changed bytes at the exact snapshot URL, while preserving originals.
            pattern='**/outputs/blobs/'+entry['record']['sha256']+'.json'
            page.route(pattern,lambda route:route.fulfill(body=b'{}',content_type='application/json'))
            page.locator('#outputs').select_option('');page.locator('#outputs').select_option(entry['id'])
            page.locator('#status.error').filter(has_text='bytes differ').wait_for()
            require(page.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256')==entry['record']['sha256'],'Corrupt server bytes replaced prior display')
            page.unroute(pattern);checks.append('Changed server snapshot bytes visibly refuse and preserve prior recording')
            if args.rigid:
                rigid_entry=next(e for e in choices if e['record']['sha256']==originals[str(args.rigid)])
                rigid_pattern='**/outputs/blobs/'+rigid_entry['record']['sha256']+'.json';held=[]
                page.route(rigid_pattern,lambda route:held.append(route))
                page.locator('#outputs').select_option(rigid_entry['id'])
                for _ in range(100):
                    if held:break
                    page.wait_for_timeout(50)
                require(held,'Delayed HTTP recording read did not start')
                page.locator('#outputs').select_option(entry['id']);page.locator('#status').filter(has_text='Imported for visualization').wait_for()
                held[0].fulfill(body=args.rigid.read_bytes(),content_type='application/json');page.wait_for_timeout(200);page.unroute(rigid_pattern)
                require(page.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256')==entry['record']['sha256'],'Delayed HTTP read overwrote newer recording choice')
                page.locator('#outputs').select_option(rigid_entry['id']);page.locator('#title').filter(has_text='Rigid mesh').wait_for()
                raw=json.loads(args.rigid.read_text());actual=page.locator('rheon-viewer').evaluate('(e)=>e.frames().map(frame=>frame.vertices)')
                require(actual==[raw['initial']['vertices']]+[step['stored']['vertices'] for step in raw['steps']],'Discovered rigid geometry changed')
                checks.append('Delayed HTTP selection cannot overwrite a newer choice; original rigid geometry stays exact')
            page.unroute('**/outputs/catalog.json');page.route('**/outputs/catalog.json',lambda route:route.fulfill(json={'schema':'future','entries':[]}))
            previous=page.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256')
            page.locator('#refresh-outputs').click();page.locator('#status.error').filter(has_text='catalog unavailable').wait_for()
            require(page.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256')==previous and page.locator('#outputs option').count()==len(choices)+1,'Bad catalog replaced retained view/list')
            page.unroute('**/outputs/catalog.json');page.locator('#refresh-outputs').click();page.locator('#outputs-summary').filter(has_text='completed recordings').wait_for();page.locator('#status').filter(has_text='Choose a completed').wait_for()
            checks.append('Invalid refreshed catalog leaves the prior recording and catalog intact')
            page.locator('#outputs-panel').evaluate('(e)=>e.open=true');page.screenshot(path=str(output/'producer-desktop.png'))
            page.set_viewport_size({'width':390,'height':844});require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Phone output catalog overflow');page.screenshot(path=str(output/'producer-phone.png'))
            page.locator('#pose').click();require(page.locator('#refresh-outputs').is_hidden() and page.locator('#outputs-panel').is_hidden(),'Producer discovery leaks into authoring mode')
            checks.append('Producer controls fit phone and remain separate from authoring/playback')
            require(not errors,'Browser errors: '+str(errors));require(originals=={name:digest(Path(name)) for name in originals},'Original producer data changed')
            receipt={'rheon_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=Path(__file__).resolve().parents[3],text=True).strip(),'kenoma_commit':PIN,'browser':browser.version,'checks':checks,'completed_recordings':len(choices),'original_inputs':originals,'runtime_package_sha256':digest(site/'package-receipt.json'),'original_catalog_sha256':digest(site/'outputs/catalog.json'),'page_network_errors':errors,'physics_runs':0}
            (output/'producer-browser-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2));browser.close()
    finally:server.shutdown();server.server_close()

if __name__=='__main__':main()
