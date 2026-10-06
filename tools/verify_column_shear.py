"""Independent liquid-only mass, edge force, traction and energy evidence gate.

Only the fixed-flat-surface periodic shear prerequisite is qualified. All
requirements remain active under python -O. No coupled carrier claim is made.
"""
import csv
import json
import math
from pathlib import Path
import struct
import sys
import numpy as np
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]

def require(ok,message):
    if not ok:raise ValueError(message)

def near(a,b,tol=1e-12):
    require(math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,f'{a} != {b}')

def f32(x):return struct.unpack('f',struct.pack('f',x))[0]

def finite(value):
    if isinstance(value,dict):
        for v in value.values():finite(v)
    elif isinstance(value,list):
        for v in value:finite(v)
    elif isinstance(value,(int,float)):require(math.isfinite(value),'nonfinite metadata')

def read(path):
    with path.open() as stream:rows=[{k:float(v) for k,v in row.items()} for row in csv.DictReader(stream)]
    require(all(math.isfinite(v) for row in rows for v in row.values()),'nonfinite CSV: '+str(path))
    return rows

def face_profile(path,n,normal,wet,tangents):
    rows=read(path);expected=[]
    for d in range(3):
        m=[n[q]+(q==d) for q in range(3)]
        for k in range(m[2]):
            for j in range(m[1]):
                for i in range(m[0]):expected.append((d,i,j,k))
    require(len(rows)==len(expected),'complete face layout')
    data={}
    for row,key in zip(rows,expected):
        require(tuple(row[name] for name in ('axis','i','j','k'))==key,'face ordering')
        data[key]=row['velocity']
    def sample(d,j):
        p=[0,0,0];p[normal]=j;return data[(d,*p)]
    profiles=[[sample(d,j) for j in range(wet)] for d in tangents]
    for (d,*p),value in data.items():
        expected=0 if d==normal or p[normal]>=wet else sample(d,p[normal])
        require(value==expected,'periodic uniform shear, zero normal/dry fields')
    return profiles

def pixel(path,profile):
    with Image.open(path) as image:
        require(image.mode=='L' and image.size==(24,len(profile)),'diagnostic PNG layout')
        for j,u in enumerate(profile):
            expected=min(255,max(0,math.floor(255*(.5+.5*u)+.5)))
            require(all(image.getpixel((i,len(profile)-1-j))==expected for i in range(24)),'native diagnostic pixel')

def case(path):
    meta=json.loads((path/'case.json').read_text());finite(meta)
    require(meta['lateral_boundary']=='periodic' and meta['endpoint_shear_traction']=='zero' and meta['coupled_carrier_step'] is False,'declared prerequisite boundaries')
    require(meta['density']==3 and meta['dynamic_viscosity']==.15 and meta['duration']==.1 and meta['geometry_stamp']==[1,0],'material units and geometry identity')
    n=meta['counts'];normal=meta['normal_axis'];full=meta['full_layers'];top=meta['top_fraction'];h=meta['spacing'][normal]
    require(normal in (0,1,2) and n[normal]==full+2 and h==1/(full+top),'normal geometry')
    for d in range(3):
        if d!=normal:require(n[d]==[2,2,3][d] and meta['spacing'][d]==[.5,.5,.25][d],'lateral geometry')
    require(meta['workspace_bytes']==40*n[normal],'five capped profile arrays')
    wet=full+(top>.5);height=(full+top)*h;area=math.prod(meta['spacing'][d]*n[d] for d in range(3) if d!=normal)
    widths=[h]*(wet-1)+[height-(wet-1)*h];mass=[3*(area*w) for w in widths];k=.15*area/h
    require(all(w>0 for w in widths),'positive liquid-only dual volumes')
    near(math.fsum(area*w for w in widths),area*height,64*sys.float_info.epsilon*2*area*height)
    tangents=[d for d in range(3) if d!=normal];initial=face_profile(path/'initial-faces.csv',n,normal,wet,tangents);final=face_profile(path/'final-faces.csv',n,normal,wet,tangents)
    for t,profile in enumerate(initial):
        for j,u in enumerate(profile):
            y=(j+.5)*h;shape=.25 if meta['kind']=='translation' else y if meta['kind']=='affine' else math.cos(math.pi*y)
            near(u,f32(shape*(1 if t==0 else .5)),1e-7)
    ledger=read(path/'steps.csv');records=read(path/'profiles.csv');steps=meta['steps'];dt=.1/steps
    require(len(ledger)==steps and len(records)==steps*wet,'complete interval/profile records')
    previous=initial
    max_reference_error=0.0
    for step,row in enumerate(ledger,1):
        require(row['step']==step and row['dt']==dt and row['geometry_id']==1 and row['geometry_version']==0 and row['workspace_bytes']==meta['workspace_bytes'],'interval geometry/time/capacity')
        require(row['volume']==area*height,'exact held liquid volume')
        near(row['mass_volume_error'],math.fsum(area*w for w in widths)-area*height,1e-15)
        require(row['mass_volume_budget']==64*sys.float_info.epsilon*2*area*height,'geometry volume budget')
        require(abs(row['mass_volume_error'])<=row['mass_volume_budget'],'mass partition gate')
        stage=records[(step-1)*wet:step*wet]
        current=[[r[f'before{t}'] for r in stage] for t in range(2)];after=[[r[f'after{t}'] for r in stage] for t in range(2)]
        require(current==previous,'exact profile carry-forward')
        for j,r in enumerate(stage):
            require(r['step']==step and r['layer']==j,'profile interval/layout')
            require(r['position']==(j+.5)*h and r['dual_length']==widths[j] and r['mass']==mass[j],'actual cut dual mass')
        forces=[];dissipation=[];update_energy=[];roundwork=[];absolute_work=[];before_energy=[];after_energy=[]
        for t in range(2):
            # Independent tridiagonal row law: zero exterior traction, no air
            # edge, no boundary damping term. Equal opposite internal forces.
            force=[(k*(current[t][j+1]-current[t][j]) if j+1<wet else 0)-(k*(current[t][j]-current[t][j-1]) if j else 0) for j in range(wet)]
            forces.append(force)
            for j,r in enumerate(stage):
                require(r[f'force{t}']==force[j],'exact shared-edge shear force')
                delta=dt*force[j]/mass[j];proposed=current[t][j]+delta;rounding=after[t][j]-proposed
                require(after[t][j]==f32(proposed),'actual stored f32 update')
                require(min(current[t])<=proposed<=max(current[t]) and min(current[t])<=after[t][j]<=max(current[t]),'raw convex bounds')
                before_energy.append(.5*(mass[j]*(current[t][j]*current[t][j])))
                after_energy.append(.5*(mass[j]*(after[t][j]*after[t][j])))
                update_energy.append(.5*(mass[j]*(delta*delta)))
                work=mass[j]*(rounding*(proposed+.5*rounding));roundwork.append(work);absolute_work.append(abs(work))
            dissipation.extend(k*(current[t][j+1]-current[t][j])**2 for j in range(wet-1))
            force_sum=math.fsum(force);force_budget=64*sys.float_info.epsilon*math.fsum(abs(f) for f in force)
            near(row[f'force{t}'],force_sum,1e-15);near(row[f'force_budget{t}'],force_budget,1e-28)
            require(abs(row[f'force{t}'])<=row[f'force_budget{t}'],'force cancellation gate')
            po=math.fsum(m*u for m,u in zip(mass,current[t]));pn=math.fsum(m*u for m,u in zip(mass,after[t]));pr=math.fsum(m*(v-(u+dt*f/m)) for m,u,v,f in zip(mass,current[t],after[t],force))
            near(row[f'momentum_before{t}'],po,1e-14);near(row[f'momentum_after{t}'],pn,1e-14);near(row[f'rounding_momentum{t}'],pr,1e-15)
            require(row[f'momentum_error{t}']==row[f'momentum_after{t}']-row[f'momentum_before{t}']-row[f'rounding_momentum{t}'],'independent stored momentum residual')
            terms=[abs(m*u)+abs(m*v)+abs(m*(v-(u+dt*f/m))) for m,u,v,f in zip(mass,current[t],after[t],force)]
            expected_budget=64*sys.float_info.epsilon*math.fsum(terms)
            near(row[f'momentum_budget{t}'],expected_budget,1e-27)
            require(abs(row[f'momentum_error{t}'])<=row[f'momentum_budget{t}'],'momentum balance gate')
        for key,expected in [('kinetic_before',math.fsum(before_energy)),('kinetic_after',math.fsum(after_energy)),('dissipation_before',math.fsum(dissipation)),('update_energy',math.fsum(update_energy)),('rounding_work',math.fsum(roundwork))]:near(row[key],expected,2e-13)
        loss=dt*row['dissipation_before'];identity=row['kinetic_after']-row['kinetic_before']+loss-row['update_energy']-row['rounding_work']
        require(row['identity_error']==identity,'independent stored energy residual')
        floating=64*sys.float_info.epsilon*(row['kinetic_before']+row['kinetic_after']+loss+row['update_energy']+math.fsum(absolute_work))
        near(row['energy_budget'],math.fsum(absolute_work)+floating,1e-27)
        require(abs(identity)<=floating and row['kinetic_after']-row['kinetic_before']<=row['energy_budget'],'energy gates')
        require(all(row[key]>=0 for key in ('kinetic_before','kinetic_after','dissipation_before','update_energy','energy_budget')),'nonnegative diagnostics')
        previous=after
    require(previous==final,'final native face/profile equality')
    pixel(path/'initial.png',initial[0]);pixel(path/'final.png',final[0])
    if meta['kind']=='translation':require(initial==final and all(r['dissipation_before']==0 for r in ledger),'translation is a null mode')
    if meta['kind']=='affine':
        # f32 sampled affine values have small representation error. The row
        # law above qualifies every actual force, including both endpoint loads.
        require(ledger[0]['dissipation_before']>0,'affine shear deforms')
    # Independently assembled dense generalized operator: lumped M, graph K.
    stiffness=np.zeros((wet,wet))
    for j in range(wet-1):stiffness[j,j]+=k;stiffness[j+1,j+1]+=k;stiffness[j,j+1]-=k;stiffness[j+1,j]-=k
    operator=stiffness/np.array(mass)[:,None]
    stability=dt*max(np.diag(operator));require(all(abs(r['stability']-stability)<=1e-15 and 0<=r['stability']<=1 for r in ledger),'generalized explicit stability')
    symmetric=stiffness/np.sqrt(np.outer(mass,mass));eigenvalues,eigenvectors=np.linalg.eigh(symmetric)
    require(min(eigenvalues)>=-1e-12,'dense operator PSD')
    propagator=np.linalg.matrix_power(np.eye(wet)-dt*operator,steps)
    reference=propagator@np.array(initial[0]);max_reference_error=max(abs(np.array(final[0])-reference))
    require(max_reference_error<2e-6,'discrete double-reference error')
    coeff=eigenvectors.T@(np.sqrt(mass)*np.array(initial[0]));exact_discrete=(eigenvectors@(np.exp(-eigenvalues*.1)*coeff))/np.sqrt(mass)
    time_error=math.sqrt(math.fsum(w*(v-e)**2 for w,v,e in zip(widths,reference,exact_discrete))/height)
    native_time_error=math.sqrt(math.fsum(w*(v-e)**2 for w,v,e in zip(widths,final[0],exact_discrete))/height)
    continuum=[u*math.exp(-(.15/3)*math.pi**2*.1/height**2) for u in initial[0]]
    error=math.sqrt(math.fsum(w*(v-e)**2 for w,v,e in zip(widths,final[0],continuum))/height)
    return {'case':path.name,'kind':meta['kind'],'normal_axis':normal,'full_layers':full,'top_fraction':top,'wet_nodes':wet,'intervals':steps,'continuum_rms':error,'time_error':time_error,'native_time_error':native_time_error,'double_reference_error':float(max_reference_error),'liquid_volume':area*height,'initial_energy':ledger[0]['kinetic_before'],'final_energy':ledger[-1]['kinetic_after']}

def verify(root):
    root=Path(root)
    expected={f'decay-a1-n{n}-f{f}' for n in (8,16,32,64) for f in (.25,.5,.75)}|{f'time-a1-n12-f0.75-s{s}' for s in (64,128,256)}|{f'affine-a{a}-n3-f0.75' for a in (0,1,2)}|{f'translation-a{a}-n3-f{f}' for a in (0,1,2) for f in (.25,.75)}
    require({p.name for p in root.iterdir() if p.is_dir()}==expected,'complete native scenario inventory')
    results=[case(root/name) for name in sorted(expected)];lookup={r['case']:r for r in results}
    for f in (.25,.5,.75):
        for lo,hi in ((8,16),(16,32),(32,64)):
            require(lookup[f'decay-a1-n{hi}-f{f}']['continuum_rms']<.4*lookup[f'decay-a1-n{lo}-f{f}']['continuum_rms'],'traction-free spatial convergence')
    for lo,hi in ((64,128),(128,256)):
        require(lookup[f'time-a1-n12-f0.75-s{hi}']['time_error']<.55*lookup[f'time-a1-n12-f0.75-s{lo}']['time_error'],'double fixed-grid time convergence')
        require(lookup[f'time-a1-n12-f0.75-s{hi}']['native_time_error']<.55*lookup[f'time-a1-n12-f0.75-s{lo}']['native_time_error'],'native fixed-grid time convergence')
    return {'results':results,'prerequisite_only':True,'coupled_carrier_free_surface_viscosity_claimed':False,'varying_height_tensor_operator_implemented':False}
if __name__=='__main__':print(json.dumps(verify(Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'evidence/column-shear/demo'),indent=2))
