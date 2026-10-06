"""Readonly source/result integrity, actual ELF binding and preserved failures."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,struct,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1];BASE='8fc718ff603e7a7123037e592b4f40f59762e011'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def inventory(commit):
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',commit],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in [b'100644',b'100755'] and kind==b'blob','regular frozen source/evidence files');data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'preserved inherited byte '+name.decode());digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def run():
 old=inventory(BASE);require(old['files']==8134,'complete prior preparation inventory')
 for folder in ['case41-six-fd-columns-v1','case47-pure-e2-128-v1']:
  d=ROOT/'evidence'/folder;r=json.loads((d/'blocked-receipt.json').read_text())
  for name,digest in r['artifact_sha256'].items():require(sha((d/name).read_bytes())==digest,'preserved failed preflight/harness result')
 c41=ROOT/'evidence/case41-six-fd-columns-v1';require(not(c41/'capture-before.json').exists() and not(c41/'native.log').exists(),'case41 stopped before all seven equations')
 oldpublic=ROOT/'evidence/forcing-case47-public-call-v1'
 for name in ['candidate.rs','reference.rs','scalar.rs','candidate_helpers.rs','reference_helpers.rs','candidate_probe.rs','reference_probe.rs','inputs.rs','trajectory_inputs.rs','fixed_inputs.rs','FittedHeightWorkspace-overlay.rs','TranslatedViscousFlow-overlay.rs']:
  require((P/name).read_bytes()==(oldpublic/name).read_bytes(),'ACKed numerical/owner source unchanged '+name)
 require((P/'harness.rs').read_bytes()==(oldpublic/'harness.rs').read_bytes()+b'\ninclude!("pure_e2.rs");\n','only appended bounded test')
 native=(P/'pure_e2.rs').read_text();require(all(x not in native for x in ['candidate_snapshot(', 'from_cloned_parts(', 'linear(', 'retry', 'owner.clone']), 'no hidden accepted snapshot, correction or retry operation')
 b=json.loads((P/'binary-binding.json').read_text());require(subprocess.check_output(['objdump','-d','-C',b['binary']])==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()),'actual linked instructions')
 for name,args in [('symbols',['nm','-S','-nC']),('headers',['readelf','-hW']),('relocations',['readelf','-rW'])]:require(subprocess.check_output([*args,b['binary']])==(P/('native-'+name+'.txt')).read_bytes(),'same actual ELF '+name)
 checks=load('actual_analysis',P/'analyze.py').run();require(checks==json.loads((P/'analysis-normal.json').read_text()) and (P/'analysis-normal.json').read_bytes()==(P/'analysis-optimized.json').read_bytes(),'fresh result analysis and mode parity')
 for roster in ['analysis-commands.json','physical-commands.json','memory-commands.json']:
  cmds=json.loads((P/roster).read_text());require(len(cmds)==2 and all(c['exit']==0 for c in cmds),'two passing reader modes '+roster)
  for c in cmds:
   for channel in ['stdout','stderr']:require(sha((P/c[channel]).read_bytes())==c[channel+'_sha256'],'actual reader command journal')
 # Borrowed-state cancellation validator corruption must be rejected.
 a=load('actual_data',P/'analyze.py');states=[x for x in a.rows(P/'cancel-10-native.log') if x.get('event')=='accepted_bits'];before,after=states[-2:];bad=json.loads(json.dumps(after));bad['velocity'][0][0]^=1;require(a.fields(before)==a.fields(after) and a.fields(before)!=a.fields(bad),'one-bit cancelled state corruption rejected')
 m=load('actual_memory',P/'audit_observation.py').run();require(m==json.loads((P/'memory-normal.json').read_text()),'fresh complete memory accounting')
 # Match original case47 initial bit fixture independently of the new Rust check.
 initial=next(x for x in a.rows(oldpublic/'E2-native.log') if 'model' in x);fixture=json.loads((P/'initial-bits.json').read_text());actual={k:[int.from_bytes(struct.pack('>d',v),'big') for v in initial[src]] for k,src in [('mass','mass'),('pressure','pressure_coefficients')]}
 require(actual['mass']==fixture['mass'] and actual['pressure']==fixture['pressure'],'archived original nodal mass/pressure bits')
 return {'status':'PASS_FROZEN_PURE_E2_RESULT_AND_PRESERVATION','base':BASE,'base_tree':'d9b51b3e2c1b186ea2d4b139d53bfe38a4ff678f','preserved_base':old,'analysis_status':checks['status'],'native_source':b['prototype'],'native_source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'memory_bound':m['bound'],'actual_main_steps':checks['actual_main_accepted_steps'],'actual_controls':checks['cancellation_stages_passed'],'total_accepted_advances':checks['total_accepted_advances'],'total_public_attempts':checks['total_public_attempts'],'negative_controls':checks['negative_controls']+['one-bit cancelled-state corruption'],'case41_candidate_equations':0,'case41_memory_preflight_still_blocked':True,'step_retries':0,'new_native_executions_by_reader':0,'reference_integrations':0,'all_original_failures_retained':True,'production_adoption':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
