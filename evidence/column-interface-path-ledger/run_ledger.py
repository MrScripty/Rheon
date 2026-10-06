"""Record the rows checked by the actual path-filter unittest, in either mode."""
import hashlib,json,subprocess,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
from test_column_interface_verifier import ColumnVerifierTest
case=ColumnVerifierTest('test_rust_ci_triggers_for_each_example_and_column_verifier');result=unittest.TestResult();case.run(result)
if not result.wasSuccessful():raise ValueError(str(result.errors+result.failures))
ledger=case.path_ledger
if result.testsRun!=1 or any(not row['matched_patterns']for row in ledger['controls']):raise ValueError('missing actual path test result')
sha=lambda path:hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
print(json.dumps(dict(status='PASS_ACTUAL_EXECUTED_PATH_LEDGER',source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),actual_test_method='ColumnVerifierTest.test_rust_ci_triggers_for_each_example_and_column_verifier',actual_test_methods=result.testsRun,tracked_example_count=len(ledger['tracked_examples']),actual_control_count=len(ledger['controls']),**ledger,source_sha256={p:sha(p)for p in ['.github/workflows/rust-rheon.yml','tools/test_column_interface_verifier.py','evidence/column-interface-path-ledger/run_ledger.py']}),indent=2))
