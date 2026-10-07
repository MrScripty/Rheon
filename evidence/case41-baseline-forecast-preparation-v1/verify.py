"""Integrity of reviewed preparation only; no tests/equations or guard reruns."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
inv=json.loads((P/'result-inventory.json').read_text())
for f,h in inv['sha256'].items():require(sha(ROOT/f)==h,'frozen result '+f)
policy=json.loads((P/'execution-protocol.json').read_text())
for f,h in policy['sha256'].items():require(sha(ROOT/f)==h,'frozen protocol '+f)
b=json.loads((P/'compile-binding.json').read_text());r=json.loads((P/'preflight-receipt.json').read_text())
require(sha(Path(b['binary']))==b['binary_sha256']==policy['binary_sha256']==r['binary_sha256'],'exact new ELF')
require(r['status']=='PASS_TWO_OBSERVATION_PREPARATION_ONLY' and r['native_guard_invocations']==5,'preflight-only acceptance')
require(r['native_equations']==r['baseline_observations']==r['forecast_observations']==r['owner_advances']==r['controller_corrections']==0,'zero numerical observation')
require(not r['execution_allowed'] and not r['independent_review_accepted'] and not r['historical_memory_qualified'] and r['complete_memory_bound'] is None,'pending review and incomplete certificate')
require(r['protocol_sha256']==sha(P/'execution-protocol.json'),'exact protocol/preflight')
for obj in [b,r]:require(git('rev-parse',obj['source']+'^{tree}')==obj['source_tree'],'source/tree')
spec=importlib.util.spec_from_file_location('readonly_source_check',P/'source_conformance.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);require(m.run()==r['source_conformance'],'source/input replay')
cmd=json.loads((P/'preflight-commands.json').read_text());expected=['research_public_scalar_preflight','research_public_type_layout','scalar_affine_preflight','case41_paired_layout_preflight','case41_forecast_input_layout_preflight']
require([c['argv'] for c in cmd]==[[b['binary'],'--exact','research_public_call::'+t,'--nocapture'] for t in expected],'closed native guard roster')
require(all(c['exit']==0 and not c['numerical_allow_variables_present'] and c['numerical_observations']==0 for c in cmd),'only passed guards')
require((P/'frames-normal.json').read_bytes()==(P/'frames-optimized.json').read_bytes(),'frame reader normal/-O parity')
require(not (P/'execution-authorization.json').exists() and not (P/'capture-before.json').exists() and not (P/'native.log').exists(),'no authority or numerical journal')
require(policy['limits']=={'address_space_bytes':268435456,'stack_bytes':8388608,'cpu_seconds':60,'wall_seconds':120,'stdout_bytes':2097152,'stderr_bytes':1048576,'file_output_bytes':8388608},'bounded reviewed proposal')
require('equations_max=2' in (P/'run_after_review_v2.py').read_text(),'two-observation future runner metadata')
changes=git('diff','--name-only','4122cd0a2cb8941c2898cdb4cd7c9badbaee2b7c','HEAD').splitlines();require(all(f.startswith('evidence/case41-baseline-forecast-preparation-v1/') for f in changes),'new packet only')
require(git('rev-parse','origin/main')=='9cd4587a54befa61bdfddc8e35014bd3c34f02fb','historical local main unchanged')
print(json.dumps({'status':'PASS_FROZEN_TWO_OBSERVATION_PREPARATION','native_source':b['source'],'preflight_source':r['source'],'binary_sha256':b['binary_sha256'],'native_guard_invocations':5,'native_equations':0,'execution_allowed':False,'historical_memory_qualified':False,'old_evidence_and_production_unchanged':True},indent=2,sort_keys=True))
