"""Frozen Q2 geometry-only gates; no coupled PDE acceptance."""
import csv,json,math,pathlib,sys
root=pathlib.Path(sys.argv[1])
rows=list(csv.DictReader((root/'summary.csv').open()))
assert len(rows)==24
results=[]
for row in rows:
 r={k:(v if k=='case' else float(v)) for k,v in row.items()}
 a=r['area']; exact=r['exact_area']; mu=r['mu']
 defects={k:r[k] for k in ['normal_closure','linear_closure','cubic_defect','solenoidal_defect','traction_defect','face_mismatch','dual_relative_defect']}
 geometry=[abs(a-exact)/exact,abs(r['mx']-r['exact_mx'])/exact,abs(r['my']-r['exact_my'])/exact,abs(r['radial_second']-r['exact_radial_second'])/exact]
 geometry.extend(abs(r[k]-r['exact_'+k])/exact for k in ['mxx','mxy','myy'])
 control=[abs((r['wrong_force_x']-r['force_x'])-mu*a)/(mu*a),abs((r['wrong_force_y']-r['force_y'])+3*mu*a)/(mu*a)]
 item={'case':r['case'],'n':int(r['n']),'mu':mu,'local_geometry_pass':all(math.isfinite(v) and v<=1e-10 for v in defects.values()),'defects':defects,'min_positive_volume':r['min_positive_volume'],'negative_pressure_control_pass':max(control)<=1e-10,'area_error':geometry[0],'radial_error':abs(r['radial_second']/r['exact_radial_second']-1),'first_second_moment_scaled_errors':geometry[1:],'flux_error':abs(r['flux']/r['exact_flux']-1),'force_errors':[abs(r['force_x']/r['exact_force']-1),abs(r['force_y']/r['exact_force']-1)],'global_geometry_pass':max(geometry[1:3]+[geometry[5]])<=1e-10 if r['case']=='disk' else max(geometry)<=1e-10}
 results.append(item)
convergence=[]
for mu in [.1,.001002]:
 group=sorted([r for r in results if r['case']=='disk' and r['mu']==mu],key=lambda r:r['n'])
 assert len(group)==3
 for metric in ['area_error','radial_error','flux_error']:
  errors=[r[metric] for r in group]
  p=[math.log(a/b,2) for a,b in zip(errors,errors[1:])]
  convergence.append(dict(mu=mu,metric=metric,errors=errors,orders=p,pass_gate=errors[-1]<=1e-4 and all(1.8<=v<=2.2 for v in p)))
 for d in range(2):
  errors=[r['force_errors'][d] for r in group];p=[math.log(a/b,2) for a,b in zip(errors,errors[1:])]
  convergence.append(dict(mu=mu,metric=f'force_{d}',errors=errors,orders=p,pass_gate=errors[-1]<=1e-4 and all(1.8<=v<=2.2 for v in p)))
positive=all(r['local_geometry_pass'] and r['negative_pressure_control_pass'] and r['global_geometry_pass'] for r in results) and all(r['pass_gate'] for r in convergence)
print(json.dumps(dict(native_geometry_candidate=positive,independent_cellwise_geometry_gate='REQUIRED',results=results,convergence=convergence,coupled_stokes_mms_or_production_water_admitted=False),indent=2,allow_nan=False))
