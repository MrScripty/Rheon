"""Bind read-only diagnosis, guard checks and all frozen predecessor bytes."""
from pathlib import Path
import collections,hashlib,json,os,subprocess
P=Path(__file__).resolve().parent;F=Path('/workspace/Rheon-fd-pure-e2');D=F/'evidence/case41-scaled-fd-preparation-v1'
def req(v,msg):
 if not v:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def git(root,*args):return subprocess.check_output(['git',*args],cwd=root).decode().strip()
req(not(P/'receipt.json').exists(),'record diagnosis once')
policy=json.loads((P/'protocol.json').read_text())
for path,digest in policy['tooling_sha256'].items():req(sha((P/path).read_bytes())==digest,'frozen tooling '+path)
req(sha((F/policy['original_blocked_receipt_path']).read_bytes())==policy['original_blocked_receipt_sha256'],'old blocked receipt unchanged')
req((P/'verify-normal.log').read_bytes()==(P/'verify-optimized.log').read_bytes(),'normal/optimized verifier parity')
for name in ['verify-normal','verify-optimized']:req(json.loads((P/(name+'.log')).read_text())['status']=='PASS_READER_CHECKS_AND_FOUR_FAIL_CLOSED_CONTROLS','reader controls passed')
r=json.loads((P/'preflight-receipt.json').read_text());req(r['native_test_invocations']==3 and r['FD_test_invocations']==0 and r['native_equations']==r['owner_advances']==0 and r['complete_memory_bound'] is None,'guard-only preflight scope')
commands=[]
for kind,argv in [('symbols',['nm','-S','-nC',r['binary']]),('relocations',['readelf','-rW',r['binary']]),('headers',['readelf','-hW',r['binary']])]:
 result=subprocess.run(argv,capture_output=True);req(result.returncode==0,'actual ELF metadata read');req(result.stdout==(D/('native-'+kind+'.txt')).read_bytes(),'actual archived '+kind)
 commands.append(dict(argv=argv,exit=0,stdout_sha256=sha(result.stdout),matches_archived=True,stderr_sha256=sha(result.stderr)))
(P/'elf-metadata-checks.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n')
m=json.loads((P/'memory-normal.log').read_text());E=m['external_transfer_ledger']
alias={}
for line in (D/'native-symbols.txt').read_text().splitlines():
 fields=line.split(maxsplit=3)
 if len(fields)==4:
  try:addr=int(fields[0],16)
  except ValueError:continue
  alias.setdefault(addr,[]).append(fields[3])
def key(e):return e['target'],tuple((e.get('binding')or{}).get('names',[]))
ref={key(e) for e in E['reference']};groups=collections.defaultdict(list)
for e in E['candidate']:groups[key(e)].append(e)
summary=dict(status='BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING',counts={k:dict(collections.Counter(e['classification']for e in v))for k,v in E.items()},distinct_candidate_targets=len(groups),distinct_reference_targets=len(ref),candidate_only_targets=[dict(address=hex(k[0]) if k[0] is not None else None,names=list(k[1]) or alias.get(k[0],[]),callers=sorted({e['caller'] for e in v}),transfer_count=len(v))for k,v in groups.items()if k not in ref],common_target_does_not_prove_common_memory=True,complete_memory_bound=None)
(P/'transfer-summary.json').write_text(json.dumps(summary,indent=2,sort_keys=True)+'\n')
req(git(F,'rev-parse','HEAD')==policy['frozen_history'],'frozen full worktree HEAD')
rows=[]
for entry in subprocess.check_output(['git','ls-tree','-r','-z','HEAD'],cwd=F).split(b'\0'):
 if not entry:continue
 meta,path=entry.split(b'\t',1);mode,typ,oid=meta.decode().split();rel=path.decode();file=F/rel
 req(typ=='blob','all inherited entries are blobs')
 data=os.readlink(file).encode() if mode=='120000' else file.read_bytes()
 req(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==oid,'unchanged inherited bytes '+rel)
 rows.append(dict(path=rel,mode=mode,git_blob=oid,sha256=sha(data)))
raw=json.dumps(rows,sort_keys=True,separators=(',',':')).encode();inventory=dict(source=policy['frozen_history'],tree=policy['frozen_tree'],files=len(rows),full_byte_inventory_sha256=sha(raw),all_bytes_match_git_blobs=True)
(P/'frozen-inventory.json').write_text(json.dumps(inventory,indent=2,sort_keys=True)+'\n')
# Existing publication helper enumerates sixteen frozen worktrees; append the
# two later frozen milestones, leaving the current tooling worktree separate.
import importlib.util
spec=importlib.util.spec_from_file_location('old_postgit',F/'evidence/case41-scaled-fd-preparation-v1/record_post_git.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
expected=dict(old.FROZEN);expected['Rheon-case41-next-plan']='8fc718ff603e7a7123037e592b4f40f59762e011';expected['Rheon-fd-pure-e2']=policy['frozen_history'];heads=[]
for name,commit in sorted(expected.items()):
 root=Path('/workspace')/name;actual=git(root,'rev-parse','HEAD');status=git(root,'status','--porcelain');req(actual==commit and not status,'frozen worktree unchanged '+name);heads.append(dict(worktree=str(root),head=actual,tree=git(root,'rev-parse','HEAD^{tree}'),clean=True))
(P/'frozen-worktrees.json').write_text(json.dumps(heads,indent=2,sort_keys=True)+'\n')
receipt=dict(status='PUBLISHED_DIAGNOSIS__FD_MEMORY_PREFLIGHT_BLOCKED',tooling_source=r['source'],tooling_source_tree=r['source_tree'],compiled_source=r['compiled_source'],compiled_source_tree=r['compiled_source_tree'],binary=r['binary'],binary_sha256=r['binary_sha256'],guard_receipt_sha256=sha((P/'preflight-receipt.json').read_bytes()),preserved_blocked_receipt_sha256=policy['original_blocked_receipt_sha256'],known_crate_and_storage_subtotal=66816,complete_memory_bound=None,cap=67584,native_equations=0,owner_advances=0,FD_test_invocations=0,native_test_invocations=3,build_invocations=0,FD_execution_allowed=False,inherited_inventory=inventory,frozen_worktree_count=len(heads),artifact_sha256={str(x.relative_to(P)):sha(x.read_bytes())for x in sorted(P.rglob('*'))if x.is_file() and x.name!='receipt.json' and '__pycache__'not in x.parts})
(P/'receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(status=receipt['status'],receipt_sha256=sha((P/'receipt.json').read_bytes()),frozen_worktrees=len(heads),inherited_files=len(rows)),indent=2,sort_keys=True))
