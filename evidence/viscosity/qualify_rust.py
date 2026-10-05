"""Run the actual Rust feature matrix; inherit the selected pinned environment."""
import json
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]).resolve();output.mkdir()
commands=[('rustc',['rustc','--version','--verbose']),('cargo',['cargo','--version']),
 ('fmt',['cargo','fmt','--all','--check']),
 ('test-default',['cargo','test','--locked']),
 ('test-core',['cargo','test','--locked','--no-default-features']),
 ('test-desktop',['cargo','test','--locked','--features','desktop']),
 ('clippy-default',['cargo','clippy','--locked','--all-targets','--','-D','warnings']),
 ('clippy-core',['cargo','clippy','--locked','--no-default-features','--all-targets','--','-D','warnings']),
 ('clippy-desktop',['cargo','clippy','--locked','--features','desktop','--all-targets','--','-D','warnings'])]
receipt=[]
for name,args in commands:
    with (output/(name+'.log')).open('w') as stream:code=subprocess.run(args,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
    receipt.append({'name':name,'command':args,'exit':code});(output/'commands.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(name,'exit',code,flush=True)
    if code:raise SystemExit(code)
print('PASS complete Rust feature matrix',flush=True)
