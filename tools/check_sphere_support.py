"""Independent Fraction geometry/P1-force/equilibrium checks of actual Rust output.
No native report supplies its own physical oracle. Exact physical correspondence
is proved only for these dyadic planar fixtures, not all nearest-rounded loads.
"""
import argparse
from fractions import Fraction as Q
import hashlib
import json
import math
from pathlib import Path
import subprocess


def scalar(x):
    if type(x) not in (int, float) or not math.isfinite(x):
        raise ValueError('finite non-Boolean scalar required')
    return Q(x)


def vector(x):
    if type(x) is not list or len(x) != 3:
        raise ValueError('3-vector required')
    return [scalar(y) for y in x]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def add(a, b): return [x+y for x,y in zip(a,b)]
def sub(a, b): return [x-y for x,y in zip(a,b)]
def scale(s, a): return [s*x for x in a]
def dot(a, b): return sum(x*y for x,y in zip(a,b))
def cross(a, b): return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def fixture(name='gravity'):
    c=[0.,0.,.25]
    return dict(name=name,radius=.25,h=.125,steps=4,mass=2.,inertia=.05,time=0.,
                center=c,velocity=[0.,0.,0.],omega=[0.,0.,0.],point=[0.,0.,0.],gravity=[0.,0.,-9.81],
                fixed=[[-8.,-8.,0.],[8.,-8.,0.],[0.,8.,0.]],fixed_triangles=[[0,1,2]],
                moving=[c,[.125,0.,.25],[0.,.125,.25]],moving_triangles=[[0,1,2]],traction=[[[0.,0.,0.]]*3])


def fixtures():
    import copy
    cases=[]
    def append(name,**kw):
        a=fixture(name); a.update(kw); cases.append(a); return a
    append('gravity',steps=1000)
    append('proposal_only',steps=0)
    append('neutral',gravity=[0.,0.,0.])
    append('lift_off',gravity=[0.,0.,1.])
    append('tangent',gravity=[1.,0.,-9.81])
    append('tiny_tangent',gravity=[2.**-100,0.,-9.81])
    append('tiny_twist',velocity=[0.,0.,-2.**-100])
    append('spin',omega=[0.,1.,0.])
    append('gap_radius',radius=math.nextafter(.25,math.inf))
    append('radius_inside',radius=math.nextafter(.25,0.))
    append('boundary',fixed=[[0.,0.,0.],[8.,0.,0.],[0.,8.,0.]])
    append('multiple',fixed_triangles=[[0,1,2],[0,2,1]])
    append('clock_absorbed',time=1e20)
    append('owner_dt_limit',h=2.)
    append('impulse_underflow',h=math.ulp(0.),gravity=[0.,0.,-math.ulp(1.)])
    append('reversed_winding',fixed_triangles=[[0,2,1]])
    for axis in range(3):
        for sign in [-1.,1.]:
            a=fixture(f'axis{axis}_side{sign}')
            def perm(p):
                out=[0.,0.,0.]
                out[axis]=sign*p[2];out[(axis+1)%3]=p[0];out[(axis+2)%3]=p[1];return out
            for key in ['center','point','gravity']:a[key]=perm(a[key])
            for key in ['fixed','moving']:a[key]=[perm(p) for p in a[key]]
            cases.append(a)
    for sign in [-1.,1.]:
        c=[3.*sign,4.*sign,0.]
        append(f'oblique{sign}',radius=5.,center=c,gravity=[-3.*sign,-4.*sign,0.],
               fixed=[[-4.,3.,-4.],[4.,-3.,-4.],[0.,0.,4.]],moving=[c,[c[0]+.125,c[1],c[2]],[c[0],c[1]+.125,c[2]]])
    for power in [-3,0,4]:
        a=fixture(f'scale{power}')
        k=2.**power
        for key in ['center','point']:a[key]=[x*k for x in a[key]]
        for key in ['fixed','moving']:a[key]=[[x*k for x in p] for p in a[key]]
        a['radius']*=k;a['inertia']*=k*k;cases.append(a)
    a=fixture('translated');shift=[2.,-4.,8.]
    for key in ['center','point']:a[key]=add(a[key],shift)
    for key in ['fixed','moving']:a[key]=[add(p,shift) for p in a[key]]
    cases.append(a)
    append('p1_force',radius=2.,center=[.5,.5,2.],point=[.5,.5,0.],
           moving=[[0.,0.,2.],[1.,0.,2.],[0.,1.,2.]],traction=[[[0.,0.,12.],[0.,0.,-12.],[0.,0.,-12.]]])
    append('p1_torque',traction=[[[0.,0.,1.]]*3])
    append('p1_tangent',traction=[[[1.,0.,0.],[1.,0.,0.],[1.,0.,0.]]])
    return copy.deepcopy(cases)


def encode(a):
    values=[a['radius'],a['h'],a['steps'],a['mass'],a['inertia'],a['time']]
    for k in ['center','velocity','omega','point','gravity']:values+=a[k]
    for key,tk in [('fixed','fixed_triangles'),('moving','moving_triangles')]:
        values += [len(a[key]),len(a[tk])]
        for p in a[key]: values+=p
        for t in a[tk]: values+=t
    for tri in a['traction']:
        for p in tri:values+=p
    return ' '.join(map(str,values))+'\n'


def loads(a):
    c=vector(a['center']); f=[Q(0)]*3; torque=f.copy()
    for ids,traction in zip(a['moving_triangles'],a['traction']):
        vs=[vector(a['moving'][i]) for i in ids];ts=list(map(vector,traction))
        n=cross(sub(vs[1],vs[0]),sub(vs[2],vs[0])); sq=dot(n,n)
        numerator=math.isqrt(sq.numerator);denominator=math.isqrt(sq.denominator)
        require(numerator*numerator==sq.numerator and denominator*denominator==sq.denominator,'fixture area rational')
        area=Q(numerator,2*denominator)
        for i in range(3):
            nf=scale(area/Q(12),add(scale(Q(2),ts[i]),add(ts[(i+1)%3],ts[(i+2)%3])))
            f=add(f,nf);torque=add(torque,cross(sub(vs[i],c),nf))
    g=scale(scalar(a['mass']),vector(a['gravity']))
    return f,torque,g,add(f,g)


def prediction(a):
    if len(a['fixed_triangles'])!=1:return 'SingleFacetOnly'
    if any(vector(a['velocity'])+vector(a['omega'])):return 'StationaryTwistOnly'
    c=vector(a['center']);p=vector(a['point']);radial=sub(c,p)
    if dot(radial,radial)!=scalar(a['radius'])**2:return 'InvalidRadiusWitness'
    vs=[vector(a['fixed'][i]) for i in a['fixed_triangles'][0]]
    if any(dot(radial,sub(v,p)) for v in vs):return 'InvalidPlaneWitness'
    areas=[dot(cross(sub(vs[(i+1)%3],vs[i]),sub(p,vs[i])),radial) for i in range(3)]
    if any(x==0 for x in areas):return 'FaceBoundary'
    if not(all(x>0 for x in areas) or all(x<0 for x in areas)):return 'OutsideFace'
    _,torque,_,f=loads(a)
    if any(torque):return 'UnsupportedTorque'
    if any(cross(radial,f)):return 'TangentialLoadUnsupported'
    if dot(radial,f)>0:return 'LiftOff'
    if a['time']==1e20:return 'Motion(ClockAbsorbed)'
    if a['h']>1:return 'Motion(InvalidDuration)'
    if a['name']=='impulse_underflow':return 'Contact(ArithmeticFailure)'
    return 'Complete'


def close_vector(actual, expected, label):
    v=vector(actual)
    for x,y in zip(v,expected):
        require(abs(x-y)<=Q(1,10**12)*max(Q(1),abs(y)),label)


def check(a, result):
    require(type(result) is dict and type(result.get('records')) is list and type(result.get('status')) is str,'schema')
    predicted=prediction(a);require(result['status']==predicted,a['name']+' status '+result['status']+' != '+predicted)
    initial=result['initial'];final=result['final']
    for state in [initial,final]+[r['after'] for r in result['records']]:
        require(type(state) is dict,'frame')
        for key in ['center','velocity','omega']:vector(state[key])
        require(type(state['q']) is list and len(state['q'])==4,'quaternion')
        for x in state['q']:scalar(x)
        scalar(state['time_s'])
        for key in ['generation','surface_version']:require(type(state[key]) is int,key)
        for p in state['vertices']:vector(p)
    if predicted!='Complete':
        require(not result['records'] and initial==final,'refusal changed owner');return
    require(len(result['records'])==max(1,a['steps']),'record count')
    mesh,tau,g,f=loads(a);support=scale(Q(-1),f);p=vector(a['point']);c=vector(a['center']);h=scalar(a['h'])
    previous=initial
    for r in result['records']:
        for key,expected in [('mesh_force',mesh),('mesh_torque',tau),('gravity_force',g),('external_force',f),('support_force',support),('support_torque',cross(sub(p,c),support)),('lever',sub(p,c)),('net_force',[Q(0)]*3),('net_torque',[Q(0)]*3),('normal',scale(1/scalar(a['radius']),sub(c,p))),('mesh_impulse',scale(h,mesh)),('gravity_impulse',scale(h,g)),('support_impulse',scale(h,support)),('angular_impulse_defect',[Q(0)]*3),('proxy_force',[Q(0)]*3)]:close_vector(r[key],expected,key)
        for key in ['external_power','support_power','external_work','support_work']:require(scalar(r[key])==0,key)
        for key in ['normal_defect','magnitude','requested_h','actual_elapsed','clock_defect']:scalar(r[key])
        for key in ['probe_point','probe_defect','impulse_defect']:vector(r[key])
        # Native proposal impulse closure is independently recomputed from its
        # retained component products, rather than silently set to zero.
        defect=[float(float(r['mesh_impulse'][i]+r['gravity_impulse'][i])+r['support_impulse'][i]) for i in range(3)]
        require(r['impulse_defect']==defect,'represented component impulse ledger')
        after=r['after']
        for key in ['center','q','velocity','omega','vertices']:require(after[key]==initial[key],'stationary '+key)
        if a['steps']:
            elapsed=scalar(after['time_s'])-scalar(previous['time_s'])
            require(scalar(r['actual_elapsed'])==elapsed and scalar(r['clock_defect'])==elapsed-h,'clock ledger')
            for key in ['generation','surface_version']:require(after[key]==previous[key]+1,'stamp')
        else:require(after==initial,'proposal mutated owner')
        previous=after
    require(final==previous,'final')


def qualify(executable, output):
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    summaries=[];negative=0
    for a in fixtures():
        raw=subprocess.run([str(executable)],input=encode(a),text=True,capture_output=True,check=True,timeout=30)
        (output/(a['name']+'.input')).write_text(encode(a));(output/(a['name']+'.json')).write_text(raw.stdout)
        result=json.loads(raw.stdout);check(a,result)
        summaries.append(dict(name=a['name'],status=result['status'],records=len(result['records'])))
    # Hostile schema is rejected even when bool compares equal to zero/one.
    import copy
    a=fixtures()[0];result=json.loads((output/(a['name']+'.json')).read_text())
    probes=[('support_force',0),('external_power',None),('normal',0),('clock_defect',None)]
    for field,index in probes:
        forged=copy.deepcopy(result)
        if index is None:forged['records'][0][field]=False
        else:forged['records'][0][field][index]=False
        try:check(a,forged)
        except ValueError:negative+=1
        else:raise ValueError('Boolean forgery accepted '+field)
    receipt=dict(qualified=True,scenarios=summaries,negative_schema_probes=negative,
                 executable_sha256=hashlib.sha256(Path(executable).read_bytes()).hexdigest(),
                 scope='single-face stationary equilibrium; rational fixture correspondence; no generic IEEE refinement')
    (output/'oracle-summary.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    print(json.dumps(qualify(args.executable,args.output),indent=2))
