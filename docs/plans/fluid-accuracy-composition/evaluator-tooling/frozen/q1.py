"""Frozen native Q1 criteria; geometry/discretization independent review remains separate."""
import csv,json,math,pathlib,sys
root=pathlib.Path(sys.argv[1])
def read(name):
    with (root/name).open() as f:return list(csv.DictReader(f))
c=read('channel-summary.csv');t=read('transport-summary.csv');g=read('geometry.csv');controls=read('constant-controls.csv')
assert len(c)==6 and len(t)==18 and len(g)==9 and len(controls)==18
steady=sorted((r for r in c if r['case']=='steady'),key=lambda r:int(r['n']))
time=sorted((r for r in c if r['case']=='temporal'),key=lambda r:int(r['steps']))
orders=lambda e:[math.log(a/b,2) for a,b in zip(e,e[1:])]
spatial=orders([float(r['relative_rms']) for r in steady]);temporal=orders([float(r['rms']) for r in time])
rho=998.2;nu=.001002/rho;volume=1e-9;speed=.01*.001**2/(8*nu)
channel={'original_coarse_failure_preserved':float(steady[0]['relative_rms'])>.02 and abs(float(steady[0]['relative_rms'])-.02139313)<=5e-9,
 'resolved_velocity_pass':all(float(r['relative_rms'])<=.02 for r in steady[1:]),'spatial_orders':spatial,'spatial_order_pass':all(1.8<=p<=2.2 for p in spatial),
 'temporal_orders':temporal,'temporal_pass':all(p>=.9 for p in temporal) and all(float(b['rms'])/float(a['rms'])<.65 for a,b in zip(time,time[1:])) and all(float(r['max_energy_increase'])<=0 for r in time),
 'native_algebra_pass':all(float(r['max_energy_identity'])/(.5*rho*volume*speed**2)<=1e-10 and float(r['max_momentum_identity'])/(rho*volume*speed)<=1e-10 and float(r['max_force_residual'])/(rho*volume*.01)<=1e-10 for r in steady)}
geometry_native=all(r['exhausted']=='false' and float(r['empirical_uncertainty'])<=1e-4 and abs(float(r['volume_fine'])-4*math.pi*.15**3/3)/(4*math.pi*.15**3/3)<=1e-4 for r in g)
control_pass=all(r['status']=='complete' and float(r['max_change'])<=1e-12 for r in controls)
transport=[]
for case in ['translation','vortex-adapted']:
 for courant in [.1,.25,.5]:
  rows=sorted((r for r in t if r['case']==case and float(r['requested_courant'])==courant),key=lambda r:int(r['n']))
  assert len(rows)==3
  completed=all(r['status']=='complete' and abs(float(r['time'])-1)<=16*sys.float_info.epsilon for r in rows)
  item={'case':case,'courant':courant,'complete':completed,'statuses':[r['status'] for r in rows],
   'conservation_bounds_pass':all(float(r['relative_drift'])<=1e-10 and float(r['min'])>=-1e-12 and float(r['max'])<=1+1e-12 for r in rows),
   'actual_courant_pass':all(r['status']!='complete' or abs(float(r['max_courant'])-courant)<=16*sys.float_info.epsilon*courant for r in rows)}
  if completed:
   errors=[float(r['relative_l1']) for r in rows];u=2e-4
   lower=[math.log((a-u)/(b+u),2) if a>u and b>0 else None for a,b in zip(errors,errors[1:])]
   item.update(errors=errors,orders=orders(errors),order_lower=lower,geometry_uncertainty_budget=u,
    finest_shape_pass=errors[-1]+u<=.05,finest_order_pass=lower[-1] is not None and lower[-1]>=.9)
  else:item.update(errors=None,orders=None,finest_shape_pass=False,finest_order_pass=False)
  item['positive_accuracy_candidate']=completed and item['conservation_bounds_pass'] and item['actual_courant_pass'] and item['finest_shape_pass'] and item['finest_order_pass'] and geometry_native and (case=='translation' or control_pass)
  transport.append(item)
print(json.dumps({'channel':channel,'geometry_native_estimate_pass':geometry_native,'independent_geometry_gate':'REQUIRED before positive scientific disposition','constant_controls_pass':control_pass,'transport':transport,'general_water_or_Q2_admitted':False},indent=2,allow_nan=False))
