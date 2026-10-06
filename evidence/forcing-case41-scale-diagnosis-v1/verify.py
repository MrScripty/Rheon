"""Fresh exact reader replay and bounded provenance checks; no native invocation."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib,importlib.util,json,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 a=load('focus',P/'analyze.py');s=load('scaling',P/'pressure_scaling.py')
 for script,m in [('analyze',a),('pressure_scaling',s)]:
  for dps in [80,120]:
   require(m.run(dps)==json.loads((P/f'{script}-{dps}-normal.json').read_text()),'fresh exact reader replay');require((P/f'{script}-{dps}-normal.json').read_bytes()==(P/f'{script}-{dps}-optimized.json').read_bytes(),'mode parity')
  lo=json.loads((P/f'{script}-80-normal.json').read_text());hi=json.loads((P/f'{script}-120-normal.json').read_text());lo.pop('precision_digits');hi.pop('precision_digits');require(lo==hi,'display precision parity')
 policy=json.loads((P/'supplement-policy.json').read_text());require(policy['base_protocol_sha256']==hashlib.sha256((P/'protocol.json').read_bytes()).hexdigest(),'original protocol binding')
 for path,digest in policy['source_sha256'].items():require(hashlib.sha256((P/path).read_bytes()).hexdigest()==digest,'supplement reader binding')
 provenance=json.loads((P/'provenance.json').read_text())
 for row in provenance['corpus']:require(hashlib.sha256((ROOT/row['path']).read_bytes()).hexdigest()==row['sha256'],'bounded local provenance corpus')
 controls=[]
 try:a.inverse([[Q(1),Q(1)],[Q(1),Q(1)]])
 except ValueError:controls.append('singular matrix refused')
 require(len(controls)==1,'negative rank control')
 A=[[Q(2),Q(1)],[Q(1),Q(3)]];inv=a.inverse(A);inv[0][0]+=1;require(a.mul(A,inv)!=[[Q(1),Q(0)],[Q(0),Q(1)]],'corrupt inverse detected');controls.append('corrupt inverse identity detected')
 oldP=a.P
 with tempfile.TemporaryDirectory()as d:
  t=Path(d);bad=json.loads((P/'protocol.json').read_text());first=next(iter(bad['inputs']));bad['inputs'][first]='0'*64;(t/'protocol.json').write_text(json.dumps(bad));a.P=t
  try:a.run(80)
  except ValueError as e:require('pinned input'in str(e),'specific binding failure');controls.append('corrupt input binding refused')
  finally:a.P=oldP
 require(len(controls)==3,'all corruption controls');return {'status':'PASS_FRESH_EXISTING_DATA_DIAGNOSIS','orders':[16,32],'precision':[80,120],'normal_optimized_parity':True,'negative_controls':controls,'provenance_files':len(provenance['corpus']),'native_executions':0,'owner_advances':0,'new_corrections':0,'arithmetic_floor_bound':False,'full_Newton_condition':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
