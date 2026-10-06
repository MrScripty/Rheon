"""Replay actual native accepted state against independently assembled host operators.

Native old velocity/mass is the accepted publication, never silently replaced by
an exactly constrained chart reconstruction. Every native equation and loss is
checked. Exported histories are bounded evidence, not owner snapshots.
"""
import copy,json,sys
from pathlib import Path
import numpy as np
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P.parent/'fitted-discrete-refinement'))
import temporal
m=temporal.m

def require(x,s):
 if not x:raise ValueError(s)
def coeff(u):
 z=np.zeros(22)
 for j in range(22):
  found=False
  for i in range(16):
   for d in range(2):
    if np.array_equal(m.R_NODE[i,d],np.eye(22)[j]):z[j]=u[i,d];found=True;break
   if found:break
  require(found,'one-hot coefficients')
 return z

def replay(rows):
 require(len(rows)==124,'all native accepted steps')
 groups={};result=[];maxima=dict(momentum=0.,direct_momentum=0.,work_ratio=0.,ledger_ratio=0.,gcl_ratio=0.,quad=0.,velocity_difference=0.,pressure_difference=0.)
 for row in rows:groups.setdefault((row['kind'],row['h']),[]).append(row)
 require(len(groups)==10,'both fields five refinements')
 for(kind,h),steps in groups.items():
  q,eta=temporal.initial(kind);s=m.spatial(m.q_to_x(q))
  if kind=='initial':u=np.zeros((16,3));u[:,0]=s['pos'][np.array([np.flatnonzero(m.IDS==i)[0]for i in range(16)]),1]
  else:u=temporal.point(q,eta,temporal.seed(q,eta),0.)['flux']['velocity']
  mass=s['mass'];clock=0.
  host=json.loads((P.parent/('fitted-discrete-refinement/trials/repaired-'+('pressure'if kind=='pressure_state'else'initial')+'-trajectory.json')).read_text())
  require(len(steps)==round(.1/h), 'complete native case')
  for index,row in enumerate(steps):
   require(row['step']==index+1,'actual stamp sequence');unknown=np.array(row['unknowns']);partition=temporal.sign_partition(q,eta,unknown,h);v=temporal.evaluate(q,eta,unknown,h,32,partition=partition);v16=temporal.evaluate(q,eta,unknown,h,16,partition=partition)
   end=v['end'];newu=np.array(row['velocity']);z1=coeff(newu);z0=coeff(u);newmass=end['spatial']['mass'];res=m.R_NODE
   require(np.max(np.abs(newu-end['flux']['velocity']))<=1e-11,'native full nodal endpoint vs physical chart')
   require(np.max(np.abs(np.array(row['end_q'])-end['q']))<=1e-11,'native actual geometry endpoint')
   require(np.max(np.abs(np.array(row['end_eta'])-end['eta']))<=1e-11,'native actual chart coordinates')
   require(np.array_equal(np.array(row['pressure_coefficients']),unknown[6:]),'native pressure publication')
   if 'mass' in row:
    require(np.max(np.abs(np.array(row['mass'])-newmass))<=1e-14,'actual native published masses')
    require(np.max(np.abs(np.array(row['positions'])-end['spatial']['pos']))<=1e-14,'actual native published PS geometry')
    require(np.array_equal(np.array(row['triangles']),m.TRI),'actual native published topology')
    pd=float(np.max(np.abs(np.array(row['physical_pressure'])-end['spatial']['Q']@unknown[6:])));require(pd<=1e-11,'actual native physical pressure publication');maxima['pressure_difference']=max(maxima['pressure_difference'],pd)
    newmass=np.array(row['mass'])
   clock+=h;require(row['time']==clock,'actual unclamped common clock')
   convection=temporal.route(v['transfers']['plus'],v['transfers']['minus'],newu)
   force=end['spatial']['K']@z1+end['spatial']['B'].T@unknown[6:]
   inertia=np.einsum('ndj,nd->j',res,newmass[:,None]*np.einsum('ndj,j->nd',res,z1-z0)+(newmass-mass)[:,None]*u)
   direct=np.einsum('ndj,nd->j',res,newmass[:,None]*newu-mass[:,None]*u)
   residual=inertia+convection+h*force;directres=direct+convection+h*force
   rate=np.linalg.norm(residual)/h;directrate=np.linalg.norm(directres)/h
   require(rate<=1e-11 and directrate<=1e-11,'all actual native 22 momentum equations')
   require(np.max(np.abs(end['spatial']['D']@z1))<=1e-11,'full actual native strong divergence')
   require(np.all(v['transfers']['maxima'][:3]<=1e-11),'full path divergence acceleration/material')
   gcl=newmass-mass;np.add.at(gcl,m.PAIR_I,v['transfers']['plus']-v['transfers']['minus']);np.add.at(gcl,m.PAIR_J,-v['transfers']['plus']+v['transfers']['minus'])
   gallow=128*temporal.EPS*(mass+newmass);gratio=float(np.max(np.abs(gcl)/gallow));require(gratio<=1,'actual native physical local GCL')
   e0=.5*np.sum(mass[:,None]*u*u);e1=.5*np.sum(newmass[:,None]*newu*newu)
   dbe=.5*np.sum(mass[:,None]*(newu-u)**2);dmix=.5*np.sum((v['transfers']['plus']+v['transfers']['minus'])*np.sum((newu[m.PAIR_I]-newu[m.PAIR_J])**2,axis=1));dmu=h*z1@end['spatial']['K']@z1;wp=h*z1@end['spatial']['B'].T@unknown[6:];wg=.5*np.sum(gcl*np.sum(newu*newu,axis=1));wr=z1@residual;ledger=e1-e0+dbe+dmix+dmu+wp+wg-wr
   allow=128*temporal.EPS*(e0+e1+dbe+dmix+abs(dmu)+abs(wp)+abs(wg)+abs(wr))
   require(min(dbe,dmix,dmu)>=0 and max(abs(wr),abs(ledger),abs(wp))<=allow,'actual accepted-state complete work gates')
   for name,value in [('energy_old',e0),('energy_new',e1),('backward_Euler_loss',dbe),('mixing_loss',dmix),('viscous_loss',dmu),('pressure_work',wp),('GCL_work',wg),('discrete_residual_work',wr),('energy_ledger_error',ledger),('fixed_work_allowance',allow)]:require(abs(row['report'][name]-value)<=allow,'native reported '+name)
   quad=max(np.max(np.abs(v['transfers'][k]-v16['transfers'][k]))for k in ['plus','minus','actual_integrated_momentum_convection']);require(quad<=1e-15,'actual physical quad refinement')
   require(row['report']['finite_momentum_rate_norm']<=1e-13 and row['report']['equation_evaluations']<=200,'unchanged native Newton and resource bounds')
   maxima['momentum']=max(maxima['momentum'],rate);maxima['direct_momentum']=max(maxima['direct_momentum'],directrate);maxima['work_ratio']=max(maxima['work_ratio'],abs(wr)/allow);maxima['ledger_ratio']=max(maxima['ledger_ratio'],abs(ledger)/allow);maxima['gcl_ratio']=max(maxima['gcl_ratio'],gratio);maxima['quad']=max(maxima['quad'],quad)
   q=np.array(row['end_q']);eta=np.array(row['end_eta']);u=newu;mass=newmass
  ref=np.array(host['references'][-1]['final_z']);refmass=m.spatial(np.array(host['references'][-1]['final_x']))['M'];dz=z1-ref;error=float(np.sqrt(dz@refmass@dz));geo=float(np.max(np.abs(m.q_to_x(q)-host['references'][-1]['final_x'])))
  hostrow=next(r for r in host['rows']if r['interval']==h);vd=float(np.max(np.abs(z1-hostrow['final_velocity'])));maxima['velocity_difference']=max(maxima['velocity_difference'],vd);require(vd<=1e-11,'native versus qualified host endpoint')
  result.append(dict(kind=kind,interval=h,steps=len(steps),velocity_lumped_L2_error=error,geometry_max_error=geo,host_final_velocity_max_difference=vd,actual_clock=clock,final_stamp=steps[-1]['step']))
 for kind in ['initial','pressure_state']:
  rr=[r for r in result if r['kind']==kind];ratios=[a['velocity_lumped_L2_error']/b['velocity_lumped_L2_error']for a,b in zip(rr,rr[1:])];require(all(1.8<x<2.2 for x in ratios),'actual native first-order temporal refinement')
 return dict(status='PASS',scope='actual native accepted-state replay; endpoint donor first order, not exact continuous momentum',steps=124,rows=result,maxima=maxima)
if __name__=='__main__':
 rows=[json.loads(line)for line in Path(sys.argv[1]).read_text().splitlines()];out=replay(rows)
 if '--negative-self-test' in sys.argv:
  controls=[]
  for name in ['velocity','pressure','clock','energy','geometry','mass']:
   changed=copy.deepcopy(rows)
   if name=='velocity':changed[0]['velocity'][0][0]+=.01
   elif name=='pressure':changed[0]['pressure_coefficients'][0]+=.01
   elif name=='clock':changed[0]['time']=0.
   elif name=='energy':changed[0]['report']['energy_new']+=.01
   elif name=='geometry':changed[0]['positions'][6][0]+=.01
   else:changed[0]['mass'][0]+=.01
   try:replay(changed)
   except ValueError as error:controls.append(dict(control=name,status='REJECTED',reason=str(error)))
   else:raise ValueError('corruption accepted '+name)
  out['actual_corruption_rejections']=controls
 print(json.dumps(out,indent=2))
