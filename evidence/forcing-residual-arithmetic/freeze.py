"""Bind diagnosis-only artifacts and byte-restored source; never overwrite."""
from pathlib import Path
import hashlib,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
if(P/'receipt.json').exists():raise ValueError('refuse overwrite receipt')
base='dad53b4054034fe0c2ff6240464df7441ab4e6a9'
paths=subprocess.check_output(['git','ls-files'],cwd=ROOT,text=True).splitlines()
for path in paths:
 if (ROOT/path).read_bytes()!=subprocess.check_output(['git','show',base+':'+path],cwd=ROOT):raise ValueError('frozen tracked source/evidence modified '+path)
r={'base':base,'base_tree':'57648befc4e108bf8d727b3367adf7e119cdebc1','forcing':'f76841e9217e781d91c5bfeabadb6b4c6f42b20b','main_read_only':'9cd4587a54befa61bdfddc8e35014bd3c34f02fb','main_tree':'69def74e09a06c46a812eaadb8530a53306b9daa','preserved_tracked_paths':len(paths),'production_changes':[],'owner_advances':0,'newton_target':1e-13,'remaining_frozen_refusals':5,'original_temporal_band':'FAIL_ORIGINAL_TEMPORAL_BAND','production_solver_defect_established':False,'arithmetic_floor_proved':False,'native_probes':[{'argv':['cargo','test','--locked','--release','--no-default-features','--lib','captured_refusal_equation_arithmetic','--','--nocapture'],'source_overlay':'probe-overlay.patch','probe_version':version,'stdout':stdout,'stderr':stderr,'target':'/workspace/.rheon-tools/forcing-arithmetic-target','exit':0,'scope':scope}for version,stdout,stderr,scope in [('native-probe-first.rs','native-probe.log','native-probe-stderr.log','five original equation vectors and isolated planar qualification'),('native-probe-start-chart.rs','native-probe-start-chart.log','native-probe-start-chart-stderr.log','original reconstruction plus start chart'),('native_probe.rs','native-neighborhood.log','native-neighborhood-stderr.log','original reconstruction plus start chart and 220 bounded neighbors')]],'file_sha256':{str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(P.rglob('*'))if p.is_file()and '__pycache__'not in p.parts and p.name!='receipt.json'}}
(P/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print('Frozen diagnosis receipt SHA256 '+hashlib.sha256((P/'receipt.json').read_bytes()).hexdigest())
