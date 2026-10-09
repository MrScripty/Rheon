"""Independent body-force sources on the frozen moving full-velocity model."""
import sys,importlib.util
from pathlib import Path
import numpy as np
from scipy.integrate import solve_ivp
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'extruded-third-velocity'))
spec=importlib.util.spec_from_file_location('frozen_third_reference',Path(__file__).resolve().parents[1]/'extruded-third-velocity/reference.py')
third=importlib.util.module_from_spec(spec);spec.loader.exec_module(third)
t=third.temporal;m=third.m
A=np.array([.0625,-.125,.03125])
H=third.H
LIMIT=1e-11
def require(ok,message):
 if not ok:raise ValueError(message)
def source(s,a):
 return np.einsum('ndj,nd->j',m.R_NODE,s['mass'][:,None]*a),third.RW.T@(s['mass']*a[2])
def instantaneous(state,a):
 x,eta=state[:4],state[4:10];s=m.spatial(x);z=s['H']@eta
 motion=m.cap_motion(z);xdot=np.array([*motion[0],*motion[1]])
 s=m.spatial(x,xdot);f=m.flux(s,z);force,_=source(s,a)
 g=np.einsum('ndj,nd->j',m.R_NODE,s['mdot'][:,None]*f['velocity']+f['convection'])+s['K']@z
 saddle=np.block([[s['M'],s['B'].T],[s['B'],np.zeros((16,16))]])
 target=s['Q'].T@(s['area']*(s['Ddot']@z));answer=np.linalg.solve(saddle,np.r_[force-g,target]);dz,p=answer[:22],answer[22:]
 require(np.linalg.norm(s['M']@dz+s['B'].T@p+g-force)<=LIMIT,'forced instantaneous full momentum')
 require(np.max(np.abs(s['D']@dz+s['Ddot']@z))<=LIMIT,'forced instantaneous moving acceleration constraint')
 mw,kx,ky,_,_=third.blocks(s);xi=state[10:];w=third.RW@xi;flux=f['flux'];cw=third.route(np.maximum(flux,0),np.maximum(-flux,0),w)
 _,fw=source(s,a);dxi=np.linalg.solve(mw,fw-third.RW.T@(s['mdot']*w+cw)-(kx+ky)@xi)
 return np.r_[xdot,dz[m.FREE],dxi],dict(spatial=s,z=z,pressure=p,acceleration=dz,flux=f,third_rate=dxi)
def seed(q,eta,a):
 s=m.spatial(m.q_to_x(q));J,_=m.cap_chart(s);z=J@eta
 _,v=instantaneous(np.r_[m.q_to_x(q),z[m.FREE],third.XI],a)
 return np.r_[m.C_CAP@v['acceleration'],v['acceleration'][m.SELECT],v['pressure']]
def evaluate(q,eta,unknown,h,a,order=16,partition=None):
 v=t.evaluate(q,eta,unknown,h,order,partition=partition);body,_=source(v['start']['spatial'],a)
 v['residual']=v['residual']-h*body;v['direct_residual']=v['direct_residual']-h*body
 v['force_work']=float(h*v['end']['z']@body);v['discrete_residual_work']=float(v['end']['z']@v['residual'])
 v['energy_ledger_error']=v['energy_new']-v['energy_old']+v['backward_Euler_loss']+v['mixing_loss']+v['viscous_loss']+v['pressure_work']+v['GCL_work']-v['force_work']-v['discrete_residual_work']
 v['fixed_work_allowance']=128*t.EPS*sum(abs(v[k])for k in ['energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','pressure_work','GCL_work','force_work','discrete_residual_work'])
 return v
def solve(q,eta,h,a):
 u=seed(q,eta,a);calls=0
 for iteration in range(7):
  v=evaluate(q,eta,u,h,a);calls+=1;rate=v['residual']/h
  if np.linalg.norm(rate)<=1e-13:break
  J=np.zeros((22,22))
  for j in range(6):
   d=1e-6*(abs(u[j])+.01);up=u.copy();up[j]+=d;vp=evaluate(q,eta,up,h,a);calls+=1;J[:,j]=(vp['residual']/h-rate)/d
  J[:,6:]=v['end']['spatial']['B'].T;u+=np.linalg.solve(J,-rate)
  require(calls<=200,'bounded forced Newton calls')
 else:raise ValueError('bounded forced Newton true residual refusal')
 partition=t.sign_partition(q,eta,u,h);coarse=evaluate(q,eta,u,h,a,16,partition);v=evaluate(q,eta,u,h,a,32,partition)
 require(max(np.linalg.norm(v['residual']),np.linalg.norm(v['direct_residual']))/h<=LIMIT,'forced full finite momentum')
 require(np.all(v['transfers']['maxima'][:3]<=LIMIT),'unchanged forced path constraints')
 require(np.all(np.abs(v['gcl'])<=128*t.EPS*(v['start']['spatial']['mass']+v['end']['spatial']['mass'])),'unchanged forced physical GCL')
 require(abs(v['horizontal_momentum_change']-h*3.375*a[0])<=LIMIT,'known horizontal external impulse')
 require(max(abs(v[k])for k in ['energy_ledger_error','discrete_residual_work','pressure_work'])<=v['fixed_work_allowance'],'forced signed planar work')
 quad=max(np.max(np.abs(v['transfers'][k]-coarse['transfers'][k]))for k in ['plus','minus','actual_integrated_momentum_convection']);require(quad<=1e-15,'unchanged forced physical quadrature')
 return u,v,dict(iterations=iteration+1,calls=calls,quadrature=float(quad))
def finite_third(v,h,oldw,oldmass,a):
 s=v['end']['spatial'];mw,kx,ky,_,_=third.blocks(s);plus=v['transfers']['plus'];minus=v['transfers']['minus']
 C=(third.RW[m.PAIR_I]-third.RW[m.PAIR_J]).T@(plus[:,None]*third.RW[m.PAIR_I]-minus[:,None]*third.RW[m.PAIR_J]);fw=third.RW.T@(oldmass*a[2])
 return np.linalg.solve(mw+C+h*(kx+ky),third.RW.T@(oldmass*oldw)+h*fw)
def reference(kind,field,a,rtol,max_step):
 q,eta=t.initial(kind);s=m.spatial(m.q_to_x(q));J,_=m.cap_chart(s);z=J@eta;initial=np.r_[m.q_to_x(q),z[m.FREE],third.initial_xi(field)];calls=0
 def rhs(time,y):
  nonlocal calls
  calls+=1;require(calls<=5000,'bounded forced DAE calls');return instantaneous(y,a)[0]
 answer=solve_ivp(rhs,(0,.1),initial,method='DOP853',rtol=rtol,atol=rtol/100,max_step=max_step)
 require(answer.success and np.all(np.isfinite(answer.y)),'forced DAE reference success');y=answer.y[:,-1];s=m.spatial(y[:4]);z=s['H']@y[4:10]
 return dict(kind=kind,field=field,acceleration=a.tolist(),duration=.1,rtol=rtol,max_step=max_step,calls=calls,final_x=y[:4].tolist(),final_velocity=(np.einsum('ndj,j->nd',m.R_NODE,z)+np.c_[np.zeros((16,2)),third.RW@y[10:]]).tolist())
