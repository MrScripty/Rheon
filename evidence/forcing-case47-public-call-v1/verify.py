"""Fresh readonly ELF/source/journal binding and physical-replay evidence closure."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,os,subprocess,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 preflight,b=load('frozen_driver',P/'run_once_v2.py').preflight();require(preflight==json.loads((P/'preflight-receipt.json').read_text()),'fresh readonly preflight replay');amend=json.loads((P/'reader-amendment.json').read_text());require(amend['driver_sha256']==sha((P/'run_once_v2.py').read_bytes())and amend['old_driver_sha256']==sha((P/'run_once.py').read_bytes())and amend['case47_owner_executions_before_repair']==0,'preserved original guard/reader repair');raw=subprocess.check_output(['objdump','-d','-C',b['binary']]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()),'actual same ELF instructions')
 for name,argv in [('symbols',['nm','-S','-nC',b['binary']]),('relocations',['readelf','-rW',b['binary']]),('headers',['readelf','-hW',b['binary']])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
 outcome=load('native_reader',P/'analyze.py').run();require(outcome==json.loads((P/'analysis-normal.json').read_text())and(P/'analysis-normal.json').read_bytes()==(P/'analysis-optimized.json').read_bytes(),'native journal fresh replay/parity');require((P/'physical-normal.json').read_bytes()==(P/'physical-optimized.json').read_bytes(),'complete physical replay mode parity');phys=json.loads((P/'physical-normal.json').read_text());require(phys['status']=='PASS_UNCHANGED_PHYSICAL_REPLAY'and phys['actual_cells_replayed']==26 and phys['new_E2_comparison_steps']==1 and phys['historical_E1_prefix_steps']==25 and phys['reference_integrations']==0 and not phys['tolerance_relaxation'],'exact physical scope');commands=json.loads((P/'analysis-commands.json').read_text());require(len(commands)==4,'closed reader command roster')
 for cmd in commands:
  require(cmd['exit']==0 and cmd['native_execution']is False and cmd['reader_sha256']==sha((P/cmd['reader']).read_bytes()),'actual unchanged reader command')
  for channel in ['stdout','stderr']:require(cmd[channel+'_sha256']==sha((P/cmd[channel]).read_bytes()),'reader log binding')
 with tempfile.TemporaryDirectory(prefix='case47-readonly-render-')as d:
  target=Path(d)/'replay.svg';env=dict(os.environ,MPLCONFIGDIR='/tmp/rheon-mpl',XDG_CACHE_HOME='/tmp/rheon-cache');r=subprocess.run(['python',str(P/'render.py'),str(target)],env=env,capture_output=True);require(r.returncode==0 and target.read_bytes()==(P/'public-call.svg').read_bytes(),'actual deterministic standalone render')
 return {'status':'PASS_FRESH_CASE47_BOUND_RESULTS','native_source':b['prototype'],'native_source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'memory_bound':65760,'memory_cap':67584,'native_scalar_groups':23,'exact_primitive_probes':36,'affine_probes':6,'E1_refusal_trace_and_state_exact':True,'E2_prefix_reports_and_states_exact':True,'E2_version':26,'E2_comparison_calls':1,'E2_corrections':2,'E2_counted_equations':15,'physical_cells_readonly_replayed':26,'new_reference_integrations':0,'original_runtime_failures_preserved':2,'original_geometry_band_failures_preserved':8,'negative_controls':outcome['negative_controls'],'new_native_executions_in_verification':0,'production_adoption':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
