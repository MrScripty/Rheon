"""Unchanged physical equations on every actually published pure-E2 cell."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('original_adapter',ROOT/'evidence/forcing-e1-trajectories-v2/replay_physics.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
def run():
 pubs=[json.loads(x) for x in (P/'main-native.log').read_text().splitlines() if x.startswith('{') and '"model"' in x];a.require(1<=len(pubs)<=129,'bounded actual publications')
 oldschema=a.frozen.schema;oldcell=a.frozen.cell;visited=0
 def observed(*args):
  nonlocal visited
  visited+=1;return oldcell(*args)
 a.frozen.schema=a.schema;a.frozen.cell=observed
 try:
  if len(pubs)==1:return {'status':'NO_ACCEPTED_CELLS','actual_cells_replayed':0,'reference_integrations':0}
  out=a.frozen.replay(pubs,references=False)
  a.require(visited==len(pubs)-1 and out['status']=='PASS','all actual pure-E2 cells')
  return {'status':'PASS_UNCHANGED_PHYSICAL_REPLAY','actual_cells_replayed':visited,'maxima':out['maxima'],'final_endpoint':out['rows'][0],'reference_integrations':0,'tolerance_relaxation':False,'scope':'Unchanged 34 momentum equations, full path constraints, donor/GCL, separate and combined forced work, pressure and both third shears; replay only, no reference integration.'}
 except ValueError as error:
  return {'status':'FAIL_UNCHANGED_PHYSICAL_REPLAY','first_failed_cell':visited,'error':str(error),'later_cells_unqualified':True,'reference_integrations':0,'tolerance_relaxation':False}
 finally:a.frozen.schema=oldschema;a.frozen.cell=oldcell
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
