"""Read-only packet integrity and exact enclosure/display checks; no ELF calls."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib
import json
import subprocess
P = Path(__file__).resolve().parent
ROOT = P.parents[1]
def require(ok, msg):
    if not ok:
        raise ValueError(msg)
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
policy = json.loads((P/'protocol.json').read_text())
for path, digest in policy['input_sha256'].items():
    require(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest, 'input '+path)
inv = json.loads((P/'result-inventory.json').read_text())
for path, digest in inv['sha256'].items():
    require(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest, 'result '+path)
old = json.loads((ROOT/'evidence/case41-exploratory-seven-v1/result-inventory.json').read_text())
for path, digest in old['sha256'].items():
    require(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest, 'old result '+path)
x = json.loads((P/'analysis-80-normal.json').read_text())
for precision in [80,120]:
    require((P/f'analysis-{precision}-normal.json').read_bytes()==(P/f'analysis-{precision}-optimized.json').read_bytes(), 'normal/-O parity')
    y=json.loads((P/f'analysis-{precision}-normal.json').read_text())
    require(y.pop('precision_digits')==precision, 'precision binding')
    z=dict(x);z.pop('precision_digits')
    require(y==z, '80/120 display parity')
    for mode in ['normal','optimized']:
        require(not (P/f'analysis-{precision}-{mode}-stderr.log').read_bytes(), 'clean reader stderr')
require(x['status']=='PASS_CAPTURED_LINEAR_PREDICTIONS_ONLY', 'status')
require(x['new_native_equations']==x['applied_corrections']==x['owner_advances']==x['native_linear_calls']==0, 'zero execution scope')
require(not x['historical_memory_qualified'] and x['historical_subtotal']==66368 and x['memory_cap']==67584, 'historical certificate incomplete')
u=Q(x['rounding_limits']['captured_matrix_Neumann_theta_upper_exact'])
require(u*u>=Q(x['rounding_limits']['captured_matrix_Neumann_theta_squared_exact']) and u<1, 'exact Neumann enclosure')
require(x['rounding_limits']['exact_stored_above_original_gate'] and x['rounding_limits']['exact_constraint_adjusted_above_original_gate'], 'original gate preserved')
require(json.loads((P/'matrix.json').read_text())['matrix']==x['matrix'], 'matrix export binding')
require(git('rev-parse', inv['analysis_source']+'^{tree}')==inv['analysis_tree'], 'analysis source/tree')
for path in git('diff','--name-only', policy['prior_publication'],'HEAD').splitlines():
    require(path.startswith('evidence/case41-captured-linear-predictions-v1/'), 'new packet only '+path)
for path in git('diff','--name-only','HEAD').splitlines():
    require(path.startswith('evidence/case41-captured-linear-predictions-v1/'), 'working change only '+path)
require(git('rev-parse','refs/remotes/origin/main')=='9cd4587a54befa61bdfddc8e35014bd3c34f02fb', 'main unchanged')
print(json.dumps(dict(status='PASS_CAPTURED_LINEAR_PREDICTION_INTEGRITY', analysis_source=inv['analysis_source'], analysis_tree=inv['analysis_tree'], inventory_sha256=hashlib.sha256((P/'result-inventory.json').read_bytes()).hexdigest(), normal_optimized_equal=True, precision_80_120_display_equal=True, historical_memory_qualified=False,new_native_equations=0,applied_corrections=0,owner_advances=0),indent=2,sort_keys=True))
