"""Run built binaries and bind exact frozen outputs, plus a new viscosity repeat."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[2]
binroot=Path(sys.argv[1]).resolve();output=Path(sys.argv[2]).resolve();output.mkdir()
receipt=[]
def sha(data):return hashlib.sha256(data).hexdigest()
def run(label,args):
    with (output/(label+'.log')).open('w') as stream:code=subprocess.run(args,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
    if code:raise ValueError(label+' exit '+str(code))
    return {'command':list(map(str,args)),'exit':code}
for old in json.loads((ROOT/'evidence/column-interface/legacy-replay/receipt.json').read_text()):
    name=Path(old['fixture']).name;dest=output/name
    command=[str(binroot/'rheon'),*old['command'][1:-1],str(dest)]
    record=run(name,command);record['fixture']=old['fixture'];record['byte_identical_sha256']={}
    for file in old['byte_identical_sha256']:
        a=(ROOT/old['fixture']/file).read_bytes();b=(dest/file).read_bytes()
        if a!=b:raise ValueError('original Jacobi bytes: '+file)
        record['byte_identical_sha256'][file]=sha(b)
    a=json.loads((ROOT/old['fixture']/'run.json').read_text());b=json.loads((dest/'run.json').read_text())
    for field in old['equal_manifest_fields']:
        if a[field]!=b[field]:raise ValueError('Jacobi manifest: '+field)
    record['equal_manifest_fields']=old['equal_manifest_fields'];receipt.append(record)
for name,fixture in [('liquid_step','evidence/liquid-step/demo'),('free_surface','evidence/free-surface/demo'),('column_interface','evidence/column-interface/demo'),('viscosity','evidence/viscosity/demo'),('column_shear','evidence/column-shear/demo')]:
    dest=output/name;record=run(name,[str(binroot/'examples'/name),str(dest)])
    a={str(p.relative_to(ROOT/fixture)) for p in (ROOT/fixture).rglob('*') if p.is_file()}
    b={str(p.relative_to(dest)) for p in dest.rglob('*') if p.is_file()}
    if a!=b:raise ValueError('complete replay inventory: '+name)
    record['fixture']=fixture;record['byte_identical_sha256']={}
    for file in sorted(a):
        first=(ROOT/fixture/file).read_bytes();second=(dest/file).read_bytes()
        if first!=second:raise ValueError('native replay bytes: '+name+'/'+file)
        record['byte_identical_sha256'][file]=sha(second)
    receipt.append(record)
(output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('PASS original Jacobi manifests/bytes and five complete native example replays')
