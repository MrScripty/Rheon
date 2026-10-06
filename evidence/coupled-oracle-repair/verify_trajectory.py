"""Strict successor to the frozen per-cell-only temporal qualification oracle."""
import copy,json,sys
from pathlib import Path
from fractions import Fraction
from functools import lru_cache
import numpy as np
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'evidence/fitted-discrete-refinement'))
import temporal as step
import trajectory
m=step.m
H=(.05,.025,.0125,.00625,.003125);DURATION=.1

def require(x,s):
 if not x:raise ValueError(s)
def keys(x,names,what):require(isinstance(x,dict)and set(names)<=set(x),'required '+what+' fields')
def array(x,shape,what):
 try:a=np.asarray(x,float)
 except (ValueError,TypeError)as e:raise ValueError('finite '+what)from e
 require(a.shape==shape and np.all(np.isfinite(a)),'finite shaped '+what);return a
def close(a,b,allow=1e-11,what='physical replay'):
 a=np.asarray(a,float);b=np.asarray(b,float);require(a.shape==b.shape and np.all(np.isfinite(a))and np.max(np.abs(a-b),initial=0)<=allow,what)
def reported(a,b,what):
 a=np.asarray(a,float);b=np.asarray(b,float);close(a,b,64*np.finfo(float).eps*max(1.,float(np.max(np.abs(b),initial=0))),'recomputed '+what)

@lru_cache(maxsize=2)
def references(kind):
 return (trajectory.reference(DURATION,1e-10,.00625,kind),trajectory.reference(DURATION,2e-12,.003125,kind))

def schema(data):
 keys(data,['status','duration','initial_kind','replays','rows','references','velocity_refinement_ratios','geometry_refinement_ratios','reference_refinement_velocity_difference','rollback','new_public_step_enabled','continuum_accuracy_claim'],'trajectory')
 require(data['status']=='PASS'and data['initial_kind']in ['initial','pressure_state'],'qualified initial field')
 require(data['duration']==DURATION,'canonical common physical endpoint')
 require(isinstance(data['replays'],list)and isinstance(data['rows'],list)and len(data['replays'])==len(data['rows'])==5,'five distinct refinement cases')
 require(isinstance(data['references'],list)and len(data['references'])==2,'two related references required')
 for h,case,row in zip(H,data['replays'],data['rows']):
  keys(case,['interval','steps'],'case');keys(row,['interval','steps','final_stamp','actual_clock','final_q','final_velocity','final_energy','mass_total','loss_sums','velocity_lumped_L2_error','geometry_max_error','max_work_allowance_ratio','max_GCL_allowance_ratio','max_momentum_rate_norm','max_quadrature_difference'],'case summary')
  require(case['interval']==row['interval']==h,'distinct ordered canonical intervals')
  count=round(DURATION/h);require(isinstance(case['steps'],list)and len(case['steps'])==count and type(row['steps'])is int and row['steps']==count,'complete case step count')
  require(Fraction(h)*count==Fraction(DURATION),'same exact real endpoint')
  for n,c in enumerate(case['steps']):
   keys(c,['step','interval','initial_q','initial_eta','unknowns','end_q','end_eta','end_pressure','energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','fixed_work_allowance','direct_momentum_rate_norm','finite_momentum_rate_norm','max_full_constraints','GCL_defect','face_sign_partition'],'physical step')
   require(type(c['step'])is int and c['step']==n+1 and c['interval']==h,'step identity and actual interval')
   for name,shape in [('initial_q',(3,)),('initial_eta',(6,)),('unknowns',(22,)),('end_q',(3,)),('end_eta',(6,)),('end_pressure',(24,)),('max_full_constraints',(3,)),('GCL_defect',(16,))]:array(c[name],shape,name)
   partition=np.asarray(c['face_sign_partition'],float);require(partition.ndim==1 and 2<=len(partition)<=66 and np.all(np.isfinite(partition))and partition[0]==0. and partition[-1]==h and np.all(np.diff(partition)>0),'bounded actual sign partition')
 return data

def verify(data):
 schema(data);kind=data['initial_kind'];fresh=references(kind)
 for supplied,actual in zip(data['references'],fresh):
  keys(supplied,list(actual),'reference')
  for name,value in actual.items():
   if name in ['calls','rtol','max_step']:require(supplied[name]==value,'related regenerated reference '+name)
   else:reported(supplied[name],value,'related regenerated reference '+name)
 refz=np.array(fresh[1]['final_z']);refx=np.array(fresh[1]['final_x']);refmass=m.spatial(refx)['M'];difference=np.array(fresh[0]['final_z'])-refz;reference_difference=float(np.sqrt(difference@refmass@difference));reported(data['reference_refinement_velocity_difference'],reference_difference,'reference gap')
 result=[];count=0
 for h,case,row in zip(H,data['replays'],data['rows']):
  q,eta=step.initial(kind);energy0=None;loss=np.zeros(3);clock=0.;max_work=max_gcl=max_rate=max_quad=0.
  for c in case['steps']:
   close(c['initial_q'],q);close(c['initial_eta'],eta);unknown=np.array(c['unknowns']);partition=step.sign_partition(q,eta,unknown,h);close(c['face_sign_partition'],partition[0],0.,'actual sign provenance')
   v=step.evaluate(q,eta,unknown,h,32,partition=partition);v16=step.evaluate(q,eta,unknown,h,16,partition=partition);step.qualify(v,h);allow=v['fixed_work_allowance'];clock+=h
   for key in ['energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','fixed_work_allowance']:close(c[key],v[key],allow)
   close(c['end_q'],v['end']['q']);close(c['end_eta'],v['end']['eta']);close(c['end_pressure'],v['end']['spatial']['Q']@unknown[6:]);close(c['GCL_defect'],v['gcl'],128*step.EPS*float(np.max(v['start']['spatial']['mass']+v['end']['spatial']['mass'])))
   close(c['direct_momentum_rate_norm'],np.linalg.norm(v['direct_residual'])/h);close(c['finite_momentum_rate_norm'],np.linalg.norm(v['residual'])/h);close(c['max_full_constraints'],v['transfers']['maxima'][:3])
   if energy0 is None:energy0=v['energy_old']
   loss+=np.array([v['backward_Euler_loss'],v['mixing_loss'],v['viscous_loss']]);max_work=max(max_work,abs(v['discrete_residual_work'])/allow);max_gcl=max(max_gcl,float(np.max(np.abs(v['gcl'])/(128*step.EPS*(v['start']['spatial']['mass']+v['end']['spatial']['mass'])))));max_rate=max(max_rate,float(np.linalg.norm(v['residual'])/h));max_quad=max(max_quad,max(float(np.max(np.abs(v['transfers'][k]-v16['transfers'][k])))for k in ['plus','minus','actual_integrated_momentum_convection']));q=v['end']['q'];eta=v['end']['eta'];count+=1
  s=m.spatial(m.q_to_x(q));J,_=m.cap_chart(s);z=J@eta;dz=z-refz;error=float(np.sqrt(dz@refmass@dz));geo=float(np.max(np.abs(m.q_to_x(q)-refx)))
  close(row['loss_sums'],loss);close(row['final_q'],q);close(row['final_velocity'],z);close(row['actual_clock'],clock,0.,'actual unclamped clock');require(type(row['final_stamp'])is int and row['final_stamp']==len(case['steps']),'common final version');close(row['final_energy'],v['energy_new']);reported(row['mass_total'],np.sum(s['mass']),'mass total')
  for name,value in [('velocity_lumped_L2_error',error),('geometry_max_error',geo),('max_work_allowance_ratio',max_work),('max_GCL_allowance_ratio',max_gcl),('max_momentum_rate_norm',max_rate),('max_quadrature_difference',max_quad)]:reported(row[name],value,name)
  require(abs(v['energy_new']-energy0+np.sum(loss))<=len(case['steps'])*allow,'cumulative separate loss ledger');result.append(dict(interval=h,steps=len(case['steps']),final_q=q.tolist(),final_velocity=z.tolist(),velocity_lumped_L2_error=error,geometry_max_error=geo,actual_clock=clock,final_stamp=len(case['steps'])))
 velocity_ratios=[a['velocity_lumped_L2_error']/b['velocity_lumped_L2_error']for a,b in zip(result,result[1:])];geometry_ratios=[a['geometry_max_error']/b['geometry_max_error']for a,b in zip(result,result[1:])]
 reported(data['velocity_refinement_ratios'],velocity_ratios,'velocity ratios');reported(data['geometry_refinement_ratios'],geometry_ratios,'geometry ratios')
 require(reference_difference<min(x['velocity_lumped_L2_error']for x in result)/1000,'reference resolves measured errors');require(all(1.8<r<2.2 for r in velocity_ratios)and all(1.7<r<2.3 for r in geometry_ratios),'recomputed original convergence bands')
 require(data['new_public_step_enabled']is False and data['continuum_accuracy_claim']is False,'host-only measured accuracy');keys(data['rollback'],['checks','successful_continuation','invalid_interval_failures'],'rollback');require(len(data['rollback']['checks'])==8 and all(c['accepted_state_unchanged']for c in data['rollback']['checks'])and data['rollback']['successful_continuation']is True and data['rollback']['invalid_interval_failures']==4,'recorded rollback evidence')
 return dict(status='PASS',actual_steps=count,kind=kind,duration=DURATION,rows=result,references=list(fresh),velocity_refinement_ratios=velocity_ratios,geometry_refinement_ratios=geometry_ratios,reference_refinement_velocity_difference=reference_difference)

if __name__=='__main__':print(json.dumps(verify(json.load(open(sys.argv[1]))),indent=2))
