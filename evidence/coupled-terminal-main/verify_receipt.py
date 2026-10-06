"""Verify frozen current-main source, original failure and hosted qualification.

Use explicit failures rather than assertions so optimized Python is equivalent.
All bindings are local Git objects and retained actual tool outputs/logs.
"""
from pathlib import Path
import copy,csv,datetime,hashlib,io,json,re,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
E=Path(__file__).resolve().parent
C059='c0591ebc4c2619d5ad6e6decfe9d3957ace8db5b'
BASE='773bd2725e35590cfe9cbca5625f5239b98f03c2'
EXPECTED_NAMES={'executable-contracts','Feature contracts (default)',
                'Feature contracts (core-only)','Feature contracts (desktop)',
                'core-and-executable'}

def require(ok,message):
    if not ok:raise ValueError(message)

def blob(commit,path):
    return subprocess.check_output(['git','show',commit+':'+path],cwd=ROOT)

def verify(r,contents):
    require(r['base']==BASE and r['local_source']==C059,'frozen baseline/local identities')
    source=r['source']
    require(subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==r['source_tree'],'source tree')
    require(subprocess.check_output(['git','rev-parse',source+'^'],cwd=ROOT,text=True).strip()==C059,'ordinary additive repair parent')
    changed=set(subprocess.check_output(['git','diff','--name-only',C059,source],cwd=ROOT,text=True).splitlines())
    require(changed=={'.github/workflows/rust-rheon.yml','tools/test_column_interface_verifier.py',
        'docs/research-book/implementation/generic-terminal-validation.md',
        'evidence/coupled-terminal-main/verify_workflow.py','evidence/coupled-terminal-main/README.md'},
        'only the exact workflow compatibility/test/explanation repair; no native changes')
    ident=subprocess.check_output(['git','show','-s','--format=%an <%ae> / %cn <%ce>',source],cwd=ROOT,text=True).strip()
    require(ident=='MrScripty <TheEnvironmentGuy@protonmail.com> / MrScripty <TheEnvironmentGuy@protonmail.com>','approved author/committer')
    for path,digest in r['source_sha256'].items():
        require(hashlib.sha256(blob(source,path)).hexdigest()==digest,'source bytes: '+path)
    for path,digest in r['evidence_sha256'].items():
        require(hashlib.sha256(contents[path]).hexdigest()==digest,'retained evidence bytes: '+path)
    local=json.loads(contents['local-matrix.json'])
    require(local['source']==C059 and len(local['commands'])==10,'complete ten-command local matrix')
    wanted=['format']+[kind+'-'+mode for mode in ['default','core-only','desktop']for kind in ['clippy','tests','doc-tests']]
    require([x['name']for x in local['commands']]==wanted and all(x['exit']==0 for x in local['commands']),'every actual local command passes')
    for path,digest in local['source_sha256'].items():
        require(hashlib.sha256(blob(C059,path)).hexdigest()==digest,'original local binding: '+path)
        if path!='.github/workflows/rust-rheon.yml':
            require(hashlib.sha256(blob(source,path)).hexdigest()==digest,'identical qualified Rust/Cargo/example/test bytes: '+path)
    jobs=json.loads(contents[r['hosted_jobs_file']])['response']['structuredContent']['jobs']
    require({j['name']for j in jobs}==EXPECTED_NAMES and len(jobs)==5,'hosted full job inventory')
    require(all(j['run_id']==r['hosted_run'] and j['status']=='completed' and j['conclusion']=='success' for j in jobs),'all hosted jobs terminal and successful')
    for name,path in r['hosted_job_logs'].items():
        log=contents[path].decode()
        require(name in EXPECTED_NAMES and '##[error]'not in log,'hosted successful log: '+name)
        times=re.findall(r'(?m)^(2026-\S+Z) ',log)
        dt=lambda x:datetime.datetime.fromisoformat(x.replace('Z','+00:00'))
        actual_span=(dt(times[-1])-dt(times[0])).total_seconds()
        require(actual_span==r['hosted_log_span_seconds'][name],'actual measured log span: '+name)
        if name!='core-and-executable':require(source in log,'actual checkout source: '+name)
        else:require('FEATURE_RESULT: success'in log and 'EXECUTABLE_RESULT: success'in log,'aggregate waited for successful needs')
    require(set(r['hosted_job_logs'])==EXPECTED_NAMES,'all raw hosted logs retained')
    common=contents[r['hosted_job_logs']['executable-contracts']].decode()
    require('Ran 38 tests'in common and '\n' in common and 'PASS executable completion manifest'in common,'actual lifecycle and smoke completion')
    require(all(0<=x<1800 for x in r['hosted_log_span_seconds'].values()),'measured hosted log spans within unchanged 30-minute job limits')
    artifacts=json.loads(contents['hosted-repair-artifacts.json'])['response']['structuredContent']['artifacts']
    require(len(artifacts)==1,'actual single original smoke artifact')
    artifact=artifacts[0]
    require(artifact['workflow_run']['head_sha']==source and artifact['workflow_run']['id']==r['hosted_run'],'artifact source/run independently bound')
    require('sha256:'+hashlib.sha256(contents['hosted-smoke-feb304e.zip']).hexdigest()==artifact['digest'],'original downloaded hosted ZIP digest')
    smoke=json.loads(contents['hosted-smoke/run.json'])
    rows=list(csv.DictReader(io.StringIO(contents['hosted-smoke/steps.csv'].decode())))
    png=contents['hosted-smoke/opacity.png']
    require(smoke['size']==16 and smoke['steps']==12 and smoke['managed_simulation_bytes']==333824 and
        smoke['last_divergence_max']<=1e-5 and smoke['whole_process_memory_cap_claimed']is False and
        smoke['real_time_performance_claimed']is False and len(rows)==12 and
        all(float(row['actual_divergence_max'])<=1e-5 for row in rows) and
        png[:8]==b'\x89PNG\r\n\x1a\n' and struct.unpack('>II',png[16:24])==(16,16),
        'every original smoke gate on actual downloaded files')
    failed=json.loads(contents['hosted-c059-final-jobs.json'])['response']['structuredContent']['jobs']
    require(len(failed)==5 and all(j['run_id']==37469020996 for j in failed),'original hosted failure retained')
    require({j['name']for j in failed if j['conclusion']=='failure'}=={'executable-contracts','core-and-executable'},'original failed lifecycle and aggregate retained')
    failure=contents['hosted-c059-executable-failed.log'].decode()
    require(C059 in failure and 'FAILED (failures=51)'in failure and 'AssertionError: [] is not true'in failure,'exact original failure/source')
    require(r['physics_claims_changed'] is False and r['frozen_research_heads']=={
        'forcing':'f76841e9217e781d91c5bfeabadb6b4c6f42b20b',
        'diagnosis':'dad53b4054034fe0c2ff6240464df7441ab4e6a9'},'frozen research scope')

r=json.loads((E/'receipt.json').read_text())
contents={path:(E/path).read_bytes()for path in r['evidence_sha256']}
verify(r,contents)
controls=[]
for name,mutate in [
    ('source tree',lambda x:x.update(source_tree='0'*40)),
    ('original source',lambda x:x.update(local_source='0'*40)),
    ('evidence hash',lambda x:x['evidence_sha256'].update({'local-matrix.json':'0'*64})),
    ('run identity',lambda x:x.update(hosted_run=x['hosted_run']+1)),
    ('checkout source hash',lambda x:x['source_sha256'].update({'src/coupled_discrete.rs':'0'*64})),
    ('incomplete logs',lambda x:x['hosted_job_logs'].pop('Feature contracts (desktop)')),
    ('physics promotion',lambda x:x.update(physics_claims_changed=True)),
]:
    corrupt=copy.deepcopy(r);mutate(corrupt)
    try:verify(corrupt,contents)
    except ValueError:controls.append(name)
    else:raise ValueError('accepted corrupt receipt: '+name)
print('PASS source/local equivalence, all hosted jobs, preserved original failure and seven binding controls: '+repr(controls))
