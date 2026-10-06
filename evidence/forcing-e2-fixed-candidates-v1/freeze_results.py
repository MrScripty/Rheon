"""Freeze actual completed E2 results before closed read-only verification."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent
s=importlib.util.spec_from_file_location('closure',P/'verify.py');v=importlib.util.module_from_spec(s);s.loader.exec_module(v)
output=P/'receipt.json';v.require(not output.exists(),'never refreeze results');raw=v.inventory();v.validate_inventory(raw)
receipt={'status':'FROZEN_FOUR_E2_FIXED_EQUATIONS_NO_TRAJECTORY','predecessor':v.BASE,'predecessor_tree':v.TREE,'predecessor_inventory_sha256':v.sha(raw),'artifacts':{f.name:v.sha(f.read_bytes())for f in v.artifacts()},'preflight_receipt_sha256':v.sha((P/'preflight-receipt.json').read_bytes()),'native_source':'cbeaa72b403cb1f3af4f95727cc29ec660c99ca3','native_source_tree':'2539d960f279d5f50aa213f3374651d7c51b5957','native_fixed_candidate_equations':4,'case41_above_original_Newton_target':True,'case47_below_original_Newton_target':True,'paired_native_planar_qualifiers_passed':2,'original_runtime_refusals_unresolved':2,'original_geometry_band_failures_unresolved':8,'accepted_owner_advances':0,'controller_iterations':0,'new_trajectories':0,'new_references':0,'third_solve':False,'publication':False,'production_adoption':False,'presentation_v2_reason':'Move shared legend outside bars; preserve v1 renderer and figure. No numerical result or protocol change.'}
output.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps({'receipt_sha256':v.sha(output.read_bytes()),'artifacts':len(receipt['artifacts'])}))
