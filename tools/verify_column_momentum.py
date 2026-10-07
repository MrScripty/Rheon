from generated_fixtures import fixture
"""Independent interval-overlap oracle for prescribed geometry remap evidence.
No assertion-dependent gates; evaluate every native stored-f32 field and ledger.
"""
import argparse
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
EPS = np.finfo(float).eps
CASES = {f'{kind}-{axis}': (kind, d, 4, steps) for d, axis in enumerate('xyz')
         for kind, steps in [('exchange', 32), ('constant', 8), ('identity', 1)]}
CASES.update({f'affine-n{n}': ('affine', 1, n+2, 1) for n in [8, 16, 32, 64]})
def require(ok, message):
    if not ok:
        raise ValueError(message)
def finite_tree(value):
    if isinstance(value, dict):
        for x in value.values(): finite_tree(x)
    elif isinstance(value, list):
        for x in value: finite_tree(x)
    elif isinstance(value, (int, float)):
        require(math.isfinite(value), 'nonfinite record')
def close(a, b, scale, label):
    require(abs(a-b) <= 128*EPS*scale, label)
def slabs(height, h):
    n = math.floor(height)+(height % 1 > .5)
    return [(j*h, (j+1)*h if j+1 < n else height*h) for j in range(n)]
def overlap(a, b):
    return max(0., min(a[1], b[1])-max(a[0], b[0]))
def verify_case(dest, name):
    data = json.loads((dest/'case.json').read_text()); finite_tree(data)
    kind, d, n, steps = CASES[name]
    counts = [1]*3; counts[d] = n; counts[(d+1) % 3] = 2
    spacing = [.5, .25, .75]; spacing[d] = 1/(n-1.75) if kind == 'affine' else 1.
    require(data['name'] == name and data['kind'] == kind and data['normal'] == d, 'case identity')
    require(data['counts'] == counts and data['spacing'] == spacing and data['density'] == 3., 'geometry/units')
    require(data['field_kind'] == 'cell_lattice_lumped_tangential_parcel_means' and
            data['pressure_published'] is False and data['physical_time_claimed'] is False and
            data['raster_range'] == [-4., 4.], 'model scope')
    require(len(data['steps']) == steps, 'complete intervals')
    size = math.prod(counts); h = spacing[d]; area = math.prod(spacing[t] for t in range(3) if t != d); rhoa = 3*area
    def cell(c, j):
        p = [0]*3; r = c
        for t in range(3):
            if t != d: p[t], r = r % counts[t], r // counts[t]
        p[d] = j
        return p[0]+counts[0]*(p[1]+counts[1]*p[2])
    heights = [n-1.75]*2 if kind == 'affine' else [2.75, 2.25]
    initial = np.zeros((2, size), dtype=np.float32)
    for c in range(2):
        for j, interval in enumerate(slabs(heights[c], h)):
            value = .25 if kind == 'constant' else 1+sum(interval)/2 if kind == 'affine' else ([0,1,3] if c == 0 else [0,-1])[j]
            initial[:, cell(c,j)] = [value, -.5*value]
    previous = initial.copy(); previous_heights = heights
    rms = 0.
    for index, step in enumerate(data['steps']):
        r = step['report']
        require(step['index'] == index and r['before_version'] == index and r['after_version'] == index+1, 'stamp/index')
        require(r['workspace_bytes'] == 24*size, 'workspace accounting')
        oldh, newh = step['before_heights'], step['after_heights']
        require(oldh == previous_heights and newh == ([v+.5 for v in oldh] if kind == 'affine' else oldh if kind == 'identity' else oldh[::-1]), 'geometry carry/transition')
        old = np.asarray(step['before'], dtype=np.float32); new = np.asarray(step['after'], dtype=np.float32); donors = np.asarray(step['donor'], dtype=np.float32)
        require(old.shape == new.shape == (2,size) and donors.shape == (2,2), 'field shape')
        require(np.array_equal(old, previous), 'exact stored carry')
        donor_expected = np.zeros((2,2), dtype=np.float32)
        for c in range(2):
            if kind == 'affine':
                value = 1+.5*(oldh[c]+newh[c])*h; donor_expected[c] = [value,-.5*value]
            elif newh[c] > oldh[c]: donor_expected[c] = old[:,cell(1-c,len(slabs(oldh[1-c],h))-1)]
        require(np.array_equal(donors, donor_expected), 'explicit donor cap origin')
        masses = {label: [] for label in ['before','after','added','removed']}
        moments = {label: [[], []] for label in ['before','after','added','removed','rounding']}
        energies = {label: [] for label in ['before','after','added','removed']}
        mixing=[]; work=[]; expected = np.zeros((2,size), dtype=np.float32); rms_terms=[]
        active=deactive=0
        for c in range(2):
            os, ns = slabs(oldh[c],h), slabs(newh[c],h)
            active += max(0,len(ns)-len(os)); deactive += max(0,len(os)-len(ns))
            require(np.all(old[:,[cell(c,j) for j in range(len(os),n)]] == 0), 'dry input')
            for i, a in enumerate(os):
                mass=rhoa*(a[1]-a[0]); removed=rhoa*overlap(a,(newh[c]*h,max(oldh[c],newh[c])*h))
                masses['before'].append(mass);masses['removed'].append(removed)
                for t in range(2):
                    u=float(old[t,cell(c,i)])
                    moments['before'][t].append(mass*u);moments['removed'][t].append(removed*u)
                    energies['before'].append(.5*mass*u*u);energies['removed'].append(.5*removed*u*u)
            for j, b in enumerate(ns):
                mass=rhoa*(b[1]-b[0]); imported=rhoa*overlap(b,(oldh[c]*h,max(oldh[c],newh[c])*h))
                masses['after'].append(mass);masses['added'].append(imported)
                parcels=[(rhoa*overlap(a,b), old[:,cell(c,i)].astype(float)) for i,a in enumerate(os) if overlap(a,b)>0]
                if imported: parcels.append((imported,donors[c].astype(float)))
                close(math.fsum(w for w,u in parcels),mass,2*mass,'overlap mass partition')
                for t in range(2):
                    same_value=all(float(u[t])==float(parcels[0][1][t]) for w,u in parcels)
                    candidate=float(parcels[0][1][t]) if same_value else math.fsum(w*float(u[t]) for w,u in parcels)/mass
                    stored=float(new[t,cell(c,j)]);expected[t,cell(c,j)]=np.float32(candidate)
                    low=min(float(u[t]) for w,u in parcels); high=max(float(u[t]) for w,u in parcels)
                    require(low <= candidate <= high and low <= stored <= high,'convex donor bounds')
                    require(stored == 0 or abs(stored) >= np.finfo(np.float32).tiny,'normal stored scale')
                    delta=stored-candidate
                    moments['after'][t].append(mass*stored);moments['added'][t].append(imported*float(donors[c,t]));moments['rounding'][t].append(mass*delta)
                    energies['after'].append(.5*mass*stored*stored);energies['added'].append(.5*imported*float(donors[c,t])**2)
                    mixing.extend(.5*w*(float(u[t])-candidate)**2 for w,u in parcels)
                    work.append(mass*delta*(candidate+.5*delta))
                    if kind == 'affine': rms_terms.append(mass*(stored-(1+sum(b)/2)*(1 if t==0 else -.5))**2)
        require(np.array_equal(new,expected),'actual f32 overlap update/dry output')
        require((r['activated_nodes'],r['deactivated_nodes']) == (active,deactive),'support activation')
        m={key:math.fsum(values) for key,values in masses.items()};e={key:math.fsum(values) for key,values in energies.items()}
        p={key:[math.fsum(v) for v in values] for key,values in moments.items()}
        mass_scale=sum(m.values());energy_scale=sum(e.values())+sum(mixing)+math.fsum(abs(v) for v in work)
        for key in m: close(r['mass_'+key],m[key],mass_scale,'mass ledger '+key)
        me=m['after']-m['before']-m['added']+m['removed'];mb=64*EPS*mass_scale
        close(r['mass_error'],me,mass_scale,'mass error');close(r['mass_budget'],mb,mb,'mass budget');require(abs(r['mass_error'])<=r['mass_budget'],'mass gate')
        for t in range(2):
            scale=math.fsum(abs(v) for values in moments.values() for v in values[t]);budget=64*EPS*scale
            for key in p: close(r['rounding_momentum' if key=='rounding' else 'momentum_'+key][t],p[key][t],scale,'momentum '+key)
            error=p['after'][t]-p['before'][t]-p['added'][t]+p['removed'][t]-p['rounding'][t]
            close(r['momentum_error'][t],error,scale,'momentum error')
            # Rounding-work reconstruction varies at binary64 last bits. The
            # declared budget must still equal the native magnitudes tightly.
            declared_scale=sum(abs(r['momentum_'+key][t]) for key in ['before','after','added','removed'])+abs(r['rounding_momentum'][t])
            require(r['momentum_budget'][t] <= budget*(1+128*EPS) and r['momentum_budget'][t]>=64*EPS*declared_scale*(1-128*EPS),'momentum budget')
            require(abs(r['momentum_error'][t])<=r['momentum_budget'][t],'momentum gate')
        for key in e: close(r['kinetic_'+key],e[key],energy_scale,'kinetic ledger '+key)
        loss=math.fsum(mixing);roundwork=math.fsum(work);absolute_work=math.fsum(abs(v) for v in work)
        close(r['mixing_loss'],loss,energy_scale,'mixing loss');close(r['rounding_work'],roundwork,energy_scale,'rounding work')
        err=e['after']-e['before']-e['added']+e['removed']+loss-roundwork
        close(r['energy_error'],err,energy_scale,'energy identity')
        # Use a scale-aware binary64 allowance for independent candidate
        # evaluation, not an accuracy relaxation of the production gate.
        close(r['energy_budget'],absolute_work+64*EPS*energy_scale,energy_scale,'energy budget')
        require(r['mixing_loss']>=0 and abs(r['energy_error'])<=64*EPS*energy_scale and e['after']-e['before']-e['added']+e['removed']<=r['energy_budget']+128*EPS*energy_scale,'energy gates')
        if kind in ['exchange','constant']:
            close(m['added'],m['removed'],mass_scale,'closed exchanged mass')
            close(e['added'],e['removed'],energy_scale,'closed exchanged energy')
            for t in range(2):close(p['added'][t],p['removed'][t],math.fsum(abs(v) for values in moments.values() for v in values[t]),'closed exchanged momentum')
        if kind in ['identity','constant']:require(loss==0.,'identity/constant mixing')
        if rms_terms:rms=math.sqrt(math.fsum(rms_terms)/(2*m['after']))
        previous=new; previous_heights=newh
    for filename,fields in [('initial.png',initial),('final.png',previous)]:
        pixels=np.array([[int(math.floor((float(fields[t,cell(c,j)])+4)*255/8+.5)) for c in range(2) for t in range(2)] for j in reversed(range(n))],dtype=np.uint8)
        with Image.open(dest/filename) as img:require(img.mode=='L' and np.array_equal(np.asarray(img),pixels),'native raster '+filename)
    require({p.name for p in dest.iterdir()} == {'case.json','initial.png','final.png'},'case inventory')
    return {'name':name,'intervals':steps,'final_energy':data['steps'][-1]['report']['kinetic_after'],'affine_weighted_rms':rms}
def verify(root):
    root=Path(root);require({p.name for p in root.iterdir()}==set(CASES),'complete case inventory')
    results=[verify_case(root/name,name) for name in CASES]
    affine=[r['affine_weighted_rms'] for r in results if r['name'].startswith('affine')]
    require(all(b<a and a/b>2.5 for a,b in zip(affine,affine[1:])),'restricted affine refinement')
    return {'cases':len(results),'intervals':sum(r['intervals'] for r in results),'results':results}
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('directory');parser.add_argument('--summary');args=parser.parse_args();result=verify(args.directory)
    if args.summary:Path(args.summary).write_text(json.dumps(result,indent=2)+'\n')
    print('PASS',result['cases'],'cases /',result['intervals'],'native remaps, overlap/ledgers/f32/pixels/refinement')
