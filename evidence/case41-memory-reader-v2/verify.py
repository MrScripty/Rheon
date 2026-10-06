"""Reader checks and fail-closed mutations; no Rust or numerical execution."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('reader_v2',P/'audit_memory.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
def req(v,msg):
 if not v:raise ValueError(msg)
r=a.run();req(r['status']=='BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING','blocker retained')
req(r['known_crate_and_storage_subtotal']==66816 and r['cap']==67584 and r['unallocated_headroom']==768,'subtotal arithmetic only')
req(r['workspace_bytes']==62096 and r['outer_frame']['frame_bound']==62192 and r['outer_residual_bytes']==96 and r['explicit_arrays_bytes']==528 and r['added_crate_frame_peak']==4096,'all known storage charges')
req(r['absorbed_operations']==['Work::equation'],'only justified jointly absent operation')
req(r['deltas']['Work::case41_fd_equation']['candidate']==[183472] and r['deltas']['Work::case41_fd_equation']['reference']==[184064],'entire absorbed observer frames')
req(r['native_equations']==r['owner_advances']==0 and r['absolute_stack_bound_not_established'] and r['shared_recursions'],'limitations retained')
req({k:len(v) for k,v in r['frames'].items()}=={'candidate':46,'reference':52},'all reached crate functions')
for kind in ['candidate','reference']:
 for suffix in ['coefficients','cross','linear','Work::partition','Work::point']:
  req(any('::'+kind+'::' in row['name'] and row['name'].endswith('::'+suffix) for row in r['frames'][kind].values()),'numerical helper retained '+suffix)
original_load=a.load;checks=[]
def altered(kind):
 def load(name,path):
  m=original_load(name,path)
  if name!='frames':return m
  if kind=='one-sided-missing-point':
   old=m.functions
   def funcs(raw,symbols):return {k:v for k,v in old(raw,symbols).items() if not v['name'].endswith('::candidate::Work::point')}
   m.functions=funcs
  elif kind=='missing-absorbed-body-edge':
   old=m.transfers
   def transfers(f):
    rows=old(f)
    if f['name'].endswith(('::candidate::Work::case41_fd_equation','::reference::Work::capture_equations')):
     rows=[e for e in rows if '::Work::partition>' not in e['instruction']]
    return rows
   m.transfers=transfers
  elif kind=='outer-smaller-than-workspace':
   old=m.fixed_frame
   def frame(f):
    row=old(f)
    if f['name'].endswith('::case41_six_columns_only'):row['frame_bound']=62095
    return row
   m.fixed_frame=frame
  elif kind=='unknown-indirect-call':
   old=m.transfers
   def transfers(f):
    rows=old(f)
    if f['name'].endswith('::candidate::case41_fd_capture'):rows.append(dict(address=f['start'],instruction='call *%rax',opcode='call',target=None,kind='call'))
    return rows
   m.transfers=transfers
  return m
 return load
for kind in ['one-sided-missing-point','missing-absorbed-body-edge','outer-smaller-than-workspace','unknown-indirect-call']:
 a.load=altered(kind)
 try:
  trial=a.run()
 except ValueError as e:
  req(kind!='unknown-indirect-call','unknown call is retained as explicit blocker');checks.append(dict(case=kind,status='REJECTED',reason=str(e)))
 else:
  req(kind=='unknown-indirect-call' and trial['status']=='BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING' and any(e['classification']=='unresolved_indirect' for e in trial['external_transfer_ledger']['candidate']),'unresolved callee cannot grant a pass')
  checks.append(dict(case=kind,status='BLOCKED_WITH_TRANSFER_RETAINED'))
 finally:a.load=original_load
print(json.dumps(dict(status='PASS_READER_CHECKS_AND_FOUR_FAIL_CLOSED_CONTROLS',controls=checks,native_equations=0,owner_advances=0,complete_memory_bound=None),indent=2,sort_keys=True))
