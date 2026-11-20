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
import re
import subprocess
import sys

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--head',required=True)
p.add_argument('--base',required=True)
p.add_argument('--approved-binding',type=Path,required=True)
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
binding_bytes=a.approved_binding.read_bytes()
binding_digest=hashlib.sha256(binding_bytes).hexdigest()
binding=json.loads(binding_bytes)
if (binding.get('candidate_head')!=a.head or binding.get('actual_pr143_merge')!=a.base
        or binding.get('actual_pr143_merge_verified') is not True
        or binding.get('independent_final_source_review_passed') is not True
        or binding.get('candidate_tree')!=git('rev-parse',a.head+'^{tree}')
        or binding.get('base_tree')!=git('rev-parse',a.base+'^{tree}')):
    raise ValueError('exact positive approved final-source/actual143 binding required')
if git('rev-parse','HEAD')!=a.head or git('status','--porcelain'):
    raise ValueError('requires exact clean committed candidate')
subprocess.run(['git','merge-base','--is-ancestor',a.base,a.head],cwd=repo,check=True)
if a.base in {'15026a966265cf9eaedf464e2b5a066bd0fef3d8','b70dbae515d01ced3762e0fff030df7f8b13363e','7befee45270280d07693b91f4afa934ec1780b66'}:
    raise ValueError('actual PR143 merged-main base required')
output=a.output.resolve()
if output.exists() or repo==output or repo in output.parents:
    raise ValueError('fresh external output required')
existing=output.parent
while not existing.exists():existing=existing.parent
if subprocess.run(['git','-C',str(existing),'rev-parse','--is-inside-work-tree'],capture_output=True).returncode==0:
    raise ValueError('output must be outside every Git worktree')
output.mkdir(parents=True,exist_ok=False)
(output/'approved-binding.json').write_bytes(binding_bytes)
tracked=git('ls-files').splitlines()
sources={name:sha(repo/name) for name in tracked if (repo/name).is_file()}
env=dict(os.environ)
unset=['GITHUB_SHA','GITHUB_REF','GITHUB_HEAD_REF','RHEON_SOURCE_REVISION','RHEON_BUILD_MATERIAL_SHA256','RHEON_ALIGNED_PACKET','RHEON_ALIGNED_NATIVE_DUMP']
for name in unset:env.pop(name,None)
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
journal={'preliminary':False,'source_head':a.head,'source_tree':git('rev-parse','HEAD^{tree}'),'actual_pr143_base':a.base,'approved_binding_sha256':binding_digest,'clean':True,'source_sha256':sources,'unset_environment_names':unset,'commands':[],'started':now(),'status':'running','sealed_campaign_selected':False,'performance_selected':False}
receipt=output/'journal.json'
def save():
    temporary=output/'.journal-next.json'
    with temporary.open('w') as f:
        f.write(json.dumps(journal,indent=2)+'\n');f.flush();os.fsync(f.fileno())
    os.replace(temporary,receipt)
def exporter_identity():
    native_receipt=json.loads((native/'qualification.json').read_text())
    digest=sha(executable)
    expected_names=['format','rust-contracts','rust-build','rust-clippy','qualification-gate-tests','qualification-gate-tests-optimized']
    for case in ['native','unit-center-cube','represented-nonmidpoint','reflected-anisotropic','large-origin-nonmidpoint']:
        expected_names.extend([case+'-run',case+'-oracle-comparison'])
    expected_names.extend(['oracle-tests','oracle-tests-optimized'])
    recorded=native_receipt.get('commands',[])
    if (native_receipt.get('status')!='passed' or native_receipt.get('clean') is not True
            or native_receipt.get('source_head')!=a.head or native_receipt.get('source_tree')!=journal['source_tree']
            or native_receipt.get('executable_sha256')!=digest
            or [c.get('name') for c in recorded]!=expected_names or any(c.get('exit')!=0 for c in recorded)):
        raise RuntimeError('exporter differs from actual native qualification')
    if (packet/'qualification.json').exists():
        packet_receipt=json.loads((packet/'qualification.json').read_text())
        if (packet_receipt.get('executable_sha256')!=digest or packet_receipt.get('status')!='passed'
                or packet_receipt.get('clean') is not True or packet_receipt.get('source_head')!=a.head
                or packet_receipt.get('source_tree')!=journal['source_tree']
                or packet_receipt.get('native_source_head')!=a.head or packet_receipt.get('native_source_tree')!=journal['source_tree']
                or packet_receipt.get('primary_qualification_sha256')!=sha(native/'qualification.json')):
            raise RuntimeError('exporter differs from actual packet qualification')
    return digest
def retain_binary(path, expected):
    if not path.is_absolute():path=repo/path
    if path.is_symlink() or path.read_bytes()[:4]!=b'\x7fELF' or sha(path)!=expected:
        raise RuntimeError('regular unchanged ELF required: '+str(path))
    destination=output/'executables'/path.name
    destination.parent.mkdir(exist_ok=True)
    with path.open('rb') as source, destination.open('xb') as target:
        while block:=source.read(1024*1024):target.write(block)
    if sha(path)!=expected or sha(destination)!=expected:
        raise RuntimeError('retained ELF identity changed: '+str(path))
    return {'original_path':str(path),'retained_path':str(destination),'sha256':expected,'size':path.stat().st_size}
def require_source_unchanged():
    if git('rev-parse','HEAD')!=a.head or git('status','--porcelain') or git('ls-files').splitlines()!=tracked:
        raise RuntimeError('source identity/status/path set changed')
    if any(sha(repo/name)!=digest for name,digest in sources.items()):
        raise RuntimeError('source bytes changed')
save()
try:
    for row in commands:
        name,argv=row[:2]
        cwd=repo/row[2] if len(row)>2 else repo
        actual_env=dict(env)
        if name.startswith('packet-tests'):actual_env['RHEON_ALIGNED_PACKET']=str(packet)
        item={'name':name,'argv':argv,'cwd':str(cwd),'environment_additions':{'RHEON_ALIGNED_PACKET':str(packet)} if name.startswith('packet-tests') else {},'started':now(),'returncode':None,'status':'running'}
        journal['commands'].append(item);save()
        log=output/(name+'.log')
        try:
            with log.open('xb') as f:
                result=subprocess.run(argv,cwd=cwd,env=actual_env,stdout=f,stderr=subprocess.STDOUT,timeout=3600)
        except BaseException as error:
            item.update(ended=now(),status='interrupted-or-uncertain',exception=type(error).__name__+': '+str(error),log_sha256=sha(log) if log.exists() else None)
            save();raise
        item.update(ended=now(),returncode=result.returncode,status='passed' if result.returncode==0 else 'failed',log_sha256=sha(log));save()
        if result.returncode:raise RuntimeError(name+' failed; no retry selected')
        if name in {'native-qualification','real-packet'}:
            item['verified_exporter_sha256']=exporter_identity();save()
    require_source_unchanged()
    exporter_digest=exporter_identity()
    binaries=[retain_binary(executable,exporter_digest)]
    paths=re.findall(r'^\s*Running .+? \(([^()]+)\)\s*$',(native/'rust-contracts.log').read_text(),re.MULTILINE)
    expected_prefixes=['aligned_strain_contract-','obstacle_flow_contract-','static_obstacle_contract-','aligned_strain-']
    if len(paths)!=4 or len(set(paths))!=4 or any(sum(Path(name).name.startswith(prefix) for name in paths)!=1 for prefix in expected_prefixes):
        raise RuntimeError('exact four native Rust test executables required')
    for name in dict.fromkeys(paths):
        path=Path(name)
        if not path.is_absolute():path=repo/path
        binaries.append(retain_binary(path,sha(path)))
    require_source_unchanged()
    if exporter_identity()!=exporter_digest:raise RuntimeError('exporter changed during retention')
    journal.update(status='passed',source_unchanged=True,exporter_sha256=exporter_digest,retained_executables=binaries)
except BaseException as error:
    journal.update(status='failed-or-interrupted',error=type(error).__name__+': '+str(error))
    raise
finally:
    journal['ended']=now();save()
print(json.dumps({'status':journal['status'],'source_head':a.head,'commands':len(journal['commands']),'journal_sha256':sha(receipt)}))
