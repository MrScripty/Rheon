"""Fresh full-public accounting plus new bounded borrowed-observation helpers."""
from pathlib import Path
import gzip,importlib.util,json,re
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 base=load('fresh_public',P/'audit_full.py').run();h=load('frames',ROOT/'evidence/forcing-increment-memory-audit/audit.py')
 raw=gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()).decode();selected=''.join(x for x in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw) if x.splitlines() and 'research_public_call::' in x.splitlines()[0]);fs=h.functions(selected,(P/'native-symbols.txt').read_text())
 rows={}
 for name in ['pure_e2_bits','pure_e2_initial']:
  matches=[f for f in fs.values() if f['name'].endswith('::'+name)];require(len(matches)==1,'new observation helper '+name)
  rows[name]={'name':matches[0]['name'],**h.fixed_frame(matches[0]),'transfers':h.transfers(matches[0])}
  require(not any(e['target'] in fs and '::candidate::' in fs[e['target']]['name'] for e in rows[name]['transfers']),'observation does not advance or solve')
 matches=re.findall(r'"control_bytes":(\d+)',(P/'type-layout.log').read_text());require(len(matches)==1,'one actual Rust-debug Control size');layout={'control_bytes':int(matches[0])}
 # New shell's Control lives during the public call. Charge a conservative
 # 1024-byte result/report/control/time allowance as well as full helper frames,
 # even when matched public frames or constructor maxima already contain them.
 require(layout['control_bytes']<=128,'fixed bounded cancellation Control')
 observation=max(r['frame_bound'] for r in rows.values());bound=(base['bound']+1024+observation+15)//16*16
 return {'status':'PASS_PURE_E2_MEMORY_PREFLIGHT' if bound<=67584 else 'BLOCKED_PURE_E2_MEMORY_CAP','bound':bound,'cap':67584,'remaining':67584-bound,'full_public':base,'observation_helpers':rows,'observation_helper_peak':observation,'explicit_result_report_control_time_allowance':1024,'control_bytes':layout['control_bytes'],'new_accepted_snapshot_bytes':0,'simultaneous_owners':1,'no_shrink_credit':True,'no_baseline_slack_credit':True,'owner_advances':0,'scope':'Fresh actual ELF constructor, public force/Newton/quadrature/third/publish/cancellation paths, bounded control and all new bit-journal frames. Existing emit body and bounded formatting remain common diagnostics. No whole-process RSS claim.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
