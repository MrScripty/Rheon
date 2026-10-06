"""Regenerate the original paired references once without parameter search."""
from pathlib import Path
import hashlib,json,os,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/'evidence/forced-extruded-liquid'))
import reference as r
def sha(data):return hashlib.sha256(data).hexdigest()
def require(ok,message):
 if not ok:raise ValueError(message)
roster=json.loads((P/'roster.json').read_text());policy=json.loads((P/'protocol.json').read_text())
for path,digest in roster['original_sources_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen original reference source '+path)
require(not(P/'reference-authorization.json').exists(),'never rerun reference experiment')
(P/'reference-authorization.json').write_text(json.dumps({'driver_sha256':sha(Path(__file__).read_bytes()),'protocol_sha256':sha((P/'protocol.json').read_bytes()),'original_sources_sha256':roster['original_sources_sha256'],'resource_env':{k:os.environ.get(k)for k in ['OPENBLAS_NUM_THREADS','OMP_NUM_THREADS']},'parameters':policy['reference_integrations'],'rhs_cap':5000,'parameter_search':False},indent=2,sort_keys=True)+'\n')
for kind in r.KINDS if hasattr(r,'KINDS')else ['initial','pressure_state']:
 for field in ['nonconstant','constant']:
  for load in ['forward','reversed']:
   label=f'reference-{kind}-{field}-{load}';results=[]
   for params in policy['reference_integrations']:
    start=time.monotonic()
    try:result=r.reference(kind,field,r.A*(1 if load=='forward'else -1),params['rtol'],params['max_step']);results.append({'status':'COMPLETE','parameters':params,'result':result})
    except Exception as error:results.append({'status':'FAILED','parameters':params,'error':repr(error)})
    (P/(label+'.json')).write_text(json.dumps({'kind':kind,'field':field,'load':load,'rows':results},indent=2,sort_keys=True)+'\n')
    print(json.dumps({'event':'reference_finished','kind':kind,'field':field,'load':load,'rtol':params['rtol'],'status':results[-1]['status'],'seconds':time.monotonic()-start}),flush=True)
