"""Derive CI coverage from executed rows and preserve the superseded receipt."""
import copy,hashlib,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent;ROOT=P.parents[1];RECEIPT=P/'final-receipt.json';BASE='bc66591cb88d07a4c55cb5f9b586dd08a36b2143'
def require(ok,message):
 if not ok:raise ValueError(message)
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT)
def sha(data):return hashlib.sha256(data).hexdigest()
def tree(c):return {l.split(b'\t',1)[1].decode():l.split(b'\t',1)[0].split()[2].decode()for l in git('ls-tree','-r',c).splitlines()}
def verify(r,evidence_commit=None):
 require(r['status']=='PASS_EXECUTED_PATH_LEDGER_COUNT_CORRECTION','count correction status')
 source=r['qualified_source_commit'];s=tree(source);b=tree(BASE);allowed={'tools/test_column_interface_verifier.py'}
 require(git('rev-parse',source+'^{tree}').decode().strip()==r['qualified_source_tree'],'source tree')
 require(git('show','-s','--format=%P',source).decode().split()==[BASE]and r['ordered_source_parents']==[BASE],'separate exact repair parent')
 require(r['preserved_git_blobs']=={p:o for p,o in b.items()if p not in allowed}and all(s.get(p)==o for p,o in r['preserved_git_blobs'].items()),'unchanged workflow/Rust/old source and evidence')
 for p,d in r['source_sha256'].items():require(sha(git('show',source+':'+p))==d and sha((ROOT/p).read_bytes())==d,'candidate source '+p)
 e=tree(evidence_commit)if evidence_commit else None;prefix=str(P.relative_to(ROOT))+'/'
 files={p for p in e if p.startswith(prefix)and p!=str(RECEIPT.relative_to(ROOT))}if e is not None else{str(p.relative_to(ROOT))for p in P.rglob('*')if p.is_file()and '__pycache__'not in p.parts and p!=RECEIPT}
 require(files==set(r['file_sha256']),'complete correction inventory')
 for p,d in r['file_sha256'].items():require(sha(git('show',evidence_commit+':'+p)if e else(ROOT/p).read_bytes())==d,'correction evidence '+p)
 examples=sorted(p for p in s if p.startswith('examples/')and p.endswith('.rs'))
 additional=['tools/verify_column_interface.py','tools/test_column_interface_verifier.py','evidence/column-interface/demo/jacobi-pcg-v1-activation/final.png'];expected=[(event,p)for event in ['pull_request','push']for p in examples+additional]
 for mode in ['normal','optimized']:
  ledger=json.loads((P/(mode+'-ledger.json')).read_text())
  require(ledger['status']=='PASS_ACTUAL_EXECUTED_PATH_LEDGER'and ledger['source_commit']==source and ledger['actual_test_methods']==1,'actual executed method '+mode)
  require(ledger['tracked_examples']==examples and ledger['additional_paths']==additional,'exact tracked source inventory')
  require([(row['event'],row['path'])for row in ledger['controls']]==expected and all(row['matched_patterns']for row in ledger['controls']),'every actual matched event/path')
  require(ledger['tracked_example_count']==len(examples)and ledger['actual_control_count']==len(expected),'derived ledger counts')
  for p,d in ledger['source_sha256'].items():require(r['source_sha256'][p]==d,'executed source '+p)
 require((P/'normal-ledger.json').read_bytes()==(P/'optimized-ledger.json').read_bytes(),'normal optimized path ledger parity')
 require(r['tracked_example_count']==len(examples)and r['actual_control_count']==len(expected),'reported counts derive from rows')
 oldpath='evidence/column-interface-final-artifacts/final-receipt.json';old=json.loads(git('show',BASE+':'+oldpath))
 require(sha((ROOT/oldpath).read_bytes())==r['superseded_receipt_sha256']==sha(git('show',BASE+':'+oldpath)),'unchanged superseded receipt')
 require(old['ci_path_controls']==r['superseded_unsupported_control_claim']and old['ci_path_controls']!=len(expected),'explicit old overstatement')
 for p,d in old['source_sha256'].items():require(sha(git('show',old['qualified_source_commit']+':'+p))==d,'original thirteen source hashes')
 for p,d in old['file_sha256'].items():require(sha(git('show',BASE+':'+p))==d and sha((ROOT/p).read_bytes())==d,'original forty packet hashes')
 chain=json.loads((P/'prior-parent-chain.json').read_text());parent=chain['base']
 require(chain['head']==BASE and len(chain['ordered_commits'])==5,'original five repair commits')
 for row in chain['ordered_commits']:
  require(row['parent']==parent and git('show','-s','--format=%P',row['commit']).decode().split()==[parent]and git('rev-parse',row['commit']+'^{tree}').decode().strip()==row['tree'],'ordered exact parent/tree '+row['commit']);parent=row['commit']
 require(parent==BASE and r['hosted_CI_qualified']is False,'chain endpoint and hosted scope')
 require(git('show','-s','--format=%an%n%ae%n%cn%n%ce',source).decode().splitlines()==['MrScripty','TheEnvironmentGuy@protonmail.com']*2,'new source identity')
 return dict(status=r['status'],source=source,tree=r['qualified_source_tree'],tracked_examples=len(examples),actual_control_count=len(expected),source_paths=len(r['source_sha256']),packet_files=len(files),superseded_unsupported_control_claim=old['ci_path_controls'],unchanged_original_source_hashes=len(old['source_sha256']),unchanged_original_packet_hashes=len(old['file_sha256']),hosted_CI_qualified=False)
if __name__=='__main__':
 r=json.loads(RECEIPT.read_text());commit=sys.argv[sys.argv.index('--evidence-commit')+1]if'--evidence-commit'in sys.argv else None;print(json.dumps(verify(r,commit),indent=2))
 if '--negative-self-test'in sys.argv:
  for field in ['source_sha256','file_sha256','preserved_git_blobs']:
   bad=copy.deepcopy(r);bad[field][next(iter(bad[field]))]='0'*len(next(iter(bad[field].values())))
   try:verify(bad,commit)
   except ValueError as error:print('REJECTED '+field+': '+str(error))
   else:raise ValueError('corrupt binding accepted '+field)
  bad=copy.deepcopy(r);bad['actual_control_count']+=2
  try:verify(bad,commit)
  except ValueError as error:print('REJECTED unsupported count: '+str(error))
  else:raise ValueError('unsupported count accepted')
