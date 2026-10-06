"""Readonly extraction from the existing case47 ELF; no rebuild or execution."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,re,subprocess
P=Path(__file__).resolve().parent
F=Path('/workspace/Rheon-fd-pure-e2');OLD=F/'evidence/case47-pure-e2-128-v2'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 require(not(P/'outer-frame.json').exists(),'never overwrite extraction evidence')
 b=json.loads((OLD/'binary-binding.json').read_text());require(sha(Path(b['binary']).read_bytes())==b['binary_sha256']=='6e6ea0505c81cbf8624b177c6c782ff8cb29cc0830c0524e79088e8307a10989','exact existing ELF')
 raw=subprocess.check_output(['objdump','-d','-C',b['binary']]);require(raw==gzip.decompress((OLD/'native-disassembly.txt.gz').read_bytes()),'actual existing instructions equal frozen archive')
 require(sha(raw)==(OLD/'native-disassembly.txt.sha256').read_text().strip(),'archived disassembly digest')
 h=load('fixed_frames',F/'evidence/forcing-increment-memory-audit/audit.py');parts=re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode());selected=''.join(x for x in parts if x.splitlines() and 'research_public_call::' in x.splitlines()[0]);fs=h.functions(selected,(OLD/'native-symbols.txt').read_text())
 records={}
 for suffix,label in [('case47_pure_e2_original_128_or_cancel','outer'),('verified_prefix','native-prefix'),('candidate::public_call','candidate-public-call'),('observe','shared-observe')]:
  f=next(f for f in fs.values() if f['name'].endswith('::'+suffix));block=next(x for x in parts if x.startswith(f'{f["start"]:016x} <'));(P/(label+'.asm.txt')).write_text(block)
  records[label]={'name':f['name'],'address':hex(f['start']),'size':f['end']-f['start'],**h.fixed_frame(f),'direct_calls':[{'address':hex(e['address']),'instruction':e['instruction'],'name':fs[e['target']]['name'] if e['target'] in fs else None} for e in h.transfers(f) if e['kind']=='call'],'stack_mutations':[{'address':hex(a),'instruction':asm} for a,asm in f['code'].items() if asm.startswith(('push','pop')) or re.search(r',%rsp$',asm)]}
 layout=re.search(r'"native":\(([^)]+)\),"candidate":\(([^)]+)\).*"control_bytes":(\d+),"workspace_bytes":(\d+)',(OLD/'type-layout.log').read_text());require(layout is not None,'actual archived Rust layout tuple');native=list(map(int,layout[1].split(',')));candidate=list(map(int,layout[2].split(',')));control=int(layout[3]);workspace=int(layout[4]);require(native[3]==8848 and candidate[3]==8856 and control==120 and workspace==62096,'frozen type sizes')
 outer=records['outer']['frame_bound'];residual=outer-workspace-2*native[3];require(outer==81152 and residual==1360,'complete conservative outer shell residual')
 m=json.loads((OLD/'memory-normal.json').read_text());base=m['full_public']['bound'];observation=m['observation_helper_peak'];require(base==65760 and observation==304 and m['bound']==67088,'original certificate values')
 # Keep its existing alignment overcharge, replace the one descriptor and
 # guessed 1024 with the entire outer residual, which already contains both
 # candidate descriptors. No smaller frame/unused native residual credit.
 supplemented=(base-8+residual+observation+15)//16*16
 report={
  'status':'READONLY_OUTER_FRAME_CLARIFICATION_SUPPLEMENT_FOR_REVIEW',
  'binary':b['binary'],'binary_sha256':b['binary_sha256'],
  'compiled_source':b['prototype'],'compiled_source_tree':b['prototype_tree'],
  'actual_disassembly_sha256':sha(raw),'archive_sha256':sha((OLD/'native-disassembly.txt.gz').read_bytes()),
  'symbol_table_sha256':sha((OLD/'native-symbols.txt').read_bytes()),'layout_sha256':sha((OLD/'type-layout.log').read_bytes()),
  'frames':records,'workspace_bytes':workspace,'native_owner_descriptor_bytes':native[3],'candidate_owner_descriptor_bytes':candidate[3],'control_bytes':control,
  'disjoint_outer_regions':[
   {'name':'constructor return slot, later public Result output','offset':1232,'bytes':8856},
   {'name':'settled candidate owner','offset':10096,'bytes':8856},
   {'name':'ChartWorkspace','offset':18960,'bytes':62096}],
  'common_native_owner_regions_credited':2,'common_native_owner_bytes_credited':2*native[3],
  'remaining_outer_residual_bytes':residual,'candidate_descriptor_additions_in_residual':16,
  'original_guessed_allowance':1024,'original_bound':67088,
  'original_allowance_complete_frame_accounting_established':False,
  'supplemented_bound_for_review':supplemented,'unchanged_cap':67584,'supplemented_remaining':67584-supplemented,
  'full_public_positive_stack_peak':m['full_public']['added_public_stack'],
  'no_shrink_or_native_residual_slack_credit':True,
  'numerical_test_invocations':0,'rebuilds':0,'new_equations':0,'new_owner_advances':0,
  'scope':'Supplement the declared additional numerical-memory certificate. Credit only two source-matched ordinary native owner descriptor regions demonstrated by the same ELF native-prefix constructor move. Keep the whole remaining conservative outer frame, both candidate descriptor additions and existing public/observation charges. Common libtest, allocator, environment parsing and bounded journal formatting remain outside this numerical certificate as in its original scope; this is not process RSS.'}
 (P/'outer-frame.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
 print(json.dumps({k:report[k] for k in ['status','binary_sha256','original_bound','original_allowance_complete_frame_accounting_established','supplemented_bound_for_review','supplemented_remaining','numerical_test_invocations','rebuilds']},indent=2,sort_keys=True))
if __name__=='__main__':run()
