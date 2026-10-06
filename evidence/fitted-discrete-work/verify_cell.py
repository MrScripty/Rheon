"""Actual first-order discrete-work cell replay; retain different equation failures."""
import argparse
import copy
import json
import numpy as np
import step
m=step.m


def close(a,b,tolerance=1e-11):
 a=np.asarray(a,float);b=np.asarray(b,float)
 m.require(a.shape==b.shape and np.all(np.isfinite(a))and np.max(np.abs(a-b),initial=0)<=tolerance,'actual cell replay arithmetic')


def verify(data):
 q=np.array(data['initial_q']);eta=np.array(data['initial_eta']);unknown=np.array(data['unknowns']);h=data['interval']
 m.require(q.shape==(3,)and eta.shape==(6,)and unknown.shape==(22,),'complete fixed system')
 m.require(np.isfinite(h)and 0<h<=.05,'bounded actual replay interval')
 v=step.evaluate(q,eta,unknown,h,32,True);allow=step.qualify(v,h)
 for key in ['energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','pressure_work','GCL_work','discrete_residual_work','energy_ledger_error','fixed_work_allowance']:close(data[key],v[key],v['fixed_work_allowance'])
 close(data['finite_momentum_residual'],v['residual']);close(data['finite_momentum_rate_norm'],np.linalg.norm(v['residual'])/h)
 close(data['actual_local_GCL_defect'],v['gcl'],min(allow));close(data['local_GCL_allowance'],allow,min(allow))
 close(data['face_sign_partition'],v['transfers']['partition'],1e-15)
 close(data['full_path_maxima'],v['transfers']['maxima']);close(data['minimum_sampled_quality'],v['transfers']['minimum_quality'])
 close(data['horizontal_momentum_change'],v['horizontal_momentum_change'])
 close(data['endpoint_donor_momentum_convection'],v['endpoint_momentum_convection'])
 close(data['actual_integrated_physical_momentum_convection'],v['transfers']['actual_integrated_momentum_convection'])
 close(data['endpoint_vs_actual_momentum_max'],np.max(np.abs(v['endpoint_vs_actual_momentum_convection'])))
 close(data['actual_integrated_physical_momentum_residual'],v['actual_integrated_physical_momentum_residual'])
 close(data['actual_integrated_physical_momentum_norm'],np.linalg.norm(v['actual_integrated_physical_momentum_residual']))
 m.require(data['actual_physical_integrated_momentum_gate_pass']==(np.linalg.norm(v['actual_integrated_physical_momentum_residual'])<=1e-11),'honest different exact integrated physical equation')
 m.require(data['continuous_momentum_gate_pass']==(v['transfers']['maxima'][3]<=1e-11),'honest pointwise continuous momentum gate')
 m.require(data['discrete_work_gate_pass']is True and data['research_discrete_candidate_qualified']is True and data['new_public_step_enabled']is False,'qualified research scope only')
 close(data['end_q'],v['end']['q']);close(data['end_eta'],v['end']['eta']);close(data['end_reduced_velocity'],v['end']['z']);close(data['end_pressure'],v['end']['spatial']['Q']@unknown[6:])
 m.require(len(data['positive_negative_face_integrals'])==len(m.PAIR),'complete physical face inventory')
 for row,pair,plus,minus in zip(data['positive_negative_face_integrals'],m.PAIR,v['transfers']['plus'],v['transfers']['minus']):close(row,[*pair,plus,minus],1e-15)
 m.require(len(data['samples'])==5 and len(data['quadrature_points'])==32*(len(v['transfers']['partition'])-1),'complete physical trajectory')
 for row,t in zip(data['samples'],np.linspace(0,h,5)):
  p=step.point(q,eta,unknown,t);s=p['spatial']
  for key,value in [('time',t),('q',p['q']),('eta',p['eta']),('nodes',s['pos']),('velocity',p['flux']['velocity']),('reduced_velocity',p['z']),('mass',s['mass']),('pressure',s['Q']@unknown[6:])]:close(row[key],value)
 for a,b in zip(data['quadrature_points'],v['transfers']['points']):
  m.require(set(a)==set(b),'complete quadrature replay')
  for key in a:close(a[key],b[key])
 return dict(interval=h,finite_momentum_rate_norm=float(np.linalg.norm(v['residual'])/h),discrete_work=float(v['discrete_residual_work']),fixed_allowance=float(v['fixed_work_allowance']),ledger_error=float(v['energy_ledger_error']),continuous_momentum_gate_pass=data['continuous_momentum_gate_pass'],actual_physical_integrated_momentum_gate_pass=data['actual_physical_integrated_momentum_gate_pass'],new_public_step_enabled=False)


def negative_tests(data):
 controls=['false_public_step','zero_pressure','omit_normal_acceleration','omit_BE_loss','omit_mixing_loss','mislabel_viscosity','hide_continuous_momentum','claim_exact_physical_integrated_momentum','substitute_physical_momentum_flux','face_circulation','stale_endpoint_mass','wrong_pressure_work']
 for name in controls:
  bad=copy.deepcopy(data)
  if name=='false_public_step':bad['new_public_step_enabled']=True
  elif name=='zero_pressure':bad['unknowns'][6:]=[0.]*16
  elif name=='omit_normal_acceleration':bad['unknowns'][2]=0.
  elif name=='omit_BE_loss':bad['backward_Euler_loss']=0.
  elif name=='omit_mixing_loss':bad['mixing_loss']=0.
  elif name=='mislabel_viscosity':bad['viscous_loss']+=bad['mixing_loss']
  elif name=='hide_continuous_momentum':bad['continuous_momentum_gate_pass']=True
  elif name=='claim_exact_physical_integrated_momentum':bad['actual_physical_integrated_momentum_gate_pass']=True
  elif name=='substitute_physical_momentum_flux':bad['endpoint_donor_momentum_convection']=bad['actual_integrated_physical_momentum_convection'][:]
  elif name=='face_circulation':
   rows=bad['positive_negative_face_integrals'];lookup={(i,j):k for k,(i,j,_,_)in enumerate(rows)};tri=list(map(int,m.IDS[m.TRI[0]]));before=np.zeros(16)
   for i,j,p,n in rows:before[i]+=p-n;before[j]-=p-n
   for i,j in zip(tri,tri[1:]+tri[:1]):rows[lookup[min(i,j),max(i,j)]][2 if i<j else 3]+=1/1024
   after=np.zeros(16)
   for i,j,p,n in rows:after[i]+=p-n;after[j]-=p-n
   m.require(np.max(np.abs(before-after))<1e-15,'circulation leaves node marginals intact')
  elif name=='stale_endpoint_mass':bad['samples'][-1]['mass']=bad['samples'][0]['mass'][:]
  else:bad['pressure_work']=1e-5
  try:verify(bad)
  except ValueError:print('REJECTED',name)
  else:raise ValueError('corruption accepted '+name)

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('payload');p.add_argument('--negative-self-test',action='store_true');args=p.parse_args();data=json.load(open(args.payload));result=verify(data)
 if args.negative_self_test:negative_tests(data)
 print('PASS',json.dumps(result,sort_keys=True))
