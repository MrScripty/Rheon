#!/usr/bin/env python3
"""Fresh rational finite-row oracle against actual Rust owned-state actions.
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
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'research'))
from check_proof_audit import require_external_output

CAP=16_000_000

def require(condition, message="oracle validation failed"):
    if not condition:
        raise ValueError(message)

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(cmd,ok=True):
    r=subprocess.run(list(map(str,cmd)),capture_output=True,text=True)
    require((r.returncode==0)==ok, (cmd,r.stdout,r.stderr))
    return r

def encode_state(g,v,refs):
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
    for value in [1.,1.,0.,0.]:x(value)
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
        require(q in cells and len(cells) in [2,4])
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
    require(separation>0)
    inverse=1./separation
    return dict(component=a,derivative=b,boundary=boundary,weight=weight,endpoints=[dict(position=position,source=source,face=f,coefficient=coefficient) for (position,source,f),coefficient in zip(pairs,[-inverse,inverse])])

def validate_record(g,sites,v,q,native,expected_slopes=None):
    require(native['qualification']=='Unqualified' and not native['pressure_available'] and not native['physical_load_qualified'])
    require(native['cap_bytes']==CAP and max(native['retained_action_payload_bytes'],native['parse_phase_payload_bytes'])<=CAP)
    require(native['velocity']==v and native['row_test']==q)
    require(len(native['rows'])==len(sites))
    gather=[];transpose=[[F(0)]*len(values) for values in v];row_pairing=F(0);ideal_errors=[]
    for i,(site,row,test) in enumerate(zip(sites,native['rows'],q)):
        expected=expected_row(g,site)
        for key in ['component','derivative','boundary','weight','endpoints']:require(row[key]==expected[key], (site,key,row[key],expected[key]))
        require(row['active_term_count']==sum(e['source']=='VelocityFace' for e in expected['endpoints']))
        a=row['component'];value=F(0)
        for e in row['endpoints']:
            if e['source']=='VelocityFace':
                value+=F(e['coefficient'])*F(v[a][e['face']])
                transpose[a][e['face']]+=F(row['weight'])*F(test)*F(e['coefficient'])
        gather.append(value);row_pairing+=F(row['weight'])*value*F(test)
        if expected_slopes is not None:
            lo,hi=row['endpoints'];uminus=F(v[a][lo['face']]) if lo['face'] is not None else F(0);uplus=F(v[a][hi['face']]) if hi['face'] is not None else F(0)
            ideal=(uplus-uminus)/(F(hi['position'][row['derivative']])-F(lo['position'][row['derivative']]))
            require(ideal==expected_slopes[i], (site,ideal,expected_slopes[i]))
            ideal_errors.append(float(abs(value-expected_slopes[i])))
    face_pairing=sum((F(value)*adj for values,adjs in zip(v,transpose) for value,adj in zip(values,adjs)),F(0))
    require(row_pairing==face_pairing)
    def error(a,b):return abs(float(F(a)-b))
    grad_error=max([error(a,b) for a,b in zip(native['gradient'],gather)]+[0.])
    trans_error=max([error(a,b) for values,expected in zip(native['transpose'],transpose) for a,b in zip(values,expected)]+[0.])
    work_error=max(error(native['work']['row_pairing'],row_pairing),error(native['work']['face_pairing'],face_pairing))
    scale=1.+max([abs(float(x)) for x in gather]+[abs(float(row_pairing))]+[abs(float(x)) for values in transpose for x in values])
    require(max(grad_error,trans_error,work_error)<2e-12*scale)
    for a,values in enumerate(native['transpose']):
        require(all(values[f]==0 for f in range(len(values)) if not active(g,a,f)))
    return dict(rows=len(sites),zero_rows=sum(r['active_term_count']==0 for r in native['rows']),nonzero_gather=sum(x!=0 for x in native['gradient']),nonzero_transpose=sum(x!=0 for arr in native['transpose'] for x in arr),exact_rational_transpose_identity=True,exact_real_affine_quotients_checked=expected_slopes is not None,stored_affine_coefficient_defect_max=max(ideal_errors+[0.]),native_gather_error=grad_error,native_transpose_error=trans_error,native_work_error=work_error,unenclosed_work_defect=native['work']['unenclosed_defect'],combined_payload_bytes=native['combined_payload_bytes'],retained_action_payload_bytes=native['retained_action_payload_bytes'],parse_phase_payload_bytes=native['parse_phase_payload_bytes'])

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--repo',required=True,type=Path);p.add_argument('--binary',required=True,type=Path);p.add_argument('--out',required=True,type=Path);a=p.parse_args()
    repo,binary,out=a.repo.resolve(),a.binary.resolve(),a.out.resolve();out=require_external_output(out,repo);out.mkdir(parents=True,exist_ok=False)
    geometries={}
    for kind in ['3','6','12','nonmidpoint']:
        path=out/('geometry-'+kind+'.json');run([binary,'describe_geometry',kind,path]);g=json.loads(path.read_text());require(g['counts']==[int(kind) if kind.isdigit() else 6]*3)
        geometries[kind]=g
    matrix=[[F(1,2),F(-1),F(1,4)],[F(3,4),F(3,2),F(-1,2)],[F(-1,4),F(1,2),F(-1)]]
    rotation=[[F(0),F(-1,2),F(1,4)],[F(1,2),F(0),F(-3,4)],[F(-1,4),F(3,4),F(0)]]
    results=[];input_sources={}
    cases=[('3','general'),('6','affine'),('6','rotation'),('6','flat'),('6','general'),('12','general'),('nonmidpoint','flat')]
    for kind,mode in cases:
        folder=out/(kind+'-'+mode);folder.mkdir();g=geometries[kind]
        sites=normal_sites(g) if kind=='3' else flat_sites(g) if mode=='flat' else interior_sites()
        if mode=='general' and kind!='3':sites+=flat_sites(g)
        v=[[0.]*len(arr) for arr in g['areas']];slopes=None
        if mode in ['affine','rotation']:
            M=matrix if mode=='affine' else rotation
            for component,values in enumerate(v):
                for face in range(len(values)):
                    if active(g,component,face):
                        pos=face_position(g,component,coordinate(face,face_counts(g,component)))
                        values[face]=float(F(2 if mode=='affine' else 0)+sum((M[component][d]*F(pos[d]) for d in range(3)),F(0)))
            slopes=[M[s[1]][s[1] if s[0]=='N' else s[2]] for s in sites]
        elif mode=='flat':
            slopes=[];assigned={}
            for site in sites:
                row=expected_row(g,site);slope=F(2**row['component']) if row['boundary']=='FlatWallRay' else F(0);slopes.append(slope)
                for endpoint in row['endpoints']:
                    if endpoint['face'] is not None:
                        trace=next(e for e in row['endpoints'] if e['face'] is None)
                        value=float(slope*(F(endpoint['position'][row['derivative']])-F(trace['position'][row['derivative']])))
                        key=row['component'],endpoint['face'];require(key not in assigned or assigned[key]==value);assigned[key]=value;v[key[0]][key[1]]=value
        else:
            for component,values in enumerate(v):
                for face in range(len(values)):
                    if active(g,component,face):values[face]=float(F((face*17+component*3)%29-14,16))
        q=[float(F((i*7+3)%17-8,8)) for i in range(len(sites))]
        velocity_file=folder/'supplied-velocities.json';velocity_file.write_text(json.dumps(v,separators=(',',':'))+'\n')
        bodies=[('problem','Explicit manufactured local operator test input; transient Stokes declarations; density1, viscosity1 in SI; no physical solution/error/load acceptance. Geometry: '+json.dumps(g['counts'])+' origin '+json.dumps(g['origin'])+' spacing '+json.dumps(g['spacing'])+' solid bounds '+json.dumps([g['lower'],g['upper']])+'\n'),('boundary','Explicit stationary no-slip solid and sealed free-slip outer walls; stored inactive normal faces exactly0; no additional wall observations.\n'),('initial',json.dumps({'kind':'supplied_initial_data','time_seconds':0,'velocity_file':velocity_file.name,'velocity_sha256':sha(velocity_file),'manufactured_local_test_only':True,'errors':'all_unknown'})+'\n'),('forcing','Explicit no forcing declared on [0,0] seconds. No time advancement.\n')]
        refs=[]
        for name,body in bodies:file=folder/(name+'.txt');file.write_text(body);refs.append(sha(file))
        checkpoint=folder/'supplied-state.rheon-os1';checkpoint.write_bytes(encode_state(g,v,refs))
        request_file=folder/'requests.txt';request_file.write_text(''.join(' '.join(map(str,s))+'\n' for s in sites))
        qfile=folder/'row-test.txt';qfile.write_text(''.join(repr(x)+'\n' for x in q))
        run([binary,kind,checkpoint,request_file,qfile,folder/'native'])
        record=folder/'native/gradient-record.json';native=json.loads(record.read_text());r=validate_record(g,sites,v,q,native,slopes);r.update(kind=kind,mode=mode,record_sha256=sha(record),state_sha256=sha(checkpoint),references=refs);results.append(r)
        input_sources[str(folder.relative_to(out))]={p.name:sha(p) for p in [velocity_file,request_file,qfile,checkpoint,*[folder/(name+'.txt') for name,_ in bodies]]}
    # Reuse the newly authored explicit N6 input, never edit a prior fixture.
    folder=out/'6-general';negatives=[]
    for name,query in [('coarse-fine','CF\n'),('corner','C 0 1 2 2 2 0\n'),('outer-cross','C 0 1 0 1 0 0\n'),('same-axis','C 0 0 1 1 1 0\n')]:
        f=out/(name+'-request.txt');f.write_text(query);test=out/(name+'-test.txt');test.write_text('1\n');r=run([binary,'6',folder/'supplied-state.rheon-os1',f,test,out/(name+'-unexpected-output')],False);require(not (out/(name+'-unexpected-output')).exists());negatives.append(dict(case=name,native_error=r.stderr.strip(),native_exit=r.returncode))
    files=['src/obstacle_gradient.rs','src/lib.rs','examples/obstacle_gradient.rs','tests/obstacle_gradient_contract.rs','research/obstacle-gradient/GradientInterface.lean','research/obstacle-gradient/check_proofs.py','tools/check_obstacle_gradient.py','docs/research-book/implementation/discrete-obstacle-gradient-foundation-20261009.md']
    report=dict(source_head=run(['git','-C',repo,'rev-parse','HEAD']).stdout.strip(),sources={f:sha(repo/f) for f in files},binary_sha256=sha(binary),cases=results,input_sources=input_sources,actual_native_unsupported_refusals=negatives,qualification='Unqualified',all_checks_passed=True,hard_cap_bytes=CAP,physical_load_accepted=False,accepted_dynamics_authorized=False,exact_real_scope='finite rows and compatible local affine/trace quotients only; stored reciprocal defects reported; native tolerances unenclosed')
    path=out/'gradient-oracle-report.json';path.write_text(json.dumps(report,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(report=str(path),sha256=sha(path),cases=len(results),all_checks_passed=True)))
if __name__=='__main__':main()
