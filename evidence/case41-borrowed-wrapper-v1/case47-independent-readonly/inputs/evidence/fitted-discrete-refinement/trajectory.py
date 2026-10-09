"""Bounded host research publication, rollback and coupled ODE refinement."""
from dataclasses import dataclass,asdict
import json
import sys
import numpy as np
from scipy.integrate import solve_ivp
import temporal as step
m=step.m
MAX_STEPS=64
MAX_ODE_EVALUATIONS=5000
BARRIERS=('before_solve','after_solve','after_qualify','before_publish')

@dataclass(frozen=True)
class State:
 q:tuple
 eta:tuple
 pressure:tuple
 clock:float
 stamp:int

class Cancelled(Exception):pass

class HostOwner:
 """One accepted tuple state; one bounded candidate, no history or snapshots.

 This standalone host experiment is not a second production geometry authority
 or a native memory ledger. Python allocator overhead is not bounded in bytes.
 """
 def __init__(self,kind='initial'):
  q,eta=step.initial(kind);s=m.spatial(m.q_to_x(q));J,_=m.cap_chart(s)
  _,v=m.evaluate(np.r_[m.q_to_x(q),(J@eta)[m.FREE]])
  self.accepted=State(tuple(q),tuple(eta),tuple(v['pressure']),0.,0)
 def advance(self,h,cancel=lambda stage:False,fail=None):
  def barrier(stage):
   if cancel(stage):raise Cancelled(stage)
   if stage==fail:raise ValueError('injected failure '+stage)
  m.require(self.accepted.stamp<MAX_STEPS,'bounded research steps')
  barrier('before_solve')
  old=self.accepted
  cell=step.solve(h,state=(np.array(old.q),np.array(old.eta)),diagnostics=False)
  barrier('after_solve')
  # Reassemble/gate actual curve after candidate completion, not metadata alone.
  evaluated=step.evaluate(np.array(old.q),np.array(old.eta),np.array(cell['unknowns']),h,32)
  step.qualify(evaluated,h)
  candidate=State(tuple(cell['end_q']),tuple(cell['end_eta']),tuple(cell['unknowns'][6:]),old.clock+h,old.stamp+1)
  m.require(np.isfinite(candidate.clock)and candidate.clock>old.clock,'candidate clock')
  barrier('after_qualify');barrier('before_publish')
  self.accepted=candidate
  return cell


def fingerprint(state):return json.dumps(asdict(state),sort_keys=True,separators=(',',':'))


def reference(end=.1,rtol=2e-12,max_step=.003125,kind='initial'):
 q,eta=step.initial(kind);z=step.point(q,eta,step.seed(q,eta),0.)['z']
 initial=np.r_[m.q_to_x(q),z[m.FREE]];calls=0
 def rhs(t,state):
  nonlocal calls
  calls+=1;m.require(calls<=MAX_ODE_EVALUATIONS,'bounded independent ODE calls')
  return m.evaluate(state)[0]
 answer=solve_ivp(rhs,(0,end),initial,method='DOP853',rtol=rtol,atol=rtol/100,max_step=max_step)
 m.require(answer.success,'coupled ODE reference success')
 y=answer.y[:,-1];s=m.spatial(y[:4]);z=s['H']@y[4:]
 return dict(rtol=rtol,max_step=max_step,calls=calls,final_x=y[:4].tolist(),final_z=z.tolist(),strong_divergence_max=float(np.max(np.abs(s['D']@z))),geometry_height_sum_error=float(abs(y[1]+y[3]-2.25)))


def rollback_checks():
 owner=HostOwner();owner.advance(.05);checks=[]
 m.require(owner.accepted.clock>0 and owner.accepted.stamp==1,'nonzero accepted state')
 for failure in [False,True]:
  for stage in BARRIERS:
   before=fingerprint(owner.accepted)
   try:
    owner.advance(.025,cancel=(lambda s,target=stage:s==target)if not failure else (lambda s:False),fail=stage if failure else None)
   except (Cancelled,ValueError)as error:
    m.require(fingerprint(owner.accepted)==before,'all accepted geometry velocity pressure clock stamp preserved')
    checks.append(dict(mode='failure'if failure else 'cancel',barrier=stage,exception=str(error),accepted_state_unchanged=True))
   else:raise ValueError('rollback injection did not trigger')
 before=fingerprint(owner.accepted)
 for h in [0.,-.01,float('nan'),.051]:
  try:owner.advance(h)
  except ValueError:m.require(fingerprint(owner.accepted)==before,'invalid interval rollback')
  else:raise ValueError('invalid interval accepted')
 # Continue from that unchanged nonzero state, rather than reconstructing owner.
 cell=owner.advance(.025)
 m.require(owner.accepted.stamp==2 and owner.accepted.clock==.07500000000000001,'successful continuation')
 return dict(checks=checks,invalid_interval_failures=4,successful_continuation=True,final_state=asdict(owner.accepted),final_energy=cell['energy_new'],production_cancellation_claim=False)


def study(end=.1,kind='initial',progress=None):
 coarse=reference(end,1e-10,.00625,kind);fine=reference(end,2e-12,.003125,kind)
 print('two coupled ODE references complete',coarse['calls'],fine['calls'],file=sys.stderr,flush=True)
 refz=np.array(fine['final_z']);refx=np.array(fine['final_x']);refmass=m.spatial(refx)['M']
 reference_difference=float(np.sqrt((np.array(coarse['final_z'])-refz)@refmass@(np.array(coarse['final_z'])-refz)))
 rows=[];replays=[]
 for h in [.05,.025,.0125,.00625,.003125]:
  owner=HostOwner(kind);count=round(end/h);m.require(count<=MAX_STEPS,'bounded refinement')
  max_work=max_gcl=max_rate=max_quad=0.;loss=np.zeros(3);replay=[]
  for i in range(count):
   cell=owner.advance(h);loss+=np.array([cell['backward_Euler_loss'],cell['mixing_loss'],cell['viscous_loss']]);max_work=max(max_work,abs(cell['discrete_residual_work'])/cell['fixed_work_allowance']);max_gcl=max(max_gcl,max(abs(g)/a for g,a in zip(cell['actual_local_GCL_defect'],cell['local_GCL_allowance'])));max_rate=max(max_rate,cell['finite_momentum_rate_norm']);max_quad=max(max_quad,cell['quadrature_16_32_difference'])
   replay.append(dict(step=i+1,interval=h,initial_q=cell['initial_q'],initial_eta=cell['initial_eta'],unknowns=cell['unknowns'],end_q=cell['end_q'],end_eta=cell['end_eta'],end_pressure=cell['end_pressure'],energy_old=cell['energy_old'],energy_new=cell['energy_new'],backward_Euler_loss=cell['backward_Euler_loss'],mixing_loss=cell['mixing_loss'],viscous_loss=cell['viscous_loss'],fixed_work_allowance=cell['fixed_work_allowance'],direct_momentum_rate_norm=cell['direct_momentum_rate_norm'],finite_momentum_rate_norm=cell['finite_momentum_rate_norm'],max_full_constraints=cell['full_path_maxima'][:3],GCL_defect=cell['actual_local_GCL_defect'],face_sign_partition=cell['face_sign_partition']))
   if progress is not None:progress(dict(kind=kind,status='in progress',completed_cases=replays,current_case=dict(interval=h,steps=replay),references=[coarse,fine]))
  x=m.q_to_x(np.array(owner.accepted.q));s=m.spatial(x);J,_=m.cap_chart(s);z=J@np.array(owner.accepted.eta);dz=z-refz
  rows.append(dict(interval=h,steps=count,velocity_lumped_L2_error=float(np.sqrt(dz@refmass@dz)),geometry_max_error=float(np.max(np.abs(x-refx))),final_q=owner.accepted.q,final_velocity=z.tolist(),final_energy=cell['energy_new'],mass_total=float(np.sum(s['mass'])),max_work_allowance_ratio=max_work,max_GCL_allowance_ratio=max_gcl,max_momentum_rate_norm=max_rate,max_quadrature_difference=max_quad,loss_sums=loss.tolist(),final_stamp=owner.accepted.stamp,actual_clock=owner.accepted.clock))
  replays.append(dict(interval=h,steps=replay))
  print('actual host refinement',json.dumps(rows[-1]),file=sys.stderr,flush=True)
 ratios=[rows[i]['velocity_lumped_L2_error']/rows[i+1]['velocity_lumped_L2_error']for i in range(len(rows)-1)]
 geometry_ratios=[rows[i]['geometry_max_error']/rows[i+1]['geometry_max_error']for i in range(len(rows)-1)]
 m.require(reference_difference<min(r['velocity_lumped_L2_error']for r in rows)/1000,'reference refinement resolves candidate error')
 m.require(all(1.8<r<2.2 for r in ratios),'measured first-order velocity refinement')
 m.require(all(1.7<r<2.3 for r in geometry_ratios),'measured first-order geometry refinement')
 return dict(initial_kind=kind,scope='bounded repeated HOST research first-order endpoint donor momentum, same spatial DAE reference; no public native step',duration=end,rows=rows,velocity_refinement_ratios=ratios,geometry_refinement_ratios=geometry_ratios,reference_refinement_velocity_difference=reference_difference,references=[coarse,fine],replays=replays,rollback=rollback_checks(),new_public_step_enabled=False,continuum_accuracy_claim=False)

if __name__=='__main__':
 from pathlib import Path
 import argparse
 parser=argparse.ArgumentParser();parser.add_argument('kind',choices=['initial','pressure_state']);parser.add_argument('output');args=parser.parse_args();out=Path(args.output)
 def progress(data):out.write_text(json.dumps(data,indent=2)+'\n')
 try:result=study(kind=args.kind,progress=progress)
 except (ValueError,Cancelled)as error:
  prior=json.loads(out.read_text())if out.exists()else {};prior.update(status='REJECTED',exception=str(error));progress(prior);raise
 progress(dict(status='PASS',**result));print('PASS',args.kind,'actual repeated work, refinement and rollback',flush=True)

