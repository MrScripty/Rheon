"""Supplement frozen diagnosis with exact captured pressure column scaling."""
from pathlib import Path
from fractions import Fraction as Q
import importlib.util,json,math,sys
import mpmath as mp
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('focus',P/'analyze.py');f=importlib.util.module_from_spec(s);s.loader.exec_module(f)
def run(dps):
 mp.mp.dps=dps;C=ROOT/'evidence/forcing-e2-fixed-candidates-v1';c=next(x for x in json.loads((C/'inputs.json').read_text())['cases']if x['index']==41);c=dict(c,acceleration=[.0625,-.125,.03125]);records=f.a.records(C/'candidate-fixed.log');rows=[]
 for order in [16,32]:
  e=next(x for x in records if x['event']=='fixed_equation'and x['index']==41 and x['order']==order);end=next(x for x in records if x['event']=='fixed_point'and x['index']==41 and x['order']==order and x['label']=='end');r,groups=f.a.exact(c,e);latent=[f.a.scalar(x)for x in end['chart_endpoint_parts']];rl,_=f.a.exact(c,e,z=latent);rho=[x-y for x,y in zip(r,rl)];B=f.transpose([[Q(x)for x in row]for row in e['end_b']]);Bt=f.transpose(B);ni=f.inverse(f.mul(Bt,B));scales=[max(map(abs,col))for col in Bt];Bs=[[x/s for x,s in zip(row,scales)]for row in B];Ns=f.mul(f.transpose(Bs),Bs)
  def project(v):
   p=f.mv(B,f.mv(ni,f.mv(Bt,v)));q=[x-y for x,y in zip(v,p)];f.require(all(x==0 for x in f.mv(Bt,q)),'exact pressure complement');return {'range':f.summary(p),'complement':f.summary(q)}
  coeff=f.mv(ni,f.mv(Bt,r));rows.append({'order':order,'column_max_abs':[float(x)for x in scales],'max_to_min_column_ratio':float(max(scales)/min(scales)),'column_inf_normalized_Gram_condition':f.condition(Ns),'stored':project(r),'latent_inertia_only':project(rl),'rho_rate':project(rho),'component3_groups':{k:float(v[3])for k,v in groups.items()if k!='direct_inertia'},'pressure_range_fit_coefficients':[{'column':i,'fit_coefficient':float(x),'current_pressure':c['final_authorized_unknowns'][6+i],'current_ulp':math.ulp(c['final_authorized_unknowns'][6+i]),'fit_coefficient_per_current_ulp':float(abs(x)/Q(math.ulp(c['final_authorized_unknowns'][6+i])))}for i,x in enumerate(coeff)]})
 return {'status':'PASS_EXISTING_DATA_PRESSURE_SCALING','precision_digits':dps,'rows':rows,'new_native_equations':0,'new_corrections':0,'scope':'Exact linear pressure-range fit coefficients compared with current binary64 spacing. No rounded alternative pressure candidate, fresh forces or native correction is evaluated. Normalizing columns changes only this diagnostic matrix, never solver/gate.'}
if __name__=='__main__':print(json.dumps(run(int(sys.argv[1])if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
