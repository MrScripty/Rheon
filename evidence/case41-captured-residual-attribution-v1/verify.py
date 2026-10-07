"""Check frozen captured attribution; no native execution."""
from pathlib import Path
import hashlib,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
inv=json.loads((P/'result-inventory.json').read_text())
for f,h in inv['sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'frozen '+f)
policy=json.loads((P/'protocol.json').read_text())
for f,h in policy['sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'input '+f)
model=[]
for d in [80,120]:
    normal=(P/f'analysis-{d}-normal.json').read_bytes()
    require(normal==(P/f'analysis-{d}-optimized.json').read_bytes(),'normal/-O parity')
    x=json.loads(normal);require(x.pop('precision_digits')==d,'precision binding');model.append(x)
    for mode in ['normal','optimized']:require(not (P/f'analysis-{d}-{mode}-stderr.log').read_bytes(),'stderr clean')
require(model[0]==model[1],'80/120 displays identical')
x=model[0];require(x['status']=='PASS_ARCHIVED_BASELINE_COMPLEMENT_ATTRIBUTION_ONLY','status')
require(x['new_native_equations']==x['controller_corrections']==x['owner_advances']==0,'no numerical run')
require(x['exact_stored_above_Newton_gate'] and not x['frozen_ideal_endpoint_remainder_above_Newton_gate'],'honest diagnostic classification')
require(not x['historical_memory_qualified'] and not x['arithmetic_floor_claimed'],'limits preserved')
changes=subprocess.check_output(['git','diff','--name-only','42114ab','HEAD'],cwd=ROOT,text=True).splitlines()
require(all(f.startswith('evidence/case41-captured-residual-attribution-v1/') for f in changes),'separate attribution packet only')
print(json.dumps({'status':'PASS_FROZEN_ARCHIVED_ATTRIBUTION','source':inv['analysis_source'],'tree':inv['analysis_tree'],'normal_optimized_equal':True,'precision_80_120_equal':True,'new_native_equations':0,'historical_memory_qualified':False},indent=2,sort_keys=True))
