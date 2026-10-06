"""Bounded quadratic material-cap cell; all xy momentum plus separate work gate."""
import json
import numpy as np
from scipy.optimize import root
import model as m
from exact import instantaneous

EPS=np.finfo(float).eps
MAX_EVALUATIONS=200


def initial(kind='initial'):
 z=np.array(instantaneous(kind)['z'],float)
 return np.array([0.,.5,1.]),np.r_[m.C_CAP@z,z[m.SELECT]]


def quadrature(q0,eta0,unknown,h,order,diagnostics=False):
 alpha,p=unknown[:6],unknown[6:]
 nodes,weights=np.polynomial.legendre.leggauss(order)
 momentum=np.zeros(22);convection_integral=np.zeros(22);Fpos=np.zeros(len(m.PAIR));Fneg=np.zeros(len(m.PAIR))
 values=np.zeros(5);points=[]
 max_strong=0.;max_acceleration=0.;max_material=0.;max_point_residual=0.;max_point_norm=0.;min_mass=float('inf');min_area=float('inf')
 for node,weight in zip(nodes,weights):
  time=h*(1+node)/2;w=h*weight/2
  v=m.cell_point(q0,eta0,alpha,time,p);s=v['spatial'];f=v['flux'];z=v['z']
  force=np.einsum('ndj,nd->j',m.R_NODE,f['convection'])+s['K']@z+s['B'].T@p
  momentum+=w*force
  convection_integral+=w*np.einsum('ndj,nd->j',m.R_NODE,f['convection'])
  Fpos+=w*np.maximum(f['flux'],0.);Fneg+=w*np.maximum(-f['flux'],0.)
  gcl=s['mdot']+f['div_flux'];gcl_work=.5*np.dot(gcl,np.sum(f['velocity']**2,axis=1))
  values+=w*np.array([f['dissipation'],z@s['K']@z,z@s['B'].T@p,z@v['momentum_residual'],gcl_work])
  max_strong=max(max_strong,float(np.max(np.abs(s['D']@z))))
  max_acceleration=max(max_acceleration,float(np.max(np.abs(v['strong_acceleration_residual']))))
  max_material=max(max_material,float(np.max(np.abs(v['material_cap_residual']))))
  max_point_residual=max(max_point_residual,float(np.max(np.abs(v['momentum_residual']))))
  max_point_norm=max(max_point_norm,float(np.linalg.norm(v['momentum_residual'])))
  min_mass=min(min_mass,float(np.min(s['mass'])));min_area=min(min_area,float(np.min(s['area'])))
  if diagnostics:
   points.append(dict(time=float(time),q=v['q'].tolist(),velocity=z.tolist(),pressure=p.tolist(),
                     reconstructed_pressure=(s['Q']@p).tolist(),strong_divergence=(s['D']@z).tolist(),
                     full_acceleration_residual=v['strong_acceleration_residual'].tolist(),
                     momentum_residual=v['momentum_residual'].tolist(),material_cap_residual=v['material_cap_residual'].tolist()))
 start=m.cell_point(q0,eta0,alpha,0.,p);end=m.cell_point(q0,eta0,alpha,h,p)
 res=end['spatial']['M']@end['z']-start['spatial']['M']@start['z']+momentum
 gcl=end['spatial']['mass']-start['spatial']['mass']
 np.add.at(gcl,m.PAIR_I,Fpos-Fneg);np.add.at(gcl,m.PAIR_J,-Fpos+Fneg)
 energy0=.5*start['z']@start['spatial']['M']@start['z'];energy1=.5*end['z']@end['spatial']['M']@end['z']
 adv,strain,pwork,rwork,gwork=values
 ledger=energy1-energy0+adv+strain+pwork-rwork+gwork
 allowance=128*EPS*(energy0+energy1+abs(adv)+abs(strain)+abs(pwork)+abs(rwork)+abs(gwork))
 return dict(residual=res,convection_integral=convection_integral,integrals=values,positive=Fpos,negative=Fneg,gcl=gcl,start=start,end=end,
  energy0=energy0,energy1=energy1,energy_ledger_error=float(ledger),energy_allowance=float(allowance),
  max_strong_divergence=max_strong,max_strong_acceleration=max_acceleration,max_material_mismatch=max_material,
  max_pointwise_momentum_residual=max_point_residual,max_pointwise_momentum_norm=max_point_norm,min_mass=min_mass,min_microarea=min_area,points=points)


def solve_cell(h,kind='initial',guess=None):
 q0,eta0=initial(kind)
 exact=instantaneous(kind);acc=np.array(exact['acceleration'],float)
 seed=np.r_[m.C_CAP@acc,acc[m.SELECT],np.array(exact['pressure'],float)]if guess is None else guess
 calls=0
 def equation(unknown):
  nonlocal calls
  calls+=1
  m.require(calls<=MAX_EVALUATIONS,'bounded nonlinear evaluations')
  return quadrature(q0,eta0,unknown,h,16)['residual']/h
 answer=root(equation,seed,method='hybr',options={'xtol':1e-11,'maxfev':MAX_EVALUATIONS})
 v16=quadrature(q0,eta0,answer.x,h,16);v32=quadrature(q0,eta0,answer.x,h,32,True)
 m.require(np.linalg.norm(v32['residual'])<=1e-11,'all 22 integrated momentum equations')
 m.require(v32['max_strong_divergence']<=1e-11 and v32['max_strong_acceleration']<=1e-11,'full velocity/acceleration path constraints')
 m.require(v32['max_material_mismatch']<=1e-11,'actual quadratic cap kinematics')
 qerror=max(np.max(np.abs(v32[name]-v16[name]))for name in ['positive','negative','integrals','residual'])
 gallow=128*EPS*(v32['start']['spatial']['mass']+v32['end']['spatial']['mass'])
 m.require(np.all(np.abs(v32['gcl'])<=gallow),'actual finite local physical GCL')
 m.require(qerror<=1e-15,'physical flux / work quadrature refinement')
 m.require(abs(v32['energy_ledger_error'])<=v32['energy_allowance'],'complete integrated energy ledger')
 work_pass=abs(v32['integrals'][3])<=v32['energy_allowance']
 rows=[]
 for t in np.linspace(0,h,5):
  r=m.cell_point(q0,eta0,answer.x[:6],t,answer.x[6:]);s=r['spatial']
  rows.append(dict(time=float(t),q=r['q'].tolist(),eta=r['eta'].tolist(),nodes=s['pos'].tolist(),
    velocity=r['flux']['velocity'].tolist(),reduced_velocity=r['z'].tolist(),mass=s['mass'].tolist(),
    reconstructed_pressure=(s['Q']@answer.x[6:]).tolist(),divergence=(s['D']@r['z']).tolist(),
    pointwise_momentum_residual=r['momentum_residual'].tolist()))
 return dict(scope='quadratic cap with full divergence and integrated conservative momentum; separate fixed work gate',
    interval=h,initial_kind=kind,unknowns=answer.x.tolist(),nonlinear_evaluations=calls,solver_success=bool(answer.success),solver_message=answer.message,
    integrated_momentum_residual=v32['residual'].tolist(),
    integrated_convection=v32['convection_integral'].tolist(),
    positive_negative_face_integrals=[[i,j,float(v32['positive'][k]),float(v32['negative'][k])]for k,(i,j)in enumerate(m.PAIR)],max_integrated_momentum_residual=float(np.max(np.abs(v32['residual']))),
    integrated_momentum_norm=float(np.linalg.norm(v32['residual'])),
    max_strong_divergence=v32['max_strong_divergence'],max_strong_acceleration=v32['max_strong_acceleration'],max_material_mismatch=v32['max_material_mismatch'],
    max_pointwise_momentum_residual=v32['max_pointwise_momentum_residual'],max_pointwise_momentum_norm=v32['max_pointwise_momentum_norm'],pointwise_momentum_gate_pass=v32['max_pointwise_momentum_norm']<=1e-11,
    actual_local_GCL_defect=v32['gcl'].tolist(),max_actual_local_GCL_defect=float(np.max(np.abs(v32['gcl']))),
    local_GCL_allowance=gallow.tolist(),quadrature_16_32_difference=float(qerror),
    energy_old=float(v32['energy0']),energy_new=float(v32['energy1']),advection_loss=float(v32['integrals'][0]),strain_loss=float(v32['integrals'][1]),
    pressure_work=float(v32['integrals'][2]),actual_momentum_residual_work=float(v32['integrals'][3]),GCL_defect_work=float(v32['integrals'][4]),
    energy_ledger_error=v32['energy_ledger_error'],fixed_work_allowance=v32['energy_allowance'],work_gate_pass=bool(work_pass),
    candidate_accepted=bool(work_pass and answer.success and v32['max_pointwise_momentum_norm']<=1e-11),new_public_step_enabled=False,quadrature_points=v32['points'],samples=rows)


if __name__=='__main__':
 result=solve_cell(.05)
 print(json.dumps(result,indent=2))
