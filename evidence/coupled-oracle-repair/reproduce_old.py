"""Actual retained demonstrations of frozen oracle gaps, not false real data."""
import copy,importlib.util,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent;R=P.parents[1]
sys.path.insert(0,str(R/'evidence/fitted-discrete-refinement'))
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
oldhost=load('frozen_per_cell_oracle',R/'evidence/fitted-discrete-refinement/verify_trajectory.py')
oldnative=load('frozen_optional_physics_oracle',R/'evidence/native-coupled-discrete/replay.py')
data=json.loads((R/'evidence/fitted-discrete-refinement/trials/repaired-initial-trajectory.json').read_text());bad=copy.deepcopy(data);bad['replays']=[copy.deepcopy(data['replays'][0])for _ in range(5)];bad['rows']=[copy.deepcopy(data['rows'][0])for _ in range(5)];bad['duration']=999;bad['references']=[];bad['velocity_refinement_ratios']=[999]*4;bad['geometry_refinement_ratios']=[999]*4;bad['reference_refinement_velocity_difference']=999
for r in bad['rows']:r['velocity_lumped_L2_error']=999;r['geometry_max_error']=999
(P/'trials/duplicated-coarse-fabricated-study.json').write_text(json.dumps(bad,indent=2)+'\n');count=oldhost.verify(bad);print('FROZEN ORACLE INCORRECTLY ACCEPTED',count,'cells, five duplicate coarse cases, duration999, empty refs, fabricated errors/ratios',flush=True)
rows=[json.loads(x)for x in(R/'evidence/native-coupled-discrete/native-qualification/native-default.jsonl').read_text().splitlines()]
for row in rows:
 for name in ['mass','positions','triangles','physical_pressure']:del row[name]
(P/'trials/native-physical-fields-omitted.jsonl').write_text(''.join(json.dumps(r)+'\n'for r in rows));result=oldnative.replay(rows);print('FROZEN ORACLE INCORRECTLY ACCEPTED',result['steps'],'native cells with physical publication fields omitted',flush=True)
(P/'trials/frozen-oracle-counterexamples.json').write_text(json.dumps(dict(actual_frozen_host_accepted_malformed_cells=count,actual_frozen_native_accepted_omitted_field_cells=result['steps'],actual_stored_trajectories_remain_valid=True,not_a_governing_equation_counterexample=True),indent=2)+'\n')
