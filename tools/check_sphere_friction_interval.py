"""Fresh bounded Coulomb labs: actual stored-state events, full ledgers and pose.

80-digit convex-distance CCD (or independent exact-rational interior planes),
quadratic disk minimization and piecewise Rodrigues rotation. Exact Fraction
trajectories additionally cover dyadic repeated corridors. Stored-state event
audits explicitly condition on each published prefix; they do not silently
claim an unrounded global trajectory or an IEEE enclosure. Strict checks remain
active under python -O. The isolated checker audits normalized local diagnostic
frames; actual native orientation/mesh are checked separately here.
"""
import argparse, copy, hashlib, json, math, subprocess
from fractions import Fraction as F
from pathlib import Path
import mpmath as mp
import check_sphere_contact as contact
import check_sphere_friction as friction
import check_sphere_interval as interval
from check_sphere_contact import require, num, vector, dot, length, keys
from check_rigid_motion import exponential, quaternion_matrix

mp.mp.dps=80
TOL=mp.mpf('2e-11')
TOP=interval.TOP|{'coulomb_coefficient'}
SEGMENT=interval.SEGMENT|{'normal_proposal','represented_elapsed_s'}
ACCOUNT=(interval.ACCOUNT-{'momentum_defect'})|{
    'summed_spin_impulse_n_m_s','summed_world_impulse_n_m_s','momentum_defect_n_s',
    'spin_momentum_defect_n_m_s','world_angular_defect_n_m_s',
    'summed_point_midpoint_work_j','work_defect_j'}
VECTOR_ACCOUNT={'summed_impulse_n_s','summed_spin_impulse_n_m_s','summed_world_impulse_n_m_s',
                'momentum_defect_n_s','spin_momentum_defect_n_m_s','world_angular_defect_n_m_s'}

def fixtures():
    out=interval.cases()
    for c in out:
        c['mu']=0.
        if c.get('admission'):c['admission']='Interval('+c['admission']+')'
        if c.get('status'):c['status']='Stopped(Contact('+c['status'][8:-1]+'))'
    def add(name,**changes):
        c=copy.deepcopy(out[0]);c.update(name=name,mu=1/1024,h=.2,budget=3,
            v=[4.,1.,0.],vertices=[[-.375,-128.,-128.],[-.375,128.,-128.],[-.375,0.,128.],
             [.375,-128.,-128.],[.375,128.,-128.],[.375,0.,128.]],walls=[-.375,.375],rational=True)
        c.update(changes);out.append(c);return c
    add('repeated_dyadic_capped')
    add('dyadic_budget',budget=2)
    add('dyadic_terminal_miss',h=.04,budget=1)
    add('friction_rotation_stop',mu=.01,h=1.,budget=8,vertices=copy.deepcopy(out[0]['vertices']),
        walls=[-1.,1.],rational=False,prefix=1,status='Stopped(Contact(Motion(RotationLimit)))',stop='RotationLimit')
    add('cancel_rotation_stop',mu=.1,h=1.,budget=8,vertices=copy.deepcopy(out[0]['vertices']),
        walls=[-1.,1.],rational=False,prefix=1,status='Stopped(Contact(Motion(RotationLimit)))',stop='RotationLimit')
    add('rounded_departure_stop',mu=.1,budget=8,rational=False,prefix=2,
        status='Stopped(Contact(InvalidDeparture))',stop='InvalidDeparture')
    add('spin_driven_end',v=[4.,0.,0.],h=.03125,mu=.1,rational=False)
    add('cancellation_exact_end',h=.03125,mu=.1,rational=False)
    add('invalid_mu',mu=-.1,admission='InvalidCoefficient',rational=False)
    for axis in [1,2]:
        c=add('capped_axis_'+str(axis));
        def perm(p):p=list(p);p[0],p[axis]=p[axis],p[0];return p
        c.update(axis=axis,vertices=list(map(perm,c['vertices'])),v=perm(c['v']),
                 w=[-x for x in perm(c['w'])]) # improper map: spin is axial
    shift=[.125,.25,-.5]
    c=add('translated_capped');c.update(c=shift,vertices=[[p[i]+shift[i]for i in range(3)]for p in c['vertices']],walls=[-.25,.5])
    add('reversed_capped_winding',triangles=[[0,2,1],[3,5,4]])
    # Mode ties use a first/endpoint isolated hit, with exact stop=cap=4.
    for label,mu in [('tie',.5),('tie_below',math.nextafter(.5,0.)),('tie_above',math.nextafter(.5,1.))]:
        add(label,c=[0.,0.,.5],v=[7.,0.,-2.],w=[0.,0.,0.],h=.125,mu=mu,budget=1,
            axis=None,vertices=[[-4.,-4.,0.],[4.,-4.,0.],[0.,4.,0.]],triangles=[[0,1,2]],rational=False)
    add('tiny_nonzero_slip',c=[0.,0.,.5],v=[2**-40,0.,-2.],w=[0.,0.,0.],h=.125,mu=.5,budget=1,
        axis=None,vertices=[[-4.,-4.,0.],[4.,-4.,0.],[0.,4.,0.]],triangles=[[0,1,2]],rational=False)
    add('finite_edge_friction_end',c=[.5,-.1875,1.],v=[0.,0.,-2.],w=[0.,0.,0.],radius=.3125,mu=.03125,h=.375,budget=1,axis=None,vertices=[[0.,0.,0.],[4.,0.,0.],[0.,4.,0.]],triangles=[[0,1,2]],rational=False)
    return out

def native_input(c):
    a=interval.input_text(c).split();a.insert(2,str(c['mu']));return ' '.join(a)+'\n'

def event_reference(c,before,remaining,previous):
    local=copy.deepcopy(c);local.update(c=before['center'],v=before['velocity'],w=before['omega'],h=remaining)
    if c.get('axis') is not None:
        axis=c['axis'];center=list(map(F,before['center']));v=list(map(F,before['velocity']));r=F(c['radius']);lo,hi=map(F,c['walls'])
        speed=v[axis];dt=((hi-r-center[axis])/speed if speed>0 else (lo+r-center[axis])/speed)if speed else None
        if dt is None or dt>F(remaining):return dict(time=num(remaining),hit=None)
        require(dt>0,'strict independent progress')
        t=interval.real(dt);p=vector(before['center'])+t*vector(before['velocity']);q=p.copy();q[axis]=interval.real(hi if speed>0 else lo)
        ti=1 if speed>0 else 0;n=mp.zeros(3,1);n[axis]=-1 if speed>0 else 1
        feature,bary=contact.classify(q,[vector(c['vertices'][j])for j in c['triangles'][ti]])
        require(feature=='Face','finite plane fixture has actual interior support')
        j=-(1+num(c['e']))*num(c['mass'])*dot(vector(before['velocity']),n)*n
        return dict(time=t,hit=ti,center=p,point=q,normal=n,velocity=vector(before['velocity'])+j/num(c['mass']),
                    impulse=j,energy_change=-num(c['mass'])/2*(1-num(c['e'])**2)*dot(vector(before['velocity']),n)**2,feature=feature,barycentric=bary)
    ordinals=[j for j in range(len(c['triangles']))if j!=previous]
    local['triangles']=[c['triangles'][j]for j in ordinals]
    event=contact.oracle(local)
    if event['hit']is not None:event['hit']=ordinals[event['hit']]
    return event

def normalized_isolated(raw,c,before,segment,event):
    """Reuse the independently audited isolated response formulas. Geometry and
    response are copied from actual native records; only pose metadata/render
    diagnostics are normalized to a local identity orientation for that checker.
    Actual piecewise native pose is audited in evaluate, never replaced there.
    """
    local=copy.deepcopy(c);local.update(c=before['center'],v=before['velocity'],w=before['omega'],h=float(num(segment['duration_s'])+num(segment['remaining_s'])))
    local.pop('error',None)
    b=copy.deepcopy(before);a=copy.deepcopy(segment['frame']);t=num(segment['duration_s']);rot=exponential(vector(before['omega']),t)
    b.update(time_s=0.,q=[1.,0.,0.,0.],generation=2,surface_version=4)
    b['vertices']=[[float(x)for x in vector(b['center'])+num(c['radius'])/2*mp.matrix(p)]for p in contact.REFERENCE]
    a.update(time_s=float(num(segment['represented_elapsed_s'])),generation=3,surface_version=5)
    # The existing checker compares quaternion_matrix to rotation; a direct
    # exponential quaternion supplies the local orientation with no integrator.
    w=vector(before['omega']);angle=t*length(w)
    a['q']=[float(mp.cos(angle/2))]+([float(mp.sin(angle/2)*x/length(w))for x in w]if length(w)else[0.,0.,0.])
    a['vertices']=[[float(x)for x in vector(a['center'])+rot*(num(c['radius'])/2*mp.matrix(p))]for p in contact.REFERENCE]
    isolated={k:copy.deepcopy(raw[k])for k in contact.TOP-{'before','after','error','result'}}
    isolated.update(before=b,after=a,error=None,coulomb_coefficient=c['mu'],requested_interval_s=local['h'])
    isolated['result']={k:copy.deepcopy(segment[k])for k in ['hit','impact','normal_proposal','clock_defect_s','represented_elapsed_s','rotation_increment_rad','quaternion_norm_defect','translation_defect_m']}
    isolated['result'].update(requested_event_dt_s=segment['duration_s'],unused_interval_s=segment['remaining_s'])
    # A local reference has its own q norm rounding; actual q norm is separately
    # audited below. Other diagnostics remain the native values.
    isolated['result']['quaternion_norm_defect']=float(mp.sqrt(sum(num(q)**2 for q in a['q']))-1)
    if 'barycentric' not in event and event['hit']is not None:event['barycentric']=event['bary']
    return friction.evaluate(isolated,local,event,friction.disk_oracle(local,event))

def evaluate(raw,c):
    keys(raw,TOP);require(raw['collision_shape']=='declared_sphere'and raw['moving_mesh_role']=='render_and_traction','collider roles')
    for k,ck in [('radius_m','radius'),('restitution','e'),('requested_interval_s','h'),('mass_kg','mass'),('inertia_kg_m2','inertia'),('coulomb_coefficient','mu')]:require(num(raw[k])==num(c[ck]),'physical input')
    require(type(raw['impact_budget'])is int and raw['impact_budget']==c['budget'],'budget type/input')
    require(raw['static_vertices']==c['vertices']and raw['static_triangles']==c['triangles']and raw['moving_triangles']==contact.MOVING,'geometry input')
    for p in raw['static_vertices']:vector(p)
    for t in raw['static_triangles']+raw['moving_triangles']:require(type(t)is list and len(t)==3 and all(type(j)is int for j in t),'triangle types')
    for k,v in [('relative_tolerance',1e-12),('max_gap_residual_m',1e-10),('simultaneous_window_s',1e-10)]:require(num(raw[k])==num(v),'numerical policy')
    require(type(raw['static_triangle_limit'])is int and raw['static_triangle_limit']==64,'triangle cap')
    contact.frame_schema(raw['before']);contact.frame_schema(raw['after']);require(type(raw['sequence'])is list,'sequence type')
    initial=raw['before'];require(initial['time_s']==0 and initial['q']==[1.,0.,0.,0.]and initial['generation']==2 and initial['surface_version']==4 and initial['peak_payload_bytes']==816,'initial metadata')
    maximum=0.;physical={'time_s':0.,'point_m':0.,'normal':0.,'velocity_m_s':0.,'omega_rad_s':0.}
    def compare(a,b,kind=None):
        nonlocal maximum
        error=abs(num(a)-b);require(error<=TOL,'interval comparison '+str(error));maximum=max(maximum,float(error))
        if kind:physical[kind]=max(physical[kind],float(error))
    def cv(a,b,kind=None):
        vector(a)
        for j in range(3):compare(a[j],b[j],kind)
    cv(initial['center'],vector(c['c']));cv(initial['velocity'],vector(c['v']));cv(initial['omega'],vector(c['w']))
    for p,ref in zip(initial['vertices'],contact.REFERENCE):cv(p,vector(c['c'])+num(c['radius'])/2*mp.matrix(ref))
    if c.get('admission'):
        require(type(raw['admission_error'])is str and raw['admission_error']==c['admission']and raw['result']is None,'exact admission');require(raw['sequence']==[]and raw['after']==initial,'admission atomic');return dict(maximum_algebra_error=maximum,physical_errors=physical,segments=0,status='admission')
    require(raw['admission_error']is None,'unexpected admission');result=raw['result'];keys(result,interval.RESULT)
    for k in ['remaining_interval_s','consumed_interval_s']:num(result[k])
    for k in ['accepted_segments','accepted_impacts']:require(type(result[k])is int and result[k]>=0,'count type')
    require(type(result['status'])is str,'status type');require(result['accepted_segments']==len(raw['sequence'])<=c['budget']+1<=65,'bounded records')
    before=initial;remaining=c['h'];previous=None;rotation=mp.eye(3);impacts=0;durations=mp.mpf(0);predicted=mp.mpf(0);events=mp.mpf(0);work=mp.mpf(0);impulse=mp.zeros(3,1);spin=mp.zeros(3,1);world=mp.zeros(3,1)
    for index,seg in enumerate(raw['sequence']):
        keys(seg,SEGMENT);contact.frame_schema(seg['frame'])
        for k in ['duration_s','remaining_s','clock_defect_s','rotation_increment_rad','quaternion_norm_defect','represented_elapsed_s']:num(seg[k])
        vector(seg['translation_defect_m']);t=num(seg['duration_s']);require(t>0 and float(num(remaining)-t)==seg['remaining_s'],'stored duration subtraction/progress');require(seg['remaining_s']<remaining,'strict remaining progress')
        if index==0:require(seg['departure']is None,'no initial departure')
        else:
            d=seg['departure'];keys(d,interval.DEPARTURE);vector(d['point']);require(type(d['triangle'])is int and d['triangle']==previous,'only last facet')
            require(d['point']==raw['sequence'][index-1]['hit']['point'],'actual departure point')
            require(type(d['from_generation'])is int and type(d['from_surface_version'])is int and d['from_generation']==before['generation']and d['from_surface_version']==before['surface_version'],'actual departure stamps')
            gap,support,speed=interval.certificate(before,d['point'],d['triangle'],c)
            require(gap>=0 and all(x<=0 for x in support)and speed>=0,'exact represented departure predicates')
            require(d['kind']==('Separating'if speed>0 else'Tangent'),'departure kind')
            a=[F(before['center'][j])-F(d['point'][j])for j in range(3)]
            endpoint=sum(a[j]*(F(seg['frame']['center'][j])-F(before['center'][j]))for j in range(3));require(endpoint>=0,'actual endpoint outward')
        event=event_reference(c,before,remaining,previous);compare(seg['duration_s'],event['time'],'time_s')
        if event['hit']is not None:
            keys(seg['hit'],contact.HIT);require(seg['hit']['triangle']==event['hit'],'earliest event')
            impacts+=1;require(impacts<=c['budget'],'impact budget');previous=event['hit']
        else:require(seg['hit']is None and index==len(raw['sequence'])-1,'terminal miss');previous=None
        audited=normalized_isolated(raw,c,before,seg,event);maximum=max(maximum,audited['maximum_algebra_error'])
        for k,v in audited['physical_errors'].items():physical[k]=max(physical[k],v)
        actual=seg['frame'];rotation=exponential(vector(before['omega']),t)*rotation
        actual_rotation=quaternion_matrix(actual['q'])
        for j in range(3):
            for k in range(3):compare(float(actual_rotation[j,k]),rotation[j,k])
        for p,ref in zip(actual['vertices'],contact.REFERENCE):cv(p,vector(actual['center'])+rotation*(num(c['radius'])/2*mp.matrix(ref)),'point_m')
        require(actual['generation']==3+index and actual['surface_version']==5+index and actual['peak_payload_bytes']==816,'actual publication')
        compare(actual['time_s'],num(before['time_s'])+t,'time_s');compare(seg['represented_elapsed_s'],num(actual['time_s'])-num(before['time_s']))
        compare(seg['quaternion_norm_defect'],mp.sqrt(sum(num(q)**2 for q in actual['q']))-1)
        durations+=t
        if seg['impact']is not None:
            i=seg['impact'];j=vector(i['impulse_n_s']);impulse+=j;spin+=vector(i['angular_impulse_n_m_s']);world+=friction.cross(vector(seg['hit']['point']),j)
            predicted+=num(i['predicted_energy_change_j']);events+=num(i['kinetic_after_j'])-num(i['kinetic_before_j']);work+=num(i['point_midpoint_work_j'])
        remaining=seg['remaining_s'];before=actual
    require(raw['after']==before,'actual last publication retained');require(result['accepted_impacts']==impacts,'impact count');require(result['remaining_interval_s']==remaining,'remaining truthful');require(result['consumed_interval_s']==float(num(c['h'])-num(remaining)),'consumed truthful')
    if c.get('prefix')is not None:
        require(len(raw['sequence'])==c['prefix']and result['status']==c['status']and remaining>0,'exact unfinished status/prefix')
        if c['stop']=='InvalidDeparture':
            gap,support,speed=interval.certificate(before,raw['sequence'][-1]['hit']['point'],previous,c);require(gap<0 or any(x>0 for x in support)or speed<0,'independently explain departure refusal')
        elif c['stop']=='RotationLimit':
            future=event_reference(c,before,remaining,previous);require(length(vector(before['omega']))*future['time']>mp.mpf('.25'),'independently explain rotation refusal')
    elif remaining==0:require(result['status']=='Complete','all time completion')
    else:require(result['status']=='ImpactBudgetExhausted'and impacts==c['budget']and event_reference(c,before,remaining,previous)['hit']is not None,'budget stop before next hit')
    a=result['accounting'];keys(a,ACCOUNT)
    for k in VECTOR_ACCOUNT:vector(a[k])
    for k in ACCOUNT-VECTOR_ACCOUNT:num(a[k])
    m,I=num(c['mass']),num(c['inertia']);v0,w0=vector(initial['velocity']),vector(initial['omega']);v1,w1=vector(before['velocity']),vector(before['omega'])
    kinetic=friction.energy(m,I,v1,w1)-friction.energy(m,I,v0,w0)
    world_delta=friction.cross(vector(before['center']),m*v1)+I*w1-friction.cross(vector(initial['center']),m*v0)-I*w0
    for k,value in {'summed_impulse_n_s':impulse,'summed_spin_impulse_n_m_s':spin,'summed_world_impulse_n_m_s':world,'momentum_defect_n_s':m*(v1-v0)-impulse,'spin_momentum_defect_n_m_s':I*(w1-w0)-spin,'world_angular_defect_n_m_s':world_delta-world}.items():cv(a[k],value)
    elapsed=num(before['time_s'])-num(initial['time_s'])
    for k,value in {'summed_predicted_energy_change_j':predicted,'summed_event_energy_change_j':events,'summed_point_midpoint_work_j':work,'kinetic_change_j':kinetic,'energy_defect_j':kinetic-predicted,'work_defect_j':kinetic-work,'summed_segment_duration_s':durations,'represented_elapsed_s':elapsed,'clock_defect_s':elapsed-durations,'duration_defect_s':num(result['consumed_interval_s'])-durations}.items():compare(a[k],value)
    return dict(maximum_algebra_error=maximum,physical_errors=physical,segments=len(raw['sequence']),status=result['status'])

def rational_trajectory(raw,c):
    center=list(map(F,c['c']));v=list(map(F,c['v']));w=list(map(F,c['w']));remaining=F(c['h']);m,I,r,e,mu=map(F,[c['mass'],c['inertia'],c['radius'],c['e'],c['mu']]);axis=c['axis'];lo,hi=map(F,c['walls']);rows=[]
    def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
    for seg in raw['sequence']:
        speed=v[axis];dt=((hi-r-center[axis])/speed if speed>0 else(lo+r-center[axis])/speed)if speed else None
        hit=dt is not None and dt<=remaining;t=dt if hit else remaining;center=[x+t*y for x,y in zip(center,v)];remaining-=t
        if hit:
            n=[F(0)]*3;n[axis]=-1 if speed>0 else 1;lever=[-r*x for x in n];g=[v[j]+cross(w,lever)[j]for j in range(3)];vn=sum(v[j]*n[j]for j in range(3));vt=[g[j]-sum(g[k]*n[k]for k in range(3))*n[j]for j in range(3)]
            slip2=sum(x*x for x in vt);s=F(math.isqrt(slip2.numerator),math.isqrt(slip2.denominator));require(s*s==slip2,'exact rational axial-slip premise');k=1/m+r*r/I;jn=-(1+e)*m*vn;qt=min(s/k,mu*jn);jt=[-qt*x/s if s else F(0)for x in vt];j=[jn*n[i]+jt[i]for i in range(3)];dw=cross(lever,j);v=[v[i]+j[i]/m for i in range(3)];w=[w[i]+dw[i]/I for i in range(3)]
        for key,values in [('center',center),('velocity',v),('omega',w)]:
            for actual,expected in zip(seg['frame'][key],values):require(abs(num(actual)-interval.real(expected))<=TOL,'independent whole exact-rational trajectory')
        rows.append({k:[str(x)for x in values]for k,values in [('center',center),('velocity',v),('omega',w)]})
    return rows

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    require(not a.output.exists()and not a.output.resolve().is_relative_to(Path(__file__).resolve().parents[1]),'fresh external output');a.output.mkdir(parents=True);summaries=[];frozen={}
    for c in fixtures():
        text=native_input(c);run=subprocess.run([a.executable],input=text,text=True,capture_output=True,timeout=20);require(run.returncode==0,'native bridge failed '+run.stderr)
        raw=json.loads(run.stdout);(a.output/(c['name']+'.input')).write_text(text);(a.output/(c['name']+'.native.json')).write_text(run.stdout)
        result=evaluate(raw,c);result['name']=c['name'];summaries.append(result);frozen[c['name']]=(raw,c)
        if c.get('rational'):(a.output/(c['name']+'.rational.json')).write_text(json.dumps(rational_trajectory(raw,c),indent=2)+'\n')
    raw,c=frozen['repeated_dyadic_capped'];probes=[
        lambda x:x['result'].update(status='Complete forged'),lambda x:x['result'].update(accepted_segments=True),
        lambda x:x['sequence'][1]['departure'].update(from_generation=2),lambda x:x['sequence'][1]['departure'].update(triangle=0),
        lambda x:x['sequence'][1]['departure']['point'].__setitem__(2,False),
        lambda x:x['sequence'][1]['frame']['omega'].__setitem__(2,42.),lambda x:x['sequence'][1]['frame']['q'].__setitem__(0,True),
        lambda x:x['sequence'][0]['impact'].update(tangent_impulse_n_s=[0.,0.,0.]),
        lambda x:x['result']['accounting'].update(world_angular_defect_n_m_s=[1.,0.,0.]),
        lambda x:x['result']['accounting'].update(summed_spin_impulse_n_m_s=[0.,0.,0.]),
        lambda x:x['result'].update(remaining_interval_s=.1),lambda x:x['sequence'][0].pop('normal_proposal'),
        lambda x:x['sequence'][1]['frame']['vertices'][0].__setitem__(0,42.),
        lambda x:x['static_triangles'][0].__setitem__(0,False)]
    count=0
    for probe_index,mutate in enumerate(probes):
        bad=copy.deepcopy(raw);mutate(bad)
        try:evaluate(bad,c)
        except ValueError:count+=1
        else:raise ValueError('forgery accepted '+str(probe_index))
    for label in ['friction_rotation_stop','rounded_departure_stop','dyadic_budget']:
        raw,c=frozen[label];bad=copy.deepcopy(raw);bad['result'].update(status='Complete',remaining_interval_s=0.)
        try:evaluate(bad,c)
        except ValueError:count+=1
        else:raise ValueError('unfinished forgery accepted')
    raw,c=frozen['tiny_nonzero_slip'];bad=copy.deepcopy(raw);bad['sequence'][0]['impact'].update(candidate='NoTangentialImpulse',slip_before_m_s=0.,tangent_magnitude_n_s=0.,tangent_impulse_n_s=[0.,0.,0.])
    try:evaluate(bad,c)
    except ValueError:count+=1
    else:raise ValueError('tiny slip forgery accepted')
    receipt={'scope':__doc__,'cases':summaries,'negative_probes':count,'algebra_tolerance':str(TOL),
        'maximum_algebra_error':max(x['maximum_algebra_error']for x in summaries),
        'physical_errors':{k:max(x['physical_errors'][k]for x in summaries)for k in summaries[0]['physical_errors']},
        'executable_sha256':hashlib.sha256(a.executable.read_bytes()).hexdigest(),
        'evidence_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in a.output.iterdir()if p.is_file()},
        'source_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in [Path(__file__),Path(contact.__file__),Path(friction.__file__),Path(interval.__file__)]}}
    (a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in {'cases','source_sha256','scope'}},indent=2))
if __name__=='__main__':main()
