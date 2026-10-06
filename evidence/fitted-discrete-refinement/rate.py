"""Exact leading endpoint-varying-donor error coefficient, retained coarse failure."""
from fractions import Fraction as Q
import json
from pathlib import Path
import temporal as t
from exact import instantaneous
b=t.m.base

def leading(kind):
 exact=instantaneous(kind);acc=exact['acceleration'];raw=b.mv(t.m.INITIAL['R'],acc);un=[[Q(0)]*3 for _ in range(16)]
 for raw_index,node in enumerate(t.m.INITIAL['ids']):un[node]=raw[3*raw_index:3*raw_index+3]
 route=b.zeros(16,3)
 for (i,j),f in exact['physical_flux'].items():
  donor=un[i]if f>0 else un[j]
  for d in range(3):route[i][d]+=f*donor[d]/2;route[j][d]-=f*donor[d]/2
 Rn=b.zeros(48,22)
 for raw_index,node in enumerate(t.m.INITIAL['ids']):
  for d in range(3):Rn[3*node+d]=t.m.INITIAL['R'][3*raw_index+d]
 L=b.mv(t.m.old.transpose(Rn),[v for row in route for v in row])
 if kind=='initial':t.m.require(L==[Q(0)]*22,'zero initial physical relative flux gives zero quadratic coefficient')
 return L

def summary():
 base=json.loads((Path(__file__).resolve().parents[1]/'fitted-discrete-work/trials/final-cells.json').read_text());rows=[]
 for row in base['rows']:
  L=leading(row['kind']);samples=[]
  for cell in row['cells']:
   delta=[a-b for a,b in zip(cell['endpoint_donor_momentum_convection'],cell['actual_integrated_physical_momentum_convection'])];h=cell['interval']
   samples.append(dict(interval=h,delta_over_h_squared=[x/(h*h)for x in delta],distance_to_exact_leading_max=max(abs(x/(h*h)-float(y))for x,y in zip(delta,L))))
  rows.append(dict(kind=row['kind'],exact_quadratic_coefficient=[str(x)for x in L],coefficient_max=float(max(map(abs,L))),frozen_coarse_ratio_diagnostic=row['momentum_flux_difference_order_diagnostic_pass'],frozen_ratios=row['momentum_flux_difference_ratios'],samples=samples))
 return dict(derivation='C_endpoint-C_path = h^2 R^T route(f(0), Udot(0))/2 + O(h^3) on smooth stable-donor faces; initial f(0)=0 gives zero coefficient',rows=rows,physical_gate_changes=0)

if __name__=='__main__':print(json.dumps(summary(),indent=2))
