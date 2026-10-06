"""Freeze additive diagnostic artifacts after capture and analysis, before validation."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('closure',P/'verify.py');v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
output=P/'receipt.json'
if output.exists():raise ValueError('receipt is immutable; no refreeze')
raw=v.inventory();v.verify_inventory(raw)
receipt={'status':'FROZEN_SEPARATE_DIAGNOSTIC_RESULTS','predecessor':v.BASE,'predecessor_tree':v.TREE,'predecessor_inventory_sha256':v.sha(raw),'preserved_predecessor_files':7768,'native_source':'00d7de13b82396836e8f1efad6e3049034f5c2e7','native_source_tree':'e6487beb81407818a0280513fd2a1bbbc986d883','artifacts':{str(p.relative_to(v.ROOT)):v.sha(p.read_bytes())for p in v.artifact_paths()},'accepted_owners_constructed':0,'accepted_owners_advanced':0,'new_controller_iterations':0,'new_reference_integrations':0,'new_trajectory_steps':0,'native_terminal_failures_retained':2,'original_geometry_band_failures_retained':8,'trajectory_remedy_established':False,'production_adoption':False,'memory_cap_expanded':False,'arithmetic_floor_assumed':False,'separate_geometry_receipt_sha256':v.sha((v.G/'receipt.json').read_bytes())}
output.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
print(json.dumps({'event':'freeze_diagnostic_results','receipt_sha256':v.sha(output.read_bytes()),'artifact_count':len(receipt['artifacts'])}))
