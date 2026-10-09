"""Exact independent oracle for the frozen observation experiment; never steps.

Consumes actual Rust records. Generic load reconstruction uses only observations;
analytic derivatives/target are confined to acquisition and oracle comparisons.
Outputs outside Git, create-new. No existing fixtures or tolerances are edited.
"""
from fractions import Fraction as Q
from itertools import product
from pathlib import Path
import argparse, hashlib, json, math, struct, subprocess
import sympy as S

ROOT=Path(__file__).resolve().parents[1]
PARENT='10fcda307ca42fad9b93b83679cd97b52c64ee03'
NATIVE=(1,2,4); WALL=(1,2,4,8,16,32)

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def val(b):return Q.from_float(struct.unpack('>d',bytes.fromhex(b))[0])
def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def serial(x):
    if isinstance(x,Q):return str(x)
    if isinstance(x,dict):return {k:serial(v) for k,v in x.items()}
    if isinstance(x,(list,tuple)):return [serial(v) for v in x]
    return x

def reconstruct(rows,second=False,reference=(Q(3,2),)*3):
    """Geometry/trace/value-only finite load action; trace is known stationary0."""
    total=[Q(0)]*6;absolute=[Q(0)]*6;faces={}
    for r in rows:
        n,i,side=r['normal'],r['component'],r['side'];a,b,U,V,A=r['a'],r['b'],r['u'],r['v'],r['area']
        if n==i or side not in (-1,1) or a<=0 or A<=0:raise ValueError('unsupported observation')
        if second:
            if V is None:return None
            if b<=a:raise ValueError('distinct outward distances required')
            t=b*U/(a*(b-a))-a*V/(b*(b-a))
        else:t=U/a
        f=[Q(0)]*3;f[i]=A*t;w=f+cross([x-y for x,y in zip(r['p'],reference)],f)
        key=f'{n}:{side}';face=faces.setdefault(key,[Q(0)]*6)
        for k in range(6):total[k]+=w[k];absolute[k]+=abs(w[k]);face[k]+=w[k]
    return {'wrench':total,'sum_abs_terms':absolute,'faces':faces}

# Independent analytic acquisition/oracle; not visible to reconstruct().
x=S.symbols('x0:3');p=[(a-1)*(a-2) for a in x]
psi=225*(x[0]-S.Rational(1,2))*S.prod(a*a for a in p)
u=[S.diff(psi,x[1]),-S.diff(psi,x[0]),S.Integer(0)]
polys=[[(powers,Q(int(S.numer(c)),int(S.denom(c)))) for powers,c in S.Poly(a,*x).terms()] for a in u]
stress=[[S.diff(u[i],x[n])+S.diff(u[n],x[i]) for n in range(3)] for i in range(3)]

def sample(i,point):
    # Factored exact formula independently checked against symbolic derivatives.
    pp=[(a-1)*(a-2) for a in point];tilt=point[0]-Q(1,2)
    if i==0:return 450*tilt*pp[0]**2*pp[1]*(2*point[1]-3)*pp[2]**2
    if i==1:return -225*(pp[0]**2+2*tilt*pp[0]*(2*point[0]-3))*pp[1]**2*pp[2]**2
    return Q(0)

def acquisition_bound(i,point):
    pp=[(a-1)*(a-2) for a in point];tilt=point[0]-Q(1,2)
    scale=abs(sample(i,point)) if i!=1 else 225*(abs(pp[0]**2)+abs(2*tilt*pp[0]*(2*point[0]-3)))*pp[1]**2*pp[2]**2
    return Q(128,2**52)*scale

def continuum():
    total=[S.Integer(0)]*6
    for n,side in product(range(3),(-1,1)):
        wall=1 if side<0 else 2;tang=[k for k in range(3) if k!=n]
        t=[side*stress[i][n].subs(x[n],wall) for i in range(3)]
        lever=[a-S.Rational(3,2) for a in x];lever[n]=S.Rational(wall)-S.Rational(3,2)
        for k,e in enumerate(t+cross(lever,t)):
            total[k]+=S.integrate(e,(x[tang[0]],1,2),(x[tang[1]],1,2))
    return [Q(int(S.numer(v)),int(S.denom(v))) for v in total]

def decoded(raw):
    rows=[]
    for r in raw['observations']:
        r=r.copy();r['p']=list(map(val,r['p']))
        for k in ('a','b','u','area'):r[k]=val(r[k])
        r['v']=None if r['v'] is None else val(r['v']);rows.append(r)
    return rows

def expected_rows(mode,s):
    h=Q(1,s);out=[]
    for n,side in product(range(3),(-1,1)):
        for i in range(3):
            if i==n:continue
            j=3-n-i
            for ki,kj in product(range(s+(mode=='native')),range(s)):
                point=[Q(0)]*3;point[n]=Q(1 if side<0 else 2);point[i]=1+h*(ki+(Q(0) if mode=='native' else Q(1,2)));point[j]=1+h*(kj+Q(1,2))
                a,b=h/2,3*h/2;q=point.copy();q[n]+=side*a;t=point.copy();t[n]+=side*b
                out.append({'normal':n,'component':i,'side':side,'p':point,'a':a,'b':b,'u':sample(i,q),'v':None if s==1 else sample(i,t),'area':h*h*(Q(1,2) if mode=='native' and ki in (0,s) else 1)})
    return out

def normal_comparator(raw):
    if raw['mode']!='native':return None
    s=raw['scale'];h=Q(1,s);samples={(r['component'],tuple(map(val,r['p']))):val(r['u']) for r in raw['native']['samples']}
    result=[Q(0)]*6;outer=0
    for n,side in product(range(3),(-1,1)):
        i,j=[a for a in range(3) if a!=n]
        for ki,kj in product(range(s),repeat=2):
            p=[Q(0)]*3;p[n]=Q(1 if side<0 else 2);p[i]=1+h*(ki+Q(1,2));p[j]=1+h*(kj+Q(1,2));q=p.copy();q[n]+=side*h
            if (n,tuple(q)) in samples:un=samples[n,tuple(q)]
            elif q[n] in (0,3):un=Q(0);outer+=1
            else:raise ValueError('missing normal sample')
            f=[Q(0)]*3;f[n]=2*h*un
            for k,w in enumerate(f+cross([a-Q(3,2) for a in p],f)):result[k]+=w
    return {'wrench':result,'known_outer_traces':outer,'zero_normal_stress_not_inferred_from_MAC':True}

BOUND_CACHE={}
def polynomial_bound(poly,radii):
    key=(str(poly),tuple(radii))
    if key not in BOUND_CACHE:
        z=S.symbols('z0:3');shifted=S.Poly(S.expand(poly.subs({x[k]:z[k]+S.Rational(3,2) for k in range(3)})),*z)
        BOUND_CACHE[key]=sum(Q(abs(int(S.numer(c))),int(S.denom(c)))*math.prod(radii[k]**powers[k] for k in range(3)) for powers,c in shifted.terms())
    return BOUND_CACHE[key]

def analytic_bounds(rows,mode,s,second):
    # Oracle-only derivative bounds; never numerical estimator inputs.
    h=Q(1,s);ray=[Q(0)]*6;quad=[Q(0)]*6;noise=[Q(0)]*6
    for n,side in product(range(3),(-1,1)):
        wall=1 if side<0 else 2
        for i in range(3):
            if i==n:continue
            j=3-n-i;group=[r for r in rows if (r['normal'],r['component'],r['side'])==(n,i,side)]
            radii=[Q(1,2)]*3;radii[n]+=3*h/2 if second else h/2
            M=polynomial_bound(S.diff(u[i],x[n],3 if second else 2),radii)
            traction=side*stress[i][n].subs(x[n],wall)
            Mi=polynomial_bound(S.diff(traction,x[i]),[Q(1,2)]*3);Mj=polynomial_bound(S.diff(traction,x[j]),[Q(1,2)]*3)
            Mii=polynomial_bound(S.diff(traction,x[i],2),[Q(1,2)]*3);Mjj=polynomial_bound(S.diff(traction,x[j],2),[Q(1,2)]*3)
            Ci=Q(1,12) if mode=='native' else Q(1,24);Cj=Q(1,24)
            quad[i]+=h*h*(Ci*Mii+Cj*Mjj)
            for k in range(3):
                if k==i:continue
                ell=3-k-i
                quad[k+3]+=h*h*(Ci*(Mii/2+2*Mi*(ell==i))+Cj*(Mjj/2+2*Mj*(ell==j)))
            for r in group:
                e=r['a']*r['b']*M/6 if second else r['a']*M/2
                gain=r['b']/(r['a']*(r['b']-r['a']))+r['a']/(r['b']*(r['b']-r['a'])) if second else 1/r['a']
                ray[i]+=r['area']*e;noise[i]+=r['area']*gain
                for k in range(3):
                    if k!=i:
                        lever=abs(r['p'][3-k-i]-Q(3,2));ray[k+3]+=r['area']*lever*e;noise[k+3]+=r['area']*lever*gain
    return {'ray_error_bound':ray,'surface_quadrature_bound':quad,'unit_sample_error_load_gain':noise,'uniform_C3_closed_face_premise':True,'bounds_are_oracle_diagnostics_not_operator_inputs':True}

def controls():
    for a,b in ((Q(1,4),Q(3,4)),(Q(2,7),Q(9,11))):
        for side in (-1,1):
            slope=side*Q(7,5);curve=Q(-3,7)
            row={'normal':0,'component':1,'side':side,'p':[Q(1),Q(3,2),Q(3,2)],'a':a,'b':b,'u':slope*a,'v':slope*b,'area':Q(1)}
            assert reconstruct([row])['wrench'][1]==slope
            row.update(u=slope*a+curve*a*a,v=slope*b+curve*b*b)
            assert reconstruct([row],True)['wrench'][1]==slope
            row['v']=None;assert reconstruct([row],True) is None
    witness=sample(1,[Q(1,2),Q(3,2),Q(3,2)]);assert witness==Q(-2025,4096)
    for i in range(3):
        point=[Q(1,2),Q(3,2),Q(3,2)]
        expanded=sum(c*math.prod(point[k]**powers[k] for k in range(3)) for powers,c in polys[i]);assert expanded==sample(i,point)
    return {'pair_distinguishing_scalar':witness,'general_finite_sample_load_identifiability':False,'P1_noise_gain_at_h':2,'P2_noise_gain_at_h':Q(10,3)}

def verify(raw,target):
    mode,s=raw['mode'],raw['scale'];rows=decoded(raw);oracle=expected_rows(mode,s)
    assert len(rows)==len(oracle)
    assert raw['observation_bytes']<=2_000_000 and raw['observation_bytes']==len(rows)*raw['row_size']
    maxnoise=Q(0)
    for r,e in zip(rows,oracle):
        for k in ('normal','component','side','p','a','b','area'):assert r[k]==e[k],(k,r,e)
        for name,distance in (('u',r['a']),('v',r['b'])):
            if e[name] is None:assert r[name] is None;continue
            point=r['p'].copy();point[r['normal']]+=r['side']*distance
            error=abs(r[name]-e[name]);maxnoise=max(maxnoise,error)
            assert error<=acquisition_bound(r['component'],point)
    native=raw.get('native')
    if native:
        assert native['active_count']==78*s**3-30*s**2 and native['row_count']==390*s**3-216*s**2+36*s
        assert native['combined_bytes']==112*native['active_count']+208*native['row_count']+8*(81*s**3+27*s**2)
        assert native['combined_bytes']<=16_000_000
        keys=set()
        for point in native['samples']:
            i=point['component'];p=list(map(val,point['p']));U=val(point['u']);assert abs(U-sample(i,p))<=acquisition_bound(i,p)
            assert (i,tuple(p)) not in keys;keys.add((i,tuple(p)))
        expected=set();h=Q(1,s);N=3*s
        for i in range(3):
            shape=[N]*3;shape[i]=N-1
            for c0 in product(*(range(t) for t in shape)):
                c=list(c0);c[i]+=1;left=c.copy();left[i]-=1
                if any(all(s<=a<2*s for a in cell) for cell in (left,c)):continue
                expected.add((i,tuple(h*(a+(Q(0) if k==i else Q(1,2))) for k,a in enumerate(c))))
        assert keys==expected
    out={'mode':mode,'scale':s,'h':Q(1,s),'observations':len(rows),'observation_bytes':raw['observation_bytes'],'max_sample_error':maxnoise,'methods':{},'normal_comparator':normal_comparator(raw)}
    if native:out['old_generalized_wrench']=list(map(val,native['old_generalized_wrench']));out['native_payload']= {k:v for k,v in native.items() if k not in ('samples','old_generalized_wrench')}
    for method,second in (('P1',False),('P2',True)):
        result=reconstruct(rows,second);ideal=reconstruct(oracle,second)
        if result is None:assert raw[method] is None;out['methods'][method]={'supported':False,'reason':'actual second outward point absent; no substitution'};continue
        rust=list(map(val,raw[method]));n=len(rows);gamma=Q(32*n+64,2**52)
        arithmetic=[gamma*a for a in result['sum_abs_terms']]
        assert all(abs(a-b)<=tol for a,b,tol in zip(rust,result['wrench'],arithmetic))
        shift=[Q(2,7),Q(-3,11),Q(5,13)];shifted=reconstruct(rows,second,[Q(3,2)+a for a in shift])['wrench'];change=cross(shift,result['wrench'][:3]);assert shifted[3:]==[a-b for a,b in zip(result['wrench'][3:],change)]
        twist=[Q(2),Q(-3),Q(5),Q(7,2),Q(-4),Q(9,5)];direct=Q(0)
        for r in rows:
            t=r['u']/r['a'] if not second else r['b']*r['u']/(r['a']*(r['b']-r['a']))-r['a']*r['v']/(r['b']*(r['b']-r['a']))
            rv=[a+b for a,b in zip(twist[:3],cross(twist[3:],[a-Q(3,2) for a in r['p']]))];direct+=r['area']*t*rv[r['component']]
        assert direct==sum(a*b for a,b in zip(twist,result['wrench']))
        error=[a-b for a,b in zip(result['wrench'],target)];idealerror=[a-b for a,b in zip(ideal['wrench'],target)]
        out['methods'][method]={'supported':True,'wrench':result['wrench'],'exact_input_wrench':ideal['wrench'],'error':error,'exact_input_error':idealerror,'relative_Fy_error':abs(error[1])/Q(1,2),'relative_Tz_error':abs(error[5]),'sample_load_error':[a-b for a,b in zip(result['wrench'],ideal['wrench'])],'rust_arithmetic_error':[a-b for a,b in zip(rust,result['wrench'])],'rust_arithmetic_bound':arithmetic,'faces':result['faces'],'reference_covariance':True,'surface_rigid_work':True,'physical_solver_qualified':False}
        bounds=analytic_bounds(rows,mode,s,second);totalbound=[a+b+abs(c-d) for a,b,c,d in zip(bounds['ray_error_bound'],bounds['surface_quadrature_bound'],result['wrench'],ideal['wrench'])];assert all(abs(e)<=b for e,b in zip(error,totalbound));out['methods'][method].update(bounds);out['methods'][method]['analytic_conditional_bound']=totalbound
        if native:out['methods'][method]['surface_minus_old_generalized']=[a-b for a,b in zip(result['wrench'],out['old_generalized_wrench'])]
    return out

def main():
    a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--output',type=Path,required=True);args=a.parse_args();output=args.output.resolve();assert not output.is_relative_to(ROOT);output.mkdir(parents=True,exist_ok=False)
    binary=args.binary.resolve();target=continuum();assert target==[0,Q(-1,2),0,0,0,-1]
    records=[]
    for mode,scales in (('native',NATIVE),('wall',WALL)):
        for s in scales:
            dest=output/f'{mode}-{s}.json';subprocess.run([str(binary),mode,str(s),str(dest)],check=True)
            assert dest.stat().st_size<=(2_000_000 if mode=='native' else 8_000_000)
            raw=json.loads(dest.read_text());result=verify(raw,target);result['input_sha256']=sha(dest);records.append(result)
    refused=output/'native-8-refused.json';p=subprocess.run([str(binary),'native','8',str(refused)],text=True,capture_output=True);assert p.returncode!=0 and not refused.exists()
    for mode in ('native','wall'):
        prev={}
        for record in records:
            if record['mode']!=mode:continue
            for method,r in record['methods'].items():
                if not r['supported']:continue
                e=[abs(r['exact_input_error'][k]) for k in (1,5)]
                if method in prev:r['observed_Fy_Tz_orders']=[math.log2(float(a/b)) if a and b else None for a,b in zip(prev[method],e)]
                prev[method]=e
    report={'schema':'independent-wall-observations-research-v1','parent':PARENT,'binary_sha256':sha(binary),'source_sha256':{str(p.relative_to(ROOT)):sha(p) for p in (ROOT/'examples/research_wall_observations.rs',Path(__file__))},'target':target,'controls':controls(),'native8_refusal':{'returncode':p.returncode,'stderr':p.stderr,'required_combined_bytes':43_321_344,'cap':16_000_000},'records':records,'existing_fixtures_modified':False,'new_wall_observations_available_to_old_solver':False,'pressure_stepping_coupling_publication':False}
    (output/'report.json').write_text(json.dumps(serial(report),indent=2)+'\n')
    for r in records:
        for method,t in r['methods'].items():
            if t['supported']:print(r['mode'],r['scale'],method,'Fy',float(t['wrench'][1]),'Tz',float(t['wrench'][5]),'relative errors',float(t['relative_Fy_error']),float(t['relative_Tz_error']))

if __name__=='__main__':main()
