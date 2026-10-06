"""Reference and finite-rule diagnosis; original convergence bands stay fixed."""
import json,sys
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'evidence/forced-extruded-liquid';sys.path.insert(0,str(BASE));import reference as r
P=Path(__file__).resolve().parent
KINDS=['initial','pressure_state'];LOADS=['forward','reversed'];H=list(r.H)
def require(ok,message):
 if not ok:raise ValueError(message)
def acceleration(load):return r.A*(1 if load=='forward'else-1)
def initial(kind):
 q,e=r.t.initial(kind);s=r.m.spatial(r.m.q_to_x(q));j,_=r.m.cap_chart(s);z=j@e
 return np.r_[r.m.q_to_x(q),z[r.m.FREE],r.third.XI]
def rk4(kind,a,h):
 n=round(.1/h);require(n<=128 and n*h==.1,'bounded independent fixed-step reference');y=initial(kind);calls=0
 for step in range(n):
  k1=r.instantaneous(y,a)[0];k2=r.instantaneous(y+h*k1/2,a)[0];k3=r.instantaneous(y+h*k2/2,a)[0];k4=r.instantaneous(y+h*k3,a)[0];calls+=4;y=y+h*(k1+2*k2+2*k3+k4)/6
  require(np.all(np.isfinite(y))and calls<=5000,'bounded finite independent RK4')
 return dict(method='fixed RK4',h=h,steps=n,calls=calls,final_x=y[:4].tolist(),final_state=y.tolist())
def native_groups():
 rows=[json.loads(line)for line in (BASE/'local-qualification/native-default.jsonl').read_text().splitlines()];groups={}
 for row in rows:groups.setdefault((row['kind'],row['field'],row['load'],row['h']),[]).append(row)
 return groups
def references():
 groups=native_groups();old=json.loads((BASE/'local-qualification/normal-diagnostic-replay.json').read_text());results=[];paired=0
 for kind in KINDS:
  for load in LOADS:
   a=acceleration(load);tight=r.reference(kind,'nonconstant',a,1e-13,.0015625);rk=[rk4(kind,a,h)for h in [.00625,.003125,.0015625]]
   existing=[x for x in old['references']if x['kind']==kind and x['field']=='nonconstant'and np.sign(x['acceleration'][0])==np.sign(a[0])]
   xf=np.array(tight['final_x']);errors=[]
   for h in H:
    native=groups[kind,'nonconstant',load,h];constant=groups[kind,'constant',load,h]
    for x,y in zip(native,constant):
     require(all(x[name]==y[name]for name in ['end_q','end_eta','positions','mass','pressure_coefficients','physical_pressure','unknowns'])and np.array_equal(np.array(x['velocity'])[:,:2],np.array(y['velocity'])[:,:2]),'actual planar field independence');paired+=1
    error=r.m.q_to_x(np.array(native[-1]['end_q']))-xf
    errors.append(dict(h=h,error_components=error.tolist(),geometry_max_error=float(np.max(abs(error))),dominant_coordinate=int(np.argmax(abs(error))),normalized_error=(error/h).tolist()))
   gaps=[float(np.max(abs(np.array(ref['final_x'])-xf)))for ref in existing];rk_gap=float(np.max(abs(np.array(rk[-1]['final_x'])-xf)));rk_delta=float(np.max(abs(np.array(rk[-2]['final_x'])-rk[-1]['final_x'])))
   resolution=min(x['geometry_max_error']for x in errors)/1000
   require(max(gaps[-1],rk_gap,rk_delta)<=resolution,'original reference-resolution scale for geometry')
   hs=np.array([x['h']for x in errors[-3:]]);ys=np.array([x['error_components']for x in errors[-3:]])/hs[:,None];h0=hs[0]
   scaled=np.linalg.solve(np.c_[np.ones(3),hs/h0,(hs/h0)**2],ys);coeff=np.array([scaled[0],scaled[1]/h0,scaled[2]/h0**2]);holdout=[]
   for e in errors[:2]:
    h=e['h'];predicted=h*coeff[0]+h*h*coeff[1]+h*h*h*coeff[2];holdout.append(dict(h=h,predicted_components=predicted.tolist(),actual_components=e['error_components'],prediction_max_gap=float(np.max(abs(predicted-e['error_components'])))))
   ratios=[x['geometry_max_error']/y['geometry_max_error']for x,y in zip(errors,errors[1:])]
   results.append(dict(kind=kind,load=load,tight_DOP853=tight,independent_RK4=rk,existing_geometry_gaps_to_tight=gaps,RK4_geometry_gap_to_tight=rk_gap,RK4_finest_refinement_gap=rk_delta,geometry_resolution_allowance=resolution,errors=errors,original_geometry_band=[1.7,2.3],original_ratios=ratios,original_band_pass=all(1.7<x<2.3 for x in ratios),component_fit=dict(training_h=hs.tolist(),coefficients_c1_c2_c3=coeff.tolist(),coarse_holdouts=holdout,scope='Descriptive small-h component expansion with coarse holdouts; no extrapolated qualification')))
   print('reference diagnosis',kind,load,flush=True,file=sys.stderr)
 return dict(status='PASS_REFERENCE_DIAGNOSIS_ORIGINAL_FAILURE_RETAINED',actual_paired_planar_publications=paired,rows=results,original_reversed_geometry_qualification='FAIL_ORIGINAL_TEMPORAL_BAND')
def finite():
 groups=native_groups();rows=[]
 for kind in KINDS:
  for load in LOADS:
   a=acceleration(load)
   for h in H:
    q,e=r.t.initial(kind);maxgap=0.;maxeta=0.;rate=0.;maxcalls=0;accepted=0;refusal=None
    for actual in groups[kind,'nonconstant',load,h][1:]:
     try:u,v,counts=r.solve(q,e,h,a)
     except ValueError as error:refusal=str(error);break
     q=v['end']['q'];e=v['end']['eta'];accepted+=1;maxgap=max(maxgap,float(np.max(abs(q-actual['end_q']))));maxeta=max(maxeta,float(np.max(abs(e-actual['end_eta']))));rate=max(rate,float(np.linalg.norm(v['residual'])/h));maxcalls=max(maxcalls,counts['calls'])
    rows.append(dict(kind=kind,load=load,h=h,status='COMPLETE'if refusal is None else'RESEARCH_REFUSAL',actual_independent_steps=accepted,expected_steps=round(.1/h),refusal=refusal,max_native_geometry_gap=maxgap,max_native_eta_gap=maxeta,max_independent_momentum_rate=rate,max_equation_calls=maxcalls,independent_final_q=q.tolist(),native_final_q=groups[kind,'nonconstant',load,h][-1]['end_q']))
    print('finite diagnosis',kind,load,h,rows[-1]['status'],flush=True,file=sys.stderr)
 return dict(status='FINITE_DIAGNOSIS_RECORDED',rows=rows,original_newton_rate_gate=1e-13,original_full_momentum_gate=1e-11)
if __name__=='__main__':print(json.dumps(references()if sys.argv[1]=='references'else finite(),indent=2))
