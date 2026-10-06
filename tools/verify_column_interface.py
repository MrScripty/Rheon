"""Independent finite fields, native ledger, geometry and manufactured oracles.

Coupled two-column dynamics demonstrate consecutive atomic publication and
changing pressure classification. Accuracy convergence is imposed transverse
advection, distinct from general liquid dynamics.
"""
import csv
import json
import math
from pathlib import Path
import sys
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]

def require(ok,message):
    if not ok: raise ValueError(message)

def near(a,b,tol=1e-12):
    require(math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,f"{a} differs from {b}")

def read_csv(path):
    with path.open() as stream: rows=[{key:float(value) for key,value in row.items()} for row in csv.DictReader(stream)]
    require(all(math.isfinite(value) for row in rows for value in row.values()),"nonfinite field: "+path.name)
    return rows

def cells(path,dims):
    rows=read_csv(path);nx,ny,nz=dims
    require(len(rows)==nx*ny*nz,"cell count")
    for index,row in enumerate(rows):
        require((row['i'],row['j'],row['k'])==(index%nx,(index//nx)%ny,index//(nx*ny)),"cell layout")
        require(0<=row['fraction']<=1,"raw fraction bounds")
    return rows

def volume(rows,h): return math.fsum(row['fraction']*(h[0]*h[1]*h[2]) for row in rows)

def face_fields(path,dims):
    rows=read_csv(path);expected=set()
    require(all(row[key].is_integer() for row in rows for key in ('axis','i','j','k')),'integer face layout')
    for d in range(3):
        shape=list(dims);shape[d]+=1
        for k in range(shape[2]):
            for j in range(shape[1]):
                for i in range(shape[0]): expected.add((d,i,j,k))
    result={(int(row['axis']),int(row['i']),int(row['j']),int(row['k'])):row['velocity'] for row in rows}
    require(len(rows)==len(expected) and set(result)==expected,"face layout")
    return result

def divergence(fields,p,h):
    result=0
    for d in range(3):
        hi=list(p);hi[d]+=1
        result+=(fields[(d,*hi)]-fields[(d,*p)])/h[d]
    return result

def heights(rows,dims):
    nx,ny,nz=dims;result=[]
    for k in range(nz):
        for i in range(nx):
            full=0;top=0;air=False
            for j in range(ny):
                f=rows[i+nx*(j+ny*k)]['fraction']
                require(not air or f==0,"canonical bottom-attached column")
                if f==1: full+=1
                else:
                    if not air: top=f
                    air=True
            require(full+top>.5 and full+top<=ny-.5,"resolved height range")
            result.append((full,top))
    return result

def ledger(row,previous):
    require(all(math.isfinite(v) for v in row.values()),"nonfinite ledger")
    require(row['volume_before']==previous,"exact volume carry-forward")
    require(row['inward']>=0 and row['outward']>=0 and row['source']==0,"boundary/source scenario ledger")
    b,a,inc,out=row['volume_before'],row['volume_after'],row['inward'],row['outward']
    balance=a-b+out-inc-row['source'];budget=(64*sys.float_info.epsilon)*(abs(b)+abs(a)+out+inc+abs(row['source']))
    require(row['balance']==balance,"independent balance")
    require(row['budget']==budget,"independent rounding budget")
    require(abs(balance)<=budget,"native ledger gate")
    raw=a-row['reconstruction_change']
    raw_balance=raw-b+out-inc-row['source']
    raw_budget=(64*sys.float_info.epsilon)*(abs(b)+abs(raw)+out+inc+abs(row['source']))
    require(raw>=0 and abs(raw_balance)<=raw_budget,'independent raw ledger before closure')
    return a

def primitive(x):
    return x if x<=.25 else .5 if x>=.75 else .25+.5*(x-.25)+math.sin(2*math.pi*(x-.25))/(4*math.pi)

def initial_profile(n): return [(primitive((i+1)/n)-primitive(i/n))*n for i in range(n)]

def donor_profile(initial,c,step):
    return [math.fsum(math.comb(step,j)*c**j*(1-c)**(step-j)*(initial[i-j] if i>=j else 1) for j in range(step+1)) for i in range(len(initial))]

def pixel_equation(case,stem,rows,dims,h):
    nx,ny,nz=dims
    with Image.open(case/(stem+'.png')) as image:
        require(image.mode=='L' and image.size==(nx,ny),'native guidance dimensions')
        for j in range(ny):
            for i in range(nx):
                integral=math.fsum(rows[i+nx*(j+ny*k)]['fraction']*h[2] for k in range(nz))
                expected=math.floor(255*-math.expm1(-8*integral)+.5)
                require(image.getpixel((i,ny-1-j))==expected,'native pixel equation')

def coupled(case,meta):
    dims=meta['counts'];h=meta['spacing'];nx,ny,nz=dims
    require(nx==2 and nz==1 and ny==(4 if meta['kind']=='coupled-activation' else 3) and h==[1,1,1],'coupled geometry')
    require(case.name==meta['method']+'-'+meta['kind'].removeprefix('coupled-'),'method/kind identity')
    require(meta['method'] in ('jacobi-pcg-v1','sgs-pcg-v1') and meta['pressure_is_held_interval'] is True,'coupled model')
    rest=meta['kind']=='coupled-mixed-rest';activation=meta['kind']=='coupled-activation'
    expected_steps=16 if rest else 8;require(meta['steps']==expected_steps and meta['time']==expected_steps*.125,'coupled clock')
    # F=sum face payloads, N=cell payloads, C=two column descriptors.
    faces=sum((dims[d]+1)*math.prod(dims[q] for q in range(3) if q!=d) for d in range(3))
    require(meta['owned_array_bytes']==16*faces+80*math.prod(dims)+48*nx*nz,'bounded geometry inventory')
    steps=read_csv(case/'steps.csv');require(len(steps)==expected_steps,'interval count')
    old_cells=cells(case/'initial-cells.csv',dims);old_heights=heights(old_cells,dims)
    expected_volume=2.5 if rest else 3 if activation else 2
    near(volume(old_cells,h),expected_volume,1e-14);previous=volume(old_cells,h)
    for stem,index in [('initial',0)]+[(f'frame-{step:02}',step) for step in range(1,expected_steps+1)]:
        phase=cells(case/(stem+'-cells.csv'),dims);end_heights=heights(phase,dims)
        geometry=read_csv(case/(stem+'-geometry.csv'));require(len(geometry)==nx*nz,'geometry count')
        held=old_heights
        for col,row in enumerate(geometry):
            require(row['column']==col and row['geometry_version']==index and row['pressure_geometry_version']==max(0,index-1),'geometry source versions')
            require((row['full_layers'],row['top_fraction'])==end_heights[col],'end geometry fraction identity')
            require((row['pressure_full_layers'],row['pressure_top_fraction'])==held[col],'held pressure geometry identity')
        fields=face_fields(case/(stem+'-faces.csv'),dims);wet_div=[]
        for k in range(nz):
            for j in range(ny):
                for i in range(nx):
                    full,top=held[i+nx*k];wet=j<full or (j==full and top>.5)
                    p=phase[i+nx*(j+ny*k)]['pressure']
                    if wet: wet_div.append(abs(divergence(fields,(i,j,k),h)))
                    else: require(p==0,'prescribed atmospheric air storage')
        maximum=max(wet_div);require(maximum<=1e-5,'independent held wet divergence')
        pixel_equation(case,stem,phase,dims,h)
        if index:
            row=steps[index-1];require(row['step']==index and row['time']==index*.125 and row['dt']==.125,'actual interval clock')
            previous=ledger(row,previous);require(row['inward']==row['outward']==0,'sealed coupled box')
            near(previous,volume(phase,h),2e-14);near(previous,expected_volume,2e-14)
            near(row['wet_divergence'],maximum,1e-15)
            require(row['geometry_version']==index and row['pressure_geometry_version']==index-1,'ledger source versions')
            require(0<=row['collapsed_columns']<=nx*nz and row['collapsed_columns'].is_integer(),'closure count')
            if rest:
                require([x['fraction'] for x in phase]==[1,1,.25,.25,0,0],'mixed hydrostatic fractions')
                near(phase[0]['pressure'],.1875);near(phase[1]['pressure'],.1875)
            elif index==1:
                expected=1/6 if activation else .125
                near(phase[0]['pressure'],-expected,1e-10);near(phase[1]['pressure'],expected,1e-10)
        old_cells,old_heights=phase,end_heights
    final=cells(case/'final-cells.csv',dims);require(final==old_cells,'final accepted frame identity')
    final_fields=face_fields(case/'final-faces.csv',dims)
    require(final_fields==fields,'final accepted face identity')
    final_geometry=read_csv(case/'final-geometry.csv')
    require(final_geometry==geometry,'final accepted geometry identity')
    pixel_equation(case,'final',final,dims,h)
    if activation:
        require(final[2]['fraction']>.5 and final[2]['pressure']!=0,'new pressure center activation')
    return {'scenario':case.name,'coupled_intervals':expected_steps,'volume':previous,'changing_pressure_classification':activation,'general_fluid_accuracy_claimed':False}

def imposed(case,meta):
    require(meta['pressure_solved'] is False,'imposed velocity scope')
    dims=meta['counts'];h=meta['spacing'];nx,ny,nz=dims
    initial=cells(case/'initial-cells.csv',dims);final=cells(case/'final-cells.csv',dims)
    heights(initial,dims);heights(final,dims)
    require(all(row['pressure']==0 for row in initial+final),'imposed fixture pressure placeholder')
    fields=face_fields(case/'faces.csv',dims)
    vertical=meta['kind']=='imposed-vertical';speed=meta['speed'];dt=meta['dt'];count=meta['steps']
    require(speed==(0.125 if case.name=='vertical-up' else -.125) if vertical else speed==.25,'manufactured speed')
    for (d,i,j,k),value in fields.items():
        near(value,speed if (d==1 if vertical else d==0 and j<2) else 0,0)
    require(max(abs(divergence(fields,(i,j,k),h)) for k in range(nz) for j in range(ny) for i in range(nx))==0,'independent imposed divergence')
    rows=read_csv(case/'steps.csv');require(len(rows)==count,'imposed interval count');previous=volume(initial,h)
    profile=initial_profile(nx) if not vertical else None
    for step,row in enumerate(rows,1):
        require(row['step']==step and row['time']==step*dt and row['dt']==dt,'imposed clock')
        previous=ledger(row,previous)
        require(row['geometry_version']==step and row['pressure_geometry_version']==step-1,'imposed geometry versions')
        require(row['wet_divergence']==0,'reported imposed divergence')
        if vertical:
            expected=(1.25 if speed>0 else 2.25)+speed*dt*step
            near(previous,expected,0);near(row['inward'],max(speed,0)*dt,0);near(row['outward'],max(-speed,0)*dt,0)
        else:
            old=donor_profile(profile,meta['courant'],step-1)
            near(row['inward'],dt*.25*.5,1e-15)
            near(row['outward'],dt*.25*(.25+.25*old[-1]),2e-14)
    near(previous,volume(final,h),2e-14)
    if vertical:
        expected=(1.25 if speed>0 else 2.25)+speed*dt*count
        full=int(expected);top=expected-full
        require([row['fraction'] for row in final]==[1 if j<full else top if j==full else 0 for j in range(ny)],'exact vertical swept geometry')
        return {'scenario':case.name,'geometric_intervals':count,'height':expected,'volume':previous,'height_error':0}
    require(dims==[nx,4,2] and h==[1/nx,.25,.5] and meta['time']==.5,'transverse fixture geometry/time')
    expected=donor_profile(profile,meta['courant'],count);error=0;maximum=0
    for row in final:
        i,j=int(row['i']),int(row['j']);oracle=1 if j==0 else expected[i] if j==1 else 0
        near(row['fraction'],oracle,3e-13)
        maximum=max(maximum,abs(row['fraction']-oracle))
        if j==1 and row['k']==0:
            translated=(primitive((i+1)/nx-.125)-primitive(i/nx-.125))*nx
            error+=abs(row['fraction']-translated)*(.25/nx)
    return {'scenario':case.name,'geometric_intervals':count,'volume':previous,'height_l1_error':error,'binomial_max_error':maximum,'courant':meta['courant'],'x_cells':nx,'scope':'Imposed X translation on a 3D stored grid; not coupled multidirectional accuracy.'}

def verify(directory):
    directory=Path(directory);manifest=json.loads((directory/'complete.json').read_text())
    require(manifest=={'scenarios':13,'model':'bounded-column-closure','imposed_advection_separate_from_pressure':True},'complete scope')
    cases=sorted(p for p in directory.iterdir() if p.is_dir())
    expected={method+'-'+kind for method in ('jacobi-pcg-v1','sgs-pcg-v1') for kind in ('pulse','activation','mixed-rest')}|{'vertical-up','vertical-down','advection-n64-c0125'}|{f'advection-n{n}-c025' for n in (16,32,64,128)}
    require({p.name for p in cases}==expected,'complete scenario inventory')
    results=[]
    for case in cases:
        meta=json.loads((case/'run.json').read_text());require(all(math.isfinite(x) for x in meta.values() if isinstance(x,(int,float))),'nonfinite metadata')
        results.append(coupled(case,meta) if meta['kind'].startswith('coupled-') else imposed(case,meta))
    convergence=sorted((r for r in results if r.get('courant')==.25),key=lambda r:r['x_cells'])
    require([r['x_cells'] for r in convergence]==[16,32,64,128],'convergence resolutions')
    for a,b in zip(convergence,convergence[1:]):
        require(b['height_l1_error']<a['height_l1_error'],'spatial refinement behavior')
        b['observed_order']=math.log(a['height_l1_error']/b['height_l1_error'],2)
    fine=next(r for r in results if r.get('courant')==.125)
    coarse=next(r for r in convergence if r['x_cells']==64)
    require(fine['height_l1_error']>coarse['height_l1_error'],'recorded fixed-grid donor diffusion limitation')
    return {'finite_fields_geometry_pixels_ledgers_carry_forward':'passed','results':results,'scope':'Bounded column geometry closure. Pressure activation/atomic publication is demonstrated; convergence applies only to prescribed transverse advection.'}
if __name__=='__main__': print(json.dumps(verify(Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'evidence/column-interface/demo'),indent=2))
