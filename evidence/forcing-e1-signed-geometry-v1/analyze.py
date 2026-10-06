"""Signed geometry errors divided by h, from existing endpoints only."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib,json,sys
import numpy as np
P=Path(__file__).resolve().parent;ROOT=P.parents[1];OLD=ROOT/'evidence/forcing-e1-trajectories-v2'
sys.path.insert(0,str(ROOT/'evidence/forced-extruded-liquid'));import reference as r
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def forbidden(*args,**kwargs):raise ValueError('new integration/trajectory is forbidden')
r.reference=r.instantaneous=r.solve=forbidden
LABELS=['left_cap_x','middle_cap_x','left_cap_height','middle_cap_height']
def run():
 packet=json.loads((P/'inputs.json').read_text());previous=json.loads((OLD/'convergence-outcome.json').read_text())
 for path,digest in packet['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'all existing source/evidence unchanged '+path)
 families=[];failed=[]
 for ref in packet['references']:
  key=(ref['kind'],ref['field'],ref['load']);coarse,tight=[x['result']for x in ref['rows']];xf=np.array(tight['final_x']);paired=np.array(coarse['final_x'])-xf;original=next(f for f in previous['families']if(f['kind'],f['field'],f['load'])==key);errors=[]
  selected=sorted([x for x in packet['cases']if(x['case']['kind'],x['case']['field'],x['case']['load'])==key],key=lambda x:-x['case']['h'])
  for frozen in selected:
   case=frozen['case'];endpoint=frozen['endpoint'];row={'index':case['index'],'h':case['h'],'status':frozen['terminal']['status'],'accepted_steps':frozen['terminal']['accepted_steps'],'expected_steps':case['steps']}
   if endpoint is not None:
    x=r.m.q_to_x(np.array(endpoint['end_q']));error=x-xf;scaled=error/case['h'];norm=float(np.max(abs(error)));dominant=int(np.argmax(abs(error)));old=next(x for x in original['errors']if x['index']==case['index']);require(norm==old['geometry_max_error'],'same actual endpoint/reference error')
    row.update(actual_clock=endpoint['time'],actual_coordinates=x.tolist(),signed_error_components=error.tolist(),signed_error_over_h=scaled.tolist(),exact_error_over_h=[str((Q(float(v))-Q(float(w)))/Q(case['h']))for v,w in zip(x,xf)],geometry_max_error=norm,normalized_max_error=float(np.max(abs(scaled))),dominant_coordinate=dominant,dominant_label=LABELS[dominant],dominant_mode='height'if dominant>=2 else LABELS[dominant],paired_reference_difference_over_h=(paired/case['h']).tolist(),height_pair_signed_error_sum=float(error[2]+error[3]))
   else:require(frozen['retained_partial_last_publication']is not None,'retain refusal prefix without invented endpoint');row.update(endpoint_missing=True,retained_partial_clock=frozen['retained_partial_last_publication']['time'])
   errors.append(row)
  checks=[]
  for a,b in zip(errors,errors[1:]):
   check={'coarser_index':a['index'],'finer_index':b['index'],'coarser_h':a['h'],'finer_h':b['h'],'original_check':a['h']in r.H and b['h']in r.H,'original_band':[1.7,2.3]}
   if a['status']==b['status']=='COMPLETE':
    ea=np.array(a['signed_error_components']);eb=np.array(b['signed_error_components']);j=b['dominant_coordinate'];ratio=a['geometry_max_error']/b['geometry_max_error'];require(ratio==next(c['ratio']for c in original['checks']if c['quantity']=='geometry_max_error'and c['coarser_h']==a['h']and c['finer_h']==b['h']),'original ratio reproduced exactly');fixed=abs(ea[j])/abs(eb[j]);amplification=a['geometry_max_error']/abs(ea[j])if ea[j]!=0 else None
    component_ratios=[float(v/w)if w!=0 else None for v,w in zip(ea,eb)];normalized_ratios=[float(v/w)if w!=0 else None for v,w in zip(np.array(a['signed_error_over_h']),np.array(b['signed_error_over_h']))]
    check.update(status='PASS_ORIGINAL_BAND'if 1.7<ratio<2.3 else'FAIL_ORIGINAL_BAND',ratio=ratio,signed_margin=min(ratio-1.7,2.3-ratio),dominant_coordinate_switch=a['dominant_coordinate']!=b['dominant_coordinate'],dominant_mode_switch=a['dominant_mode']!=b['dominant_mode'],fixed_finer_dominant_coordinate=j,fixed_coordinate_ratio=fixed,fixed_coordinate_in_original_band=1.7<fixed<2.3,coarse_dominance_amplification=amplification,ratio_factorization_gap=abs(ratio-fixed*amplification)if amplification is not None else None,signed_component_ratios=component_ratios,signed_normalized_component_ratios=normalized_ratios,normalized_component_change=(np.array(a['signed_error_over_h'])-np.array(b['signed_error_over_h'])).tolist(),same_sign_components=[bool(v*w>0)for v,w in zip(ea,eb)])
    if check['status']=='FAIL_ORIGINAL_BAND'and check['original_check']:failed.append({'kind':key[0],'field':key[1],'load':key[2],**check})
   else:check.update(status='INCOMPLETE_NO_ENDPOINT')
   checks.append(check)
  families.append({'kind':key[0],'field':key[1],'load':key[2],'existing_tight_reference_rtol':tight['rtol'],'existing_tight_reference_max_step':tight['max_step'],'paired_reference_geometry_difference':paired.tolist(),'existing_reference_resolution_status':original['reference_resolution_status'],'errors':errors,'checks':checks})
 require(len(failed)==8,'all original eight geometry failures retained')
 switched=[x for x in failed if x['dominant_mode_switch']];within=[x for x in failed if x['fixed_coordinate_in_original_band']]
 return {'status':'PASS_SIGNED_COMPONENT_DIAGNOSIS_WITH_ORIGINAL_BAND_FAILURES','new_trajectory_steps':0,'new_reference_integrations':0,'existing_complete_endpoints':46,'missing_finer_endpoints':2,'original_geometry_failures':8,'failed_checks_with_dominant_mode_switch':len(switched),'failed_checks_fixed_finer_coordinate_in_band':len(within),'failed_checks_without_mode_switch':len(failed)-len(switched),'original_temporal_qualification':'FAIL_ORIGINAL_TEMPORAL_BAND','reference_resolution_already_passed':True,'thresholds_changed':False,'remedy_established':False,'scope':'Descriptive signed components of existing discrete endpoints. Paired reference differences are observations, not rigorous error enclosures. Dominant-mode changes and variation of error/h are separated; no continuum proof or altered metric qualification.','families':families,'failed_checks':failed}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
