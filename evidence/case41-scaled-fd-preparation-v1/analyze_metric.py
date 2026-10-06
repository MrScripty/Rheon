"""Existing-data units/scales and exact dual-mass pressure projection; no native call."""
from pathlib import Path
from fractions import Fraction as Q
import importlib.util,json,struct,sys
import mpmath as mp
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('fixed_focus',ROOT/'evidence/forcing-case41-scale-diagnosis-v1/analyze.py');f=importlib.util.module_from_spec(s);s.loader.exec_module(f)
def exact_positive_definite(M):
 n=len(M);L=[[Q(i==j)for j in range(n)]for i in range(n)];d=[]
 for j in range(n):
  value=M[j][j]-sum(L[j][k]**2*d[k]for k in range(j));f.require(value>0,'exact positive LDL pivots');d.append(value)
  for i in range(j+1,n):L[i][j]=(M[i][j]-sum(L[i][k]*L[j][k]*d[k]for k in range(j)))/value
 f.require(f.mul(f.mul(L,[[d[i]if i==j else Q(0)for j in range(n)]for i in range(n)]),f.transpose(L))==M,'exact LDL factor identity');return d

def run(dps):
 mp.mp.dps=dps;C=ROOT/'evidence/forcing-e2-fixed-candidates-v1';c=next(x for x in json.loads((C/'inputs.json').read_text())['cases']if x['index']==41);c=dict(c,acceleration=[.0625,-.125,.03125]);records=f.a.records(C/'candidate-fixed.log');out=[];L=Q(1);U=Q(1);mass=sum(map(Q,c['accepted_publication']['mass']));f.require(mass==Q(27,8),'actual conserved fixture mass');T=L/U;acc=U/T;force=mass*acc;pressure_coordinate=force/L
 for order in [16,32]:
  e=next(x for x in records if x['event']=='fixed_equation'and x['index']==41 and x['order']==order);r,_=f.a.exact(c,e);M=[[sum(Q(e['end_mass'][i])*Q(e['r'][i][d][j])*Q(e['r'][i][d][k])for i in range(16)for d in range(2))for k in range(22)]for j in range(22)];pivots=exact_positive_definite(M);W=[[x/(mass*acc**2)for x in row]for row in f.inverse(M)];B=f.transpose([[Q(x)for x in row]for row in e['end_b']]);BtW=f.mul(f.transpose(B),W);N=f.mul(BtW,B);coef=f.mv(f.inverse(N),f.mv(BtW,r));p=f.mv(B,coef);q=[x-y for x,y in zip(r,p)];f.require(f.mv(BtW,q)==[Q(0)]*16,'exact weighted complement');sq=lambda v:f.dot(v,f.mv(W,v));f.require(sq(r)==sq(p)+sq(q),'exact weighted Pythagoras');native=list(map(Q,e['rate']));error=[x-y for x,y in zip(native,r)]
  norm=lambda v:float(mp.sqrt(f.a.mpq(sq(v))))
  # Independent congruence check under one declared velocity-basis reparameterization.
  scales=[Q(2)if j==3 else Q(1)for j in range(22)];Mp=[[M[i][j]*scales[i]*scales[j]for j in range(22)]for i in range(22)];rp=[r[j]*scales[j]for j in range(22)];f.require(f.dot(r,f.mv(f.inverse(M),r))==f.dot(rp,f.mv(f.inverse(Mp),rp)),'dual-mass invariance under exact basis rescale');row={'order':order,'raw_exact_force_norm':f.a.normq(r),'uniform_physical_scaled_residual_norm':f.a.normq([x/force for x in r]),'uniform_physical_scaled_target':float(Q(1e-13)/force),'dual_mass_dimensionless_residual_norm':norm(r),'dual_mass_dimensionless_pressure_range_norm':norm(p),'dual_mass_dimensionless_complement_norm':norm(q),'dual_mass_native_minus_exact_norm':norm(error),'dual_mass_pressure_squared_share':float(sq(p)/sq(r)),'weighted_squared_residual_exact':str(sq(r)),'weighted_squared_range_exact':str(sq(p)),'weighted_squared_complement_exact':str(sq(q)),'raw_weighted_range_components':[float(x)for x in p],'raw_weighted_complement_components':[float(x)for x in q],'exact_mass_LDL_pivots':[str(x)for x in pivots],'weighted_normal_Gram_condition':f.condition(N),'original_raw_gate_unchanged':True,'mass_metric_is_not_a_replacement_gate':True,'FD_columns_available':0};out.append(row)
 deltas=[]
 for j in range(6):
  alpha=c['final_authorized_unknowns'][j];delta=1e-6*(abs(alpha)+.01);perturbed=alpha+delta;bits=lambda x:struct.pack('>d',x).hex();actual=Q(perturbed)-Q(alpha);deltas.append({'column':j,'alpha':alpha,'alpha_bits':bits(alpha),'delta':delta,'delta_bits':bits(delta),'perturbed_alpha':perturbed,'perturbed_alpha_bits':bits(perturbed),'actual_offset_exact':str(actual),'actual_to_nominal_ratio':float(actual/Q(delta))})
 return {'status':'PASS_EXISTING_DATA_DUAL_MASS_METRIC','precision_digits':dps,'scales':{'length':float(L),'velocity':float(U),'time':float(T),'mass':float(mass),'acceleration':float(acc),'force':float(force),'pressure_coordinate':float(pressure_coordinate),'pressure_sample':float(pressure_coordinate/L),'native_target_force':1e-13,'integrated_momentum_target':float(Q(c['h'])*Q(1e-13))},'rows':out,'prescribed_FD_roster':deltas,'new_native_equations':0,'new_owners':0,'new_corrections':0,'full_Newton_condition_claimed':False,'physical_state_error_bound_claimed':False,'arithmetic_floor_claimed':False,'scope':'Exact captured pressure-space diagnostics in two fixed metrics; missing six native FD columns prohibit full Jacobian and complementary acceleration-range qualification.'}
if __name__=='__main__':print(json.dumps(run(int(sys.argv[1])if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
