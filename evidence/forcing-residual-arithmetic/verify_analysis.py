"""Check captured-data sensitivity analysis and forbid acceptance promotion."""
from pathlib import Path
import copy,importlib.util,json,math,struct
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('arithmetic_analysis',P/'analyze.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
def verify(packet,captures,result):
 a.require(len(packet['rows'])==len(captures)==len(result['rows'])==5,'all original remaining cases')
 a.require(result['native_newton_target']==1e-13 and result['remaining_native_refusals']==5 and result['owner_advances']==0,'original refusal gate, inventory and no publication')
 for case,cap,row in zip(packet['rows'],captures,result['rows']):
  a.require(cap['case']==result['rows'].index(row),'actual case ordering')
  a.require(all(a.bits(x)==a.bits(y)for x,y in zip(cap['rate'],case['expected_rate'])) and a.bits(cap['norm'])==a.bits(case['expected_norm']),'native frozen vector/norm')
  a.require(cap['norm']>1e-13 and row['native_norm']==cap['norm'] and cap['published']is False and row['published']is False,'no threshold relaxation or observation promotion')
  a.require(cap['planar_qualification_pass']is True and 0<=cap['work_fraction']<1 and 0<=cap['ledger_fraction']<1,'actual planar gates, not a complete step')
  a.require(row['exact_sum_same_native_endpoints_norm']>1e-13 and row['exact_sum_max_component_gap']<1e-17,'summation alone retains each refusal')
  a.require(row['accepted_to_rebuilt_start_max_gap']==max(abs(x-y)for x,y in zip(cap['start_z'],cap['rebuilt_start_z'])),'stored start reconstruction discrepancy')
 a.require(result['original_temporal_band']=='FAIL_ORIGINAL_TEMPORAL_BAND','original temporal failure retained')
packet=json.loads((P/'inputs.json').read_text());captures=[json.loads(x)for x in (P/'native-probe-start-chart.log').read_text().splitlines()if x.startswith('{')]
normal=json.loads((P/'analysis-normal.json').read_text());optimized=json.loads((P/'analysis-optimized.json').read_text());a.require(normal==optimized,'ordinary/optimized analysis parity')
verify(packet,captures,normal)
higher=a.run(120);a.require(higher['rows']==normal['rows'],'80/120-digit sensitivity outputs agree after binary export')
controls=[]
for name,kind,mutate in [
 ('native vector','capture',lambda x:x[0]['rate'].__setitem__(0,x[0]['rate'][0]+1e-12)),
 ('false publication','capture',lambda x:x[0].update(published=True)),
 ('missing refused case','capture',lambda x:x.pop()),
 ('weakened target','result',lambda x:x.update(native_newton_target=1e-12)),
 ('new accepted endpoint','result',lambda x:x.update(owner_advances=1)),
 ('temporal promotion','result',lambda x:x.update(original_temporal_band='PASS')),
 ('work failure ignored','capture',lambda x:x[0].update(work_fraction=2.)),
]:
 c=copy.deepcopy(captures);r=copy.deepcopy(normal);mutate(c if kind=='capture'else r)
 try:verify(packet,c,r)
 except ValueError:controls.append(name)
 else:raise ValueError('accepted corrupt research packet '+name)
print(json.dumps({'status':'PASS_CAPTURED_REFUSAL_ANALYSIS','actual_native_fixed_candidate_checks':5,'ordinary_optimized_parity':True,'precision_digits_checked':[80,120],'corruptions_rejected':controls,'remaining_actual_native_refusals':5,'accepted_endpoints_added':0,'original_temporal_failure_retained':True,'production_solver_defect_established':False,'arithmetic_floor_proved':False},indent=2))
