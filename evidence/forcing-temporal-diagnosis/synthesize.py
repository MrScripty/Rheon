"""Classify the original failure from measured independent checks."""
import json,sys
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[2];P=Path(__file__).resolve().parent;sys.path.insert(0,str(ROOT/'evidence/forced-extruded-liquid'));import reference as r
def require(ok,message):
 if not ok:raise ValueError(message)
def run():
 reference=json.loads((P/'first-reference-diagnosis.json').read_text());finite=json.loads((P/'first-finite-diagnosis.json').read_text());probe=json.loads((P/'first-probe-replay.json').read_text())
 require(np.array_equal(r.m.q_to_x(np.array([0.,.5,1.])),[0.,1.,.5,1.25]),'actual coordinate ordering')
 require(reference['status']=='PASS_REFERENCE_DIAGNOSIS_ORIGINAL_FAILURE_RETAINED'and reference['actual_paired_planar_publications']==268,'independent references and exact field independence')
 require(len(finite['rows'])==20 and all(x['status']=='COMPLETE'and x['actual_independent_steps']==x['expected_steps']for x in finite['rows']),'all original finite trajectories independently reproduced')
 require(sum(x['actual_independent_steps']for x in finite['rows'])==248,'actual independent finite steps')
 geometry_gap=max(x['max_native_geometry_gap']for x in finite['rows']);eta_gap=max(x['max_native_eta_gap']for x in finite['rows'])
 require(geometry_gap<=1e-14 and eta_gap<=1e-11,'original geometry/coordinate comparison scales')
 require((P/'first-native-probe.jsonl').read_bytes()==(P/'native-no-default-probe.jsonl').read_bytes(),'actual native build-mode parity')
 require(probe['completed_cases']==2 and probe['refused_cases']==6 and probe['actual_publications']==143,'all actual probe outcomes retained')
 outcomes=[]
 for row in reference['rows']:
  require(max(row['existing_geometry_gaps_to_tight'][-1],row['RK4_geometry_gap_to_tight'],row['RK4_finest_refinement_gap'])<=row['geometry_resolution_allowance'],'reference uncertainty resolved at original scale')
  if row['load']=='reversed':
   require(not row['original_band_pass']and [x['dominant_coordinate']for x in row['errors']]==[2,0,0,0,0],'retained reversed failure and horizontal-coordinate switch')
   c1,c2,c3=np.array(row['component_fit']['coefficients_c1_c2_c3']);require(c1[0]>0 and c2[0]<0,'measured leading/correction cancellation')
   outcomes.append(dict(kind=row['kind'],dominant_coarse_coordinate='cap x1 (middle representative)',dominant_finer_coordinate='cap x0 (periodic representative)',left_cap_first_order_coefficient=float(c1[0]),left_cap_second_order_coefficient=float(c2[0]),coarse_second_to_first_term_ratio=float(abs(.05*c2[0]/c1[0])),coarse_original_ratios=row['original_ratios'][:2],finer_original_ratios=row['original_ratios'][2:],max_coarse_holdout_prediction_gap=max(x['prediction_max_gap']for x in row['component_fit']['coarse_holdouts'])))
  else:require(row['original_band_pass'],'forward control retains original band')
 fine=next(x for x in probe['rows']if x['kind']=='initial'and x['load']=='reversed'and x['probe_status']=='COMPLETE')
 require(fine['additional_pair_in_original_band']and not fine['original_five_interval_band_pass'],'one measured additional pair cannot promote original five intervals')
 return dict(status='DIAGNOSIS_SUPPORTED_ORIGINAL_GATE_FAILURE_RETAINED',classification='PRE_ASYMPTOTIC_DISCRETIZATION_ERROR_CANCELLATION_AND_MAX_NORM_COMPONENT_SWITCH',coordinate_order=['cap x0','cap H0','cap x1','cap H1'],correction_to_previous_interpretation='Earlier frozen prose mislabeled coordinate 2 as cap height; it is cap x1. Earlier source/evidence bytes are preserved.',native_vs_independent_finite_max_geometry_gap=geometry_gap,native_vs_independent_finite_max_eta_gap=eta_gap,independent_finite_cases=20,independent_finite_steps=248,independent_geometry_references=dict(tighter_DOP853=4,fixed_RK4=12,original_resolution_factor=1000),reversed_cases=outcomes,additional_initial_reversed_ratio=fine['additional_halving_ratio'],additional_initial_reversed_error=fine['geometry_max_error'],original_geometry_band=[1.7,2.3],original_five_interval_geometry_qualification='FAIL_ORIGINAL_TEMPORAL_BAND',finer_native_complete_cases=2,finer_native_refusals=6,finer_refusal='IterationLimit with exact accepted-state preservation; error does not expose residual magnitude. Cause is not separately established here.',scope='Diagnosis of the original interval-set failure in the declared finite discretization. No production change, gate relaxation, arbitrary-fine-step or general-liquid qualification; no formal asymptotic-order theorem.')
if __name__=='__main__':print(json.dumps(run(),indent=2))
