"""Bounded independent scalar block of the reviewed z-invariant donor ALE DAE.

The fixed twelve-column trace embedding is inferred from the frozen scalar x
space, not from native exports. Geometry/flux use the reviewed xy equations;
third gradients and conservative momentum are assembled separately here.
"""
from functools import lru_cache
import sys
from pathlib import Path
import numpy as np
from scipy.integrate import solve_ivp
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P.parent/'fitted-discrete-refinement'))
import temporal
m=temporal.m
RW=m.R_NODE[:,0,:12].copy()
RAW=RW[m.IDS]
XI=np.array([.25,.5,-.125,.375,.625,-.25,.1875,.4375,-.0625,.5625,.3125,.125])
H=(.05,.025,.0125,.00625,.003125)
MAX_REFERENCE_CALLS=5000
def require(ok,s):
 if not ok:raise ValueError(s)
require(np.array_equal(RW.sum(axis=1),np.ones(16))and np.linalg.matrix_rank(RW)==12,'constant complete third embedding')
def initial_xi(field):
 require(field in ['nonconstant','constant'],'declared third field')
 return XI.copy()if field=='nonconstant'else np.full(12,.25)
def blocks(s):
 p=s['pos'][m.TRI];a=p[:,1]-p[:,0];b=p[:,2]-p[:,0];a2=a[:,0]*b[:,1]-a[:,1]*b[:,0]
 require(np.all(a2>0),'positive independently assembled third areas')
 grad=np.empty((24,3,2))
 for i in range(3):
  j,k=(i+1)%3,(i+2)%3;grad[:,i,0]=(p[:,j,1]-p[:,k,1])/a2;grad[:,i,1]=(p[:,k,0]-p[:,j,0])/a2
 ex=np.einsum('ti,tij->tj',grad[:,:,0],RAW[m.TRI]);ey=np.einsum('ti,tij->tj',grad[:,:,1],RAW[m.TRI])
 kx=.05*np.einsum('t,tj,tk->jk',a2/2,ex,ex);ky=.05*np.einsum('t,tj,tk->jk',a2/2,ey,ey)
 return RW.T@(s['mass'][:,None]*RW),kx,ky,ex,ey
def route(plus,minus,w):
 c=np.zeros(16);f=plus*w[m.PAIR_I]-minus*w[m.PAIR_J];np.add.at(c,m.PAIR_I,f);np.add.at(c,m.PAIR_J,-f);return c
def rate(v,xi):
 s=v['spatial'];mass,kx,ky,_,_=blocks(s);w=RW@xi;f=v['flux']['flux'];c=route(np.maximum(f,0),np.maximum(-f,0),w)
 rhs=-(RW.T@(s['mdot']*w+c)+(kx+ky)@xi);return np.linalg.solve(mass,rhs)
def finite(v,h,oldw,oldmass):
 s=v['end']['spatial'];mass,kx,ky,_,_=blocks(s);plus=v['transfers']['plus'];minus=v['transfers']['minus']
 c=(RW[m.PAIR_I]-RW[m.PAIR_J]).T@(plus[:,None]*RW[m.PAIR_I]-minus[:,None]*RW[m.PAIR_J])
 a=mass+c+h*(kx+ky);rhs=RW.T@(oldmass*oldw);return np.linalg.solve(a,rhs)
def reference(kind,field,rtol,max_step):
 q,e=temporal.initial(kind);z=temporal.point(q,e,temporal.seed(q,e),0.)['z'];state=np.r_[m.q_to_x(q),z[m.FREE],initial_xi(field)];calls=0
 def rhs(t,y):
  nonlocal calls
  calls+=1;require(calls<=MAX_REFERENCE_CALLS,'bounded full reference calls');xy,v=m.evaluate(y[:10]);return np.r_[xy,rate(v,y[10:])]
 answer=solve_ivp(rhs,(0,.1),state,method='DOP853',rtol=rtol,atol=rtol/100,max_step=max_step)
 require(answer.success and np.all(np.isfinite(answer.y)),'full reference success');y=answer.y[:,-1];s=m.spatial(y[:4]);z=s['H']@y[4:10]
 return dict(kind=kind,field=field,duration=.1,rtol=rtol,max_step=max_step,calls=calls,final_x=y[:4].tolist(),final_z=z.tolist(),final_xi=y[10:].tolist(),final_velocity=(np.einsum('ndj,j->nd',m.R_NODE,z)+np.c_[np.zeros((16,2)),RW@y[10:]]).tolist())
