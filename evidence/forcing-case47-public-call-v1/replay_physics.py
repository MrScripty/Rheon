"""Apply unchanged physical equations/gates to actual 25-prefix + one-call data."""
from pathlib import Path
import importlib.util,json,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('original_adapter',ROOT/'evidence/forcing-e1-trajectories-v2/replay_physics.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
def run():
 pubs=[json.loads(x)for x in(P/'E2-native.log').read_text().splitlines()if x.startswith('{')and '"model"'in x];a.require(len(pubs)==27 and pubs[-1]['stamp']['version']==26,'exact authorized historical prefix plus one comparison');oldschema=a.frozen.schema;oldcell=a.frozen.cell;visited=0
 def observed(*args):
  nonlocal visited
  visited+=1;return oldcell(*args)
 a.frozen.schema=a.schema;a.frozen.cell=observed
 try:out=a.frozen.replay(pubs,references=False)
 finally:a.frozen.schema=oldschema;a.frozen.cell=oldcell
 a.require(visited==26 and out['status']=='PASS','all actual captured publications replayed');return {'status':'PASS_UNCHANGED_PHYSICAL_REPLAY','historical_E1_prefix_steps':25,'new_E2_comparison_steps':1,'actual_cells_replayed':visited,'maxima':out['maxima'],'final_endpoint':out['rows'][0],'reference_integrations':0,'tolerance_relaxation':False,'scope':'Unchanged frozen 34 momentum equations, full path constraints, donor/GCL, separate/combined work, pressure and two third shears on captured publications. Replaying historical prefix is read-only, not trajectory extension.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
