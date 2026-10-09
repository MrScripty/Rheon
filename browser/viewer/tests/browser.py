#!/usr/bin/env python3
"""Real Chromium presentation checks against supplied existing files; no solves."""
import argparse
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
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
            page.set_viewport_size({'width': 390, 'height': 844})
            require(page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Outer phone overflow')
            require(host.locator('rheon-viewer').evaluate('(e)=>e.getBoundingClientRect().width<=innerWidth+1'), 'Viewer phone overflow')
            require(editor.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Kenoma phone overflow')
            page.screenshot(path=str(output / 'pose-phone.png'))
            checks.append('Iframe/project-subpath and phone layout')
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
