from generated_fixtures import fixture
"""Independent mass/flux/pressure and existing-owner publication oracle.
Checks every native stored-f32 field, accepted epoch, ledger and grayscale pixel.
"""
import argparse
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from verify_column_momentum import finite_tree, require
EPS=np.finfo(float).eps
CASES={f'pulse-{axis}-{top}-{method}':(d,top,2,method,4,'pulse') for d,axis in enumerate('xyz') for top in [.25,.5,.75] for method in ['jacobi','sgs']}
CASES.update({f'curl-n{n}':(1,.25,n,'jacobi',1,'curl') for n in [8,16,32,64]})
def close(a,b,scale,label):
    require(abs(a-b)<=128*EPS*scale,label)
def compensated(values):
    value=0.;correction=0.
    for x in values:
        x=float(x);new=value+x
        correction += (value-new)+x if abs(value)>=abs(x) else (x-new)+value
        value=new
    return value+correction

def coords(i,n):return [i%n[0],i//n[0]%n[1],i//(n[0]*n[1])]
def index(p,n):return p[0]+n[0]*(p[1]+n[1]*p[2])
def fields(value,lengths):
    require(len(value)==len(lengths),'component count')
    result=[]
    for values,length in zip(value,lengths):
        a=np.asarray(values,dtype=np.float32).astype(float)
        require(a.shape==(length,) and np.all(np.isfinite(a)),'field shape/finite f32')
        require(np.all((a==0)|(np.abs(a)>=np.finfo(np.float32).tiny)),'normal f32 scale')
        result.append(a)
    return result
def verify_case(dest,name):
    data=json.loads((dest/'case.json').read_text());finite_tree(data)
    normal,top,n,method,steps,kind=CASES[name];tangents=[d for d in range(3) if d!=normal]
    counts=[n]*3;counts[normal]=4;spacing=[1/n]*3;spacing[normal]=.25
    require(data['name']==name and data['kind']==kind and data['normal']==normal and data['top_fraction']==top,'identity')
    require(data['counts']==counts and data['spacing']==spacing and data['density']==1 and data['projection_dt']==.125,'geometry/units')
    require(data['implementation']==('jacobi-pcg-v1' if method=='jacobi' else 'sgs-pcg-v1'),'solver identity')
    require(data['model']=='fixed_flat_static_mac_materialization' and data['wall_velocity']==0 and data['atmospheric_pressure']==0 and data['physical_time_advanced'] is False and data['moving_surface_dynamics'] is False and data['velocity_raster_range']==[-1.,1.],'model scope')
    require(len(data['steps'])==steps,'complete publications')
    size=math.prod(counts);face_counts=[counts.copy() for _ in range(3)]
    for d in range(3):face_counts[d][d]+=1
    lengths=[math.prod(m) for m in face_counts];h=spacing[normal];height=(2+top)*h;wet=2+(top>.5);area=math.prod(spacing[d] for d in tangents)
    dual=[h]*(wet-1)+[height-(wet-1)*h]
    cell_volume=np.array([area*dual[p[normal]] if p[normal]<wet else 0 for p in map(lambda i:coords(i,counts),range(size))]);cell_mass=cell_volume.copy()
    fractions=[max(0.,min(1.,2+top-coords(i,counts)[normal])) for i in range(size)]
    require(data['initial_fractions']==fractions,'authoritative phase geometry')
    mass=[];q=[];ends=[]
    for d in range(3):
        mm=[];qq=[];ee=[]
        for i in range(lengths[d]):
            p=coords(i,face_counts[d]);wall=p[d] in [0,counts[d]]
            if d==normal:
                length=h/2 if p[d]==0 else h if p[d]<wet else height-(wet-.5)*h if p[d]==wet else 0.
                m=area*length;flux=area if not wall and p[d]<=wet else 0.
            else:
                length=dual[p[normal]] if p[normal]<wet else 0.
                m=area*length*(.5 if wall else 1.);other=next(t for t in tangents if t!=d);flux=spacing[other]*length if not wall else 0.
            mm.append(m);qq.append(flux)
            if flux:
                low=p.copy();low[d]-=1;ee.append((index(low,counts),index(p,counts)))
            else:ee.append(None)
        mass.append(np.array(mm));q.append(np.array(qq));ends.append(ee)
    physical_mass=math.fsum(cell_mass)
    require(all(abs(math.fsum(m)-physical_mass)<=64*EPS*2*physical_mass for m in mass),'component mass partitions')
    def moments(v):return [math.fsum(m*u for m,u in zip(mass[d],v[d])) for d in range(3)]
    def energy(v):return math.fsum(.5*m*(u*u) for d in range(3) for m,u in zip(mass[d],v[d]))
    def integrated(v):
        terms=[[] for _ in range(size)]
        for d in range(3):
            for i,pair in enumerate(ends[d]):
                if pair:
                    a,b=pair;flux=q[d][i]*v[d][i]
                    if cell_mass[a]:terms[a].append(-flux)
                    if cell_mass[b]:terms[b].append(flux)
        return np.array([math.fsum(t) for t in terms])
    def gradient(p):
        result=[]
        for d in range(3):
            result.append(np.array([(q[d][i]/mass[d][i])*((p[b] if cell_mass[b] else 0)-(p[a] if cell_mass[a] else 0)) if pair else 0 for i,pair in enumerate(ends[d]) for a,b in ([pair] if pair else [(0,0)])]))
        return result
    def ledger(report,before,after,wall,wall_energy,mixing,rounding,roundwork,abswork,absolute,direction,normal_p=0.,normal_e=0.):
        require(report['direction']==direction and report['workspace_bytes']==8*sum(lengths),'transfer/workspace')
        close(report['profile_mass'],physical_mass,physical_mass,'profile mass')
        for d in range(3):
            close(report['mac_mass'][d],math.fsum(mass[d]),physical_mass,'MAC mass')
            close(report['mass_error'][d],math.fsum(mass[d])-physical_mass,physical_mass,'mass error')
        mb=64*EPS*2*physical_mass;close(report['mass_budget'],mb,mb,'mass budget')
        pb,eb=before;pa,ea=after
        for t in range(2):
            scale=absolute[t];budget=64*EPS*scale
            for key,val in [('momentum_before',pb[t]),('momentum_after',pa[t]),('wall_impulse',wall[t]),('rounding_momentum',rounding[t])]:close(report[key][t],val,scale,'transfer '+key)
            close(report['momentum_error'][t],pa[t]-pb[t]-wall[t]-rounding[t],scale,'transfer momentum error')
            close(report['momentum_budget'][t],budget,budget,'transfer momentum budget')
            require(abs(report['momentum_error'][t])<=report['momentum_budget'][t],'transfer momentum gate')
        scale=eb+ea+wall_energy+mixing+abswork;floating=64*EPS*scale;budget=abswork+floating
        for key,val in [('kinetic_before',eb),('kinetic_after',ea),('wall_energy_removed',wall_energy),('mixing_loss',mixing),('rounding_work',roundwork),('energy_error',ea-eb+wall_energy+mixing-roundwork)]:close(report[key],val,scale,'transfer '+key)
        close(report['energy_budget'],budget,budget,'transfer energy budget')
        require(report['mixing_loss']>=0 and abs(report['energy_error'])<=floating and ea-eb+wall_energy<=budget,'transfer energy gates')
        close(report['omitted_normal_momentum'],normal_p,abs(normal_p)+math.fsum(abs(m*v) for m,v in zip(mass[normal],previous[normal])),'omitted normal momentum')
        close(report['omitted_normal_energy'],normal_e,normal_e,'omitted normal energy')
    def profile_measure(v):return ([math.fsum(cell_mass[i]*v[t][i] for i in range(size)) for t in range(2)],math.fsum(.5*cell_mass[i]*v[t][i]**2 for t in range(2) for i in range(size)))
    previous=[np.zeros(length) for length in lengths];previous_export=None;initial_lift=None;max_div=0.;max_dense_difference=0.
    for k,step in enumerate(data['steps']):
        require(step['index']==k and step['physical_time']==0 and step['phase_unchanged'] is True and step['flat_column_mac'] is True,'accepted state/time')
        pub=step['publication'];tr=pub['transfer'];pr=pub['projection'];er=step['export']
        require(pub['before_carrier_version']==k and pub['after_carrier_version']==k+1 and pub['before_volume_version']==12+k and pub['after_volume_version']==13+k and step['pressure_geometry_version']==step['end_geometry_version']==13+k,'shared accepted revisions')
        require(tr['geometry_version']==12+k and er['geometry_version']==13+k,'transfer/export revisions')
        require(pub['before_carrier_id']==pub['after_carrier_id']==step['export_carrier_id']==17 and pub['before_volume_id']==pub['after_volume_id']==tr['geometry_id']==er['geometry_id']==step['export_volume_id']==step['pressure_geometry_id']==step['end_geometry_id']==11 and step['export_carrier_version']==k+1 and step['export_volume_version']==13+k,'complete accepted epoch identities')
        owned=16*sum(lengths)+80*size+48*(size//4);require(pub['owned_array_bytes']==owned and pub['workspace_array_bytes']==8*sum(lengths) and pub['total_array_bytes']==owned+8*sum(lengths),'existing owner memory accounting')
        source=fields(step['input_profiles'],[size,size]);before=fields(step['before_velocity'],lengths);lifted=fields(step['lifted_velocity'],lengths);after=fields(step['after_velocity'],lengths);exported=fields(step['exported_profiles'],[size,size]);pressure=np.asarray(step['pressure'],dtype=float)
        require(pressure.shape==(size,) and np.all(pressure[cell_mass==0]==0),'pressure shape/atmospheric air')
        require(all(np.array_equal(a,b) for a,b in zip(before,previous)),'exact accepted MAC carry')
        if previous_export is not None:require(all(np.array_equal(a,b) for a,b in zip(source,previous_export)),'exact accepted-profile export carry')
        else:
            expected=[np.zeros(size),np.zeros(size)]
            for i in range(size):
                p=coords(i,counts)
                if p[normal]>=wet:continue
                for t in range(2):
                    x=(p[tangents[0]]+.5)/n;z=(p[tangents[1]]+.5)/n
                    value=(math.sin(math.pi*x)*math.cos(math.pi*z) if t==0 else -math.cos(math.pi*x)*math.sin(math.pi*z)) if kind=='curl' else .1*(1+p[tangents[t]])*(1+p[normal])
                    expected[t][i]=float(np.float32(value))
            require(all(np.array_equal(a,b) for a,b in zip(source,expected)),'declared initial parcel field')
        candidate=[np.zeros(length) for length in lengths];mix=[];wall=[[],[]];wall_e=[];roundp=[[],[]];work=[];absolute=[[],[]]
        pb,eb=profile_measure(source)
        for t,d in enumerate(tangents):
            for i in range(size):
                if not cell_mass[i]:require(source[t][i]==0,'dry source');continue
                m=cell_mass[i];u=source[t][i];absolute[t].append(abs(m*u));p=coords(i,counts)
                for boundary in [p[d]==0,p[d]+1==counts[d]]:
                    if boundary:wall[t].append(-.5*m*u);wall_e.append(.25*m*u*u);absolute[t].append(abs(.5*m*u))
            for i,pair in enumerate(ends[d]):
                if not pair:continue
                a,b=pair;ua=source[t][a];ub=source[t][b];raw=ua if ua==ub else .5*(ua+ub);v=float(np.float32(raw));candidate[d][i]=v;m=mass[d][i];roundp[t].append(m*(v-raw));absolute[t].extend([abs(m*v),abs(m*(v-raw))]);mix.extend([.25*m*(ua-raw)**2,.25*m*(ub-raw)**2]);wr=m*((v-raw)*(raw+.5*(v-raw)));work.append(wr)
        require(all(np.array_equal(a,b) for a,b in zip(candidate,lifted)),'actual lift/fixed-wall f32 fields')
        ledger(tr,(pb,eb),([moments(lifted)[d] for d in tangents],energy(lifted)),[math.fsum(v) for v in wall],math.fsum(wall_e),math.fsum(mix),[math.fsum(v) for v in roundp],math.fsum(work),math.fsum(abs(v) for v in work),[math.fsum(v) for v in absolute],'LiftToMac')
        phase_mass=math.fsum(fractions)*math.prod(spacing)
        close(pub['represented_phase_mass'],phase_mass,physical_mass,'represented phase mass')
        close(pub['phase_mass_error'],physical_mass-phase_mass,physical_mass,'phase/momentum mass identity')
        phase_budget=64*EPS*(physical_mass+phase_mass)
        close(pub['phase_mass_budget'],phase_budget,phase_budget,'phase mass budget')
        require(abs(pub['phase_mass_error'])<=pub['phase_mass_budget'],'phase mass gate')
        pbefore=moments(before);ebefore=energy(before)
        for d in range(3):
            close(pub['accepted_momentum_before'][d],pbefore[d],math.fsum(abs(m*u) for m,u in zip(mass[d],before[d])),'accepted momentum')
            prescribed=(pb[tangents.index(d)] if d in tangents else 0)-pbefore[d]
            close(pub['prescribed_momentum_change'][d],prescribed,abs(prescribed)+math.fsum(abs(m*u) for m,u in zip(mass[d],before[d])),'explicit prescribed momentum')
        close(pub['accepted_energy_before'],ebefore,ebefore,'accepted energy');close(pub['prescribed_energy_change'],eb-ebefore,eb+ebefore,'explicit prescribed energy')
        grad=gradient(pressure);dt=.125;raw=[lifted[d]-dt*grad[d] for d in range(3)];stored=[np.asarray(v,dtype=np.float32).astype(float) for v in raw]
        require(all(np.array_equal(a,b) for a,b in zip(stored,after)),'actual mass-compatible pressure f32 correction')
        require(all(np.all(after[d][q[d]==0]==0) for d in range(3)),'sealed wall/inactive velocities')
        rhs=integrated(lifted)/dt;ap=integrated(grad);residual=rhs-ap;active=cell_mass>0
        residual_scale=math.fsum(abs(v) for v in rhs)+math.fsum(abs(v) for v in ap);res_l2=math.sqrt(math.fsum(v*v for v in residual));res_max=float(np.max(np.abs(residual)))
        pred=float(np.max(np.abs(residual[active])*dt/cell_volume[active]));stored_div=float(np.max(np.abs(integrated(after)[active])/cell_volume[active]));max_div=max(max_div,stored_div)
        close(pr['true_residual_l2'],res_l2,residual_scale,'true residual L2');close(pr['true_residual_max'],res_max,residual_scale,'true residual max')
        close(pr['predicted_divergence_max'],pred,residual_scale*dt/float(np.min(cell_volume[active])),'predicted liquid-volume divergence')
        divergence_scale=0.
        for cell in np.flatnonzero(active):
            p=coords(cell,counts);magnitudes=[]
            for d in range(3):
                hi=p.copy();hi[d]+=1;loindex=index(p,face_counts[d]);hiindex=index(hi,face_counts[d])
                magnitudes.extend([abs(q[d][loindex]*after[d][loindex]),abs(q[d][hiindex]*after[d][hiindex])])
            divergence_scale=max(divergence_scale,math.fsum(magnitudes)/cell_volume[cell])
        close(pr['actual_divergence_max'],stored_div,divergence_scale,'actual divergence')
        require(pr['actual_divergence_max']<=1e-5,'native actual divergence gate')
        threshold=max(1e-12,1e-12*math.sqrt(math.fsum(v*v for v in rhs)))
        require(res_l2<=threshold+128*EPS*residual_scale and pred<=1e-9+128*EPS*residual_scale*dt/float(np.min(cell_volume[active])) and stored_div<=1e-5,'unchanged pressure/divergence gates')
        require(0<=pr['iterations']<=4000,'pressure iteration range')
        delta=[raw[d]-lifted[d] for d in range(3)];rounding=[after[d]-raw[d] for d in range(3)];roundwork=math.fsum(m*(r*(u+.5*r)) for d in range(3) for m,r,u in zip(mass[d],rounding[d],raw[d]));abswork=math.fsum(abs(m*(r*(u+.5*r))) for d in range(3) for m,r,u in zip(mass[d],rounding[d],raw[d]));corr=math.fsum(.5*m*(dt*g)**2 for d in range(3) for m,g in zip(mass[d],grad[d]));reswork=-dt*math.fsum(p*v for p,v in zip(pressure,integrated(raw)));epbefore=energy(lifted);epafter=energy(after);scale=epbefore+epafter+corr+abs(reswork)+abswork;floating=64*EPS*scale;budget=max(0.,reswork)+abswork+floating
        for key,value in [('kinetic_before',epbefore),('kinetic_after',epafter),('correction_energy',corr),('residual_work',reswork),('rounding_work',roundwork),('energy_error',epafter-epbefore+corr-reswork-roundwork)]:close(pr[key],value,scale,'projection '+key)
        # The independent B assembly above accumulates six flux parcels. The
        # native diagnostic groups each pair before compensated accumulation.
        # Reconstruct that declared rounding order for a tight budget check;
        # retain the independent residual-work and energy identity checks.
        work_terms=[];work_magnitudes=[]
        for cell in np.flatnonzero(active):
            p=coords(cell,counts);terms=[];magnitudes=[]
            for d in range(3):
                hi=p.copy();hi[d]+=1;li=index(p,face_counts[d]);ri=index(hi,face_counts[d])
                low=float(q[d][li])*float(raw[d][li]);high=float(q[d][ri])*float(raw[d][ri])
                terms.append(low-high);magnitudes.extend([abs(low),abs(high)])
            work_terms.append(-dt*(float(pressure[cell])*compensated(terms)))
            work_magnitudes.append(dt*abs(pressure[cell])*math.fsum(magnitudes))
        native_reswork=compensated(work_terms)
        close(pr['residual_work'],reswork,math.fsum(work_magnitudes),'independent residual pressure work')
        close(pr['residual_work'],native_reswork,abs(native_reswork),'native grouped residual work')
        native_budget=max(0.,native_reswork)+abswork+64*EPS*(epbefore+epafter+corr+abs(native_reswork)+abswork)
        close(pr['energy_budget'],native_budget,native_budget,'projection energy budget');require(abs(pr['energy_error'])<=floating and epafter-epbefore<=native_budget,'projection energy gates')
        for d in range(3):
            impulse=math.fsum(m*(-dt*g) for m,g in zip(mass[d],grad[d]));roundmoment=math.fsum(m*r for m,r in zip(mass[d],rounding[d]));mb=64*EPS*math.fsum(abs(m*o)+abs(m*v)+abs(m*(-dt*g))+abs(m*r) for m,o,v,g,r in zip(mass[d],lifted[d],after[d],grad[d],rounding[d]));mom_before=moments(lifted)[d];mom_after=moments(after)[d];mscale=mb/(64*EPS)
            for key,value in [('momentum_before',mom_before),('momentum_after',mom_after),('pressure_impulse',impulse),('rounding_momentum',roundmoment),('momentum_error',mom_after-mom_before-impulse-roundmoment)]:close(pr[key][d],value,mscale,'projection '+key)
            close(pr['momentum_budget'][d],mb,mb,'projection momentum budget');require(abs(pr['momentum_error'][d])<=pr['momentum_budget'][d],'projection momentum gate')
        if kind=='pulse':
            rows=np.flatnonzero(active);lookup={cell:i for i,cell in enumerate(rows)};A=np.zeros((len(rows),len(rows)))
            for d in range(3):
                for i,pair in enumerate(ends[d]):
                    if not pair:continue
                    a,b=pair;w=q[d][i]**2/mass[d][i]
                    for cell in [a,b]:
                        if active[cell]:A[lookup[cell],lookup[cell]]+=w
                    if active[a] and active[b]:A[lookup[a],lookup[b]]-=w;A[lookup[b],lookup[a]]-=w
            eigen=float(np.min(np.linalg.eigvalsh(A)));require(eigen>0,'SPD atmospheric graph');exact=np.linalg.solve(A,rhs[active]);difference=float(np.max(np.abs(pressure[active]-exact)));max_dense_difference=max(max_dense_difference,difference);require(difference<=res_l2/eigen+128*EPS*(float(np.max(np.abs(exact)))+1),'independent dense pressure solution')
        # Restriction is the mass-adjoint averaging operator, not an inverse.
        expected=[np.zeros(size),np.zeros(size)];mix=[];roundp=[[],[]];work=[];absolute=[[],[]]
        for t,d in enumerate(tangents):
            absolute[t].extend(abs(m*u) for m,u in zip(mass[d],after[d]))
            for i in range(size):
                m=cell_mass[i]
                if not m:continue
                p=coords(i,counts);hi=p.copy();hi[d]+=1;a=after[d][index(p,face_counts[d])];b=after[d][index(hi,face_counts[d])];r=a if a==b else .5*(a+b);v=float(np.float32(r));expected[t][i]=v;roundp[t].append(m*(v-r));absolute[t].extend([abs(m*v),abs(m*(v-r))]);mix.extend([.25*m*(a-r)**2,.25*m*(b-r)**2]);work.append(m*((v-r)*(r+.5*(v-r))))
        require(all(np.array_equal(a,b) for a,b in zip(expected,exported)),'actual mass-adjoint f32 restriction')
        previous=after
        tangential_before=([moments(after)[d] for d in tangents],math.fsum(.5*m*u*u for d in tangents for m,u in zip(mass[d],after[d])))
        normalp=moments(after)[normal];normale=math.fsum(.5*m*u*u for m,u in zip(mass[normal],after[normal]));ledger(er,tangential_before,profile_measure(exported),[0.,0.],0.,math.fsum(mix),[math.fsum(v) for v in roundp],math.fsum(work),math.fsum(abs(v) for v in work),[math.fsum(v) for v in absolute],'RestrictTangents',normalp,normale)
        previous_export=exported
        if initial_lift is None:initial_lift=lifted
    rms=0.
    if kind=='curl':
        terms=[]
        for d in range(3):
            for i,u in enumerate(previous[d]):
                p=coords(i,face_counts[d]);value=0.
                if d in tangents and p[normal]<wet and p[d] not in [0,counts[d]]:
                    x=(p[tangents[0]]+(0 if d==tangents[0] else .5))/n;z=(p[tangents[1]]+(0 if d==tangents[1] else .5))/n
                    value=math.sin(math.pi*x)*math.cos(math.pi*z) if d==tangents[0] else -math.cos(math.pi*x)*math.sin(math.pi*z)
                terms.append(mass[d][i]*(u-value)**2)
        rms=math.sqrt(math.fsum(terms)/(3*physical_mass))
    for filename,velocity in [('initial.png',initial_lift),('final.png',previous)]:
        d=tangents[0];pixels=[]
        for j in range(n):
            row=[]
            for i in range(n+1):
                p=[0]*3;p[normal]=wet-1;p[d]=i;p[tangents[1]]=j;value=velocity[d][index(p,face_counts[d])];require(-1<=value<=1,'raster range');row.append(math.floor((value+1)*127.5+.5))
            pixels.append(row)
        with Image.open(dest/filename) as image:require(image.mode=='L' and np.array_equal(np.asarray(image),np.array(pixels,dtype=np.uint8)),'native velocity raster')
    require({p.name for p in dest.iterdir()}=={'case.json','initial.png','final.png'},'case inventory')
    return {'name':name,'publications':steps,'initial_prescribed_energy':data['steps'][0]['publication']['transfer']['kinetic_before'],'final_energy':data['steps'][-1]['publication']['projection']['kinetic_after'],'max_stored_divergence':max_div,'max_dense_pressure_difference':max_dense_difference,'curl_weighted_rms':rms}
def verify(root):
    root=Path(root);require({p.name for p in root.iterdir()}==set(CASES),'complete case inventory');results=[verify_case(root/name,name) for name in CASES];errors=[r['curl_weighted_rms'] for r in results if r['name'].startswith('curl')];require(all(3.8<a/b<4.1 for a,b in zip(errors,errors[1:])),'restricted instantaneous transfer refinement');return {'cases':len(results),'publications':sum(r['publications'] for r in results),'results':results}
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('directory');parser.add_argument('--summary');args=parser.parse_args();result=verify(args.directory)
    if args.summary:Path(args.summary).write_text(json.dumps(result,indent=2)+'\n')
    print('PASS',result['cases'],'cases /',result['publications'],'existing-owner publications; independent M/B/projection/epochs/ledgers/pixels/refinement')
