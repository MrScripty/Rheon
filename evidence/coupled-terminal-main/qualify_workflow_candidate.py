"""Run existing trigger ledger and revised strict budget checks on proposed YAML.

Only the workflow read is substituted; actual tracked inventory and immutable
baseline snapshot remain in this checkout. No Rust source or active YAML changes.
"""
from pathlib import Path
from unittest import mock
import sys,unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import test_column_interface_verifier as module
candidate=(ROOT/'evidence/coupled-terminal-main/proposed-readable-workflow.yml').read_text()
original=Path.read_text

def read_text(path,*args,**kwargs):
    if path==ROOT/'.github/workflows/rust-rheon.yml':return candidate
    return original(path,*args,**kwargs)

suite=unittest.TestSuite(module.ColumnVerifierTest(name)for name in[
    'test_rust_ci_triggers_for_each_example_and_column_verifier',
    'test_ci_budget_preserves_every_existing_job_check',
    'test_split_ci_budget_rejects_removed_checks_and_early_success'])
if '--all' in sys.argv:suite=unittest.defaultTestLoader.discover(str(ROOT/'tools'),pattern='test_*.py')
with mock.patch.object(Path,'read_text',read_text):
    result=unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():raise SystemExit(1)
print('PASS actual tracked path ledger and original command/pin inventory; ten corruptions rejected')
