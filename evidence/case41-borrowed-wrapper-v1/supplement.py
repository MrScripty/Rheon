"""Actual ELF/source targeted evidence, no builds or native invocations."""
from pathlib import Path
import gzip, hashlib, importlib.util, json, re, subprocess, sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1];OUT=Path(sys.argv[1]).resolve()
def require(ok,msg):
    if not ok:raise ValueError(msg)
def sha(data):return hashlib.sha256(data).hexdigest()
binding=json.loads((OUT/'compile-binding.json').read_text())
require(sha(Path(binding['binary']).read_bytes())==binding['binary_sha256'],'actual dedicated ELF')
for name,digest in binding['source_inventory'].items():
    require(sha((ROOT/name).read_bytes())==digest,'compiled source inventory '+name)
old=ROOT/'evidence/case41-scaled-fd-preparation-v1/native'
new=P/'native';checks={}
for name in ('candidate.rs','reference.rs'):
    a=(old/name).read_text().split('include!("'+name.replace('.rs','_helpers.rs')+'");')[0]
    b=(new/name).read_text().split('include!("'+name.replace('.rs','_helpers.rs')+'");')[0]
    require(a==b,'unchanged complete numerical module '+name);checks[name]=sha(a.encode())
for name in ('scalar.rs','fixed_inputs.rs','inputs.rs','trajectory_inputs.rs','candidate_helpers.rs','reference_helpers.rs','candidate_probe.rs','reference_probe.rs','TranslatedViscousFlow-overlay.rs'):
    require((old/name).read_bytes()==(new/name).read_bytes(),'unchanged fixture/helper '+name);checks[name]=sha((new/name).read_bytes())
require((new/'FittedHeightWorkspace-overlay.rs').read_bytes().startswith((old/'FittedHeightWorkspace-overlay.rs').read_bytes().rstrip()),'exact original geometry body before appended type-layout function')
target=json.loads((P/'baseline-target.json').read_text())
require(sha((ROOT/target['archived_log']).read_bytes())==target['archived_log_sha256'],'frozen E2 baseline source')
baseline=json.dumps(target['rate_bits'],separators=(',',':'))
text=(new/'borrowed_candidate.rs').read_text()
match=re.search(r'const EXPECTED: \[u64; V\] = \[(.*?)\];',text,re.S)
require(match is not None and [int(x.strip()) for x in match[1].split(',') if x.strip()]==target['rate_bits'],'baseline22 bit target')
require(text.index('if !case41_baseline_accepts(&e.rate)')<text.index('(Ok(e), Some(j))'),'baseline gate precedes column branch')
require('rate[i].to_bits() != EXPECTED[i]' in text,'exact bit comparison without rate-vector copy')
raw=subprocess.check_output(['objdump','-d','-C',binding['binary']])
require(raw==gzip.decompress((OUT/'native-disassembly.txt.gz').read_bytes()),'fresh ELF equals frozen assembly')
memory=json.loads((OUT/'memory-normal.json').read_text());parts={}
for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode()):
    m=re.match(r'^([a-f0-9]+) <(.+)>:',part)
    if m:parts[int(m[1],16)]=(m[2],part)
ref={e['target'] for e in memory['external_transfer_ledger']['reference']}
targets={e['target'] for e in memory['external_transfer_ledger']['candidate'] if e['target'] is not None and e['target'] not in ref}
files={}
for address in sorted(targets):
    name,body=parts[address];path=OUT/('candidate-specific-'+hex(address)+'.asm.txt');path.write_text(body)
    files[path.name]=dict(symbol=name,sha256=sha(path.read_bytes()),caller_qualification='see COMMON-COSTS; all unclosed paths retained',complete_cost=None)
# Freeze only the official installed library files needed for these arguments.
sysroot=Path(subprocess.check_output(['rustc','--print','sysroot']).decode().strip())
library=sysroot/'lib/rustlib/src/rust/library'
for name in ['core/src/array/drain.rs','core/src/slice/sort/stable/mod.rs','core/src/slice/sort/stable/drift.rs','core/src/slice/sort/stable/quicksort.rs','core/src/slice/sort/unstable/mod.rs','core/src/slice/sort/shared/smallsort.rs','alloc/src/raw_vec/mod.rs','alloc/src/alloc.rs']:
    source=library/name;dest=OUT/'rust-1.92-source'/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(source.read_bytes())
    files[str(dest.relative_to(OUT))]=dict(installed_source=str(source),sha256=sha(dest.read_bytes()))
receipt=dict(status='PASS_FROZEN_SOURCE_AND_TARGETED_EVIDENCE__COMPLETE_PREFLIGHT_BLOCKED',source=binding['source'],source_tree=binding['source_tree'],binary_sha256=binding['binary_sha256'],unchanged_sources=checks,baseline_target=target,geometry_requested_payload=23584,geometry_runtime_capacity_limit=64<<20,geometry_allocator_cost=None,geometry_recursion_active_frames=2,geometry_recursion_fixed_bytes=640,partition_max_elements=66,stable_sort_stack_scratch=4096,stable_sort_heap_scratch_success_path=0,candidate_specific_array_success_frames=[64,16],callback_success_heap=0,files=files,native_equations=0,native_invocations=0,complete_memory_bound=None,FD_execution_allowed=False)
print(json.dumps(receipt,indent=2,sort_keys=True))
