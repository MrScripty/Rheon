#!/usr/bin/env python3
"""Package static presentation and the exact Kenoma component; never run physics."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

PIN = '3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4'
ROOT = Path(__file__).resolve().parent

def require(ok, text):
    if not ok: raise ValueError(text)

def external(path):
    resolved = path.resolve()
    ancestor = resolved
    while not ancestor.is_dir(): ancestor = ancestor.parent
    env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
    probe = subprocess.run(['git', '-C', str(ancestor), 'rev-parse', '--show-toplevel'], env=env, capture_output=True, text=True, timeout=30)
    require(probe.returncode != 0 and 'not a git repository' in probe.stderr, 'Generated paths must be outside Git checkouts; uncertain Git probes refuse')
    return resolved

def run(args, cwd, **options):
    return subprocess.check_output(args, cwd=cwd, text=True, timeout=600, **options).strip()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--kenoma-source', type=Path, required=True)
    parser.add_argument('--asset-dir', type=Path, required=True, help='Rheon education node_modules, locked three@0.180.0')
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--wasm-bindgen', type=Path, required=True)
    args = parser.parse_args()
    output = external(args.output); target = external(args.target_dir)
    require(not output.exists(), 'Choose a fresh output directory')
    source = args.kenoma_source.resolve(strict=True)
    require(run(['git', 'rev-parse', 'HEAD'], source) == PIN, 'Kenoma checkout must match the reviewed pin')
    require(not run(['git', 'status', '--porcelain', '--untracked-files=all'], source), 'Kenoma checkout must be clean')
    require(run(['git', 'remote', 'get-url', 'origin'], source) == 'https://github.com/MrScripty/Kenoma.git', 'Unexpected Kenoma source repository')
    lock = tomllib.loads((source / 'Cargo.lock').read_text())
    version = next(p['version'] for p in lock['package'] if p['name'] == 'wasm-bindgen')
    require(run([str(args.wasm_bindgen.resolve(strict=True)), '--version'], source) == 'wasm-bindgen ' + version, 'WASM glue generator must match Cargo.lock')
    assets = args.asset_dir.resolve(strict=True) / 'three'
    require(json.loads((assets / 'package.json').read_text())['version'] == '0.180.0', 'Three.js version must be 0.180.0')
    # Compile only the independent graph/surface generator. All generated files
    # go outside both repositories; Kenoma-owned tracked files stay untouched.
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    subprocess.run(['cargo', 'build', '--locked', '-p', 'human_wasm', '--release', '--target', 'wasm32-unknown-unknown'], cwd=source, env=env, check=True, timeout=600)
    output.mkdir(parents=True)
    for name in ['index.html', 'viewer.css', 'shell.css', 'shell.js', 'view.js', 'adapters.js', 'catalog.js', 'app.js']:
        shutil.copyfile(ROOT / name, output / name)
    vendor = output / 'vendor'; vendor.mkdir()
    for rel in ['build/three.module.js', 'build/three.core.js', 'examples/jsm/controls/OrbitControls.js', 'LICENSE']:
        shutil.copyfile(assets / rel, vendor / Path(rel).name)
    component = output / 'kenoma'; component.mkdir()
    for rel in run(['git', 'ls-tree', '-r', '--name-only', PIN, 'browser/simple-graph'], source).splitlines():
        path = Path(rel).relative_to('browser/simple-graph')
        if path.parts[0] == 'tests' or path.name in ['package.json', 'package-lock.json', 'build.sh']: continue
        dest = component / path; dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(source / rel, dest)
    shutil.copyfile(source / 'LICENSE', component / 'KENOMA-LICENSE')
    subprocess.run([str(args.wasm_bindgen.resolve()), '--target', 'web', '--out-dir', str(component / 'pkg'), str(target / 'wasm32-unknown-unknown/release/human_wasm.wasm')], check=True, timeout=120)
    (output / 'component.json').write_text(json.dumps({'commit': PIN, 'protocol': 1, 'rig_version': 1, 'repository': 'MrScripty/Kenoma', 'mode': 'kinematic editor'}, indent=2) + '\n')
    hashes = {str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.rglob('*')) if p.is_file()}
    receipt = {'schema': 'rheon-viewer-package-v1', 'rheon_head': run(['git', 'rev-parse', 'HEAD'], ROOT), 'rheon_dirty': bool(run(['git', 'status', '--porcelain'], ROOT)), 'kenoma_commit': PIN, 'kenoma_tree': run(['git', 'rev-parse', 'HEAD^{tree}'], source), 'wasm_bindgen': version, 'files_sha256': hashes, 'physics_runs': 0, 'numerical_qualification_claim': False}
    (output / 'package-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    require(not run(['git', 'status', '--porcelain', '--untracked-files=all'], source), 'Kenoma source changed during packaging')
    print(json.dumps({'output': str(output), 'files': len(hashes), 'kenoma_commit': PIN, 'physics_runs': 0}))

if __name__ == '__main__': main()
