"""Actual debug/release builds, binary hashes and native outputs; no time step."""
import hashlib,json,os
from pathlib import Path
import subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]).resolve();output.mkdir()
commands=[]
for profile in ['debug','release']:
    command=['cargo','build','--locked','--example','fitted_height']+(['--release'] if profile=='release' else [])
    with (output/(profile+'-build.log')).open('w') as stream:
        code=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
    commands.append({'command':command,'exit':code})
    if code:raise SystemExit(code)
    binary=Path(os.environ['CARGO_TARGET_DIR'])/profile/'examples/fitted_height'
    with (output/(profile+'.json')).open('w') as stream:
        code=subprocess.run([str(binary)],cwd=ROOT,stdout=stream).returncode
    commands.append({'command':[str(binary)],'exit':code,'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()})
    if code:raise SystemExit(code)
    for optimized in [False,True]:
        command=[sys.executable]+(['-O'] if optimized else [])+[str(ROOT/'evidence/fitted-height-periodic-repair/verify_native.py'),str(output/(profile+'.json')),'--negative-self-test']
        name=profile+('-optimized' if optimized else '-normal')
        with (output/(name+'.log')).open('w') as stream:
            code=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
        commands.append({'command':command,'exit':code})
        if code:raise SystemExit(code)
(output/'receipt.json').write_text(json.dumps({'commands':commands,'debug_release_byte_equal':(output/'debug.json').read_bytes()==(output/'release.json').read_bytes(),
    'scope':'actual native instantaneous operators, no advancing solve'},indent=2)+'\n')
print('PASS debug/release native evidence and normal/optimized independent checks')
