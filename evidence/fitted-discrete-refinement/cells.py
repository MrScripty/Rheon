"""Refined tangent and endpoint donor coefficient diagnostics; coarse flag retained."""
import json
import numpy as np
import temporal as step
from exact import instantaneous
from rate import leading

def study():
 rows=[]
 for kind in ['initial','pressure_state']:
  exact=instantaneous(kind);true=np.r_[step.m.C_CAP@np.array(exact['acceleration'],float),np.array(exact['acceleration'],float)[step.m.SELECT],np.array(exact['pressure'],float)]
  cells=[step.solve(h,kind)for h in [.05,.025,.0125,.00625,.003125]]
  errors=[float(np.linalg.norm(np.array(c['unknowns'])-true))for c in cells];ratios=[errors[i]/errors[i+1]for i in range(4)]
  step.m.require(all(1.7<r<2.3 for r in ratios),'unchanged tangent first-order diagnostic')
  face_ratios=[cells[i]['endpoint_vs_actual_momentum_max']/cells[i+1]['endpoint_vs_actual_momentum_max']for i in range(4)];expected=8 if kind=='initial'else 4
  coarse=all(.85*expected<r<1.15*expected for r in face_ratios[:2]);fine=all(.85*expected<r<1.15*expected for r in face_ratios[-2:]);step.m.require(fine,'same band on genuinely refined flux differences')
  L=np.array(leading(kind),float);distance=[]
  for c in cells:
   delta=np.array(c['endpoint_donor_momentum_convection'])-c['actual_integrated_physical_momentum_convection'];distance.append(float(np.max(np.abs(delta/c['interval']**2-L))))
  distance_ratios=[distance[i]/distance[i+1]for i in range(4)];step.m.require(all(1.7<r<2.3 for r in distance_ratios),'first-order approach to exact quadratic coefficient')
  rows.append(dict(kind=kind,cells=cells,tangent_coefficient_errors=errors,tangent_first_order_ratios=ratios,momentum_flux_difference_ratios=face_ratios,coarse_diagnostic_pass=coarse,refined_diagnostic_pass=fine,exact_quadratic_coefficient=[str(x)for x in leading(kind)],scaled_flux_coefficient_error=distance,scaled_coefficient_error_ratios=distance_ratios))
 step.m.require(rows[1]['coarse_diagnostic_pass']is False,'frozen coarse pressure-state diagnostic failure retained')
 return dict(scope='same first-order finite equations; improved arithmetic and explicit pre-asymptotic coefficient diagnostics',rows=rows,new_public_step_enabled=False)

if __name__=='__main__':print(json.dumps(study(),indent=2))
