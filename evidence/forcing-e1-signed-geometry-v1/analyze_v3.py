"""Additive coordinate-label repair; numerical calculations remain frozen."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('json_scalar_reader',P/'analyze_v2.py');v2=importlib.util.module_from_spec(spec);spec.loader.exec_module(v2)
LABELS=['left_cap_x','left_cap_height','middle_cap_x','middle_cap_height']
def run():
 v2.original.LABELS=LABELS
 result=v2.run();lookup={}
 for family in result['families']:
  rows={row['index']:row for row in family['errors']}
  for row in rows.values():
   if row['status']=='COMPLETE':row['dominant_mode']='height'if row['dominant_coordinate']in [1,3]else LABELS[row['dominant_coordinate']]
  for check in family['checks']:
   if 'ratio'in check:check['dominant_mode_switch']=rows[check['coarser_index']]['dominant_mode']!=rows[check['finer_index']]['dominant_mode']
   lookup[check['coarser_index'],check['finer_index']]=check
 for check in result['failed_checks']:check['dominant_mode_switch']=lookup[check['coarser_index'],check['finer_index']]['dominant_mode_switch']
 result['failed_checks_with_dominant_mode_switch']=sum(x['dominant_mode_switch']for x in result['failed_checks']);result['failed_checks_without_mode_switch']=8-result['failed_checks_with_dominant_mode_switch'];result['coordinate_labels']=LABELS
 # Frozen q_to_x orders x/height pairs, rather than both x coordinates first.
 for family in result['families']:
  for row in family['errors']:
   if row['status']=='COMPLETE':row['height_pair_signed_error_sum']=row['signed_error_components'][1]+row['signed_error_components'][3]
 return result
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
