"""Verify active job split retains every current-main command and required gate."""
from pathlib import Path
import subprocess,yaml,copy
ROOT=Path(__file__).resolve().parents[2]
BASE='773bd2725e35590cfe9cbca5625f5239b98f03c2'
old=yaml.safe_load(subprocess.check_output(['git','show',BASE+':.github/workflows/rust-rheon.yml'],cwd=ROOT,text=True));original=old['jobs']['core-and-executable']['steps']
def require(ok,msg):
 if not ok:raise ValueError(msg)
def verify(p):
 triggers=copy.deepcopy(old[True]);triggers['push']['branches'].append('repair/coupled-terminal-validation')
 require(p.get('on',p.get(True))==triggers and p['permissions']==old['permissions'] and p['name']==old['name'],'original triggers/name/permissions plus exact candidate push scope')
 f=p['jobs']['feature-contracts'];c=p['jobs']['executable-contracts'];g=p['jobs']['core-and-executable']
 for job in [f,c,g]:require(job['runs-on']=='ubuntu-24.04'and job['timeout-minutes']==30,'unchanged runner and per-job timeout')
 require(f['env']==c['env']=={'CARGO_BUILD_JOBS':'1'},'same bounded Cargo worker count')
 require(f['steps'][:2]==[original[0],original[2]] and c['steps'][:3]==original[:3],'exact immutable checkout/Rust/Python dependencies and credential policy')
 require(c['steps'][3]['run']=='cargo fmt --all --check\n'and c['steps'][4:]==original[5:],'exact formatting, smoke assertions, lifecycle and pinned artifact retention')
 require(f['strategy']=={'fail-fast':False,'matrix':{'include':[{'mode':'default','flags':''},{'mode':'core-only','flags':'--no-default-features'},{'mode':'desktop','flags':'--features desktop'}]}},'all original feature modes run independently')
 require(f['steps'][2]['run']=='cargo clippy --locked ${{ matrix.flags }} --all-targets -- -D warnings\n'and f['steps'][3]['run']=='cargo test --locked ${{ matrix.flags }}\n','all strict lint/test commands preserved')
 require(f['steps'][4]['if']=="matrix.mode == 'core-only'"and f['steps'][4]['run']=='cargo tree --locked --no-default-features\n','dependency inspection retained')
 require('needs'not in f and 'needs'not in c and g['needs']==['feature-contracts','executable-contracts']and g['if']=='always()'and g['name']=='core-and-executable','old required check waits for every independent result')
 require(g['steps']==[{'name':'Require every existing qualification path','env':{'FEATURE_RESULT':'${{ needs.feature-contracts.result }}','EXECUTABLE_RESULT':'${{ needs.executable-contracts.result }}'},'run':'test "$FEATURE_RESULT" = success\ntest "$EXECUTABLE_RESULT" = success\n'}],'gate fails for failure/cancel/skip; no early green')
p=yaml.safe_load((ROOT/'.github/workflows/rust-rheon.yml').read_text());verify(p);controls=[]
for name,mutate in [('drop desktop',lambda x:x['jobs']['feature-contracts']['strategy']['matrix']['include'].pop()),('weaken lint',lambda x:x['jobs']['feature-contracts']['steps'][2].update(run='cargo clippy\n')),('drop smoke',lambda x:x['jobs']['executable-contracts']['steps'].pop(4)),('early green gate',lambda x:x['jobs']['core-and-executable'].update(needs=[])),('credential persistence',lambda x:x['jobs']['feature-contracts']['steps'][0]['with'].update({'persist-credentials':True}))]:
 broken=copy.deepcopy(p);mutate(broken)
 try:verify(broken)
 except ValueError:controls.append(name)
 else:raise ValueError('accepted corrupt workflow '+name)
print('PASS all original checks/pins/permissions/timeouts and required gate; rejected '+repr(controls))
