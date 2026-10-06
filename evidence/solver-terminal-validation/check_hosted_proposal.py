"""Verify inert proposal retains existing executable steps and all Rust modes."""
from pathlib import Path
import copy,yaml
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def check(proposal):
 old=yaml.safe_load((ROOT/'.github/workflows/rust-rheon.yml').read_text()); original=old['jobs']['core-and-executable']['steps']
 require(proposal['on']==old[True] and proposal['permissions']==old['permissions'],'all original triggers and permissions')
 feature=proposal['jobs']['feature-contracts'];common=proposal['jobs']['executable-and-lifecycle']
 for job in [feature,common]:
  require(job['runs-on']=='ubuntu-24.04' and job['timeout-minutes']==30 and job['env']=={'CARGO_BUILD_JOBS':'1'},'same runner, timeout and bounded build parallelism')
 require(feature['steps'][:2]==[original[0],original[2]],'same pinned checkout and Rust setup')
 require(common['steps'][:3]==original[:3],'same checkout, isolated Python and Rust setup')
 require(common['steps'][3]['run']=='cargo fmt --all --check\n','format retained')
 require(common['steps'][4:]==original[5:],'exact original smoke assertions, lifecycle tests and artifact retention')
 require(feature['strategy']=={'fail-fast':False,'matrix':{'include':[{'mode':'default','flags':''},{'mode':'core-only','flags':'--no-default-features'},{'mode':'desktop','flags':'--features desktop'}]}},'all independent feature modes')
 require(feature['steps'][2]['run']=='cargo clippy --locked ${{ matrix.flags }} --all-targets -- -D warnings\n' and feature['steps'][3]['run']=='cargo test --locked ${{ matrix.flags }}\n','exact locked strict lint and test commands in all modes')
 require(feature['steps'][4]=={'name':'Inspect core-only dependency tree','if':"matrix.mode == 'core-only'",'run':'cargo tree --locked --no-default-features\n'},'dependency inspection retained')
 require(not any('needs'in job for job in proposal['jobs'].values()),'independent critical paths')
proposal=yaml.safe_load((P/'hosted-proposal.yml').read_text());check(proposal)
controls=[]
for name,mutate in [('drop desktop',lambda p:p['jobs']['feature-contracts']['strategy']['matrix']['include'].pop()),('weaken lint',lambda p:p['jobs']['feature-contracts']['steps'][2].update(run='cargo clippy\n')),('drop smoke assertion',lambda p:p['jobs']['executable-and-lifecycle']['steps'][4].update(run='cargo run --bin rheon\n')),('credential persistence',lambda p:p['jobs']['feature-contracts']['steps'][0]['with'].update({'persist-credentials':True}))]:
 broken=copy.deepcopy(proposal);mutate(broken)
 try:check(broken)
 except ValueError:controls.append(name)
 else:raise ValueError('accepted corrupted proposal '+name)
print('PASS inert proposal retains all commands; rejected '+repr(controls))
