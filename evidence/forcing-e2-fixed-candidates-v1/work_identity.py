"""Independent nodal exact-input momentum/work identities on archived E2 fields."""
from pathlib import Path
from fractions import Fraction as Q
import importlib.util,json
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('equation_algebra',ROOT/'evidence/forcing-e1-fixed-candidates-v1/analyze.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
def run():
 packet=json.loads((P/'inputs.json').read_text());data=a.records(P/'candidate-fixed.log');output=[]
 for c in packet['cases']:
  c=dict(c,acceleration=[.0625,-.125,.03125]if c['load']=='forward'else[-.0625,.125,-.03125])
  for eq in [r for r in data if r['event']=='fixed_equation'and r['index']==c['index']]:
   h=Q(c['h']);u0=[[Q(v)for v in row[:2]]for row in c['accepted_publication']['velocity']];u1=[[Q(v)for v in row[:2]]for row in eq['end_velocity']];m0=list(map(Q,eq['start_mass']));m1=list(map(Q,eq['end_mass']));force=[[Q(v)for v in row[:2]]for row in eq['end_force']];g=[x-y for x,y in zip(m1,m0)];rn=[[m1[i]*u1[i][d]-m0[i]*u0[i][d]+h*force[i][d]-h*m0[i]*Q(c['acceleration'][d])for d in range(2)]for i in range(16)];mix=Q(0)
   for f,(i,j)in enumerate(eq['pairs']):
    plus,minus=Q(eq['plus'][f]),Q(eq['minus'][f]);g[i]+=plus-minus;g[j]-=plus-minus;mix+=(plus+minus)*sum((x-y)**2 for x,y in zip(u1[i],u1[j]))/2
    for d in range(2):transport=plus*u1[i][d]-minus*u1[j][d];rn[i][d]+=transport;rn[j][d]-=transport
   energy_delta=sum(m1[i]*sum(v*v for v in u1[i])/2-m0[i]*sum(v*v for v in u0[i])/2 for i in range(16));be=sum(m0[i]*sum((x-y)**2 for x,y in zip(u1[i],u0[i]))/2 for i in range(16));gwork=sum(g[i]*sum(v*v for v in u1[i])/2 for i in range(16));fwork=h*sum(u1[i][d]*force[i][d]for i in range(16)for d in range(2));external=h*sum(m0[i]*u1[i][d]*Q(c['acceleration'][d])for i in range(16)for d in range(2));nodal_work=sum(u1[i][d]*rn[i][d]for i in range(16)for d in range(2));lhs=energy_delta+be+mix+gwork+fwork-external;a.require(lhs==nodal_work,'independently derived stored nodal changing-mass work identity')
   stable,_=a.exact(c,eq);direct,_=a.exact(c,eq,direct=True);projected=[sum(Q(eq['r'][i][d][j])*rn[i][d]/h for i in range(16)for d in range(2))for j in range(22)];a.require(projected==direct,'nodal momentum independently projects to the complete direct equation')
   z0=list(map(Q,eq['start_z']));z1=list(map(Q,eq['end_z']));embed0=a.embedding(eq,z0);embed1=a.embedding(eq,z1);e0=[[embed0[i][d]-u0[i][d]for d in range(2)]for i in range(16)];e1=[[embed1[i][d]-u1[i][d]for d in range(2)]for i in range(16)];wr=h*sum(z*v for z,v in zip(z1,stable));term1=-sum(e1[i][d]*rn[i][d]for i in range(16)for d in range(2));term2=-sum((u1[i][d]+e1[i][d])*m1[i]*(e1[i][d]-e0[i][d])for i in range(16)for d in range(2));a.require(nodal_work-wr==term1+term2,'both exact stored embedding ledger-defect terms retained')
   output.append({'index':c['index'],'order':eq['order'],'exact_nodal_work_identity':True,'exact_direct_projection_identity':True,'exact_two_term_embedding_work_identity':True,'energy_change':str(energy_delta),'BE_loss':str(be),'mixing_loss':str(mix),'GCL_work':str(gwork),'captured_nodal_force_work':str(fwork),'external_work':str(external),'nodal_residual_work':str(nodal_work),'stored_coefficient_work':str(wr),'ledger_defect_term1':str(term1),'ledger_defect_term2':str(term2),'scope':'Exact algebra on actual captured rounded nodal forces and planar stored inputs. Force pairing need not equal native separately assembled strain/pressure power; no new allowance or gate substitution.'})
 a.require(len(output)==4,'all four evaluated equations')
 return {'status':'PASS_FOUR_INDEPENDENT_NODAL_WORK_IDENTITIES','native_execution':False,'new_trajectories':0,'rows':output}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
