#!/usr/bin/env python3
"""Fresh rational symmetric-stress and force oracle against actual Rust actions.
All velocity data are explicitly manufactured, file-bound local test inputs.
No data are generated inside the Rust gradient, no physical load is accepted.
"""
import argparse
from fractions import Fraction as F
import hashlib
import itertools
import json
import math
from pathlib import Path
import struct
import subprocess

CAP=16_000_000

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(cmd,ok=True):
    r=subprocess.run(list(map(str,cmd)),capture_output=True,text=True)
    assert (r.returncode==0)==ok,(cmd,r.stdout,r.stderr)
    return r

def encode_state(g,v,refs,mu):
    """Independent encoding of the published owned-state wire-v1 contract."""
    b=bytearray(b'RHEONOS1'+struct.pack('<I',1))
    def u(n):b.extend(struct.pack('<Q',n))
    def x(v):b.extend(struct.pack('<d',v))
    def ref(i):u(i+1);u(0);b.extend(bytes.fromhex(refs[i]))
    for n in g['counts']:u(n)
    for key in ['origin','spacing','lower','upper']:
        for value in g[key]:x(value)
    for value in g['stamp']:u(value)
    x(g['tolerance'])
    u(len(g['vertices']))
    for vertex in g['vertices']:
        for value in vertex:x(value)
    u(len(g['triangles']))
    for tri in g['triangles']:
        for index in tri:u(index)
    u(g['components'])
    for values in [g['volumes'],*g['areas']]:
        u(len(values))
        for value in values:x(value)
    u(len(g['labels']))
    for value in g['labels']:u(value)
    b.extend(bytes([1,2,1,0,0]))
    for value in [1.,mu,0.,0.]:x(value)
    u(0);ref(0);ref(1);b.append(2);ref(2);b.append(1);ref(3);x(0.);x(0.)
    b.extend(bytes([1,9]+[0]*9))
    for values in v:u(len(values))
    for values in v:
        for value in values:x(value)
    return b

def index(c,n):return c[0]+n[0]*(c[1]+n[1]*c[2])
def coordinate(f,n):return [f%n[0],(f//n[0])%n[1],f//(n[0]*n[1])]
def plane(g,a,i):return g['origin'][a]+i*g['spacing'][a]
def center(g,a,i):return g['origin'][a]+(i+.5)*g['spacing'][a]
def face_counts(g,a):n=g['counts'].copy();n[a]+=1;return n
def face_position(g,a,p):return [plane(g,d,p[d]) if d==a else center(g,d,p[d]) for d in range(3)]
def active(g,a,f):
    p=coordinate(f,face_counts(g,a))
    return 0<p[a]<g['counts'][a] and g['areas'][a][f]>0

def normal_sites(g):
    return [('N',a,*coordinate(c,g['counts'])) for c,volume in enumerate(g['volumes']) if volume>0 for a in range(3)]
def interior_sites():
    return [('N',a,1,1,1) for a in range(3)]+[('C',a,b,1,1,1,q) for a in range(3) for b in range(3) if a!=b for q in range(4)]
def flat_sites(g):
    n=g['counts'][0];lo,hi=n//3,2*n//3;middle=(lo+hi)//2
    result=[]
    for a in range(3):
        for b in range(3):
            if a==b:continue
            for wall in [lo,hi]:
                p=[lo]*3;p[a]=middle;p[b]=wall
                bit=1 if b<a else 2;varying=3^bit
                for q in [bit if wall==hi else 0,(bit if wall==hi else 0)|varying]:
                    result.extend([('C',a,b,*p,q),('C',b,a,*p,q)])
    return result

def expected_row(g,site):
    """Derive topology/endpoint coordinates independently from the query."""
    if site[0]=='N':
        _,a,*cell=site;b=a;upper=cell.copy();upper[a]+=1
        pairs=[]
        for p in [cell,upper]:
            f=index(p,face_counts(g,a));kind='VelocityFace'
            if p[a] in [0,g['counts'][a]]:kind='StationaryOuterNormal'
            elif g['areas'][a][f]==0:kind='StationarySolid'
            pairs.append((face_position(g,a,p),kind,f if kind=='VelocityFace' else None))
        weight=g['volumes'][index(cell,g['counts'])];boundary='Normal'
    else:
        _,a,b,*tail=site;p=tail[:3];q=tail[3];axes=sorted([a,b]);cells={}
        for sector in range(4):
            cell=p.copy()
            for d,bit in zip(axes,[1,2]):cell[d]-=not bool(sector&bit)
            if g['volumes'][index(cell,g['counts'])]>0:cells[sector]=cell
        assert q in cells and len(cells) in [2,4]
        remaining=3-a-b;cell=cells[q]
        widths=[abs(center(g,d,cell[d])-plane(g,d,p[d])) for d in axes]
        weight=(widths[0]*widths[1])*(plane(g,remaining,p[remaining]+1)-plane(g,remaining,p[remaining]))
        lower=p.copy();lower[b]-=1
        pairs=[(face_position(g,a,pc),'VelocityFace',index(pc,face_counts(g,a))) for pc in [lower,p]]
        boundary='Interior'
        if len(cells)==2:
            varying=min(cells)^max(cells)
            normal=axes[1] if varying==1 else axes[0]
            if normal==b:
                boundary='FlatWallRay';bit=1 if b==axes[0] else 2;side=1 if min(cells)&bit else 0;trace=1-side
                position=pairs[side][0].copy();position[b]=plane(g,b,p[b]);pairs[trace]=(position,'StationarySolid',None)
            else:
                boundary='FlatStationaryTrace'
                pairs=[(face_position(g,a,pc),'StationarySolid',None) for pc in [lower,p]]
    positions=[pair[0] for pair in pairs]
    separation=positions[1][b]-positions[0][b]
    assert separation>0
    inverse=1./separation
    return dict(component=a,derivative=b,boundary=boundary,weight=weight,endpoints=[dict(position=position,source=source,face=f,coefficient=coefficient) for (position,source,f),coefficient in zip(pairs,[-inverse,inverse])])

def pair_blocks(sites):
    """Fresh constitutive blocks from requests, never native block exports."""
    seen={};blocks=[]
    for i,s in enumerate(sites):
        assert s not in seen;seen[s]=i
    for i,s in enumerate(sites):
        if s[0]=='N':blocks.append((i,))
        else:
            reverse=('C',s[2],s[1],*s[3:]);j=seen[reverse]
            if i<j:blocks.append((i,j))
    return blocks

def validate(g,sites,v,mu,native,ideal_slopes=None):
    assert native['qualification']=='Unqualified' and not native['pressure_available'] and not native['physical_load_qualified']
    assert native['dynamic_viscosity']==mu and native['velocity']==v
    assert native['cap_bytes']==CAP and max(native[k] for k in ['retained_action_payload_bytes','parse_phase_payload_bytes','constructor_peak_payload_bytes'])<=CAP
    rows=[expected_row(g,s) for s in sites]
    assert len(native['rows'])==len(rows)
    for expected,row in zip(rows,native['rows']):
        for k in ['component','derivative','weight','boundary','endpoints']:assert expected[k]==row[k]
        assert row['active_term_count']==sum(e['face'] is not None for e in expected['endpoints'])
    terms=[];values=[];affine_defects=[]
    for i,row in enumerate(rows):
        t={(row['component'],e['face']):F(e['coefficient']) for e in row['endpoints'] if e['face'] is not None};terms.append(t)
        values.append(sum((c*F(v[a][f]) for (a,f),c in t.items()),F(0)))
        if ideal_slopes is not None:
            lo,hi=row['endpoints'];a=row['component'];b=row['derivative']
            lower=F(v[a][lo['face']]) if lo['face'] is not None else F(0);upper=F(v[a][hi['face']]) if hi['face'] is not None else F(0)
            quotient=(upper-lower)/(F(hi['position'][b])-F(lo['position'][b]))
            assert quotient==ideal_slopes[i],(sites[i],quotient,ideal_slopes[i])
            affine_defects.append(float(abs(values[i]-quotient)))
    stress=[F(0)]*len(rows);force=[[F(0)]*len(a) for a in v];power=F(0);matrix={}
    for block in pair_blocks(sites):
        i=block[0];w=F(rows[i]['weight']);E=dict(terms[i]);gamma=values[i]
        if len(block)==1:factor=2*F(mu)*w;stress[i]=2*F(mu)*gamma
        else:
            j=block[1];assert rows[i]['weight']==rows[j]['weight'];gamma+=values[j];factor=F(mu)*w
            for f,c in terms[j].items():E[f]=E.get(f,F(0))+c
            stress[i]=stress[j]=F(mu)*gamma
        power+=factor*gamma*gamma
        for (a,f),c in E.items():
            force[a][f]-=factor*c*gamma
            for h,d in E.items():matrix[((a,f),h)]=matrix.get(((a,f),h),F(0))+factor*c*d
    # Independent exact quadratic matrix route, beyond the directed transpose route.
    assert all(value==matrix.get((j,i),F(0)) for (i,j),value in matrix.items())
    quadratic=sum((F(v[a][f])*A*F(v[b][h]) for ((a,f),(b,h)),A in matrix.items()),F(0))
    assert quadratic==power and power>=0
    for a,arr in enumerate(force):
        for f,expected in enumerate(arr):
            Au=sum((A*F(v[b][h]) for ((c,k),(b,h)),A in matrix.items() if (c,k)==(a,f)),F(0))
            assert expected==-Au
    actual_work=sum((F(u)*f for us,fs in zip(v,force) for u,f in zip(us,fs)),F(0));assert actual_work==-power
    def error(actual,expected):return abs(float(F(actual)-expected))
    errs=dict(gather=max([error(a,b) for a,b in zip(native['gradient'],values)]+[0.]),stress=max([error(a,b) for a,b in zip(native['stress'],stress)]+[0.]),force=max([error(a,b) for aa,bb in zip(native['force'],force) for a,b in zip(aa,bb)]+[0.]),dissipation=error(native['work']['dissipation'],power),potential=error(native['work']['rayleigh_potential'],power/2),work=error(native['work']['force_work'],actual_work))
    scale=1+max([abs(float(x)) for x in values+stress]+[abs(float(power))]+[abs(float(x)) for arr in force for x in arr])
    assert max(errs.values())<2e-12*scale,errs
    for a,arr in enumerate(native['force']):assert all(arr[f]==0 for f in range(len(arr)) if not active(g,a,f))
    zero_rows=[i for i,r in enumerate(rows) if all(e['face'] is None for e in r['endpoints'])]
    return dict(rows=len(rows),blocks=len(pair_blocks(sites)),zero_rows=len(zero_rows),nonzero_stress_on_zero_rows=sum(native['stress'][i]!=0 for i in zero_rows),nonzero_gather=sum(x!=0 for x in native['gradient']),nonzero_stress=sum(x!=0 for x in native['stress']),nonzero_force=sum(x!=0 for arr in native['force'] for x in arr),cross_force_on_zero_velocity_component=any(all(x==0 for x in v[a]) and any(x!=0 for x in native['force'][a]) for a in range(3)),exact_rational_matrix_symmetric=True,exact_rational_matrix_action=True,exact_rational_power_nonnegative=True,exact_rational_negative_work=True,exact_rational_dissipation=str(power),exact_real_affine_quotients_checked=ideal_slopes is not None,stored_affine_coefficient_defect_max=max(affine_defects+[0.]),native_errors=errs,unenclosed_work_defect=native['work']['unenclosed_defect'],combined_payload_bytes=native['combined_payload_bytes'],constructor_peak_payload_bytes=native['constructor_peak_payload_bytes'],retained_action_payload_bytes=native['retained_action_payload_bytes'],parse_phase_payload_bytes=native['parse_phase_payload_bytes'])

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--repo',required=True,type=Path);p.add_argument('--binary',required=True,type=Path);p.add_argument('--out',required=True,type=Path);a=p.parse_args()
    repo,binary,out=a.repo.resolve(),a.binary.resolve(),a.out.resolve();assert not out.is_relative_to(repo);out.mkdir(parents=True,exist_ok=False)
    geometries={}
    for kind in ['3','6','12','anisotropic','nonmidpoint']:
        path=out/('geometry-'+kind+'.json');run([binary,'describe_geometry',kind,path]);geometries[kind]=json.loads(path.read_text())
    M=[[F(1,2),F(-1),F(1,4)],[F(3,4),F(3,2),F(-1,2)],[F(-1,4),F(1,2),F(-1)]]
    skew=[[F(0),F(-1,2),F(1,4)],[F(1,2),F(0),F(-3,4)],[F(-1,4),F(3,4),F(0)]]
    results=[];sources={};mu=2.5
    cases=[('3','general'),('6','affine'),('anisotropic','affine'),('6','rotation'),('6','flat'),('anisotropic','flat'),('nonmidpoint','flat'),('6','general'),('12','general'),('6','cross-a'),('6','cross-b')]
    for kind,mode in cases:
        folder=out/(kind+'-'+mode);folder.mkdir();g=geometries[kind]
        sites=normal_sites(g) if kind=='3' else flat_sites(g) if mode=='flat' else interior_sites()
        if mode=='general' and kind!='3':sites+=flat_sites(g)
        if mode.startswith('cross-'):sites=[('C',0,1,1,1,1,0),('C',1,0,1,1,1,0)]
        v=[[0.]*len(arr) for arr in g['areas']];slopes=None
        if mode in ['affine','rotation'] or mode.startswith('cross-'):
            matrix=skew if mode=='rotation' else M
            for component,arr in enumerate(v):
                for face in range(len(arr)):
                    if active(g,component,face) and (not mode.startswith('cross-') or component==(0 if mode=='cross-a' else 1)):
                        pos=face_position(g,component,coordinate(face,face_counts(g,component)))
                        arr[face]=float(F(0 if mode=='rotation' else 2)+sum((matrix[component][d]*F(pos[d]) for d in range(3)),F(0)))
            if not mode.startswith('cross-'):slopes=[matrix[s[1]][s[1] if s[0]=='N' else s[2]] for s in sites]
        elif mode=='flat':
            slopes=[];assigned={}
            for site in sites:
                row=expected_row(g,site);slope=F(2**row['component']) if row['boundary']=='FlatWallRay' else F(0);slopes.append(slope)
                for e in row['endpoints']:
                    if e['face'] is not None:
                        trace=next(t for t in row['endpoints'] if t['face'] is None)
                        value=float(slope*(F(e['position'][row['derivative']])-F(trace['position'][row['derivative']])))
                        key=row['component'],e['face'];assert key not in assigned or assigned[key]==value;assigned[key]=value;v[key[0]][key[1]]=value
        else:
            for component,arr in enumerate(v):
                for face in range(len(arr)):
                    if active(g,component,face):arr[face]=float(F((face*17+component*3)%29-14,16))
        vf=folder/'supplied-velocities.json';vf.write_text(json.dumps(v,separators=(',',':'))+'\n')
        bodies=[('problem',f'Explicit manufactured local test; SI density 1, dynamic viscosity {mu}; transient Stokes declaration. No physical solution/load/error acceptance. Geometry '+json.dumps({k:g[k] for k in ['counts','origin','spacing','lower','upper']})+'\n'),('boundary','Stationary no-slip solid and sealed free-slip outer; inactive normal faces zero; no extra wall observations.\n'),('initial',json.dumps(dict(kind='supplied_initial_data',time_seconds=0,velocity_file=vf.name,velocity_sha256=sha(vf),manufactured_local_test_only=True,errors='all_unknown'))+'\n'),('forcing','Explicit no forcing on [0,0] seconds; no advancement.\n')]
        refs=[]
        for name,body in bodies:f=folder/(name+'.txt');f.write_text(body);refs.append(sha(f))
        checkpoint=folder/'supplied-state.rheon-os1';checkpoint.write_bytes(encode_state(g,v,refs,mu))
        request=folder/'requests.txt';request.write_text(''.join(' '.join(map(str,s))+'\n' for s in sites))
        run([binary,kind,checkpoint,request,folder/'native']);record=folder/'native/viscous-record.json';native=json.loads(record.read_text());r=validate(g,sites,v,mu,native,slopes)
        if mode=='rotation':assert r['nonzero_gather']>0 and r['nonzero_stress']==r['nonzero_force']==0 and native['work']['dissipation']==0
        if mode.startswith('cross-'):assert r['cross_force_on_zero_velocity_component']
        r.update(kind=kind,mode=mode,record_sha256=sha(record),state_sha256=sha(checkpoint),references=refs);results.append(r)
        sources[str(folder.relative_to(out))]={f.name:sha(f) for f in [vf,request,checkpoint,*[folder/(name+'.txt') for name,_ in bodies]]}
    folder=out/'6-general';negatives=[]
    for name,text in [('coarse-fine','CF\n'),('corner','C 0 1 2 2 2 0\n'),('outer-cross','C 0 1 0 1 0 0\n'),('same-axis','C 0 0 1 1 1 0\n'),('missing-partner','C 0 1 1 1 1 0\n'),('duplicate','N 0 1 1 1\nN 0 1 1 1\n')]:
        f=out/(name+'-request.txt');f.write_text(text);dest=out/(name+'-unexpected-output');r=run([binary,'6',folder/'supplied-state.rheon-os1',f,dest],False);assert not dest.exists();negatives.append(dict(case=name,native_exit=r.returncode,native_error=r.stderr.strip()))
    files=['src/obstacle_viscous.rs','src/obstacle_gradient.rs','src/obstacle_state.rs','src/lib.rs','examples/obstacle_viscous.rs','tests/obstacle_viscous_contract.rs','research/obstacle-viscous/ViscousStress.lean','research/obstacle-viscous/check_proofs.py','tools/check_obstacle_viscous.py','docs/owned-state-viscous-stress-force-20261009.md']
    report=dict(source_head=run(['git','-C',repo,'rev-parse','HEAD']).stdout.strip(),sources={f:sha(repo/f) for f in files},binary_sha256=sha(binary),cases=results,input_sources=sources,actual_native_refusals=negatives,all_checks_passed=True,qualification='Unqualified',hard_cap_bytes=CAP,physical_load_convergence_accepted=False,accepted_dynamics_authorized=False,exact_real_scope='partial requested symmetric stress/potential/matrix/work only; no continuum or IEEE enclosure; native observation tolerance only')
    path=out/'viscous-oracle-report.json';path.write_text(json.dumps(report,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(report=str(path),sha256=sha(path),cases=len(results),all_checks_passed=True)))
if __name__=='__main__':main()
