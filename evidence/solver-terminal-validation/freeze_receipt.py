"""Freeze completed local qualification only; refuses partial command receipts."""
from pathlib import Path
import hashlib,json,re,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
source='922466bd9efdf4612462288fbe2df5a679ef4ccb';matrix=json.loads((P/'matrix-receipt.json').read_text())
require(matrix['source_commit']==source and len(matrix['commands'])==10 and matrix['status']=='PASS'and all(x['exit']==0 for x in matrix['commands']),'must complete all ten actual Rust commands first')
for name,count in [('captures',6),('trace',3),('common',5)]:
 r=json.loads((P/(name+'-receipt.json')).read_text());require(r['source_commit']==source and len(r['commands'])==count and r['status']=='PASS'and all(x['exit']==0 for x in r['commands']),'must complete native captures and trace')
counts={mode:sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',(P/('tests-'+mode+'.log')).read_text())))for mode in ['default','core-only','desktop']}
record={'status':'FROZEN_TESTED_TERMINAL_VALIDATION_CANDIDATE','source':source,'source_tree':subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip(),'baseline':'a2fcd29a927177500bcb57659c520d71e0ed0f12','source_sha256':matrix['source_sha256'],'actual_Rust_tests_passed':counts,'actual_doc_tests_executed_all_modes':True,'newton_threshold':1e-13,'correction_cap':7,'residual_check_cap':8,'separate_qualification_equations':2,'counted_equation_budget':200,'nominal_forced_bytes':474768,'preserved_baseline_paths':json.loads((P/'preservation.json').read_text())['preserved_path_count'],'original_temporal_band_pass':False,'arithmetic_floor_proved':False,'general_rate_proved':False,'hosted_CI_qualified':False,'active_workflow_changed':False,'supporting_document_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [ROOT/'docs/research-book/implementation/solver-terminal-validation-candidate.md',ROOT/'docs/research-book/implementation/solver-terminal-validation-budget-clarification.md']},'file_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(P.rglob('*'))if p.is_file()and '__pycache__'not in p.parts and p.name!='final-receipt.json'}}
(P/'final-receipt.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print(record['status'],counts)
