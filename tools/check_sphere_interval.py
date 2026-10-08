"""Independent exact-rational corridor and 80-digit convex-distance sequence labs.

Native states/vertices are compared to Fraction reflection ledgers and independent
Rodrigues drift. Finite edge/face events use convex QP distance minimization and
bisection, not the Rust feature-root formulas. Fixed algebra tolerance is not an
IEEE enclosure; exact Fraction departure checks explain conservative stops.
"""
import argparse, copy, hashlib, json, math, subprocess
from fractions import Fraction as F
from pathlib import Path
import mpmath as mp
from check_sphere_contact import (TOP as OLD_TOP, FRAME, IMPACT, MOVING, REFERENCE,
    closest, classify, keys, require, num, vector, dot, length, frame_schema)
from check_rigid_motion import exponential, quaternion_matrix
mp.mp.dps=80
TOL=mp.mpf('1e-12')
TOP=(OLD_TOP-{'error'})|{'impact_budget','sequence','admission_error'}
RESULT={'status','remaining_interval_s','consumed_interval_s','accepted_segments','accepted_impacts','accounting'}
SEGMENT={'frame','duration_s','remaining_s','clock_defect_s','translation_defect_m','rotation_increment_rad','quaternion_norm_defect','departure','hit','impact'}
DEPARTURE={'triangle','point','kind','from_generation','from_surface_version'}
HIT={'triangle','feature','point','normal','center','barycentric','gap_residual_m'}
ACCOUNT={'summed_impulse_n_s','momentum_defect','summed_predicted_energy_change_j','summed_event_energy_change_j','kinetic_change_j','energy_defect_j','summed_segment_duration_s','represented_elapsed_s','clock_defect_s','duration_defect_s'}
def cases():
    base=dict(name='elastic',radius=.25,e=1.,h=1.,budget=3,mass=2.,inertia=.05,c=[0.,0.,0.],v=[4.,0.,0.],w=[0.,0.,.1],vertices=[[-1.,-4.,-4.],[-1.,4.,-4.],[-1.,0.,4.],[1.,-4.,-4.],[1.,4.,-4.],[1.,0.,4.]],triangles=[[0,1,2],[3,4,5]],axis=0,walls=[-1.,1.])
    out=[]
    def add(name,**kw):
        c=copy.deepcopy(base);c.update(kw,name=name);out.append(c);return c
    add('elastic');add('damped',e=.5,budget=2);add('stationary_after_inelastic',e=0.,budget=1)
    for budget in [0,1,2]:add('budget_'+str(budget),budget=budget)
    add('exact_end_contact',h=.1875,budget=1)
    add('zero_budget_miss',v=[0.,0.,0.],budget=0)
    add('maximum_budget_64',v=[96.,0.,0.],budget=64,vertices=[[p[0],32*p[1],32*p[2]]for p in base['vertices']])
    add('maximum_budget_prefix_63',v=[96.,0.,0.],budget=63,vertices=[[p[0],32*p[1],32*p[2]]for p in base['vertices']])
    add('reverse_winding',triangles=[[0,2,1],[3,5,4]])
    add('reverse_velocity',v=[-4.,0.,0.])
    for axis in [1,2]:
        def perm(p):q=list(p);q[0],q[axis]=q[axis],q[0];return q
        add('axis_'+str(axis),axis=axis,vertices=list(map(perm,base['vertices'])),v=perm(base['v']),w=perm(base['w']))
    shift=[.125,.25,-.5]
    add('translated_nonmidpoint',c=shift,vertices=[[x[i]+shift[i] for i in range(3)]for x in base['vertices']],walls=[-.875,1.125])
    add('scaled_corridor',radius=.5,inertia=.2,v=[8.,0.,0.],vertices=[[2*x for x in p]for p in base['vertices']],walls=[-2.,2.])
    for label,v in [('inward',[1.,0.,0.]),('tangent',[0.,1.,0.]),('separating',[-1.,0.,0.])]:add('initial_touch_'+label,c=[.75,0.,0.],v=v,stop='InitialContact',prefix=0)
    add('duplicate_simultaneous',triangles=[[0,1,2],[3,4,5],[3,4,5]],stop='Simultaneous',prefix=0)
    add('later_spin_cap',w=[0.,0.,1.],stop='RotationLimit',prefix=1)
    add('invalid_budget',budget=65,admission='InvalidRequest')
    add('invalid_restitution',e=1.5,admission='InvalidRequest')
    add('zero_duration',h=0.,admission='InvalidRequest')
    add('radius_inertia',inertia=1.,admission='RadiusInertiaMismatch')
    add('edge_then_face_prefix',radius=.3125,c=[.5,-.1875,1.],v=[0.,0.,-2.],vertices=[[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[-4.,-1.,-4.],[4.,-1.,-4.],[0.,-1.,4.]],axis=None,stop='InvalidDeparture',prefix=2,budget=4)
    add('tangent_floor_then_wall_boundary',radius=.25,c=[0.,0.,1.],v=[1.,0.,-2.],e=0.,vertices=[[-4.,-4.,0.],[4.,-4.,0.],[0.,4.,0.],[1.,-4.,-4.],[1.,4.,-4.],[1.,0.,4.]],axis=None,stop='InitialContact',prefix=2,budget=4)
    return out

def input_text(c):
    a=[c['radius'],c['e'],c['h'],c['budget'],c['mass'],c['inertia'],*c['c'],*c['v'],*c['w'],len(c['vertices']),len(c['triangles'])]
    a += [x for p in c['vertices'] for x in p]+[i for t in c['triangles'] for i in t]
    return ' '.join(map(str,a))+'\n'

def corridor_reference(c):
    center=list(map(F,c['c']));velocity=list(map(F,c['v']));remaining=F(c['h']);elapsed=F(0);events=[];impacts=0
    if c.get('admission'):return events,'admission',remaining
    if c.get('prefix')==0:return events,'stop',remaining
    axis=c['axis'];r=F(c['radius']);lo,hi=map(F,c['walls']);e=F(c['e'])
    while remaining>0:
        if c.get('prefix')==len(events):return events,'stop',remaining
        speed=velocity[axis]
        dt=((hi-r-center[axis])/speed if speed>0 else (lo+r-center[axis])/speed) if speed else None
        hit=dt is not None and 0<dt<=remaining
        if hit and impacts==c['budget']:return events,'budget',remaining
        duration=dt if hit else remaining
        before=list(velocity);center=[x+duration*v for x,v in zip(center,velocity)];elapsed+=duration;remaining-=duration
        point=list(center);normal=[F(0)]*3;tri=None;feature=None;bary=None
        if hit:
            tri=1 if speed>0 else 0;normal[axis]=-1 if speed>0 else 1;point[axis]=hi if speed>0 else lo
            velocity[axis]=-e*velocity[axis];impacts+=1
            point_mp=mp.matrix([num(float(x)) for x in point]);vertices=list(map(vector,c['vertices']));feature,bary=classify(point_mp,[vertices[i] for i in c['triangles'][tri]])
            require(feature=='Face','corridor finite interior premise')
        events.append(dict(duration=duration,time=elapsed,remaining=remaining,center=list(center),velocity=list(velocity),before=before,point=point,normal=normal,triangle=tri,feature=feature,bary=bary))
    return events,'complete',remaining

def finite_reference(c):
    # Independent repeated-distance oracle for the two-impact finite edge fixture.
    center=vector(c['c']);v=vector(c['v']);remaining=num(c['h']);elapsed=mp.mpf(0);vertices=list(map(vector,c['vertices']));previous=None;events=[]
    for _ in range(c['prefix']):
        hits=[]
        for ti,indices in enumerate(c['triangles']):
            if ti==previous:continue
            tri=[vertices[i] for i in indices]
            def residual(t):q=closest(center+t*v,tri);d=center+t*v-q;return dot(d,d)-num(c['radius'])**2
            require(residual(0)>0,'independent other-facet clearance')
            lo,hi=mp.mpf(0),remaining
            for _ in range(220):
                x=(2*lo+hi)/3;y=(lo+2*hi)/3
                if residual(x)<residual(y):hi=y
                else:lo=x
            tm=min([mp.mpf(0),remaining,(lo+hi)/2],key=residual)
            if residual(tm)>mp.mpf('1e-40'):continue
            lo,hi=mp.mpf(0),tm
            for _ in range(260):
                mid=(lo+hi)/2
                if residual(mid)>0:lo=mid
                else:hi=mid
            dt=(lo+hi)/2;p=center+dt*v;q=closest(p,tri);n=(p-q)/length(p-q);feature,bary=classify(q,tri)
            hits.append((dt,ti,p,q,n,feature,bary))
        require(hits,'independent finite event exists')
        dt,ti,p,q,n,feature,bary=min(hits,key=lambda x:x[0]);before=v.copy();v=v-(1+num(c['e']))*dot(v,n)*n
        elapsed+=dt;remaining-=dt;center=p
        events.append(dict(duration=dt,time=elapsed,remaining=remaining,center=p.copy(),velocity=v.copy(),before=before,point=q,normal=n,triangle=ti,feature=feature,bary=bary));previous=ti
    return events,'stop',remaining

def real(x):
    if isinstance(x,F):return mp.mpf(x.numerator)/x.denominator
    if isinstance(x,mp.mpf):return x
    return num(x)
def mv(x):return mp.matrix([real(v)for v in x])
def certificate(frame,point,triangle,c):
    center=list(map(F,frame['center']));p=list(map(F,point));v=list(map(F,frame['velocity']));a=[x-y for x,y in zip(center,p)]
    gap=sum(x*x for x in a)-F(c['radius'])**2
    support=[sum(a[i]*(F(c['vertices'][j][i])-p[i])for i in range(3))for j in c['triangles'][triangle]]
    speed=sum(a[i]*v[i]for i in range(3))
    return gap,support,speed

def evaluate(raw,c,reference):
    keys(raw,TOP);require(raw['collision_shape']=='declared_sphere' and raw['moving_mesh_role']=='render_and_traction','collider roles')
    for k,ck in [('radius_m','radius'),('restitution','e'),('requested_interval_s','h'),('mass_kg','mass'),('inertia_kg_m2','inertia')]:require(num(raw[k])==num(c[ck]),'physical input')
    require(type(raw['impact_budget'])is int and raw['impact_budget']==c['budget'],'budget input')
    require(raw['static_vertices']==c['vertices'] and raw['static_triangles']==c['triangles'] and raw['moving_triangles']==MOVING,'geometry input')
    for key,value in [('relative_tolerance',1e-12),('max_gap_residual_m',1e-10),('simultaneous_window_s',1e-10)]:require(num(raw[key])==num(value),'numerical policy')
    require(type(raw['static_triangle_limit'])is int and raw['static_triangle_limit']==64,'triangle cap')
    require(type(raw['sequence'])is list,'sequence type');frame_schema(raw['before']);frame_schema(raw['after'])
    require(len(raw['sequence'])<=min(c['budget'],64)+1,'bounded sequence')
    maximum=0.;physical=dict(time_s=0.,point_m=0.,velocity_m_s=0.,normal=0.)
    def compare(x,y,kind=None):
        nonlocal maximum
        error=abs(real(x)-real(y));require(error<=TOL,'algebra comparison '+str(error));maximum=max(maximum,float(error))
        if kind:physical[kind]=max(physical[kind],float(error))
    def cv(x,y,kind=None):
        vector(x)
        for i in range(3):compare(x[i],y[i],kind)
    def pose(frame,center,v,t,index):
        frame_schema(frame);cv(frame['center'],center,'point_m');cv(frame['velocity'],v,'velocity_m_s');cv(frame['omega'],c['w']);compare(frame['time_s'],t,'time_s')
        require(frame['generation']==2+index and frame['surface_version']==4+index and frame['peak_payload_bytes']==816,'actual publication metadata')
        rotation=exponential(vector(c['w']),real(t));qrotation=quaternion_matrix(frame['q'])
        compare(sum(num(q)**2 for q in frame['q']),1)
        for i in range(3):
            for j in range(3):compare(qrotation[i,j],rotation[i,j])
        for actual,axis in zip(frame['vertices'],REFERENCE):cv(actual,mv(center)+rotation*mp.matrix(axis)*num(c['radius'])/2,'point_m')
    pose(raw['before'],c['c'],c['v'],0,0);require(raw['before']['q']==[1.,0.,0.,0.],'initial orientation')
    events,status,remaining=reference
    if c.get('admission'):
        require(type(raw['admission_error'])is str and c['admission']in raw['admission_error']and raw['result']is None,'explicit admission refusal');require(raw['sequence']==[]and raw['after']==raw['before'],'admission atomic')
        return dict(maximum_algebra_error=maximum,physical_errors=physical,status='admission',segments=0)
    require(raw['admission_error']is None,'unexpected admission');result=raw['result'];keys(result,RESULT)
    require(type(result['status'])is str,'status type')
    expected_status='Complete'if status=='complete'else 'ImpactBudgetExhausted'if status=='budget'else c['stop']
    require(result['status']==expected_status if status!='stop' else expected_status in result['status'],'honest interval status')
    require(len(raw['sequence'])==len(events),'all accepted records')
    for k,value in [('accepted_segments',len(events)),('accepted_impacts',sum(e['triangle']is not None for e in events))]:require(type(result[k])is int and result[k]==value,'count integer')
    compare(result['remaining_interval_s'],remaining,'time_s');compare(result['consumed_interval_s'],num(c['h'])-real(remaining))
    require((result['status']=='Complete')==(num(result['remaining_interval_s'])==0),'completion requires zero remainder')
    last=raw['before'];last_hit=None;impulse=mp.zeros(3,1);predicted=mp.mpf(0);event_loss=mp.mpf(0);durations=mp.mpf(0)
    for index,(s,e)in enumerate(zip(raw['sequence'],events)):
        keys(s,SEGMENT);pose(s['frame'],e['center'],e['velocity'],e['time'],index+1)
        compare(s['duration_s'],e['duration'],'time_s');compare(s['remaining_s'],e['remaining'],'time_s')
        dt=num(s['duration_s']);durations+=dt
        compare(s['clock_defect_s'],num(s['frame']['time_s'])-num(last['time_s'])-dt)
        cv(s['translation_defect_m'],vector(s['frame']['center'])-vector(last['center'])-dt*vector(last['velocity']))
        compare(s['rotation_increment_rad'],length(vector(c['w']))*dt);compare(s['quaternion_norm_defect'],mp.sqrt(sum(num(q)**2 for q in s['frame']['q']))-1)
        if last_hit is None:require(s['departure']is None,'initial no exclusion')
        else:
            d=s['departure'];keys(d,DEPARTURE);require(type(d['triangle'])is int and d['triangle']==last_hit['triangle']and d['point']==last_hit['point'],'bound exclusion')
            for k,value in [('from_generation',last['generation']),('from_surface_version',last['surface_version'])]:require(type(d[k])is int and d[k]==value,'departure stamp')
            gap,support,speed=certificate(last,d['point'],d['triangle'],c);require(gap>=0 and max(support)<=0 and speed>=0,'exact departure premises')
            require(d['kind']==('Tangent'if speed==0 else 'Separating'),'exact departure kind')
            a=[F(x)-F(y)for x,y in zip(last['center'],d['point'])];delta=[F(x)-F(y)for x,y in zip(s['frame']['center'],last['center'])];require(sum(x*y for x,y in zip(a,delta))>=0,'actual endpoint certificate')
        if e['triangle']is None:require(s['hit']is None and s['impact']is None,'terminal miss')
        else:
            h=s['hit'];keys(h,HIT);require(type(h['triangle'])is int and h['triangle']==e['triangle']and h['feature']==e['feature'],'finite feature')
            for k in ['point','normal','center','barycentric']:cv(h[k],e["bary" if k=="barycentric" else k], 'normal'if k=='normal'else'point_m'if k in ['point','center']else None)
            compare(h['gap_residual_m'],length(vector(h['center'])-vector(h['point']))-num(c['radius']))
            i=s['impact'];keys(i,IMPACT)
            for k in IMPACT-{'normal','impulse_n_s','momentum_defect'}:num(i[k])
            n=vector(i['normal']);cv(i['normal'],e['normal'],'normal');compare(i['normal_norm_defect'],length(n)-1)
            compare(i['gap_residual_m'],length(vector(s['frame']['center'])-vector(h['point']))-num(c['radius']))
            vb=mv(e['before']);va=mv(e['velocity']);mass=num(c['mass']);vn=dot(vb,mv(e['normal']));J=-(1+num(c['e']))*mass*vn*mv(e['normal'])
            cv(i['impulse_n_s'],J);cv(i['momentum_defect'],mass*(vector(s['frame']['velocity'])-vector(last['velocity']))-vector(i['impulse_n_s']))
            compare(i['normal_velocity_before_m_s'],dot(vector(last['velocity']),n));compare(i['normal_velocity_after_m_s'],dot(vector(s['frame']['velocity']),n));compare(i['restitution_defect_m_s'],dot(vector(s['frame']['velocity']),n)+num(c['e'])*dot(vector(last['velocity']),n))
            spin=num(c['inertia'])/2*dot(vector(c['w']),vector(c['w']));compare(i['kinetic_before_j'],mass/2*dot(vb,vb)+spin);compare(i['kinetic_after_j'],mass/2*dot(va,va)+spin)
            loss=-mass/2*(1-num(c['e'])**2)*vn**2;compare(i['predicted_energy_change_j'],loss);compare(i['energy_defect_j'],num(i['kinetic_after_j'])-num(i['kinetic_before_j'])-num(i['predicted_energy_change_j']))
            impulse+=vector(i['impulse_n_s']);predicted+=num(i['predicted_energy_change_j']);event_loss+=num(i['kinetic_after_j'])-num(i['kinetic_before_j'])
        last=s['frame'];last_hit=s['hit']
    require(raw['after']==last,'actual accepted prefix endpoint')
    if c.get('stop')=='InvalidDeparture':
        require(last_hit is not None,'departure halt after real impact');gap,support,speed=certificate(last,last_hit['point'],last_hit['triangle'],c);require(gap<0 or max(support)>0 or speed<0,'exact reason for conservative refusal')
    a=result['accounting'];keys(a,ACCOUNT);cv(a['summed_impulse_n_s'],impulse);cv(a['momentum_defect'],num(c['mass'])*(vector(last['velocity'])-vector(raw['before']['velocity']))-impulse)
    kinetic=num(c['mass'])/2*(dot(vector(last['velocity']),vector(last['velocity']))-dot(vector(raw['before']['velocity']),vector(raw['before']['velocity'])))
    for k,value in [('summed_predicted_energy_change_j',predicted),('summed_event_energy_change_j',event_loss),('kinetic_change_j',kinetic),('energy_defect_j',kinetic-predicted),('summed_segment_duration_s',durations),('represented_elapsed_s',num(last['time_s'])),('clock_defect_s',num(last['time_s'])-durations),('duration_defect_s',num(result['consumed_interval_s'])-durations)]:compare(a[k],value)
    return dict(maximum_algebra_error=maximum,physical_errors=physical,status=result['status'],segments=len(events))

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args();output=args.output.resolve();root=Path(__file__).resolve().parents[1]
    require(not output.exists() and not output.is_relative_to(root),'fresh external output');output.mkdir(parents=True)
    results=[];saved=[]
    for c in cases():
        reference=corridor_reference(c)if c['axis']is not None else finite_reference(c)
        text=input_text(c);(output/(c['name']+'.input')).write_text(text)
        run=subprocess.run([args.executable],input=text,text=True,capture_output=True,timeout=10);require(run.returncode==0,'native bridge '+run.stderr);(output/(c['name']+'.native.json')).write_text(run.stdout)
        raw=json.loads(run.stdout);r=evaluate(raw,c,reference);results.append(dict(name=c['name'],**r));saved.append((c,raw,reference))
        def serial(x):
            if isinstance(x,dict):return {k:serial(v)for k,v in x.items()}
            if isinstance(x,(list,tuple)):return [serial(v)for v in x]
            if isinstance(x,F):return str(x)
            if isinstance(x,mp.matrix):return [str(v)for v in x]
            if isinstance(x,mp.mpf):return str(x)
            return x
        (output/(c['name']+'.reference.json')).write_text(json.dumps(serial(reference),indent=2)+'\n')
    probes=[];c,raw,reference=saved[0]
    mutations={
        'positive_remainder_complete':lambda x:x['result'].__setitem__('remaining_interval_s',.1),
        'lost_record':lambda x:x['sequence'].pop(),
        'invented_endpoint':lambda x:x['after']['center'].__setitem__(0,99.),
        'wrong_mesh':lambda x:x['sequence'][1]['frame']['vertices'][0].__setitem__(1,99.),
        'bad_quaternion':lambda x:x['sequence'][1]['frame']['q'].__setitem__(0,2.),
        'false_departure':lambda x:x['sequence'][1]['departure'].__setitem__('triangle',1- x['sequence'][1]['departure']['triangle']),
        'wrong_feature':lambda x:x['sequence'][0]['hit'].__setitem__('feature','Vertex(0)'),
        'boolean_count':lambda x:x['result'].__setitem__('accepted_segments',True),
        'nan_energy':lambda x:x['result']['accounting'].__setitem__('energy_defect_j',float('nan')),
        'lost_impulse':lambda x:x['result']['accounting']['summed_impulse_n_s'].__setitem__(0,0.),
        'invented_policy':lambda x:x.__setitem__('max_gap_residual_m',1.),
        'extra_field':lambda x:x['sequence'][0].__setitem__('unreviewed',0),
    }
    for name,mutate in mutations.items():
        bad=copy.deepcopy(raw);mutate(bad)
        try:evaluate(bad,c,reference)
        except (ValueError,KeyError,TypeError,IndexError):probes.append(name)
        else:raise ValueError('negative probe accepted '+name)
    report=dict(cases=results,negative_probes=probes,binary_sha256=hashlib.sha256(args.executable.read_bytes()).hexdigest(),algebra_tolerance=str(TOL),reference='Fraction corridor; independent80digit convex-distance finite edges; Rodrigues stored-mesh comparison',claims='finite fixture evidence; no IEEE/CCD/global contact proof')
    report['evidence_sha256']={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in output.iterdir()if p.is_file()}
    (output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(cases=len(results),negative_probes=len(probes),maximum_algebra_error=max(r['maximum_algebra_error']for r in results)),indent=2))
if __name__=='__main__':main()
