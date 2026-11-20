"""Fixed ordinary qualification driver; review exact staged source before use.

Does not refresh admission pins, select ignored tests, run a sealed campaign,
time performance, publish artifacts, or mutate Git refs. Requires final PR143
actual merged-main ancestry and an independently approved clean PR59 head.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--head',required=True)
p.add_argument('--base',required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
repo=Path('/workspace/Rheon')
def git(*args):
    return subprocess.check_output(['git',*args],cwd=repo,text=True).strip()
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def now():return datetime.now(timezone.utc).isoformat()
for name,value in [('head',a.head),('base',a.base)]:
    if len(value)!=40 or any(c not in '0123456789abcdef' for c in value):
        raise ValueError('full lowercase commit required: '+name)
if git('rev-parse','HEAD')!=a.head or git('status','--porcelain'):
    raise ValueError('requires exact clean committed candidate')
subprocess.run(['git','merge-base','--is-ancestor',a.base,a.head],cwd=repo,check=True)
if a.base in {'15026a966265cf9eaedf464e2b5a066bd0fef3d8','b70dbae515d01ced3762e0fff030df7f8b13363e','7befee45270280d07693b91f4afa934ec1780b66'}:
    raise ValueError('actual PR143 merged-main base required')
output=a.output.resolve()
if output.exists() or repo==output or repo in output.parents:
    raise ValueError('fresh external output required')
output.mkdir(parents=True,exist_ok=False)
tracked=git('ls-files').splitlines()
sources={name:sha(repo/name) for name in tracked if (repo/name).is_file()}
env=dict(os.environ)
unset=['GITHUB_SHA','GITHUB_REF','GITHUB_HEAD_REF','RHEON_SOURCE_REVISION','RHEON_BUILD_MATERIAL_SHA256']
for name in unset:env.pop(name,None)
env.pop('RHEON_ALIGNED_PACKET',None)
env.pop('RHEON_ALIGNED_NATIVE_DUMP',None)
native=output/'native'
packet=output/'packet'
target=Path(env.get('CARGO_TARGET_DIR',str(repo/'target')))
if not target.is_absolute():target=repo/target
executable=target/'debug/examples/aligned_strain'
commands=[
 ('admission',[sys.executable,'docs/education/check_native_admission.py']),
 ('ci-budget',[sys.executable,'-m','unittest','test_column_interface_verifier.CiBudgetContract','-v'],'tools'),
 ('ci-budget-optimized',[sys.executable,'-O','-m','unittest','test_column_interface_verifier.CiBudgetContract','-v'],'tools'),
 ('native-qualification',[sys.executable,'tools/qualify_aligned_strain.py',str(native)]),
 ('real-packet',[sys.executable,'docs/education/aligned_strain_packet.py','--output',str(packet),'--executable',str(executable),'--primary-qualification',str(native/'qualification.json')]),
 ('packet-tests',[sys.executable,'-m','unittest','discover','-s','docs/education','-p','test_aligned_strain_packet.py','-v']),
 ('packet-tests-optimized',[sys.executable,'-O','-m','unittest','discover','-s','docs/education','-p','test_aligned_strain_packet.py','-v']),
]
journal={'preliminary':False,'source_head':a.head,'source_tree':git('rev-parse','HEAD^{tree}'),'actual_pr143_base':a.base,'clean':True,'source_sha256':sources,'unset_environment_names':unset,'commands':[],'started':now(),'status':'running','sealed_campaign_selected':False,'performance_selected':False}
receipt=output/'journal.json'
def save():receipt.write_text(json.dumps(journal,indent=2)+'\n')
save()
try:
    for row in commands:
        name,argv=row[:2]
        cwd=repo/row[2] if len(row)>2 else repo
        actual_env=dict(env)
        if name.startswith('packet-tests'):actual_env['RHEON_ALIGNED_PACKET']=str(packet)
        item={'name':name,'argv':argv,'cwd':str(cwd),'started':now(),'returncode':None,'status':'running'}
        journal['commands'].append(item);save()
        log=output/(name+'.log')
        with log.open('xb') as f:
            result=subprocess.run(argv,cwd=cwd,env=actual_env,stdout=f,stderr=subprocess.STDOUT,timeout=3600)
        item.update(ended=now(),returncode=result.returncode,status='passed' if result.returncode==0 else 'failed',log_sha256=sha(log));save()
        if result.returncode:raise RuntimeError(name+' failed; no retry selected')
    if git('rev-parse','HEAD')!=a.head or git('status','--porcelain') or git('ls-files').splitlines()!=tracked:
        raise RuntimeError('source identity/status/path set changed')
    if any(sha(repo/name)!=digest for name,digest in sources.items()):
        raise RuntimeError('source bytes changed')
    journal.update(status='passed',source_unchanged=True,exporter_sha256=sha(executable))
except BaseException as error:
    journal.update(status='failed-or-interrupted',error=type(error).__name__+': '+str(error))
    raise
finally:
    journal['ended']=now();save()
print(json.dumps({'status':journal['status'],'source_head':a.head,'commands':len(journal['commands']),'journal_sha256':sha(receipt)}))
