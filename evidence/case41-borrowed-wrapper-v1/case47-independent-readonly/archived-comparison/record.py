"""Record exact archived input identities, reader outcomes and preservation."""
from pathlib import Path
import hashlib,json,subprocess,sys
P=Path(__file__).resolve().parent;R=P.parents[1];F=Path('/workspace/Rheon-fd-pure-e2')
def req(v,msg):
 if not v:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*args,cwd=R):return subprocess.check_output(['git',*args],cwd=cwd).decode().strip()
def tree(commit):
 out={}
 for row in subprocess.check_output(['git','ls-tree','-r','-z',commit],cwd=R).split(b'\0'):
  if row:
   meta,path=row.split(b'\t',1);out[path.decode()]=meta.decode()
 return out
req(not(P/'receipt.json').exists(),'freeze this result once')
policy=json.loads((P/'protocol.json').read_text())
for path,h in policy['input_sha256'].items():req(sha((F/path).read_bytes())==h,'preserved input '+path)
for path,h in policy['reader_sha256'].items():req(sha((P/path).read_bytes())==h,'frozen reader '+path)
for a,b in [('analysis-normal.json','analysis-optimized.json'),('verification-normal.json','verification-optimized.json')]:req((P/a).read_bytes()==(P/b).read_bytes(),'mode parity')
result=json.loads((P/'analysis-normal.json').read_text());checks=json.loads((P/'verification-normal.json').read_text());req(result['status']=='PASS_ONE_ARCHIVED_PURE_E2_ENDPOINT_COMPARISON' and len(checks['controls'])==4,'actual comparison and mismatch guards')
base=tree(policy['preserved_base']);current=tree('HEAD');req(len(base)==8462 and all(current.get(path)==blob for path,blob in base.items()),'reviewed predecessor tree preserved')
refcommit=result['reference_publication']
for path in ['run_references.py','reference-pressure_state-nonconstant-reversed.json','reference-authorization.json']:
 rel='evidence/forcing-e1-trajectories-v2/'+path;raw=subprocess.check_output(['git','show',refcommit+':'+rel],cwd=R);req(raw==(F/rel).read_bytes(),'exact reference publication source/data '+path)
commands=[]
for label,script,optimized in [('analysis-normal','analyze.py',False),('analysis-optimized','analyze.py',True),('verification-normal','verify.py',False),('verification-optimized','verify.py',True)]:
 out=label+'.json';err=label+'-stderr.log';req(not(P/err).read_bytes(),'actual reader completed without stderr')
 commands.append(dict(argv=[sys.executable,*(['-O']if optimized else[]),str(P/script)],exit=0,resource_env=dict(OPENBLAS_NUM_THREADS='1',OMP_NUM_THREADS='1',PYTHONDONTWRITEBYTECODE='1'),stdout=out,stdout_sha256=sha((P/out).read_bytes()),stderr=err,stderr_sha256=sha((P/err).read_bytes()),reader_sha256=sha((P/script).read_bytes()),new_native_invocation=False,reference_integration=False,elapsed_time_not_recorded=True))
(P/'commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n')
# Check the reviewed packets against their own receipts, including publication
# supplements; no previously published status or limitation is edited here.
Q=R/'evidence/case41-memory-reader-v2';C=R/'evidence/case47-outer-frame-clarification-v1';reviewed={}
for directory,name,key in [(Q,'receipt.json','artifact_sha256'),(Q,'publication-supplements.json','supplement_sha256'),(C,'receipt.json','artifact_sha256')]:
 receipt=json.loads((directory/name).read_text())
 for path,h in receipt[key].items():req(sha((directory/path).read_bytes())==h,'reviewed bytes '+str(directory/path))
 reviewed[str((directory/name).relative_to(R))]=sha((directory/name).read_bytes())
source=git('rev-parse','HEAD');req(source=='b7af5b24fb020a0b6eafd0f6d546976f6845be66','exact comparison source')
receipt=dict(status='FROZEN_ONE_ARCHIVED_CASE47_ENDPOINT_COMPARISON',comparison_source=source,comparison_source_tree=git('rev-parse','HEAD^{tree}'),compiled_native_source=result['compiled_source'],compiled_native_tree=result['compiled_source_tree'],binary_sha256=result['binary_sha256'],reference_publication=result['reference_publication'],reference_publication_tree=result['reference_publication_tree'],analysis_sha256=sha((P/'analysis-normal.json').read_bytes()),protocol_sha256=sha((P/'protocol.json').read_bytes()),reviewed_receipts_unchanged=reviewed,preserved_base=policy['preserved_base'],preserved_base_tree=policy['preserved_base_tree'],preserved_base_files=len(base),input_files=len(policy['input_sha256']),new_reference_integrations=0,new_trajectories=0,new_native_invocations=0,new_FD_equations=0,new_instantaneous_equations=0,new_build_invocations=0,temporal_order_qualified=False,broader_geometry_qualified=False,artifact_sha256={str(x.relative_to(P)):sha(x.read_bytes())for x in sorted(P.rglob('*'))if x.is_file() and x.name!='receipt.json' and '__pycache__'not in x.parts})
(P/'receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(status=receipt['status'],receipt_sha256=sha((P/'receipt.json').read_bytes()),preserved_base_files=len(base)),indent=2,sort_keys=True))
