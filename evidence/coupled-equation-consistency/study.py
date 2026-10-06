"""Equation-limit, pressure-meaning and actual physical defect diagnostics.

Existing accepted cells are reassembled; no self-convergence alone establishes
consistency. Expected coefficients come from independent exact Taylor assembly.
All original physical gates retain their actual pass/fail values.
"""
import json,sys
from pathlib import Path
from fractions import Fraction as Q
import numpy as np
from scipy.integrate import solve_ivp
import taylor
step=taylor.temporal;m=step.m
P=Path(__file__).resolve().parent

def norm(x):return float(np.linalg.norm(x))
def require(ok,s):
 if not ok:raise ValueError(s)
def band(values,expected=2):return all(.85*expected<r<1.15*expected for r in values)
def ratios(values):return [a/b for a,b in zip(values,values[1:])]

def reference(kind,rtol,max_step):
 q,e=step.initial(kind);z=step.point(q,e,step.seed(q,e),0.)['z'];y=np.r_[m.q_to_x(q),z[m.FREE]];calls=0
 def rhs(t,state):
  nonlocal calls
  calls+=1;require(calls<=5000,'bounded DAE reference calls');return m.evaluate(state)[0]
 answer=solve_ivp(rhs,(0,.05),y,method='DOP853',rtol=rtol,atol=rtol/100,max_step=max_step,dense_output=True)
 require(answer.success,'independent DAE reference success');return answer,calls

def observe(kind,cells,expected):
 u0=np.array(expected['tangent_unknowns'],float);u1=np.array(expected['discrete_unknown_first_coefficient'],float);udot=np.array(expected['tangent_unknown_derivative'],float);E2=np.array(expected['actual_path_momentum_defect_coefficient'],float);I2=np.array(expected['pressure_impulse_defect_coefficient'],float);L=np.array(expected['endpoint_donor_coefficient'],float)
 coarse,ncoarse=reference(kind,1e-11,.003125);fine,nfine=reference(kind,2e-13,.0015625)
 reference_difference=0.;rows=[]
 for cell in cells:
  h=cell['interval'];q=np.array(cell['initial_q']);eta=np.array(cell['initial_eta']);unknown=np.array(cell['unknowns']);partition=step.sign_partition(q,eta,unknown,h);v=step.evaluate(q,eta,unknown,h,32,partition=partition);step.qualify(v,h)
  require(norm(v['residual'])/h<=1e-11,'unchanged all 22 discrete momentum equations')
  endpoint=m.evaluate(np.r_[m.q_to_x(v['end']['q']),v['end']['z'][m.FREE]])[1]
  E=v['actual_integrated_physical_momentum_residual'];point=v['start']['momentum_residual'];delta=v['endpoint_vs_actual_momentum_convection']
  averages=[];impulses=[]
  for answer in [coarse,fine]:
   average=np.zeros(16);impulse=np.zeros(22)
   for node,weight in zip(*step.rule(32)):
    t=h*(1+node)/2;p=m.evaluate(answer.sol(t))[1];average+=weight*p['pressure']/2;impulse+=h*weight*(p['spatial']['B'].T@p['pressure'])/2
   averages.append(average);impulses.append(impulse)
  reference_difference=max(reference_difference,float(np.max(np.abs(averages[0]-averages[1]))),float(np.max(np.abs(impulses[0]-impulses[1]))))
  impulse=h*v['end']['spatial']['B'].T@unknown[6:];paverage=averages[1];impulseerror=impulse-impulses[1]
  true_end=m.evaluate(fine.sol(h))[1];dq=m.q_to_x(v['end']['q'])-fine.sol(h)[:4];dz=v['end']['z']-true_end['z']
  rows.append(dict(interval=h,tangent_acceleration_error=norm(unknown[:6]-u0[:6]),tangent_multiplier_error=norm(unknown[6:]-u0[6:]),discrete_unknown_scaled_coefficient_error=norm((unknown-u0)/h-u1),pressure_endpoint_coefficient_error=norm(unknown[6:]-endpoint['pressure']),pressure_actual_average_coefficient_error=norm(unknown[6:]-paverage),pressure_average_scaled_bias_error=norm((unknown[6:]-paverage)/h-np.array(expected['average_pressure_coefficient_bias'],float)),first_pressure_difference_minus_true_derivative=norm((unknown[6:]-u0[6:])/h-udot[6:]),first_pressure_difference_limit_error=norm((unknown[6:]-u0[6:])/h-u1[6:]),expected_nonvanishing_first_pressure_derivative_error=norm(u1[6:]-udot[6:]),integrated_actual_path_momentum_norm=norm(E),actual_path_momentum_rate_norm=norm(E)/h,physical_integrated_original_gate_pass=bool(norm(E)<=1e-11),physical_pointwise_original_gate_pass=bool(v['transfers']['maxima'][3]<=1e-11),actual_path_scaled_defect_coefficient_error=norm(E/(h*h)-E2),initial_point_momentum_residual_norm=norm(point),endpoint_vs_path_donor_momentum_norm=norm(delta),donor_momentum_rate_defect_norm=norm(delta)/h,donor_scaled_coefficient_error=norm(delta/(h*h)-L),pressure_impulse_error_norm=norm(impulseerror),pressure_impulse_rate_error_norm=norm(impulseerror)/h,pressure_impulse_scaled_coefficient_error=norm(impulseerror/(h*h)-I2),one_step_velocity_error=float(np.sqrt(dz@true_end['spatial']['M']@dz)),one_step_geometry_error=float(np.max(np.abs(dq))),discrete_work_gate_pass=True,all_full_constraint_maxima=v['transfers']['maxima'][:3].tolist()))
 metrics={}
 for name in ['tangent_acceleration_error','tangent_multiplier_error','discrete_unknown_scaled_coefficient_error','pressure_endpoint_coefficient_error','pressure_actual_average_coefficient_error','pressure_average_scaled_bias_error','actual_path_momentum_rate_norm','actual_path_scaled_defect_coefficient_error','initial_point_momentum_residual_norm','pressure_impulse_rate_error_norm','pressure_impulse_scaled_coefficient_error','first_pressure_difference_limit_error','one_step_velocity_error','one_step_geometry_error','donor_momentum_rate_defect_norm','donor_scaled_coefficient_error']:
  values=[r[name]for r in rows];rs=ratios(values);order=4 if name=='one_step_velocity_error'else 8 if name=='one_step_geometry_error'else 4 if name=='donor_momentum_rate_defect_norm'and kind=='initial'else 2
  metrics[name]=dict(ratios=rs,expected_ratio=order,coarse_band_pass=band(rs[:2],order),fine_band_pass=band(rs[-2:],order))
  require(metrics[name]['fine_band_pass'],'independently predicted fine order '+kind+' '+name)
 require(reference_difference<1e-10,'independent pressure reference refinement')
 require(rows[-1]['first_pressure_difference_minus_true_derivative']>rows[-1]['expected_nonvanishing_first_pressure_derivative_error']/2,'retain false pressure-as-evolved derivative counterexample')
 return dict(kind=kind,rows=rows,metrics=metrics,reference_calls=[ncoarse,nfine],pressure_reference_refinement_max_difference=reference_difference,pressure_is_algebraic=True,pressure_first_difference_is_not_an_evolution_equation=True)

def repeated_pressure(kind):
 label='pressure'if kind=='pressure_state'else'initial';packet=json.loads((P.parent/('fitted-discrete-refinement/trials/repaired-'+label+'-trajectory.json')).read_text());ref=packet['references'][-1];r=m.evaluate(np.r_[ref['final_x'],np.array(ref['final_z'])[m.FREE]])[1];rows=[]
 native=[json.loads(x)for x in(P.parent/'native-coupled-discrete/native-qualification/native-default.jsonl').read_text().splitlines()]
 for case in packet['replays']:
  c=case['steps'][-1];q=np.array(c['end_q']);e=np.array(c['end_eta']);u=np.array(c['unknowns']);v=step.point(np.array(c['initial_q']),np.array(c['initial_eta']),u,c['interval']);Qn=v['spatial']['Q'];pressure=Qn@u[6:];pref=r['spatial']['Q']@r['pressure'];weights=r['spatial']['area'];error=float(np.sqrt(np.sum(weights*(pressure-pref)**2)/np.sum(weights)))
  n=next(x for x in native if x['kind']==kind and x['h']==case['interval']and x['step']==len(case['steps']));npres=np.array(n['physical_pressure']);nerror=float(np.sqrt(np.sum(weights*(npres-pref)**2)/np.sum(weights)))
  rows.append(dict(interval=case['interval'],steps=len(case['steps']),host_pressure_ALE_pullback_RMS_error=error,native_pressure_ALE_pullback_RMS_error=nerror,host_native_pressure_max_difference=float(np.max(np.abs(npres-pressure)))))
 rs=ratios([x['native_pressure_ALE_pullback_RMS_error']for x in rows]);require(band(rs[-2:]),'actual repeated native pressure first-order vs independent DAE')
 return dict(scope='physical microtriangle pressure pulled back by frozen labels; reference-area-weighted RMS, not continuum spatial error',kind=kind,rows=rows,native_pressure_error_ratios=rs)

def study():
 existing=json.loads((P.parent/'fitted-discrete-refinement/trials/refined-cells.json').read_text());rows=[];repeated=[]
 for r in existing['rows']:
  expected=taylor.derive(r['kind']);rows.append(observe(r['kind'],r['cells'],expected));repeated.append(repeated_pressure(r['kind']));print('qualified equation consistency',r['kind'],file=sys.stderr,flush=True)
 require(existing['rows'][1]['coarse_diagnostic_pass']is False,'retained original coarse pressure-state failure')
 return dict(status='PASS',scope='conditional temporal consistency of declared semidiscrete ALE donor momentum and algebraic multiplier; no continuum spatial/traction validation',rows=rows,repeated_pressure=repeated,original_coarse_pressure_flux_diagnostic_pass=False,numerical_or_physical_gates_relaxed=0,new_Lean_claims=0,new_Rust_changes=0)
if __name__=='__main__':print(json.dumps(study(),indent=2))
