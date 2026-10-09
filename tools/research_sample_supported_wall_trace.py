"""Research-only conditional wall loads from existing samples; no native edits.

Only the two already admitted physical fixtures are captured from frozen Rust.
Exact dyadic arithmetic is a diagnostic, not an IEEE/solver accuracy certificate.
Generated records must be outside Git in a previously absent directory.
"""
import argparse
from fractions import Fraction as Q
from itertools import product
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sympy as S
import check_viscous_boundary_wrench as frozen_checker

import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'research'))
from check_proof_audit import require_external_output

ROOT = Path(__file__).resolve().parents[1]
BASE = '02142c537363659ac48ac01193084360f5260664'
BINARIES = {
    '9f1f77115fac7c8907ee592cd534e030fc07d907b66ab0ce0d2064e661b6d504': 'debug',
    'cfd7d8590914f63253f3450fe1ce4240b5b3f4e9ab8e60b6fd2df58a6b39155e': 'release',
}
CHECKER = 'd25b2316e2ffe27ee9943a475a3878770f58a054d3186c05ff7bd09d3b30310c'

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def exact(bits): return Q.from_float(struct.unpack('>d', bytes.fromhex(bits))[0])
def cross(a,b): return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def add(a,b): return [x+y for x,y in zip(a,b)]
def serial(x):
    if isinstance(x,Q): return str(x)
    if isinstance(x,dict): return {k:serial(v) for k,v in x.items()}
    if isinstance(x,(list,tuple)): return [serial(v) for v in x]
    return x

def d1(delta,b,u):
    if delta <= 0: raise ValueError('positive represented distance required')
    return (u-b)/delta

def d2(a,b,trace,u,v):
    if not 0 < a < b: raise ValueError('two distinct outward active centers required')
    return -(a+b)/(a*b)*trace+b/(a*(b-a))*u-a/(b*(b-a))*v

def hat(left,right):
    if min(left,right)<0 or left+right<=0: raise ValueError('nonempty nonnegative hat support required')
    return (left+right)/2, (right-left)/3

# Independent symbolic differentiation of the specified analytic control.
x = S.symbols('x0:3')
p = [(t-1)*(t-2) for t in x]
psi = 225*(1+x[0]-S.Rational(3,2))*S.prod(t*t for t in p)
velocity = [S.diff(psi,x[1]),-S.diff(psi,x[0]),S.Integer(0)]
stress = [[S.diff(velocity[i],x[j])+S.diff(velocity[j],x[i]) for j in range(3)] for i in range(3)]

def val(poly,point):
    r=poly.subs(dict(zip(x,map(S.Rational,point))))
    return Q(int(S.numer(r)),int(S.denom(r)))

def poly_bound(poly,radii):
    z=S.symbols('z0:3')
    shifted=S.Poly(S.expand(poly.subs({x[i]:z[i]+S.Rational(3,2) for i in range(3)})),*z)
    return sum((Q(abs(int(S.numer(c))),int(S.denom(c)))*
                product_power(radii,k) for k,c in shifted.terms()),Q(0))

def product_power(r,k):
    result=Q(1)
    for a,b in zip(r,k): result*=a**b
    return result

def continuum():
    load=[S.Integer(0)]*6
    for n in range(3):
        tangent=[i for i in range(3) if i!=n]
        for wall,sign in [(1,-1),(2,1)]:
            traction=[sign*stress[i][n].subs(x[n],wall) for i in range(3)]
            lever=[x[i]-S.Rational(3,2) if i!=n else S.Rational(wall)-S.Rational(3,2) for i in range(3)]
            for k,poly in enumerate(traction+cross(lever,traction)):
                load[k]+=S.integrate(poly,(x[tangent[0]],1,2),(x[tangent[1]],1,2))
    return [Q(int(S.numer(v)),int(S.denom(v))) for v in load]

def reconstruction(raw,method):
    meta=raw['meta']; h=list(map(exact,meta['spacing'])); origin=list(map(exact,meta['origin']))
    lo,hi=meta['lo'],meta['hi']; mu=exact(meta['mu'])
    selected=next(f for f in raw['fields'] if f['name']=='polynomial-tilted-curl')
    values=list(map(exact,selected['values']))
    active={(a['axis'],tuple(a['coordinates'])):a for a in raw['active']}
    for a,U in zip(raw['active'],values):
        if val(velocity[a['axis']],list(map(exact,a['position'])))!=U:
            raise ValueError('actual Rust samples differ from analytic control')
    total=[Q(0)]*6; errorF=[Q(0)]*3; errorT=[Q(0)]*3; entries=[]
    quadF=[Q(0)]*3; quadT=[Q(0)]*3
    ref=[Q(3,2)]*3
    for n in range(3):
        for wall_index,sign,layer_index in [(lo[n],-1,lo[n]-1),(hi[n],1,hi[n])]:
            wall=origin[n]+h[n]*wall_index
            for i in range(3):
                if i==n: continue # explicitly conditional continuous div-free no-slip observable
                j=3-n-i
                traction=sign*mu*stress[i][n].subs(x[n],S.Rational(wall))
                radii=[Q(1,2)]*3
                radii[n]+=h[n]*(Q(3,2) if method=='P2' else Q(1,2))
                M=poly_bound(S.diff(velocity[i],x[n],3 if method=='P2' else 2),radii)
                maxlever=[Q(1,2)]*3
                Mii=poly_bound(S.diff(traction,x[i],2),[Q(1,2)]*3)
                Mjj=poly_bound(S.diff(traction,x[j],2),[Q(1,2)]*3)
                Mj=poly_bound(S.diff(traction,x[j]),[Q(1,2)]*3)
                A=(hi[i]-lo[i])*h[i]*(hi[j]-lo[j])*h[j]
                quadF[i]+=A*(h[i]**2*Mii/12+h[j]**2*Mjj/24)
                for k in range(3):
                    if k==i: continue
                    lever_axis=3-k-i
                    quadT[k]+=A*(maxlever[lever_axis]*h[i]**2*Mii/12+
                        h[j]**2*(maxlever[lever_axis]*Mjj+2*Mj*(lever_axis==j))/24)
                for ki,kj in product(range(lo[i],hi[i]+1),range(lo[j],hi[j])):
                    coords=[0]*3;coords[n]=layer_index;coords[i]=ki;coords[j]=kj
                    key=(i,tuple(coords))
                    if key not in active: raise ValueError('first outward active center absent')
                    a=active[key];point=list(map(exact,a['position']));delta=sign*(point[n]-wall)
                    U=values[a['id']]
                    if method=='P2':
                        coords2=coords.copy();coords2[n]+=sign
                        second=active.get((i,tuple(coords2)))
                        if second is None: return {'supported':False,'reason':'second outward active center absent','no_substitution':True}
                        b=sign*(exact(second['position'][n])-wall)
                        derivative=d2(delta,b,Q(0),U,values[second['id']]); e=mu*delta*b*M/6
                    else:
                        derivative=d1(delta,Q(0),U);e=mu*delta*M/2
                    left=h[i] if ki>lo[i] else Q(0);right=h[i] if ki<hi[i] else Q(0)
                    mass,offset=hat(left,right);area=mass*h[j]
                    centroid=point.copy();centroid[n]=wall;centroid[i]+=offset
                    force=[Q(0)]*3;force[i]=mu*area*derivative
                    lever=[v-c for v,c in zip(centroid,ref)]
                    total=add(total,force+cross(lever,force));errorF[i]+=area*e
                    for k in range(3):
                        if k!=i:errorT[k]+=area*abs(lever[3-k-i])*e
                    entries.append({'component':i,'normal':n,'sign':sign,'sample_id':a['id'],
                        'distance':delta,'area':area,'surface_centroid':centroid,'traction':mu*derivative,
                        'analytic_ray_error_bound':e})
    target=continuum();bound=add(errorF,quadF)+add(errorT,quadT)
    if not all(abs(a-b)<=e for a,b,e in zip(total,target,bound)):
        raise ValueError('conditional manufactured bounds violated')
    # Assemble independent surface work and moment with the same basis integrals.
    tests=[[Q(2),Q(-3),Q(5),Q(7,2),Q(-4),Q(9,5)], [Q(0)]*3+[Q(1),Q(0),Q(0)]]
    for test in tests:
        direct=Q(0)
        for row in entries:
            lever=[a-b for a,b in zip(row['surface_centroid'],ref)]
            rigid=add(test[:3],cross(test[3:],lever))
            direct+=row['area']*row['traction']*rigid[row['component']]
        if direct!=sum(a*b for a,b in zip(test,total)):raise ValueError('surface force/torque work inconsistent')
    # Reassemble about a displaced reference from the same surface field.
    shift=[Q(2,7),Q(-3,11),Q(5,13)]
    shifted=[Q(0)]*3
    for row in entries:
        f=[Q(0)]*3;f[row['component']]=row['area']*row['traction']
        shifted=add(shifted,cross([a-b-c for a,b,c in zip(row['surface_centroid'],ref,shift)],f))
    assert shifted==[a-b for a,b in zip(total[3:],cross(shift,total[:3]))]
    old=list(map(exact,selected['solid_wrench']));gap=[a-b for a,b in zip(total,old)]
    return {'supported':True,'wrench':total,'target':target,'load_error':[a-b for a,b in zip(total,target)],
            'physical_accuracy_qualified':False,'analytic_conditional_bound':bound,
            'ray_error_bound':errorF+errorT,'surface_quadrature_bound':quadF+quadT,
            'exact_surface_rigid_work':True,'reference_covariance':True,'old_native_wrench':old,'load_work_gap':gap,
            'gap_is_volume_residual':False,'full_endpoint_nodes_retained':True,'entries':entries}

def kinematic_normal(raw):
    m=raw['meta'];h=list(map(exact,m['spacing']));o=list(map(exact,m['origin']));mu=exact(m['mu'])
    field=next(f for f in raw['fields'] if f['name']=='polynomial-tilted-curl')
    U=list(map(exact,field['values']));active={(a['axis'],tuple(a['coordinates'])):a for a in raw['active']}
    total=[Q(0)]*6;known_outer=0
    for n in range(3):
        i,j=[a for a in range(3) if a!=n]
        for wi,sg in [(m['lo'][n],-1),(m['hi'][n],1)]:
            for ki,kj in product(range(m['lo'][i],m['hi'][i]),range(m['lo'][j],m['hi'][j])):
                c=[0]*3;c[n]=wi+sg;c[i]=ki;c[j]=kj
                a=active.get((n,tuple(c)))
                if a is None:
                    if c[n] not in (0,m['counts'][n]):raise ValueError('normal sample absent without known outer boundary trace')
                    un=Q(0);known_outer+=1
                else:un=U[a['id']]
                p=[o[d]+h[d]*(c[d]+(Q(0) if d==n else Q(1,2))) for d in range(3)];p[n]=o[n]+h[n]*wi
                f=[Q(0)]*3;f[n]=2*mu*h[i]*h[j]*un/h[n]
                total=add(total,f+cross([v-Q(3,2) for v in p],f))
    return {'symmetric_gradient_normal_secant_wrench':total,'known_sealed_outer_traces':known_outer,
            'compressible_Newtonian_model':False,'continuum_zero_normal_stress_premise_proven_by_samples':False}

def incidence_divergence(raw):
    m=raw['meta'];h=list(map(exact,m['spacing']))
    field=next(f for f in raw['fields'] if f['name']=='polynomial-tilted-curl')
    U=list(map(exact,field['values']));active={(a['axis'],tuple(a['coordinates'])):a for a in raw['active']}
    divergence=[]
    for cell in product(*(range(n) for n in m['counts'])):
        if all(m['lo'][d]<=cell[d]<m['hi'][d] for d in range(3)):continue
        div=Q(0)
        for d in range(3):
            left=list(cell);right=list(cell);right[d]+=1
            a=active.get((d,tuple(left)));b=active.get((d,tuple(right)))
            # Absent faces are the existing represented stationary normal traces.
            div+=((U[b['id']] if b else Q(0))-(U[a['id']] if a else Q(0)))/h[d]
        divergence.append(div)
    return {'max_absolute_MAC_incidence_divergence':max(map(abs,divergence),default=Q(0)),
            'nonzero_cell_count':sum(d!=0 for d in divergence),'continuous_divergence_proof':False}

def controls():
    records=[]
    for a,b in [(Q(1,4),Q(3,4)),(Q(2,7),Q(9,11)),(Q(5,8),Q(11,8))]:
        for reflection in (-1,1):
            slope=reflection*Q(7,5);trace=Q(2,9);curvature=Q(-3,7)
            assert d1(a,trace,trace+slope*a)==slope
            assert d2(a,b,trace,trace+slope*a+curvature*a*a,trace+slope*b+curvature*b*b)==slope
            assert d2(a,b,0,a**3,b**3)==-a*b
            # Rigid shear requires BOTH derivative terms, also on reflected anisotropic rays.
            assert d1(a,trace,trace+slope*a)-slope==0
            assert d2(a,b,trace,trace+slope*a,trace+slope*b)-slope==0
            records.append({'a':a,'b':b,'reflection':reflection,'affine_P1':True,'quadratic_P2':True,'rigid_shear_both_terms':True})
    # Every normal/tangential pair, both reflections, nonmidpoint distances.
    omega=[Q(2,7),Q(-3,11),Q(5,13)]
    rigid_pairs=[]
    for n,i,sign in product(range(3),range(3),(-1,1)):
        if i==n:continue
        normal=[Q(0)]*3;normal[n]=sign
        tangent=[Q(0)]*3;tangent[i]=1
        point=[Q(2,3),Q(-5,7),Q(11,13)]
        R=lambda q: add([Q(3,5),Q(-7,9),Q(2,11)],cross(omega,q))
        a=Q(2,7) if n==0 else Q(5,11) if n==1 else Q(7,13)
        sample=add(point,[a*v for v in normal])
        normal_ray=d1(a,R(point)[i],R(sample)[i])
        tangential_normal=sign*cross(omega,tangent)[n]
        assert normal_ray+tangential_normal==0
        assert d1(a,R(point)[n],R(sample)[n])==0
        rigid_pairs.append({'normal':n,'component':i,'reflection':sign,'represented_distance':a,'symmetric_gradient_zero':True})
    for a,b in [(Q(0),Q(1)),(Q(1),Q(1)),(Q(2),Q(1))]:
        try:d2(a,b,0,1,2)
        except ValueError:pass
        else:raise ValueError('unsupported support accepted')
    assert hat(0,Q(1))==(Q(1,2),Q(1,3));assert hat(Q(1),0)==(Q(1,2),Q(-1,3))
    assert Q(1,2)*Q(2,3)==Q(1,3)
    vector_moments=[]
    for i in range(3):
        point=[Q(2,7),Q(-3,11),Q(5,13)]
        centroid=point.copy();centroid[i]+=hat(Q(2,7),Q(4,9))[1]
        force=[Q(0)]*3;force[i]=Q(7,3)
        assert cross(point,force)==cross(centroid,force)
        assert cross(omega,point)[i]==cross(omega,centroid)[i]
        vector_moments.append({'traction_axis':i,'centroid_lumped_wrench_and_work_equal':True})
    return {'ray_controls':records,'rigid_cross_component_controls':rigid_pairs,'vector_hat_moment_controls':vector_moments,'invalid_support_rejected':True,'abstract_scalar_hat_first_moment':'1/3',
        'abstract_scalar_lumped_first_moment':'1/2','actual_other_tangent_midpoint_torque_z':'-1/4',
        'actual_other_tangent_exact_torque_z':'-1/3'}

def run(executable,output):
    output=require_external_output(output, ROOT, 'outputs must remain outside Git')
    binary_sha=sha(executable)
    if binary_sha not in BINARIES:raise ValueError('qualified frozen native binary required')
    subprocess.run(['git','diff','--exit-code',BASE,'--','src','proofs','tests','examples','Cargo.toml','Cargo.lock',
                    'tools/check_viscous_boundary_wrench.py'],cwd=ROOT,check=True,capture_output=True)
    if sha(ROOT/'tools/check_viscous_boundary_wrench.py')!=CHECKER:raise ValueError('unchanged old arithmetic checker required')
    output.mkdir(parents=True);cases={}
    for name in ('unit-center','polynomial-bounded'):
        path=output/(name+'.json');subprocess.run([str(executable),name,str(path)],check=True,capture_output=True,timeout=30)
        frozen_checker.verify(path,name);raw=json.loads(path.read_text())
        cases[name]={'raw_sha256':sha(path),'P1':reconstruction(raw,'P1'),'P2':reconstruction(raw,'P2'),
                     'general_kinematic_normal_comparator':kinematic_normal(raw),'MAC_divergence_diagnostic':incidence_divergence(raw)}
    if sha(executable)!=binary_sha:raise ValueError('binary changed')
    assert cases['unit-center']['P1']['wrench']==[Q(0)]*6
    assert not cases['unit-center']['P2']['supported']
    assert cases['polynomial-bounded']['P1']['wrench'][1]==Q(-199125,131072)
    assert cases['polynomial-bounded']['P1']['wrench'][5]==Q(-43875,32768)
    assert cases['polynomial-bounded']['P2']['wrench'][1]==Q(26325,32768)
    assert cases['polynomial-bounded']['P2']['wrench'][5]==Q(-22275,16384)
    result={'schema':'rheon-sample-supported-wall-trace-research-v1','preserved_head':BASE,
            'source_sha256':sha(Path(__file__)),'native_binary_sha256':binary_sha,'native_flavor':BINARIES[binary_sha],
            'old_checker_sha256':CHECKER,'sympy_version':S.__version__,'cases':cases,'controls':controls(),
            'contract':'conditional exact-real consistency only; unknown sample errors cannot be certified',
            'production_change':False,'physical_accuracy_qualified':False,'publication':False,
            'pressure':False,'stepping':False,'coupling':False,'new_refinement_fixtures':False}
    path=output/'research.json';path.write_text(json.dumps(serial(result),indent=2)+'\n')
    print(json.dumps({'output':str(path),'sha256':sha(path),'bounded_P1':serial(cases['polynomial-bounded']['P1']['wrench']),
                      'physical_accuracy_qualified':False}))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();run(a.executable.resolve(),a.output.resolve())
