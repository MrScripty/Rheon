#!/usr/bin/env python3
"""Exact manufactured-reference sampling and retained-field goals. NO solve.

The reference is an independent polynomial/beta-integral construction. It is
never passed to a native provider. All goal splits are benchmark observations,
not bounds for unknown physical solutions.
"""
import argparse, json, math, time
from fractions import Fraction as F
from pathlib import Path
from check_flat_wall_ritz import Model, rational, require, traction, file_digest, managed, FILE_CAP
from flat_wall_campaign_guard import write_reserved

B={6:F(1),7:F(-6),8:F(15),9:F(-20),10:F(15),11:F(-6),12:F(1)}
Z={6:F(1,2),7:F(-2),8:F(3,2),9:F(5),10:F(-25,2),11:F(12),12:F(-11,2),13:F(1)}
AMPLITUDE=F(12012**2,2)

def polynomial(p,x):return rational(sum((v*x**k for k,v in p.items()),F(0)))
def integral(p,low,high):
    return rational(sum((v*(high**(k+1)-low**(k+1))/(k+1) for k,v in p.items()),F(0)))

def continuum_reference():
    bx=integral(B,F(0),F(1));bz=integral(Z,F(0),F(1))
    moment=sum((v*(F(1,k+2)-F(1,2*(k+1))) for k,v in Z.items()),F(0))
    force=[-2*AMPLITUDE*bx*bz,F(0),F(0)]
    torque=[F(0),-2*AMPLITUDE*bx*moment,force[0]/2]
    require(force==[F(-1),F(0),F(0)] and torque==[F(0),-F(1,60),-F(1,2)],'beta-integral reference')
    return force,torque

def reference_flux_field(model,stored_coefficients):
    """Exact layer integrals of psi; separately expose rounded C representation."""
    p=list(map(rational,model.p));result={}
    for col in model.columns():
        x,y,z=col['node']
        q=AMPLITUDE*polynomial(B,p[x]-1)*p[y]**4*(1-p[y])**2*integral(Z,p[z]-1,p[z+1]-1)
        for a,face,c in col['terms']:
            if stored_coefficients:coefficient=rational(c)
            else:
                cell=model.coordinate(a,face);d=[i for i in range(3) if i!=a]
                area=(p[cell[d[0]]+1]-p[cell[d[0]]])*(p[cell[d[1]]+1]-p[cell[d[1]]])
                coefficient=F(1 if c>0 else -1)/area
            result[a,face]=rational(result.get((a,face),F(0))+coefficient*q)
    return result

def average_wall_weights(h1,h2):
    h1=rational(h1);h2=rational(h2)
    require(0<h1<h2,'two-cell average distances')
    return 2*(h1*h1+h1*h2+h2*h2)/(h1*h2*h2),-2*h1/(h2*h2)

def average_traction(model,velocity,normal_scheme):
    """New tangential two-cell-average P2; existing normal observable unchanged."""
    force,torque=traction(model,velocity,normal_scheme)
    old_x,old_t=traction(model,velocity,'p1');m=model.m;p=list(map(rational,model.p));wall=p[m]
    c1,c2=average_wall_weights(wall-p[m-1],wall-p[m-2]);fx=F(0);ty=F(0)
    for z in range(m,2*m):
        for x in range(m+1,2*m):
            stress=c1*velocity.get((0,model.index(0,(x,m-1,z))),F(0))+c2*velocity.get((0,model.index(0,(x,m-2,z))),F(0))
            area=(p[x+1]-p[x-1])*(p[z+1]-p[z])/2
            fx+=stress*area;ty+=((p[z]+p[z+1])/2-F(3,2))*stress*area
    # Reference height is fixed; normal traction and every other component remain.
    torque[2]+=(wall-F(3,2))*(old_x[0]-fx)
    force[0]=fx;torque[1]=ty
    return list(map(rational,force)),list(map(rational,torque))

def vector(values):return {'exact':list(map(str,values)),'nearest':list(map(float,values))}
def difference(a,b):return [x-y for x,y in zip(a,b)]

def pack_exact_vectors(report):
    """Lossless rational interning; avoid repeating N9's long exact fractions."""
    values=[];indices={}
    def pack(item):
        if isinstance(item,dict):
            if 'exact' in item:
                ids=[]
                for v in item['exact']:
                    if v not in indices:indices[v]=len(values);values.append(v)
                    ids.append(indices[v])
                return {'exact_indices':ids,'nearest':item['nearest']}
            return {k:pack(v) for k,v in item.items()}
        if isinstance(item,list):return [pack(v) for v in item]
        return item
    result=pack(report);result['exact_value_table']=values
    return result

def inspect_level(job,n):
    path=job/f'n{n}'/'records.jsonl';require(path.stat().st_size<=FILE_CAP,'retained input cap')
    header=None;velocity={};read=0
    with path.open('rb') as f:
        while line:=f.readline(8193):
            read+=len(line);require(read<=FILE_CAP and len(line)<=8192 and line.endswith(b'\n'),'retained bounded record')
            r=json.loads(line)
            if r['kind']=='header':require(header is None,'duplicate header');header=r
            if r['kind']=='velocity':
                key=(r['component'],r['face']);require(key not in velocity,'duplicate velocity');velocity[key]=rational(r['value'])
    require(header is not None and header['n']==n and header['mode']=='numerical_reduced_ritz_solve','retained provenance')
    require(header['physical_qualified'] is False and header['pressure_available'] is False,'retained qualifications')
    model=Model(n);stored=reference_flux_field(model,True);geometric=reference_flux_field(model,False)
    rf,rt=continuum_reference();reference=rf+rt;goals=[]
    for normal in ('p1','normal_p2'):
        for tangential in ('p1','two_cell_average_p2'):
            evaluate=traction if tangential=='p1' else average_traction
            fn,tn=evaluate(model,velocity,normal);fs,ts=evaluate(model,stored,normal);fg,tg=evaluate(model,geometric,normal)
            native=fn+tn;interp=fs+ts;sampled=fg+tg
            field=difference(native,interp);representation=difference(interp,sampled);observation=difference(sampled,reference)
            error=difference(native,reference)
            require(error==[a+b+c for a,b,c in zip(field,representation,observation)],'signed error split')
            goals.append({'tangential':tangential,'normal':normal,'retained_evaluation':vector(native),
                'exact_reference_face_average_evaluation':vector(sampled),'reference_stored_C_evaluation':vector(interp),
                'field_discrete_model_contribution':vector(field),'C_area_inverse_representation_contribution':vector(representation),
                'wall_observation_and_spatial_contribution':vector(observation),'measured_reference_error':vector(error),
                'force_error_norm_N':math.hypot(*map(float,error[:3])),
                'torque_error_norm_Nm':math.hypot(*map(float,error[3:])),
                'general_error_guarantee':False,'physical_qualified':False})
    p=list(map(rational,model.p));m=model.m
    spatial=2*AMPLITUDE*sum(((p[x+1]-p[x-1])/2*polynomial(B,p[x]-1) for x in range(m+1,2*m)),F(0))*integral(Z,p[m]-1,p[2*m]-1)
    return {'n':n,'record_head':header['head'],'records_sha256':file_digest(path),'goals':goals,
            'Fx_exact_wall_derivative_with_existing_x_hats':str(-spatial),
            'managed_bytes_checkpoint':managed(locals())}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--retained-directory',type=Path,required=True);ap.add_argument('--output-directory',type=Path,required=True);a=ap.parse_args()
    begin=time.monotonic();repo=Path(__file__).resolve().parents[1]
    require(not a.output_directory.resolve().is_relative_to(repo) and not a.output_directory.exists(),'fresh external output required')
    a.output_directory.mkdir()
    report={'mode':'retained-evaluation-and-independent-manufactured-sampling-only','native_or_PDE_solves_launched':0,
            'physical_qualified':False,'convergence_claim':False,'original_campaign_status':'refused',
            'reference':{'force':vector(continuum_reference()[0]),'torque':vector(continuum_reference()[1])},
            'runs':[],
            'remaining_error_model':'Field/source/energy/trial error is aggregated by the signed reference split; unknown smooth-field wall derivatives and application tolerances are not certified.'}
    for n in (6,9,12):
        level=pack_exact_vectors(inspect_level(a.retained_directory,n));output=a.output_directory/f'n{n}-signed-split.json'
        write_reserved(output,(json.dumps(level,indent=2)+'\n').encode(),65536)
        report['runs'].append({'n':n,'file':output.name,'bytes':output.stat().st_size,'sha256':file_digest(output),
                               'managed_bytes_checkpoint':level['managed_bytes_checkpoint']})
    report['managed_bytes_checkpoint']=managed(report);require(time.monotonic()-begin<=180,'physics model deadline')
    output=a.output_directory/'summary.json';text=(json.dumps(report,indent=2)+'\n').encode();write_reserved(output,text,65536)
    print(json.dumps({'output':str(output),'bytes':len(text),'sha256':file_digest(output),'seconds':time.monotonic()-begin}))
if __name__=='__main__':main()
