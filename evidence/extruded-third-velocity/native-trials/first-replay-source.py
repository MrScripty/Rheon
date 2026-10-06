"""Strict full published-state replay and regenerated common-endpoint references.

Only native publications are input. Convergence summaries are computed here,
never accepted as evidence from the producer. Every physical field is mandatory.
"""
import copy,importlib.util,json,sys
from functools import lru_cache
from pathlib import Path
import numpy as np
import reference as r
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('strict_oracle',P.parent/'coupled-oracle-repair/verify_trajectory.py');strict=importlib.util.module_from_spec(spec);spec.loader.exec_module(strict)
t=r.temporal;m=r.m;EPS=t.EPS;LIMIT=1e-11;BUDGET=470992
FIELDS=['nonconstant','constant'];KINDS=['initial','pressure_state']
REPORT=['energy_old','energy_new','backward_Euler_loss','mixing_loss','viscous_loss','GCL_work','discrete_residual_work','energy_ledger_error','fixed_work_allowance']
PLANAR=REPORT+['pressure_work','finite_momentum_rate_norm','direct_momentum_rate_norm','gcl_max','quadrature_error','full_constraints','iterations','equation_evaluations','sign_roots','endpoint_vs_path_momentum_max']
THIRD=REPORT+['coefficients','momentum_before','momentum_after','shear_x_loss','shear_y_loss','finite_momentum_rate_norm','direct_momentum_rate_norm']
def require(ok,s):r.require(ok,s)
def schema(rows):
 require(isinstance(rows,list)and len(rows)==268,'all twenty constructors and 248 accepted steps')
 groups={}
 for row in rows:
  strict.keys(row,['model','kind','field','h','step','stamp','unknowns','end_q','end_eta','third_coefficients','velocity','pressure_coefficients','time','positions','triangles','mass','physical_pressure','report','allocated_bytes'],'full publication')
  require(row['model']=='periodic-z-invariant'and row['kind']in KINDS and row['field']in FIELDS and row['h']in r.H,'declared canonical model/case')
  strict.keys(row['stamp'],['id','version'],'actual state identity');require(type(row['step'])is int and type(row['stamp']['id'])is int and row['stamp']['id']==91 and type(row['stamp']['version'])is int and row['step']==row['stamp']['version'],'actual exported stamp')
  require(type(row['allocated_bytes'])is int and row['allocated_bytes']==BUDGET,'actual bounded owner budget')
  for name,shape in [('end_q',(3,)),('end_eta',(6,)),('third_coefficients',(12,)),('velocity',(16,3)),('pressure_coefficients',(16,)),('time',()),('positions',(19,2)),('triangles',(24,3)),('mass',(16,)),('physical_pressure',(24,))]:strict.array(row[name],shape,name)
  if row['step']==0:require(row['unknowns']is None and row['report']is None,'constructor has no finite-step report')
  else:
   strict.array(row['unknowns'],(22,),'finite xy unknowns');strict.keys(row['report'],['planar','third','total'],'full work report')
   for part,keys in [('planar',PLANAR),('third',THIRD),('total',REPORT)]:
    strict.keys(row['report'][part],keys,part+' report')
    for key in keys:strict.array(row['report'][part][key],(12,)if key=='coefficients'else(),part+' '+key)
   for key,maximum in [('iterations',7),('equation_evaluations',200),('sign_roots',64)]:require(type(row['report']['planar'][key])is int and 0<=row['report']['planar'][key]<=maximum,'bounded '+key)
  groups.setdefault((row['kind'],row['field'],row['h']),[]).append(row)
 require(list(groups)==[(k,f,h)for k in KINDS for f in FIELDS for h in r.H],'distinct ordered complete canonical refinements')
 for(k,f,h),case in groups.items():require([s['step']for s in case]==list(range(round(.1/h)+1)),'actual constructor/step sequence')
 return groups
def coeff(u):
 z=np.zeros(22)
 for j in range(22):
  for i in range(16):
   for d in range(2):
    if np.array_equal(m.R_NODE[i,d],np.eye(22)[j]):z[j]=u[i,d];break
   else:continue
   break
 return z
def check_geometry(row,s,pressure):
 require(np.max(np.abs(np.array(row['positions'])-s['pos']))<=1e-14,'actual PS geometry')
 require(np.array_equal(np.array(row['triangles']),m.TRI),'actual topology')
 require(np.max(np.abs(np.array(row['mass'])-s['mass']))<=1e-14,'actual physical masses')
 require(np.max(np.abs(np.array(row['physical_pressure'])-s['Q']@pressure))<=1e-11,'actual physical pressure')
@lru_cache(maxsize=128)
def cell(q,eta,unknown,h):
 q=np.array(q);eta=np.array(eta);unknown=np.array(unknown);partition=t.sign_partition(q,eta,unknown,h)
 return t.evaluate(q,eta,unknown,h,32,partition=partition),t.evaluate(q,eta,unknown,h,16,partition=partition)
def work(old,new,m0,m1,gcl,plus,minus,visc,pressure,residual):
 e0=.5*np.sum(m0[:,None]*old*old);e1=.5*np.sum(m1[:,None]*new*new);be=.5*np.sum(m0[:,None]*(new-old)**2)
 mix=.5*np.sum((plus+minus)*np.sum((new[m.PAIR_I]-new[m.PAIR_J])**2,axis=1));wg=.5*np.sum(gcl*np.sum(new*new,axis=1));ledger=e1-e0+be+mix+visc+pressure+wg-residual
 allow=128*EPS*(e0+e1+be+mix+abs(visc)+abs(pressure)+abs(wg)+abs(residual))
 require(min(be,mix,visc)>=0 and max(abs(residual),abs(ledger),abs(pressure))<=allow,'unchanged separate/full work gates')
 return dict(energy_old=float(e0),energy_new=float(e1),backward_Euler_loss=float(be),mixing_loss=float(mix),viscous_loss=float(visc),GCL_work=float(wg),discrete_residual_work=float(residual),energy_ledger_error=float(ledger),fixed_work_allowance=float(allow))
def report_matches(reported,actual,allow,part):
 for key,value in actual.items():require(abs(reported[key]-value)<=allow,'actual reported '+part+' '+key)
def replay(rows,references=True):
 groups=schema(rows);endpoints=[];maxima=dict(full_momentum=0.,direct_full_momentum=0.,third_momentum_change=0.,third_work_ratio=0.,total_work_ratio=0.,GCL_ratio=0.,quadrature=0.,constant_deviation=0.)
 for(kind,field,h),case in groups.items():
  q,eta=t.initial(kind);start=t.point(q,eta,t.seed(q,eta),0.);s=start['spatial'];old=np.array(case[0]['velocity']);mass=np.array(case[0]['mass']);xi=np.array(case[0]['third_coefficients']);clock=0.
  require(np.max(np.abs(old[:,:2]-start['flux']['velocity'][:,:2]))<=1e-11,'actual constructor xy data')
  require(np.array_equal(xi,r.initial_xi(field))and np.array_equal(old[:,2],r.RW@xi),'actual constructor third data')
  require(case[0]['time']==0. and np.max(np.abs(np.array(case[0]['pressure_coefficients'])-start['pressure']))<=1e-11,'actual instantaneous constructor pressure/clock')
  check_geometry(case[0],s,np.array(case[0]['pressure_coefficients']))
  for row in case[1:]:
   unknown=np.array(row['unknowns']);v,v16=cell(tuple(q),tuple(eta),tuple(unknown),h);end=v['end'];s=end['spatial'];new=np.array(row['velocity']);newmass=np.array(row['mass']);newxi=np.array(row['third_coefficients']);p=np.array(row['pressure_coefficients'])
   require(np.max(np.abs(new[:,:2]-end['flux']['velocity'][:,:2]))<=1e-11,'actual accepted xy chart')
   require(np.max(np.abs(new[:,2]-r.RW@newxi))<=1e-14,'actual accepted third embedding')
   require(np.array_equal(newxi,np.array(row['report']['third']['coefficients'])),'third report belongs to publication')
   require(np.array_equal(p,unknown[6:]),'pressure report belongs to publication')
   require(np.max(np.abs(np.array(row['end_q'])-end['q']))<=1e-11 and np.max(np.abs(np.array(row['end_eta'])-end['eta']))<=1e-11,'actual accepted geometry coordinates')
   check_geometry(row,s,p);clock+=h;require(row['time']==clock,'actual unclamped common clock')
   plus=v['transfers']['plus'];minus=v['transfers']['minus'];z0=coeff(old);z1=coeff(new)
   require(np.max(np.abs(s['D']@z1))<=LIMIT and np.all(v['transfers']['maxima'][:3]<=LIMIT),'all full path divergence/acceleration/material rows')
   gcl=newmass-mass;np.add.at(gcl,m.PAIR_I,plus-minus);np.add.at(gcl,m.PAIR_J,minus-plus);gr=float(np.max(np.abs(gcl)/(128*EPS*(mass+newmass))));require(gr<=1,'actual integrated physical local GCL')
   conv=t.route(plus,minus,new);force=s['K']@z1+s['B'].T@p
   xy=m.R_NODE;inertia=np.einsum('ndj,nd->j',xy,newmass[:,None]*np.einsum('ndj,j->nd',xy,z1-z0)+(newmass-mass)[:,None]*old)
   direct=np.einsum('ndj,nd->j',xy,newmass[:,None]*new-mass[:,None]*old)
   rp=inertia+conv+h*force;dp=direct+conv+h*force
   mw,kx,ky,_,_=r.blocks(s);cw=r.route(plus,minus,new[:,2]);kw=(kx+ky)@newxi
   rw=r.RW.T@(newmass*(r.RW@(newxi-xi))+(newmass-mass)*old[:,2]+cw)+h*kw
   dw=r.RW.T@(newmass*new[:,2]-mass*old[:,2]+cw)+h*kw
   rate=float(np.linalg.norm(np.r_[rp,rw])/h);drate=float(np.linalg.norm(np.r_[dp,dw])/h);require(max(rate,drate)<=LIMIT,'all 34 actual accepted momentum equations')
   require(np.max(np.abs(newxi-r.finite(v,h,old[:,2],mass)))<=1e-11,'independent actual third finite solve')
   mom0=float(mass@old[:,2]);mom1=float(newmass@new[:,2]);require(abs(mom1-mom0)<=LIMIT,'actual third momentum conservation')
   sx=float(h*newxi@kx@newxi);sy=float(h*newxi@ky@newxi);viscw=sx+sy;viscp=float(h*z1@s['K']@z1);wp=float(h*z1@s['B'].T@p);wrp=float(z1@rp);wrw=float(newxi@rw)
   planar=work(old[:,:2],new[:,:2],mass,newmass,gcl,plus,minus,viscp,wp,wrp)
   third=work(old[:,2,None],new[:,2,None],mass,newmass,gcl,plus,minus,viscw,0.,wrw)
   total=work(old,new,mass,newmass,gcl,plus,minus,viscp+viscw,wp,wrp+wrw)
   for part,actual in [('planar',planar),('third',third),('total',total)]:report_matches(row['report'][part],actual,actual['fixed_work_allowance'],part)
   report_matches(row['report']['third'],dict(momentum_before=mom0,momentum_after=mom1,shear_x_loss=sx,shear_y_loss=sy),third['fixed_work_allowance'],'third')
   require(row['report']['planar']['finite_momentum_rate_norm']<=1e-13 and row['report']['third']['finite_momentum_rate_norm']<=LIMIT and row['report']['third']['direct_momentum_rate_norm']<=LIMIT,'unchanged native solve limits')
   quad=max(np.max(np.abs(v['transfers'][key]-v16['transfers'][key]))for key in ['plus','minus','actual_integrated_momentum_convection']);require(quad<=1e-15,'actual physical 16/32 refinement')
   if field=='constant':
    deviation=float(np.max(np.abs(new[:,2]-.25)));require(deviation<=128*EPS*.25,'constant third preservation');maxima['constant_deviation']=max(maxima['constant_deviation'],deviation)
   else:require(sx>0 and sy>0,'both actual third shears exercised')
   maxima['full_momentum']=max(maxima['full_momentum'],rate);maxima['direct_full_momentum']=max(maxima['direct_full_momentum'],drate);maxima['third_momentum_change']=max(maxima['third_momentum_change'],abs(mom1-mom0));maxima['third_work_ratio']=max(maxima['third_work_ratio'],abs(wrw)/third['fixed_work_allowance']);maxima['total_work_ratio']=max(maxima['total_work_ratio'],abs(wrp+wrw)/total['fixed_work_allowance']);maxima['GCL_ratio']=max(maxima['GCL_ratio'],gr);maxima['quadrature']=max(maxima['quadrature'],float(quad))
   q=np.array(row['end_q']);eta=np.array(row['end_eta']);old=new;mass=newmass;xi=newxi
  endpoints.append(dict(kind=kind,field=field,interval=h,steps=len(case)-1,final_q=q.tolist(),final_xi=xi.tolist(),final_velocity=old.tolist(),actual_clock=clock,actual_stamp=case[-1]['stamp']))
 output=dict(status='PASS',scope='actual published full velocity on the reviewed doubly periodic z-invariant donor/strain model; conditional temporal evidence, no general 3D or continuum claim',actual_constructors=20,actual_steps=248,maxima=maxima,rows=endpoints)
 if references:
  refs=[]
  for kind in KINDS:
   for field in FIELDS:
    coarse=r.reference(kind,field,1e-10,.00625);fine=r.reference(kind,field,2e-12,.003125);refs.extend([coarse,fine]);mass=m.spatial(np.array(fine['final_x']))['mass'];vf=np.array(fine['final_velocity']);gap=float(np.sqrt(np.sum(mass[:,None]*(np.array(coarse['final_velocity'])-vf)**2)));cases=[x for x in endpoints if x['kind']==kind and x['field']==field]
    for case in cases:
     vel=np.array(case['final_velocity']);case['full_velocity_lumped_L2_error']=float(np.sqrt(np.sum(mass[:,None]*(vel-vf)**2)));case['third_velocity_lumped_L2_error']=float(np.sqrt(np.sum(mass*(vel[:,2]-vf[:,2])**2)));case['geometry_max_error']=float(np.max(np.abs(m.q_to_x(np.array(case['final_q']))-fine['final_x'])));case['reference_refinement_full_velocity_gap']=gap
    require(gap<min(c['full_velocity_lumped_L2_error']for c in cases)/1000,'independent reference refinement resolves full error')
    for name,lo,hi in [('full_velocity_lumped_L2_error',1.8,2.2),('geometry_max_error',1.7,2.3)]+([('third_velocity_lumped_L2_error',1.8,2.2)]if field=='nonconstant'else[]):
     ratios=[a[name]/b[name]for a,b in zip(cases,cases[1:])];require(all(lo<x<hi for x in ratios),'independently expected original-band first order '+kind+' '+field+' '+name)
     for case,ratio in zip(cases,ratios):case[name+'_refinement_ratio']=ratio
    print('qualified independent full reference',kind,field,file=sys.stderr,flush=True)
  output['references']=refs
 return output
def negative_controls(rows):
 controls=[]
 for name in ['missing_mass','missing_geometry','missing_pressure','missing_third','missing_total_report','false_stamp','false_clock','false_third_velocity','false_third_coefficients','missing_second_shear','false_total_energy','duplicated_coarse_case']:
  bad=copy.deepcopy(rows);row=bad[1]
  if name.startswith('missing_')and name not in ['missing_second_shear','missing_total_report']:del row[{'missing_mass':'mass','missing_geometry':'positions','missing_pressure':'physical_pressure','missing_third':'third_coefficients'}[name]]
  elif name=='missing_total_report':del row['report']['total']
  elif name=='false_stamp':row['stamp']['version']=999
  elif name=='false_clock':row['time']=999.
  elif name=='false_third_velocity':row['velocity'][0][2]+=.01
  elif name=='false_third_coefficients':row['third_coefficients'][0]+=.01
  elif name=='missing_second_shear':del row['report']['third']['shear_y_loss']
  elif name=='false_total_energy':row['report']['total']['energy_new']+=.01
  else:bad[3:8]=copy.deepcopy(bad[:5])
  try:replay(bad,references=False)
  except ValueError as error:controls.append(dict(control=name,status='REJECTED',reason=str(error)))
  else:raise ValueError('corruption accepted '+name)
 return controls
if __name__=='__main__':
 rows=[json.loads(line)for line in Path(sys.argv[1]).read_text().splitlines()];out=replay(rows)
 if '--negative-self-test'in sys.argv:out['actual_corruption_rejections']=negative_controls(rows)
 print(json.dumps(out,indent=2))
