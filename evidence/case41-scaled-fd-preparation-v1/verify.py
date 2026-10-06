"""Readonly preparation checks: existing-data algebra, compiled source, zero native run."""
from pathlib import Path
from fractions import Fraction as Q
import gzip,hashlib,importlib.util,json,re,subprocess,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1];BASE='998de18438a339d44896d24ecf9634ad469cdcdf'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def inventory():
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in [b'100644',b'100755']and kind==b'blob','source modes');data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'all inherited source/evidence bytes '+name.decode());digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 require(count==8058,'full inherited inventory');return {'files':count,'path_content_sha256':digest.hexdigest()}
def run():
 policy=json.loads((P/'protocol.json').read_text());require(not policy['capture_executed']and not policy['capture_authorized']and policy['new_corrections']==policy['owner_advances']==0 and policy['FD_columns']==list(range(6))and policy['orders']==[16]and policy['candidate_equations_requested_after_future_qualification']==7,'honest fixed preparation scope')
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen preparation source '+path)
 local=json.loads((P/'local-source-bindings.json').read_text())
 for path,digest in local['sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'local numerical/provenance input')
 amend=json.loads((P/'metric-reader-amendment.json').read_text());require(amend['old_reader_sha256']==sha((P/'analyze_metric.py').read_bytes())and amend['new_reader_sha256']==sha((P/'analyze_metric_v2.py').read_bytes())and amend['old_failed_commands_sha256']==sha((P/'metric-commands.json').read_bytes()),'preserved nominal-mass reader repair');failed=json.loads((P/'metric-commands.json').read_text());require(len(failed)==1 and failed[0]['exit']!=0 and 'actual conserved fixture mass'in(P/'metric-80-normal-stderr.log').read_text()and not(P/'metric-80-normal.json').read_bytes(),'first false exact-sum assumption refused, never relabeled')
 b=json.loads((P/'compile-binding.json').read_text());require(b['source']=='c238111d8c8d159c157e489294a25998aaf36516'and b['source_tree']=='c87e9c658cad27df708a34aa63bccf9868095598'and b['protocol_sha256']==sha((P/'protocol.json').read_bytes())and b['native_test_invocations']==b['native_equations']==b['owner_advances']==0 and b['scalar_runtime_preflight_pending']and b['FD_memory_preflight_pending']and b['FD_execution_authorization_pending'],'compiled source and pending runtime obligations');require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'same actual unexecuted ELF');cmds=json.loads((P/'compile-commands.json').read_text());require([Path(c['stdout']).stem for c in cmds]==['format','install','build','clippy','restore'],'exact compile-only roster')
 for c in cmds:
  require(c['exit']==0 and c['native_test_invocation']is False,'passed compile-only command');require(c['argv'][0]in ['rustfmt','python','cargo'],'no ELF test invocation')
  for channel in ['stdout','stderr']:require(c[channel+'_sha256']==sha((P/c[channel]).read_bytes()),'actual compiler/Clippy log')
 raw=subprocess.check_output(['objdump','-d','-C',b['binary']]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes())and sha(raw)==(P/'native-disassembly.txt.sha256').read_text().strip(),'same actual linked instructions, readonly tools')
 old=ROOT/'evidence/forcing-e2-fixed-candidates-v1';N=P/'native';require((N/'candidate.rs').read_bytes()==(old/'candidate.rs').read_bytes()+b'\ninclude!("../fd_probe.rs");\n','only appended private observation include, entire equation/controller body unchanged');harness=(N/'harness.rs').read_text();require(harness.startswith((old/'harness.rs').read_text())and 'case41_six_columns_only'in harness[len((old/'harness.rs').read_text()):],'only added guarded test');probe=(P/'fd_probe.rs').read_text();require(all(x not in probe for x in ['linear(', 'public_call(', 'new_forced_extruded(', 'qualify(', 'third_step(', 'accept_ale(', 'step_extruded']), 'no correction, owner or publication operation in observation addon')
 for path in N.glob('*.rs'):
  if path.name in ['candidate.rs','harness.rs','library-overlay.rs']:continue
  require(path.read_bytes()==(old/path.name).read_bytes(),'all other frozen private native/scalar files unchanged')
 m=load('metric_v2',P/'analyze_metric_v2.py')
 for dps in [80,120]:
  require(m.run(dps)==json.loads((P/f'metric-v2-{dps}-normal.json').read_text()),'fresh exact weighted algebra');require((P/f'metric-v2-{dps}-normal.json').read_bytes()==(P/f'metric-v2-{dps}-optimized.json').read_bytes(),'mode parity')
 lo=json.loads((P/'metric-v2-80-normal.json').read_text());hi=json.loads((P/'metric-v2-120-normal.json').read_text());lo.pop('precision_digits');hi.pop('precision_digits');require(lo==hi,'display precision parity');require(lo['actual_stored_mass_minus_nominal_exact']=='-9/72057594037927936','nominal/stored distinction');commands=json.loads((P/'metric-v2-commands.json').read_text());require(len(commands)==4 and all(c['exit']==0 and not c['native_execution']for c in commands),'closed successful readonly reader roster')
 for c in commands:
  for channel in ['stdout','stderr']:require(c[channel+'_sha256']==sha((P/c[channel]).read_bytes()),'actual metric reader logs')
 rejected=[]
 try:m.exact_positive_definite([[Q(1),Q(2)],[Q(2),Q(1)]])
 except ValueError:rejected.append('indefinite mass rejected')
 else:raise ValueError('indefinite mass accepted')
 try:m.exact_positive_definite([[Q(1),Q(0)],[Q(0),Q(0)]])
 except ValueError:rejected.append('singular mass rejected')
 else:raise ValueError('singular mass accepted')
 # Every local Markdown citation resolves; external ones remain primary URLs.
 checked_links=0
 for path in P.glob('*.md'):
  for target in re.findall(r'\]\(([^)]+)\)',path.read_text()):
   if target.startswith(('https://','http://','#')):continue
   require((path.parent/target.split('#')[0]).exists(),'local evidence citation '+target);checked_links+=1
 return {'status':'PASS_COMPILED_PREPARATION_AND_EXISTING_DATA_METRICS_ONLY','compiled_source':b['source'],'compiled_source_tree':b['source_tree'],'binary_sha256':b['binary_sha256'],'preserved_base':inventory(),'metric_orders':[16,32],'metric_precision':[80,120],'normal_optimized_parity':True,'negative_controls':rejected,'local_links_checked':checked_links,'native_equations_executed':0,'native_test_ELF_invocations':0,'new_corrections':0,'owner_advances':0,'capture_equations_future_max':7,'capture_authorized':False,'runtime_scalar_preflight_pending':True,'FD_memory_preflight_pending':True,'pure_E2_execution_authorized':False,'original_runtime_failures_retained':2,'original_geometry_band_failures_retained':8,'arithmetic_floor_claimed':False,'physical_root_error_bound_established':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
