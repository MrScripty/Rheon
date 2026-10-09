#!/usr/bin/env python3
"""Real Chromium presentation checks against supplied existing files; no solves."""
import argparse
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
from threading import Thread
from playwright.sync_api import sync_playwright
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from build import external, PIN

def require(ok, message):
    if not ok: raise AssertionError(message)

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['site', 'rigid', 'shear', 'output']: parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args(); site = args.site.resolve(strict=True); output = external(args.output)
    require(not output.exists(), 'Browser evidence output must be fresh')
    before = {str(p.resolve()): digest(p) for p in [args.rigid, args.shear]}
    rigid = json.loads(args.rigid.read_text()); shear = json.loads(args.shear.read_text())
    package = json.loads((site / 'package-receipt.json').read_text())
    require(package['kenoma_commit'] == PIN, 'Packaged Kenoma pin mismatch')
    require(json.loads((site / 'component.json').read_text())['rig_version'] == 1, 'Packaged rig version mismatch')
    for name, sha in package['files_sha256'].items(): require(digest(site / name) == sha, 'Packaged bytes changed: ' + name)
    prefix = '/' + site.name + '/'
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *a): pass
        def do_GET(self):
            if self.path == '/embed.html':
                body = ('<!doctype html><meta name="viewport" content="width=device-width,initial-scale=1"><body style="margin:0"><iframe title="Embedded Rheon" src="' + prefix + 'index.html" style="border:0;width:100%;height:1100px"></iframe>').encode()
                self.send_response(200); self.send_header('Content-Type', 'text/html'); self.end_headers(); self.wfile.write(body)
            else: super().do_GET()
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(site.parent)))
    Thread(target=server.serve_forever, daemon=True).start()
    base = f'http://127.0.0.1:{server.server_port}'
    checks = []; errors = []; output.mkdir(parents=True)
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=shutil.which('chromium'), headless=True, args=['--no-sandbox', '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'])
            page = browser.new_page(viewport={'width': 1440, 'height': 1050}, device_scale_factor=1)
            page.on('pageerror', lambda e: errors.append(e.stack or str(e)))
            page.on('response', lambda r: errors.append(f'HTTP {r.status} {r.url}') if r.status >= 400 else None)
            page.goto(base + '/embed.html'); host = page.frame_locator('iframe[title="Embedded Rheon"]')
            host.locator('rheon-viewer').wait_for()
            require(host.locator('#scope').inner_text().startswith('Recorded playback'), 'Playback scope absent')
            host.locator('#file').set_input_files(str(args.rigid.resolve()))
            host.locator('#title').filter(has_text='Rigid mesh').wait_for()
            for index in range(len(rigid['steps']) + 1):
                host.locator('#timeline').fill(str(index)); host.locator('#timeline').dispatch_event('input')
                expected = rigid['initial'] if index == 0 else rigid['steps'][index - 1]['stored']
                actual = host.locator('rheon-viewer').evaluate('(e)=>({index:e.index, frame:e.frames()[e.index], rendered:Array.from(e.view.group.children[0].geometry.attributes.position.array), webgl:e.view.renderer.getContext() instanceof WebGL2RenderingContext})')
                require(actual['index'] == index and actual['frame']['time'] == expected['time_s'], 'Stored frame/time mismatch')
                require(actual['frame']['vertices'] == expected['vertices'], 'Adapter changes stored vertices')
                require(actual['webgl'], 'Actual WebGL2 unavailable')
                # Uploaded GPU positions use Float32; display conversion only.
                import struct
                uploaded = [struct.unpack('f', struct.pack('f', x))[0] for v in expected['vertices'] for x in v]
                require(actual['rendered'] == uploaded, 'Rendered buffer differs from recorded geometry')
            checks.append('All supplied rigid frames and actual WebGL buffers match')
            host.locator('#timeline').fill('0'); host.locator('#timeline').dispatch_event('input')
            host.locator('#play').click(); page.wait_for_timeout(600)
            require(host.locator('rheon-viewer').evaluate('(e)=>e.index') == 1, 'Saved-index playback failed')
            host.locator('#play').click(); old = host.locator('#time').inner_text(); page.wait_for_timeout(600)
            require(host.locator('#time').inner_text() == old, 'Pause failed')
            checks.append('Play/pause advances saved indices only')
            host.locator('#file').set_input_files({'name': 'bad.json', 'mimeType': 'application/json', 'buffer': b'{"schema":"unrecognized"}'})
            host.locator('#status.error').wait_for(); require(host.locator('#title').inner_text().startswith('Rigid'), 'Invalid input replaced retained view')
            checks.append('Invalid record visibly refuses and preserves prior recording')
            host.locator('#file').set_input_files(str(args.shear.resolve())); host.locator('#title').filter(has_text='Reduced obstacle').wait_for()
            for i, case in enumerate(shear['shear_cases']):
                host.locator('#cases').select_option(str(i))
                for j, frame in enumerate(case['frames']):
                    host.locator('#timeline').fill(str(j)); host.locator('#timeline').dispatch_event('input')
                    actual = host.locator('rheon-viewer').evaluate('(e)=>e.frames()[e.index]')
                    require(actual['profile'] == [[y, v] for y, v in zip(case['centers'], frame['velocity'])], 'Stored shear samples changed')
                    require(actual['time'] == frame['step'] * case['dt'], 'Shear display time product changed')
                    if 'energy' in frame: require(actual['metrics'][-2:] == [['Kinetic energy (J)', frame['energy'][1]], ['Body work (J)', frame['energy'][3]]], 'Displayed energy metrics changed')
            checks.append('All supplied shear cases/frames retain samples and ledgers')
            page.screenshot(path=str(output / 'recorded-desktop.png'))
            isolated = host.locator('rheon-viewer').evaluate('''(e)=>{
              const other=document.createElement('rheon-viewer');e.after(other);other.load(e.data);
              const before=e.index;other.seek(1);
              const ok=e.index===before&&other.index===1&&other.view!==e.view&&other.shadowRoot!==e.shadowRoot;
              other.remove();return ok;
            }''')
            require(isolated, 'Multiple viewer instances share UI state or view resources')
            checks.append('Reusable custom element instances have separate state and WebGL resources')
            # A real browser directory selection, with original recordings unchanged.
            folder = output / 'input-folder'; (folder / 'nested').mkdir(parents=True)
            shutil.copyfile(args.rigid, folder / 'rigid.json')
            shutil.copyfile(args.shear, folder / 'nested/shear.json')
            (folder / 'invalid.json').write_text('{"schema":"future"}')
            (folder / 'strain.tsv').write_text('unsupported recorded rows\n')
            (folder / 'wrench.jsonl').write_text('{"unsupported":"record"}\n')
            host.locator('#folder').set_input_files(str(folder))
            host.locator('#catalog-summary').filter(has_text='2 recordings / 5 files').wait_for()
            require(host.locator('#runs option').count() == 2, 'Folder omitted admitted recordings')
            host.locator('#runs').select_option(label='input-folder/rigid.json')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.frames()[1].vertices') == rigid['steps'][0]['stored']['vertices'], 'Folder rigid adapter changed stored values')
            host.locator('#runs').select_option(label='input-folder/nested/shear.json')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.frames()[0].profile') == [[y,v] for y,v in zip(shear['shear_cases'][0]['centers'],shear['shear_cases'][0]['frames'][0]['velocity'])], 'Folder shear adapter changed stored values')
            catalog = host.locator('#catalog-items').text_content()
            require('strain.tsv · Unsupported' in catalog and 'wrench.jsonl · Unsupported' in catalog and 'invalid.json · Refused' in catalog, 'Unsupported folder files are unexplained')
            require(host.locator('rheon-viewer').evaluate('(e)=>Object.isFrozen(e.catalog.entries)&&e.catalog.entries.filter(x=>x.status==="ready").every(x=>Object.isFrozen(x.data))'), 'Folder data is mutable')
            checks.append('Actual output-folder picker discovers unchanged recordings and visibly lists refused and unsupported formats')
            refused = output / 'unsupported-folder'; refused.mkdir(); (refused / 'records.tsv').write_text('unadapted\n')
            previous = host.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256')
            host.locator('#folder').set_input_files(str(refused)); host.locator('#status.error').filter(has_text='No supported recordings').wait_for()
            require(host.locator('rheon-viewer').evaluate('(e)=>e.data.provenance.sha256') == previous, 'Unsupported folder replaced prior recording')
            # Late reads may not overwrite a newer individual-file selection.
            pending = host.locator('rheon-viewer').evaluate('''e=>{
              let finish;const folder=e.$('folder');
              const file={name:'late.json',webkitRelativePath:'late/late.json',size:2,arrayBuffer:()=>new Promise(resolve=>finish=resolve)};
              Object.defineProperty(folder,'files',{configurable:true,value:[file]});
              window.lateFolder=e.$('folder').onchange({target:folder});
              window.finishLateFolder=()=>{finish(new TextEncoder().encode('{}').buffer);delete folder.files;};
              return typeof finish==='function';
            }''')
            require(pending, 'Delayed folder fixture did not start reading')
            host.locator('#file').set_input_files(str(args.rigid.resolve())); host.locator('#title').filter(has_text='Rigid mesh').wait_for()
            host.locator('rheon-viewer').evaluate('async()=>{window.finishLateFolder();await window.lateFolder;}')
            require(host.locator('#title').inner_text().startswith('Rigid'), 'Delayed folder replaced newer individual recording')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.catalog===null'), 'Single-file import retained unrelated catalog')
            checks.append('Unsupported folders retain prior data; delayed folder admission cannot overwrite newer selection')
            host.locator('#folder').set_input_files(str(folder)); host.locator('#catalog-summary').filter(has_text='2 recordings / 5 files').wait_for()
            host.locator('#catalog-panel').evaluate('(e)=>e.open=true')
            page.screenshot(path=str(output / 'folder-desktop.png'))
            host.locator('#pose').click()
            child = host.frame_locator('iframe[title="Kenoma simple-human pose editor"]')
            child.locator('canvas').wait_for(timeout=60000)
            editor = next(f for f in page.frames if f.url.endswith('/kenoma/index.html'))
            editor.wait_for_function('window.simpleGraphEditor?.ready', timeout=60000)
            editor.evaluate('window.simpleGraphEditor.renderer.whenIdle()')
            require(child.locator('#error').is_hidden(), 'Kenoma editor initialization error')
            state = editor.evaluate('window.simpleGraphEditor.model.state')
            snapshot = '''async()=>{
              const e=window.simpleGraphEditor;await e.renderer.whenIdle();
              const item=e.renderer.characters.get(e.model.state.selectedId);
              const hash=async data=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',data))).map(x=>x.toString(16).padStart(2,'0')).join('');
              return {vertices:item.mesh.positions.length,indices:item.mesh.indices.length,
                topology:await hash(new Uint32Array(item.mesh.indices).buffer),
                positions:await hash(new Float64Array(item.mesh.positions.flat()).buffer),
                current:item.meshKey===item.graphKey,pending:e.renderer.pending};
            }'''
            bound = editor.evaluate(snapshot)
            require(bound['current'] and not bound['pending'], 'Initial worker mesh is stale')
            child.locator('#add').click(); require(child.locator('#character option').count() == 2, 'Independent character addition failed')
            child.locator('#undo').click(); require(child.locator('#character option').count() == 1, 'Kenoma undo failed')
            editor.evaluate('window.simpleGraphEditor.renderer.whenIdle()')
            # A real owner-supported keyboard gizmo action, not injected model state.
            child.locator('#handle').select_option('rightArm:target')
            child.locator('canvas').focus(); page.keyboard.press('ArrowUp')
            posed = editor.evaluate('window.simpleGraphEditor.model.state')
            require(posed != state, 'Actual Kenoma keyboard posing failed')
            pose_mesh = editor.evaluate(snapshot)
            require(pose_mesh['topology'] == bound['topology'] and pose_mesh['indices'] == bound['indices'] and pose_mesh['vertices'] == bound['vertices'], 'Posing changed bound topology')
            require(pose_mesh['positions'] != bound['positions'] and pose_mesh['current'], 'Worker did not apply the current pose')
            child.locator('#undo').click(); restored = editor.evaluate(snapshot)
            require(restored['positions'] == bound['positions'] and restored['topology'] == bound['topology'], 'Worker undo did not restore bound geometry')
            child.locator('canvas').focus()
            for _ in range(30): page.keyboard.press('ArrowLeft')
            latest = editor.evaluate(snapshot)
            require(latest['current'] and not latest['pending'] and latest['topology'] == bound['topology'], 'Rapid input left stale mesh or changed contact topology')
            posed = editor.evaluate('window.simpleGraphEditor.model.state')
            checks.append('Worker rig applies pose/undo/latest queued input with unchanged bound topology')
            checks.append('Pinned Kenoma WASM editor: add, undo and real keyboard posing')
            page.screenshot(path=str(output / 'pose-desktop.png'))
            host.locator('#records').click(); host.locator('#pose').click()
            require(editor.evaluate('window.simpleGraphEditor.model.state') == posed, 'Mode switch reloaded the pose')
            checks.append('Pose survives tab switching without computation coupling')
            # Exercise the unchanged owner's source-file UI inside Rheon's iframe.
            child.locator('#add').click(); child.locator('#add').click()
            editor.evaluate('''async()=>{
              const e=window.simpleGraphEditor;
              for(const [i,c] of e.model.state.characters.entries()){
                e.model.dispatch({type:'color',id:c.id,color:['#cb7766','#698edb','#69bc91'][i]});
                e.model.dispatch({type:'placement',id:c.id,position:[i*1.4,.05*i,.15*i],yaw:.3*i});
                e.model.dispatch({type:'head',id:c.id,yaw:.2*(i+1),pitch:-.1*i});
              }
              e.update();await e.renderer.whenIdle();
            }''')
            authored = editor.evaluate('window.simpleGraphEditor.model.state')
            scene_path = output / 'authored.human.sqlite'
            with page.expect_download() as event: child.locator('#saveScene').click()
            event.value.save_as(str(scene_path))
            require(scene_path.read_bytes().startswith(b'SQLite format 3\0'), 'Scene save is not SQLite')
            with sqlite3.connect(f'file:{scene_path}?mode=ro', uri=True) as db:
                require(db.execute('SELECT version FROM skin_scene_schema').fetchall() == [(1,)], 'Unsupported scene container')
                source = json.loads(db.execute('SELECT source_json FROM skin_scenes WHERE id=1').fetchone()[0])
                require(set(source['characters'][0]) == {'id','name','color','position','yaw','head','rig','pose'}, 'Scene contains non-source fields')
                require(len(source['characters']) == 3, 'Scene save omitted authored characters')
            page.reload(); host.locator('#pose').click(); child.locator('canvas').wait_for(timeout=60000)
            editor = next(f for f in page.frames if f.url.endswith('/kenoma/index.html'))
            editor.wait_for_function('window.simpleGraphEditor?.ready', timeout=60000)
            require(len(editor.evaluate('window.simpleGraphEditor.model.state.characters')) == 1, 'Unexpected automatic scene persistence')
            def open_scene(path):
                with page.expect_file_chooser() as chooser: child.locator('#openScene').click()
                chooser.value.set_files(str(path))
            open_scene(scene_path)
            editor.wait_for_function('expected=>JSON.stringify(window.simpleGraphEditor.model.state)===JSON.stringify(expected)', arg=authored)
            editor.evaluate('window.simpleGraphEditor.renderer.whenIdle()')
            checks.append('Embedded Save downloads source SQLite; Open after reload restores exact IDs poses transforms colors and selection')
            # Import is a single edit, including exact restoration of prior state.
            editor.evaluate("()=>{const e=window.simpleGraphEditor;e.model.dispatch({type:'color',id:e.model.state.selectedId,color:'#123456'});e.update();}")
            altered = editor.evaluate('window.simpleGraphEditor.model.state')
            open_scene(scene_path)
            editor.wait_for_function('expected=>JSON.stringify(window.simpleGraphEditor.model.state)===JSON.stringify(expected)', arg=authored)
            child.locator('#undo').click()
            require(editor.evaluate('window.simpleGraphEditor.model.state') == altered, 'Scene import is not one undoable edit')
            child.locator('#redo').click()
            require(editor.evaluate('window.simpleGraphEditor.model.state') == authored, 'Redo changed authored scene')
            checks.append('Embedded scene import Undo/Redo restores exact prior and imported source states')
            future = output / 'future.human.sqlite'; shutil.copyfile(scene_path, future)
            with sqlite3.connect(future) as db: db.execute('UPDATE skin_scene_schema SET version=2')
            revision = editor.evaluate('window.simpleGraphEditor.model.revision')
            open_scene(future); child.locator('#error').wait_for(state='visible')
            require(editor.evaluate('window.simpleGraphEditor.model.state') == authored and editor.evaluate('window.simpleGraphEditor.model.revision') == revision, 'Future scene version mutated source')
            stale = editor.evaluate('''async raw=>{
              const e=window.simpleGraphEditor;let finish;
              const pending=e.files.loadFile({size:raw.length,arrayBuffer:()=>new Promise(resolve=>finish=resolve)});
              e.model.dispatch({type:'color',id:e.model.state.selectedId,color:'#abcdef'});e.update();
              const before=JSON.stringify(e.model.state);finish(new Uint8Array(raw).buffer);
              const result=await pending;return {status:result.status,same:JSON.stringify(e.model.state)===before};
            }''', list(scene_path.read_bytes()))
            require(stale == {'status':'error','same':True}, 'Delayed scene import overwrote newer authored state')
            checks.append('Embedded future-version and stale scene imports refuse without replacing authored state')
            open_scene(scene_path); child.locator('#error').wait_for(state='hidden')
            editor.evaluate('window.simpleGraphEditor.renderer.whenIdle()')
            page.screenshot(path=str(output / 'scene-desktop.png'))
            page.set_viewport_size({'width': 390, 'height': 844})
            require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Outer phone overflow')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.getBoundingClientRect().width<=innerWidth+1'), 'Viewer phone overflow')
            require(editor.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Kenoma phone overflow')
            require(child.locator('#saveScene').is_visible() and child.locator('#openScene').is_visible(), 'Scene-file controls absent on phone')
            with page.expect_download() as event: child.locator('#saveScene').click()
            phone_scene = output / 'phone.human.sqlite'; event.value.save_as(str(phone_scene))
            with sqlite3.connect(f'file:{phone_scene}?mode=ro', uri=True) as db:
                require(json.loads(db.execute('SELECT source_json FROM skin_scenes WHERE id=1').fetchone()[0]) == source, 'Phone save changed source payload')
            checks.append('Phone scene-file controls fit and save the same exact source payload')
            page.screenshot(path=str(output / 'pose-phone.png'))
            checks.append('Iframe/project-subpath and phone layout')
            host.locator('#records').click()
            host.locator('#folder').set_input_files(str(folder))
            host.locator('#catalog-summary').filter(has_text='2 recordings / 5 files').wait_for()
            host.locator('#catalog-panel').evaluate('(e)=>e.open=true')
            require(host.locator('#folder-label').is_visible() and host.locator('#runs-label').is_visible(), 'Phone folder controls absent')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.shadowRoot.querySelector(".workspace").scrollWidth<=e.clientWidth+1'), 'Folder catalog overflows phone')
            page.screenshot(path=str(output / 'folder-phone.png'))
            # Reusable element lifetime: detached viewer stops its display loop.
            host.locator('#records').click()
            stopped = host.locator('rheon-viewer').evaluate('(e)=>{e.play();e.remove();return {timer:e.timer,view:e.view,active:e.active}}')
            require(stopped == {'timer': None, 'view': None, 'active': False}, 'Detached viewer retains animation/resources')
            checks.append('Detached custom element releases timer, WebGL and iframe')
            require(not errors, 'Browser page/network errors: ' + str(errors))
            require(before == {str(x.resolve()): digest(x) for x in [args.rigid, args.shear]}, 'Original inputs changed')
            receipt = {'schema':'rheon-viewer-browser-v1','browser':browser.version,'rheon_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=Path(__file__).resolve().parents[3],text=True).strip(),'package_receipt_sha256':digest(site/'package-receipt.json'),'kenoma_commit':package['kenoma_commit'],'input_sha256':before,'checks':checks,'page_network_errors':errors,'physics_runs':0,'numerical_qualification':False}
            (output / 'browser-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n'); browser.close(); print(json.dumps(receipt,indent=2))
    finally: server.shutdown(); server.server_close()

if __name__ == '__main__': main()
