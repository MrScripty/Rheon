"""Replay new native publications through unchanged frozen physical gates."""
from pathlib import Path
import sys,importlib.util,json,tempfile,copy
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location('frozen_probe',ROOT/'evidence/forcing-temporal-diagnosis/replay_probe.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
def require(ok,msg):
 if not ok:raise ValueError(msg)
source=P/'forcing_temporal_probe-default.jsonl';answer=old.run(source)
rows=[json.loads(l)for l in source.read_text().splitlines()];i=next(i for i,r in enumerate(rows)if r.get('step')==1)
controls=[('published third velocity',lambda r:r[i]['velocity'][0].__setitem__(2,r[i]['velocity'][0][2]+.01)),('published pressure',lambda r:r[i]['pressure_coefficients'].__setitem__(0,r[i]['pressure_coefficients'][0]+.1)),('published geometry',lambda r:r[i]['positions'][0].__setitem__(1,r[i]['positions'][0][1]+.01)),('mass',lambda r:r[i]['mass'].__setitem__(0,r[i]['mass'][0]+.01)),('work allowance',lambda r:r[i]['report']['total'].__setitem__('fixed_work_allowance',1.)),('Newton report',lambda r:r[i]['report']['planar'].__setitem__('finite_momentum_rate_norm',1e-12)),('unbounded counter',lambda r:r[i]['report']['planar'].__setitem__('equation_evaluations',201)),('promote old failure',lambda r:next(x for x in r if x.get('probe_status')=='REFUSED').__setitem__('probe_status','COMPLETE'))]
rejections=[]
for name,mutate in controls:
 broken=copy.deepcopy(rows);mutate(broken)
 with tempfile.TemporaryDirectory(prefix='rheon-terminal-control-')as temp:
  path=Path(temp)/'native.jsonl';path.write_text('\n'.join(json.dumps(x)for x in broken)+'\n')
  try:old.run(path)
  except (ValueError,KeyError)as error:rejections.append({'control':name,'status':'REJECTED','reason':str(error)})
  else:raise ValueError('accepted corrupted new native publication '+name)
answer['new_capture_corruption_rejections']=rejections
answer['original_physical_verifier_changed']=False
answer['source']='922466bd9efdf4612462288fbe2df5a679ef4ccb'
print(json.dumps(answer,indent=2))
