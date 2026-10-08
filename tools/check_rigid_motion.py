"""Fresh actual-native trajectories versus independent 80-digit matrix drift.

Force/torque use independent degree-two triangle cubature. Rotation uses
Rodrigues matrices rather than the production quaternion product. Fixed absolute
1e-12; numerical reference, no IEEE enclosure or global dynamics theorem.
"""
import argparse,hashlib,json,math,subprocess,sys
from fractions import Fraction
from pathlib import Path
import mpmath as mp
import check_mesh_traction as mesh
mp.mp.dps=80
ROOT=Path(__file__).resolve().parents[1]
TOLERANCE=1e-12

def number(x):
    if type(x) not in (int,float) or not math.isfinite(x):raise ValueError('finite native number required')
    q=Fraction(x);return mp.mpf(q.numerator)/q.denominator

def vec(v):return mp.matrix([number(x) for x in v])
def cross(a,b):return mp.matrix([a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]])
def dot(a,b):return sum(a[i]*b[i] for i in range(3))
def norm(v):return mp.sqrt(dot(v,v))
def exponential(omega,h):
    n=norm(omega)
    if n==0:return mp.eye(3)
    a=omega/n;S=mp.matrix([[0,-a[2],a[1]],[a[2],0,-a[0]],[-a[1],a[0],0]])
    theta=n*h;return mp.eye(3)+mp.sin(theta)*S+(1-mp.cos(theta))*(S*S)
def quaternion_matrix(values):
    w,x,y,z=map(number,values)
    return mp.matrix([[1-2*(y*y+z*z),2*(x*y-w*z),2*(x*z+w*y)],
                      [2*(x*y+w*z),1-2*(x*x+z*z),2*(y*z-w*x)],
                      [2*(x*z-w*y),2*(y*z+w*x),1-2*(x*x+y*y)]])
def loads(mode,vertices,triangles,ordinal):
    if mode=='pressure':return [[2+v[0]/4-v[1]/2+3*v[2]/4 for v in [vertices[j] for j in t]] for t in triangles]
    t=[mp.mpf(0)]*3
    if mode=='uniform':t=list(map(number,[.2,-.1,.05]))
    return [[list(map(number,[1.2,.3,.6])) if mode=='moving' and i in (2,3) else t for _ in range(3)] for i in range(len(triangles))]
def cubature(mode,vertices,triangles,loading,com):
    F=mp.zeros(3,1);tau=mp.zeros(3,1)
    for ti,indices in enumerate(triangles):
        points=[vertices[i] for i in indices];N=cross(points[1]-points[0],points[2]-points[0]);double=norm(N);area=double/2
        tr=[-p*N/double for p in loading[ti]] if mode=='pressure' else [mp.matrix(t) for t in loading[ti]]
        for special in range(3):
            b=[mp.mpf(2)/3 if i==special else mp.mpf(1)/6 for i in range(3)]
            p=sum((b[i]*points[i] for i in range(3)),mp.zeros(3,1));t=sum((b[i]*tr[i] for i in range(3)),mp.zeros(3,1));f=area*t/3
            F+=f;tau+=cross(p-com,f)
    return F,tau
def cases():
    return [('translation-free','free',8,.125,[1,2,3],[0,0,0]),
            ('axis-spin-free','free',8,.125,[0,0,0],[0,0,1]),
            ('long-free-spin-sign-crossing','free',64,.125,[0,0,0],[0,0,1]),
            ('oblique-free-spin','free',8,.125,[.3,-.2,.1],[.4,.7,-.5]),
            ('constant-force-coarse','uniform',8,.125,[0,0,0],[0,0,0]),
            ('constant-force-fine','uniform',16,.0625,[0,0,0],[0,0,0]),
            ('moving-force-torque','moving',16,.0625,[.4,.1,0],[.15,-.2,.25]),
            ('moving-force-torque-longer','moving',32,.0625,[0,0,0],[.1,.2,.3]),
            ('rotated-affine-pressure','pressure',16,.0625,[0,0,0],[.2,-.3,.4])]
def compare(actual,expected,label):
    if isinstance(expected,mp.matrix):
        if not isinstance(actual,list) or len(actual)!=3:raise ValueError(label+' vector')
        return max(compare(a,expected[i],label) for i,a in enumerate(actual))
    error=abs(number(actual)-expected)
    if error>mp.mpf(str(TOLERANCE)):raise AssertionError(f'{label}: {error}')
    return float(error)
def shape(value,dimensions,label):
    if not dimensions:
        number(value);return
    if not isinstance(value,list) or len(value)!=dimensions[0]:raise ValueError(label+' dimensions')
    for x in value:shape(x,dimensions[1:],label)
def frame_schema(frame):
    keys={'time_s','center_of_mass','orientation','velocity_m_s','angular_velocity_rad_s','generation','surface_version','vertices'}
    if not isinstance(frame,dict) or set(frame)!=keys:raise ValueError('frame keys')
    shape(frame['time_s'],[],'time')
    for name in ['center_of_mass','velocity_m_s','angular_velocity_rad_s']:shape(frame[name],[3],name)
    shape(frame['orientation'],[4],'orientation');shape(frame['vertices'],[8,3],'vertices')
    for name in ['generation','surface_version']:
        if type(frame[name]) is not int or not 0<=frame[name]<=2**64-1:raise ValueError('integer stamp')
def validate(raw,case):
    keys={'mode','requested_dt_s','mass_kg','spherical_inertia_kg_m2','triangles','initial','steps'}
    if not isinstance(raw,dict) or set(raw)!=keys:raise ValueError('trajectory keys')
    topology=[[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],[3,7,6],[3,6,2],[0,4,7],[0,7,3],[1,2,6],[1,6,5]]
    shape(raw['triangles'],[12,3],'topology')
    if raw['triangles']!=topology or any(type(i) is not int for t in raw['triangles'] for i in t):raise ValueError('initial topology')
    if number(raw['requested_dt_s'])!=number(case[3]) or number(raw['mass_kg'])!=2 or number(raw['spherical_inertia_kg_m2'])!=1:raise ValueError('body/step metadata')
    frame_schema(raw['initial'])
    if not isinstance(raw['steps'],list) or len(raw['steps'])!=case[2]:raise ValueError('step count')
    scalar={'kinetic_before_j','kinetic_after_j','impulse_work_j','energy_defect_j','represented_elapsed_s','clock_defect_s','rotation_increment_rad','quaternion_norm_defect'}
    vectors={'force_n','torque_n_m','momentum_defect','angular_momentum_defect','translation_defect_m'}
    expected=scalar|vectors|{'traction','pressure','peak_payload_bytes','stored'}
    for step in raw['steps']:
        if not isinstance(step,dict) or set(step)!=expected:raise ValueError('step keys')
        for key in scalar:shape(step[key],[],key)
        for key in vectors:shape(step[key],[3],key)
        shape(step['traction'],[12,3,3],'traction');shape(step['pressure'],[12,3],'pressure')
        if type(step['peak_payload_bytes']) is not int or step['peak_payload_bytes']!=1152:raise ValueError('fixture peak capacity payload')
        frame_schema(step['stored'])
def evaluate(raw,case):
    validate(raw,case)
    name,mode,n,h0,v0,w0=case;h=number(h0);initial=raw['initial'];triangles=raw['triangles'];reference=list(map(vec,[[-.5,-.5,-.5],[.5,-.5,-.5],[.5,.5,-.5],[-.5,.5,-.5],[-.5,-.5,.5],[.5,-.5,.5],[.5,.5,.5],[-.5,.5,.5]]));com=mp.zeros(3,1);V=vec(v0);omega=vec(w0);R=mp.eye(3);time=mp.mpf(0);mass=2;I=1;maximum=0.;records=[]
    for p,r in zip(initial['vertices'],reference):compare(p,r,'initial vertex')
    compare(initial['center_of_mass'],mp.zeros(3,1),'initial COM');compare(initial['velocity_m_s'],V,'initial V');compare(initial['angular_velocity_rad_s'],omega,'initial omega');compare(initial['time_s'],mp.mpf(0),'initial time')
    if initial['orientation']!=[1.,0.,0.,0.] or initial['generation']!=2 or initial['surface_version']!=4:raise ValueError('initial pose/stamps')
    if raw['mode']!=mode or len(raw['steps'])!=n:raise ValueError('native trajectory schema')
    for k,step in enumerate(raw['steps']):
        vertices=[com+R*r for r in reference];values=loads(mode,vertices,triangles,k);F,tau=cubature(mode,vertices,triangles,values,com)
        maximum=max(maximum,compare(step['force_n'],F,'force'),compare(step['torque_n_m'],tau,'torque'))
        # Verify actual supplied loads, not just the result they ought to yield.
        unused=step['traction'] if mode=='pressure' else step['pressure']
        def zeros(v):return all(zeros(x) for x in v) if isinstance(v,list) else number(v)==0
        if not zeros(unused):raise ValueError('unused loading fields')
        native_loads=step['pressure'] if mode=='pressure' else step['traction']
        for a,e in zip(native_loads,values):
            for av,ev in zip(a,e):
                if mode=='pressure':maximum=max(maximum,compare(av,ev,'pressure'))
                else:maximum=max(maximum,compare(av,mp.matrix(ev),'traction'))
        oldV=V.copy();oldW=omega.copy();oldT=(mass*dot(V,V)+I*dot(omega,omega))/2
        V+=h*F/mass;omega+=h*tau/I;com+=h*V;R=exponential(omega,h)*R;time+=h
        T=(mass*dot(V,V)+I*dot(omega,omega))/2;work=h*dot(F,(oldV+V)/2)+h*dot(tau,(oldW+omega)/2)
        stored=step['stored'];maximum=max(maximum,compare(stored['center_of_mass'],com,'COM'),compare(stored['velocity_m_s'],V,'velocity'),compare(stored['angular_velocity_rad_s'],omega,'omega'),compare(stored['time_s'],time,'clock'),compare(step['kinetic_before_j'],oldT,'oldT'),compare(step['kinetic_after_j'],T,'newT'),compare(step['impulse_work_j'],work,'work'))
        if stored['generation']!=3+k or stored['surface_version']!=5+k:raise ValueError('native stamps')
        for p,r in zip(stored['vertices'],reference):maximum=max(maximum,compare(p,com+R*r,'actual vertex'))
        previous=raw['initial'] if k==0 else raw['steps'][k-1]['stored']
        actual_elapsed=number(stored['time_s'])-number(previous['time_s'])
        actual_displacement=vec(stored['center_of_mass'])-vec(previous['center_of_mass'])-h*vec(stored['velocity_m_s'])
        maximum=max(maximum,compare(step['represented_elapsed_s'],actual_elapsed,'actual elapsed'),compare(step['clock_defect_s'],actual_elapsed-h,'actual clock defect'),compare(step['rotation_increment_rad'],h*norm(vec(stored['angular_velocity_rad_s'])),'actual rotation'),compare(step['translation_defect_m'],actual_displacement,'actual translation defect'))
        qr=quaternion_matrix(stored['orientation']);maximum=max(maximum,float(max(abs(qr[i,j]-R[i,j]) for i in range(3) for j in range(3))))
        if maximum>TOLERANCE:raise AssertionError('native orientation differs from independent matrix')
        maximum=max(maximum,compare(step['momentum_defect'],mp.zeros(3,1),'momentum residual'),compare(step['angular_momentum_defect'],mp.zeros(3,1),'angular residual'),compare(step['energy_defect_j'],mp.mpf(0),'energy residual'),compare(step['quaternion_norm_defect'],mp.mpf(0),'unit residual'))
        records.append(dict(step=k,reference_com=[str(x) for x in com],reference_matrix=[[str(R[i,j]) for j in range(3)] for i in range(3)],reference_time_s=str(time),maximum_absolute_error=maximum))
    if mode=='uniform' and v0==[0,0,0] and w0==[0,0,0]:
        a=mp.matrix(list(map(number,[.2,-.1,.05])))*3
        analytic=a*time*time/2;explicit=a*time*h/2
        if max(abs(com[i]-analytic[i]-explicit[i]) for i in range(3))>mp.mpf('1e-65'):raise AssertionError('constant force error anchor')
    return maximum,records

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();binary=a.executable.resolve(strict=True);output=a.output.resolve()
    if output.exists() or output.is_relative_to(ROOT):raise ValueError('fresh external output required')
    output.mkdir(parents=True);snapshot=mesh.source_snapshot(ROOT);head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip();binary_sha=mesh.sha(binary)
    receipt=dict(source_head=head,source_sha256=snapshot,executable_sha256=binary_sha,absolute_tolerance=TOLERANCE,mpmath_version=mp.__version__,reference_digits=mp.mp.dps,oracle='independent degree2 cubature and world Rodrigues matrix exponential',fixtures=[],qualified=False,held_campaigns=0)
    try:
        for case in cases():
            name,mode,n,h,v,w=case;text=' '.join(map(str,[mode,n,h,*v,*w]))+'\n';prefix=output/name;prefix.with_suffix('.input.txt').write_text(text)
            r=subprocess.run([str(binary)],input=text,text=True,capture_output=True,timeout=10);prefix.with_suffix('.stdout.json').write_text(r.stdout);prefix.with_suffix('.stderr.txt').write_text(r.stderr)
            if r.returncode:raise AssertionError(name+': '+r.stderr)
            native=json.loads(r.stdout,object_pairs_hook=mesh.strict_object,parse_constant=lambda x:(_ for _ in ()).throw(ValueError(x)));err,records=evaluate(native,case);prefix.with_suffix('.reference.json').write_text(json.dumps(records,indent=2)+'\n')
            receipt['fixtures'].append(dict(name=name,steps=n,maximum_absolute_error=err,stdout_sha256=mesh.sha(prefix.with_suffix('.stdout.json'))))
        # Checker must refuse an actual moving vertex mutation at unchanged cap.
        import copy
        c=cases()[6];raw=json.loads((output/(c[0]+'.stdout.json')).read_text())
        mutants=[]
        bad=copy.deepcopy(raw);bad['steps'][-1]['stored']['vertices'][0][0]+=1e-10;mutants.append(bad)
        bad=copy.deepcopy(raw);bad['steps'][-1]['stored']['vertices'].pop();mutants.append(bad)
        bad=copy.deepcopy(raw);bad['steps'][0]['traction'][0].pop();mutants.append(bad)
        bad=copy.deepcopy(raw);bad['initial']['vertices'][0][0]+=.001;mutants.append(bad)
        bad=copy.deepcopy(raw);bad['initial']['generation']=True;mutants.append(bad)
        bad=copy.deepcopy(raw);bad['steps'][-1]['represented_elapsed_s']+=1e-10;mutants.append(bad)
        for bad in mutants:
            try:evaluate(bad,c)
            except (ValueError,AssertionError):pass
            else:raise AssertionError('mutated native record accepted')
        receipt['negative_checker_probes']=len(mutants)
        if snapshot!=mesh.source_snapshot(ROOT) or binary_sha!=mesh.sha(binary) or head!=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip():raise ValueError('source/executable drift')
        receipt['qualified']=True
    except Exception as e:receipt['failure']=str(e);raise
    finally:
        receipt['evidence_sha256']={p.name:mesh.sha(p) for p in sorted(output.iterdir()) if p.is_file()};(output/'qualification.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(qualified=True,cases=len(receipt['fixtures']),native_steps=sum(c['steps'] for c in receipt['fixtures']),maximum_absolute_error=max(c['maximum_absolute_error'] for c in receipt['fixtures']),output=str(output))))
if __name__=='__main__':main()
