"""Independent finite-field, shear-mode, energy and coupled liquid evidence gate.

Explicit checks remain enabled under python -O. This qualifies the bounded
sealed free-slip Newtonian model; it is not a free-surface viscosity gate.
"""
import csv
import json
import math
from pathlib import Path
import struct
import sys
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]

def require(ok,message):
    if not ok: raise ValueError(message)

def near(a,b,tol=1e-12):
    require(math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,f'{a} != {b}')

def finite_tree(value):
    if isinstance(value,dict):
        for v in value.values(): finite_tree(v)
    elif isinstance(value,list):
        for v in value: finite_tree(v)
    elif isinstance(value,(int,float)): require(math.isfinite(value),'nonfinite metadata')

def read(path):
    with path.open() as stream: rows=[{k:float(v) for k,v in row.items()} for row in csv.DictReader(stream)]
    require(all(math.isfinite(v) for r in rows for v in r.values()),'nonfinite CSV: '+str(path))
    return rows

def faces(path,n):
    rows=read(path);expected=[]
    for d in range(3):
        shape=list(n);shape[d]+=1
        for k in range(shape[2]):
            for j in range(shape[1]):
                for i in range(shape[0]):expected.append((d,i,j,k))
    require(len(rows)==len(expected),'face count')
    out={}
    for row,key in zip(rows,expected):
        require(tuple(row[name] for name in ('axis','i','j','k'))==key,'face layout')
        d=key[0];v=row['velocity'];require(key[d+1] not in (0,n[d]) or v==0,'impermeable wall')
        out[key]=v
    return out

def energy(v,mass): return 0.5*mass*math.fsum(u*u for u in v.values())

def pixel(path,v,n):
    nx,ny,nz=n
    with Image.open(path) as image:
        require(image.mode=='L' and image.size==(nx,ny),'velocity diagnostic PNG layout')
        for j in range(ny):
            for i in range(nx):
                # Match actual f32 addition of two native face samples.
                total=struct.unpack('f',struct.pack('f',v[0,i,j,nz//2]+v[0,i+1,j,nz//2]))[0]
                expected=math.floor(255*(0.5+0.25*total)+0.5)
                require(image.getpixel((i,ny-1-j))==min(255,max(0,expected)),'native velocity diagnostic pixel')

def stages(case,meta):
    rows=read(case/'steps.csv');require(len(rows)==meta['steps'],'interval count')
    nu=meta['dynamic_viscosity']/meta['density'];h=meta['spacing']
    for index,row in enumerate(rows,1):
        require(row['step']==index and row['dt']>0,'step sequence')
        near(row['stability'],nu*row['dt']*sum(1/(x*x) for x in h),1e-15)
        require(0<=row['stability']<=0.25,'explicit stability gate')
        for name in ('kinetic_before','kinetic_after','dissipation_before','update_energy','rounding_budget','modal_error'):
            require(row[name]>=0,'negative energy or diagnostic')
        loss=row['dt']*row['dissipation_before']
        identity=row['kinetic_after']-row['kinetic_before']+loss-row['update_energy']-row['rounding_work']
        near(row['identity_error'],identity,1e-18)
        require(abs(identity)<=64*sys.float_info.epsilon*(row['kinetic_before']+row['kinetic_after']+loss+row['update_energy']+row['rounding_budget']),'energy identity gate')
        require(row['kinetic_after']-row['kinetic_before']<=row['rounding_budget'],'energy nonincrease gate')
        require(abs(row['rounding_work'])<=row['rounding_budget'],'rounding work bound')
        # Gate receipts alone could claim an arbitrary permissive budget. Bound
        # it independently by one f32 conversion per velocity degree of freedom.
        bound=8*2**-24*(row['kinetic_before']+row['kinetic_after']+row['update_energy'])+128*sys.float_info.epsilon*loss
        require(row['rounding_budget']<=bound,'rounding budget inflation')
    return rows

def isolated(case,meta):
    n=meta['counts'];h=meta['spacing'];nu=meta['dynamic_viscosity']/meta['density'];nx=n[0]
    require(n==[nx,nx,3] and h==[1/nx,1/nx,.25] and meta['density']==3 and meta['duration']==.1,'manufactured scenario')
    rows=stages(case,meta);initial=faces(case/'initial-faces.csv',n);final=faces(case/'final-faces.csv',n)
    for (d,i,j,k),value in initial.items():
        expected=0 if d==2 else math.sin(math.pi*i/nx)*math.cos(math.pi*(j+.5)/nx) if d==0 else -math.cos(math.pi*(i+.5)/nx)*math.sin(math.pi*j/nx)
        if (d,i,j,k)[d+1] in (0,n[d]):expected=0
        near(value,struct.unpack('f',struct.pack('f',expected))[0],1e-7)
    mass=3*math.prod(h);initial_energy=energy(initial,mass)
    lam=8*math.sin(math.pi/(2*nx))**2*nx**2
    dt=.1/meta['steps'];factor=1-nu*dt*lam
    norm=math.fsum(u*u for u in initial.values())
    for index,row in enumerate(rows,1):
        near(row['dt'],dt,1e-18)
        near(row['kinetic_before'],initial_energy if index==1 else rows[index-2]['kinetic_after'],1e-15)
        expected=factor**index
        near(row['amplitude'],expected,3e-6)
        require(row['modal_error']<=3e-6,'modal field error')
        near(row['kinetic_after'],initial_energy*expected**2,3e-6*initial_energy)
        # Energy production for the symmetric-strain eigenmode.
        near(row['dissipation_before'],2*nu*lam*row['kinetic_before'],2e-5)
    near(rows[-1]['kinetic_after'],energy(final,mass),1e-13)
    amplitude=math.fsum(v*initial[key] for key,v in final.items())/norm
    near(amplitude,rows[-1]['amplitude'],1e-13)
    modal=max(abs(v-factor**len(rows)*initial[key]) for key,v in final.items())
    near(modal,rows[-1]['modal_error'],1e-13)
    exact=math.exp(-nu*.1*2*math.pi**2)
    discrete_exact=math.exp(-nu*.1*lam)
    continuum_rms=math.sqrt(math.fsum((v-exact*initial[key])**2 for key,v in final.items())/len(final))
    time_error=abs(amplitude-discrete_exact)
    pixel(case/'initial.png',initial,n);pixel(case/'final.png',final,n)
    return {'case':case.name,'kind':'isolated','intervals':len(rows),'amplitude':amplitude,'continuum_rms':continuum_rms,'time_error':time_error,'energy_ratio':energy(final,mass)/initial_energy,'modal_error':modal}

def coupled(case,meta):
    n=meta['counts'];require(n==[2,2,1] and meta['density']==1 and meta['spacing']==[1,1,1],'coupled scenario')
    rows=stages(case,meta);ledger=read(case/'liquid.csv');require(len(ledger)==len(rows),'coupled ledger count')
    previous=4.0;accepted=[]
    for index,(row,entry) in enumerate(zip(rows,ledger),1):
        require(entry['step']==entry['carrier_version']==entry['liquid_version']==index,'atomic generations')
        require(entry['time']==index*.125 and row['dt']==.125,'atomic accepted time')
        require(entry['volume_before']==previous and entry['volume_after']==4,'liquid exact carry')
        require(entry['inward']==entry['outward']==entry['source']==0,'sealed phase boundary')
        balance=entry['volume_after']-entry['volume_before'];budget=64*sys.float_info.epsilon*(abs(entry['volume_before'])+abs(entry['volume_after']))
        require(entry['balance']==balance and entry['budget']==budget,'independent liquid ledger')
        require(abs(balance)<=budget,'liquid gate');previous=entry['volume_after']
        require(entry['owned_bytes']==meta['owned_bytes'] and entry['workspace_bytes']==meta['workspace_bytes'] and entry['total_bytes']==entry['owned_bytes']+entry['workspace_bytes'],'all retained capacities')
        v=faces(case/f'frame-{index:02}-faces.csv',n);cells=read(case/f'frame-{index:02}-cells.csv')
        require(len(cells)==4 and all(r['cell']==i and r['fraction']==1 for i,r in enumerate(cells)),'fully filled native liquid')
        divergence=max(abs(v[0,i+1,j,0]-v[0,i,j,0]+v[1,i,j+1,0]-v[1,i,j,0]+v[2,i,j,1]-v[2,i,j,0]) for j in range(2) for i in range(2))
        require(entry['divergence']==divergence and divergence<=1e-5,'direct accepted divergence')
        near(entry['accepted_energy'],energy(v,1),1e-15)
        require(all(r['pressure']==0 for r in cells),'symmetric pulse zero pressure')
        near(row['kinetic_after'],energy(v,1),1e-15)
        # First force pulse creates four +-1/16 faces; K shear mode lambda=4.
        if index==1:near(energy(v,1),2*(.0625*(1-.125*meta['dynamic_viscosity']*4))**2,1e-15)
        pixel(case/f'frame-{index:02}.png',v,n);accepted.append(entry['accepted_energy'])
    return {'case':case.name,'kind':'coupled','intervals':len(rows),'final_energy':accepted[-1]}

def verify(root):
    root=Path(root);names={f'space-n{n}' for n in (8,16,32,64)}|{f'time-n12-s{s}' for s in (32,64,128)}|{'material-n16-nu0.01','material-n16-nu0.1'}|{f'{m}-coupled-mu{mu}' for m in ('jacobi-pcg-v1','sgs-pcg-v1') for mu in ('0','0.125')}
    require({p.name for p in root.iterdir() if p.is_dir()}==names,'complete native scenario inventory')
    results=[]
    for name in sorted(names):
        case=root/name;meta=json.loads((case/'case.json').read_text());finite_tree(meta)
        require(meta['wall']=='stationary-impermeable-free-slip' and meta['free_surface'] is False,'declared model boundary')
        n=meta['counts'];require(meta['workspace_bytes']==8*sum(math.prod([n[q]+(d==q) for q in range(3)]) for d in range(3)),'workspace accounting')
        results.append(isolated(case,meta) if meta['kind']=='isolated' else coupled(case,meta))
    lookup={r['case']:r for r in results}
    for lo,hi in zip((8,16,32),(16,32,64)):
        require(lookup[f'space-n{hi}']['continuum_rms']<.35*lookup[f'space-n{lo}']['continuum_rms'],'spatial convergence')
    for lo,hi in ((32,64),(64,128)):
        require(lookup[f'time-n12-s{hi}']['time_error']<.55*lookup[f'time-n12-s{lo}']['time_error'],'time convergence')
    require(lookup['material-n16-nu0.1']['energy_ratio']<lookup['space-n16']['energy_ratio']<lookup['material-n16-nu0.01']['energy_ratio'],'material coefficient decay')
    for m in ('jacobi-pcg-v1','sgs-pcg-v1'):
        require(lookup[m+'-coupled-mu0.125']['final_energy']<lookup[m+'-coupled-mu0']['final_energy'],'coupled viscosity comparison')
    return {'results':results,'free_surface_viscosity_validated':False,'physical_material_calibration_claimed':False}
if __name__=='__main__': print(json.dumps(verify(Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'evidence/viscosity/demo'),indent=2))
