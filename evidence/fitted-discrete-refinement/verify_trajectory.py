"""Replay every recorded finite step, endpoint pressure and complete discrete work."""
import copy
import json
import sys
import numpy as np
import temporal as step
m=step.m

def close(a,b,allow=1e-11):
 a=np.asarray(a,float);b=np.asarray(b,float)
 m.require(a.shape==b.shape and np.all(np.isfinite(a))and np.max(np.abs(a-b),initial=0)<=allow,'actual trajectory replay')


def verify(data):
 count=0
 m.require(data['status']=='PASS'and len(data['replays'])==len(data['rows'])==5,'complete five-case temporal qualification')
 for case,row in zip(data['replays'],data['rows']):
  q,eta=step.initial(data['initial_kind']);energy0=None;loss=np.zeros(3);clock=0.
  for n,c in enumerate(case['steps']):
   close(c['initial_q'],q);close(c['initial_eta'],eta);m.require(c['step']==n+1,'actual stamp sequence')
   h=c['interval'];m.require(h==case['interval']and 0<h<=.05,'same clock interval')
   v=step.evaluate(q,eta,np.array(c['unknowns']),h,32);step.qualify(v,h);allow=v['fixed_work_allowance'];clock+=h
   for key in ['energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','fixed_work_allowance']:close(c[key],v[key],allow)
   close(c['end_q'],v['end']['q']);close(c['end_eta'],v['end']['eta']);close(c['end_pressure'],v['end']['spatial']['Q']@np.array(c['unknowns'][6:]))
   if energy0 is None:energy0=v['energy_old']
   loss+=np.array([v['backward_Euler_loss'],v['mixing_loss'],v['viscous_loss']]);q=v['end']['q'];eta=v['end']['eta'];count+=1
  close(row['loss_sums'],loss);close(row['final_q'],q);close(row['actual_clock'],clock,0.);m.require(row['final_stamp']==len(case['steps']),'same geometry/time stamp');close(row['final_energy'],v['energy_new'])
  m.require(abs(v['energy_new']-energy0+np.sum(loss))<=len(case['steps'])*allow,'cumulative separate loss ledger')
 m.require(data['new_public_step_enabled']is False and data['continuum_accuracy_claim']is False,'host-only measured accuracy')
 m.require(len(data['rollback']['checks'])==8 and all(c['accepted_state_unchanged']for c in data['rollback']['checks']),'recorded real cancellation/failure checks')
 return count

if __name__=='__main__':
 data=json.load(open(sys.argv[1]));print('actual replayed steps',verify(data))
 for name in ['stale_geometry','stale_pressure','fake_clock','zero_mixing']:
  bad=copy.deepcopy(data)
  if name=='stale_geometry':bad['replays'][0]['steps'][0]['end_q'][0]=0.
  elif name=='stale_pressure':bad['replays'][0]['steps'][0]['end_pressure']=[0.]*24
  elif name=='fake_clock':bad['rows'][0]['actual_clock']=0.
  else:bad['replays'][0]['steps'][0]['mixing_loss']=0.
  try:verify(bad)
  except ValueError:print('REJECTED',name)
  else:raise ValueError('trajectory corruption accepted '+name)
 print('PASS complete recorded finite work/pressure/geometry/clock replay')
