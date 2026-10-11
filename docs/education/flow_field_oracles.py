"""Analytic display-field oracles, independent of the JavaScript algorithms.

Manufactured tensors qualify sampling/display geometry only. They are never
substituted for native flow fields or used as native screenshot evidence.
"""
from fractions import Fraction
import math
import struct


def require(condition, message):
    if not condition:
        raise RuntimeError('Flow field oracle: '+message)


def close(actual, expected, message, tolerance=2e-11):
    require(type(actual) in (int,float) and math.isfinite(actual), 'finite '+message)
    require(abs(actual-expected)<=tolerance, f'{message}: {actual!r} != {expected!r}')


def tensor_state(counts, spacing, origin, velocity, concentration=lambda p,i,j,k:0.):
    """Sample prescribed physical formulas on independently located MAC nodes."""
    state={'grid':{'counts':counts,'spacing':spacing,'origin':origin}}
    for axis,name in enumerate(('x','y','z')):
        shape=[n+int(d==axis) for d,n in enumerate(counts)]
        state[name]=[]
        for k in range(shape[2]):
            for j in range(shape[1]):
                for i in range(shape[0]):
                    position=[origin[d]+((i,j,k)[d]+(0 if d==axis else .5))*spacing[d] for d in range(3)]
                    state[name].append(velocity(position)[axis])
    state['tracer']=[concentration([origin[d]+((i,j,k)[d]+.5)*spacing[d] for d in range(3)],i,j,k)
                     for k in range(counts[2]) for j in range(counts[1]) for i in range(counts[0])]
    return state


def sample_mac_rational(state, position):
    """Exact-rational tensor-product interpolation, including physical clamps."""
    counts=state['grid']['counts']
    spacing=list(map(Fraction,state['grid']['spacing']))
    origin=list(map(Fraction,state['grid']['origin']))
    answer=[]
    for axis,name in enumerate(('x','y','z')):
        shape=[n+int(d==axis) for d,n in enumerate(counts)]
        supports=[]
        for d in range(3):
            physical=max(origin[d],min(origin[d]+counts[d]*spacing[d],Fraction(position[d])))
            coordinate=(physical-origin[d])/spacing[d]-(0 if axis==d else Fraction(1,2))
            coordinate=max(Fraction(0),min(Fraction(shape[d]-1),coordinate))
            lower=coordinate.numerator//coordinate.denominator
            upper=min(lower+1,shape[d]-1)
            weight=coordinate-lower
            supports.append(((lower,1-weight),(upper,weight)))
        value=Fraction(0)
        for k,wz in supports[2]:
            for j,wy in supports[1]:
                for i,wx in supports[0]:
                    value+=Fraction(state[name][i+shape[0]*(j+shape[1]*k)])*wx*wy*wz
        answer.append(float(value))
    return answer


def fixture_cases():
    affine=lambda p:(.25+.5*p[0]-.25*p[1]+.125*p[2],
                     -.375-.25*p[0]+.125*p[1]+.5*p[2],
                     .5+.125*p[0]+.25*p[1]-.25*p[2])
    samples=[{'name':'anisotropic-affine','state':tensor_state([4,5,6],[.25,.5,.125],[-1,2,-.5],affine),
              'points':[[-.5,3,-.125],[-.75,2.75,0],[-1,2,-.5],[0,4.5,.25],
                        [-1e308,-1e308,-1e308],[1e308,1e308,1e308],[-.875,4.25,.1875]]},
             {'name':'singleton-affine','state':tensor_state([1,2,1],[.5,.25,.75],[-.25,.5,1],affine),
              'points':[[-.25,.5,1],[.25,1,1.75],[0,.6875,1.25]]}]
    impulse=tensor_state([2,3,4],[.5,.25,.125],[-.25,.5,1],lambda p:(0.,0.,0.))
    for axis,name in enumerate(('x','y','z')):
        shape=[n+int(d==axis) for d,n in enumerate(impulse['grid']['counts'])]
        impulse[name][1+shape[0]*(1+shape[1]*2)]=(1.,-2.,4.)[axis]
    samples.append({'name':'individual-donor-weights','state':impulse,
                    'points':[[.3125,.84375,1.296875],[.125,.875,1.3125],[.5,.75,1.25],[-1,0,0]]})
    light=[]
    for name,concentration,extinction in (
            ('empty',lambda p,i,j,k:0.,12.),
            ('constant',lambda p,i,j,k:.125,12.),
            ('affine',lambda p,i,j,k:.03125+.015625*i+.0078125*j+.00390625*k,12.),
            ('changed-extinction',lambda p,i,j,k:.03125+.015625*i+.0078125*j+.00390625*k,3.5)):
        light.append({'name':name,'state':tensor_state([3,4,2],[.25,.5,.75],[-1,2,-.5],lambda p:(0.,0.,0.),concentration),
                      'extinction':extinction})
    uniform=lambda scale:tensor_state([4]*3,[.25]*3,[0.]*3,lambda p:(.25*scale,.5*scale,-.125*scale))
    curves=[{'name':'uniform','state':uniform(1),'seed':[.5]*3,'options':{'step':.015625,'maxLength':.125,'maxSteps':64}},
            {'name':'scaled-uniform','state':uniform(8),'seed':[.5]*3,'options':{'step':.015625,'maxLength':.125,'maxSteps':64}},
            {'name':'zero','state':uniform(0),'seed':[.5]*3,'options':{}},
            {'name':'below-min-speed','state':uniform(.0001),'seed':[.5]*3,'options':{'minSpeed':.0001}},
            {'name':'boundary','state':tensor_state([4]*3,[.25]*3,[0.]*3,lambda p:(1.,0.,0.)),
             'seed':[.25,.5,.5],'options':{'step':.125,'maxLength':2.,'maxSteps':64}},
            {'name':'step-limit','state':uniform(1),'seed':[.5]*3,'options':{'step':.03125,'maxLength':1.,'maxSteps':3}},
            {'name':'partial-final-step','state':uniform(1),'seed':[.5]*3,'options':{'step':.03125,'maxLength':.04,'maxSteps':16}}]
    rotation=tensor_state([8,8,4],[.125,.125,.25],[0.]*3,lambda p:(-(p[1]-.5),p[0]-.5,0.))
    for step in (.05,.025,.0125):
        curves.append({'name':'rotation-'+str(step),'state':rotation,'seed':[.7,.5,.5],
                       'options':{'step':step,'maxLength':.3,'maxSteps':100,'minSpeed':1e-8}})
    return {'samples':samples,'light':light,'curves':curves,'instantaneous':uniform(1)}


JS_PROBE=r'''async ({url,cases})=>{
  const m=await import(url), before=JSON.stringify(cases);
  const samples=cases.samples.map(({name,state,points})=>{
    const reusable=m.createMacSampler(state);
    return {name,direct:points.map(p=>m.sampleMacVelocity(state,p)),reusable:points.map(p=>reusable(p))};
  });
  const light=cases.light.map(({name,state,extinction})=>({name,values:Array.from(m.buildLightDepth(state,{extinction}))}));
  const curves=cases.curves.map(({name,state,seed,options})=>({name,...m.traceStreamline(state,seed,options)}));
  const built=m.buildInstantaneousStreamlines(cases.instantaneous);
  const rejected=[];
  for(const [name,fn] of [
    ['missing-mac-field',()=>m.sampleMacVelocity({...cases.instantaneous,x:[]},[.5,.5,.5])],
    ['nonfinite-query',()=>m.sampleMacVelocity(cases.instantaneous,[NaN,.5,.5])],
    ['negative-tracer',()=>m.buildLightDepth({...cases.light[0].state,tracer:Array(24).fill(-.125)})],
    ['nonfinite-extinction',()=>m.buildLightDepth(cases.light[0].state,{extinction:Infinity})],
  ]){try{fn();rejected.push({name,rejected:false});}catch(error){rejected.push({name,rejected:true});}}
  return {samples,light,curves,built,rejected,inputUnchanged:before===JSON.stringify(cases),
    defaults:Object.fromEntries(['FLOW_LIGHT_EXTINCTION','FLOW_LIGHT_AMBIENT','FLOW_STREAMLINE_SEEDS',
      'FLOW_STREAMLINE_STEP','FLOW_STREAMLINE_MAX_LENGTH','FLOW_STREAMLINE_MIN_SPEED','FLOW_STREAMLINE_SPEED_MAX'].map(k=>[k,m[k]]))};
}'''


def check_sampling(cases,result):
    require(len(result['samples'])==len(cases['samples']),'all manufactured sampling fields')
    count=0
    for fixture,observed in zip(cases['samples'],result['samples']):
        require(observed['name']==fixture['name'],'sampling case identity')
        for method in ('direct','reusable'):
            require(len(observed[method])==len(fixture['points']),'all sampler queries')
            for point,actual in zip(fixture['points'],observed[method]):
                expected=sample_mac_rational(fixture['state'],point)
                require(len(actual)==3,'three actual MAC components')
                for x,y in zip(actual,expected):close(x,y,'rational staggered interpolation')
        count+=len(fixture['points'])
    return count


def check_light(cases,result):
    require(len(result['light'])==len(cases['light']),'all optical-depth fields')
    count=0
    for fixture,observed in zip(cases['light'],result['light']):
        require(observed['name']==fixture['name'],'light case identity')
        nx,ny,nz=fixture['state']['grid']['counts']
        values=fixture['state']['tracer']
        require(len(observed['values'])==nx*ny*nz,'full optical-depth tensor')
        for k in range(nz):
            for j in range(ny):
                for i in range(nx):
                    # Integral of a clamped piecewise-linear column from the
                    # current center to the +Y box boundary, computed exactly.
                    depth=Fraction(values[i+nx*(j+ny*k)])/2
                    depth+=sum(Fraction(values[i+nx*(above+ny*k)]) for above in range(j+1,ny))
                    depth*=Fraction(fixture['state']['grid']['spacing'][1])*Fraction(fixture['extinction'])
                    expected=struct.unpack('<f',struct.pack('<f',float(depth)))[0]
                    close(observed['values'][i+nx*(j+ny*k)],expected,'exact column optical depth',tolerance=0.)
                    count+=1
    return count


def check_curves(cases,result):
    require(len(result['curves'])==len(cases['curves']),'all streamline cases')
    by_name={row['name']:row for row in result['curves']}
    branch_count=0
    for fixture,observed in zip(cases['curves'],result['curves']):
        require(observed['name']==fixture['name'] and observed['seed']==fixture['seed'],'fixed physical streamline seed')
        for direction,sign in (('forward',1),('backward',-1)):
            branch=observed[direction]
            points=branch['points'];speeds=branch['speeds']
            require(len(points)==len(speeds) and len(points)>=1,'point/speed correspondence')
            require(points[0]==fixture['seed'],'each branch includes its seed')
            for point,speed in zip(points,speeds):
                require(len(point)==3 and all(type(v) in (int,float) and math.isfinite(v) and 0<=v<=1 for v in point),'finite in-domain curve vertices')
                close(speed,math.hypot(*sample_mac_rational(fixture['state'],point)),'actual physical speed at curve vertex',tolerance=2e-10)
            if fixture['name'] in ('zero','below-min-speed'):
                require(len(points)==1 and branch['length']==0 and branch['termination']=='stagnation','stagnation has no invented motion')
            elif fixture['name']=='boundary':
                require(branch['termination']=='boundary','uniform streamline reaches physical boundary')
                expected=[1. if sign==1 else 0.,.5,.5]
                for actual,value in zip(points[-1],expected):close(actual,value,'uniform boundary endpoint')
                close(branch['length'],.75 if sign==1 else .25,'uniform boundary distance')
            elif fixture['name'] in ('uniform','scaled-uniform','step-limit','partial-final-step'):
                length={'uniform':.125,'scaled-uniform':.125,'step-limit':.09375,'partial-final-step':.04}[fixture['name']]
                velocity=sample_mac_rational(fixture['state'],fixture['seed']);speed=math.hypot(*velocity)
                for actual,start,v in zip(points[-1],fixture['seed'],velocity):
                    close(actual,start+sign*length*v/speed,'uniform analytical endpoint')
                close(branch['length'],length,'physical arclength budget')
                require(branch['termination']==('steps' if fixture['name']=='step-limit' else 'length'),'bounded curve termination reason')
            branch_count+=1
    for direction in ('forward','backward'):
        a,b=by_name['uniform'][direction],by_name['scaled-uniform'][direction]
        require(len(a['points'])==len(b['points']),'scaling velocity keeps geometry count')
        for p,q in zip(a['points'],b['points']):
            for x,y in zip(p,q):close(x,y,'positive velocity scaling preserves frozen geometry')
        for x,y in zip(a['speeds'],b['speeds']):close(y,8*x,'speed retains physical magnitude')
    errors=[]
    for step in (.05,.025,.0125):
        worst=0.
        for direction,sign in (('forward',1),('backward',-1)):
            branch=by_name['rotation-'+str(step)][direction]
            require(branch['termination']=='length','rotation reaches fixed arclength')
            close(branch['length'],.3,'matched rotation arclength')
            expected=[.5+.2*math.cos(1.5),.5+sign*.2*math.sin(1.5),.5]
            worst=max(worst,math.dist(branch['points'][-1],expected))
            for point in branch['points']:
                close(point[2],.5,'rotation stays in its plane')
                require(abs(math.hypot(point[0]-.5,point[1]-.5)-.2)<2e-5,'rotation radius matches analytic circle')
        errors.append(worst)
    require(0<errors[2]<errors[1]<errors[0] and errors[2]<1e-6,'refinement reduces analytic rotation endpoint error')
    ratios=[errors[0]/errors[1],errors[1]/errors[2]]
    require(all(10<ratio<24 for ratio in ratios),'RK4 fourth-order refinement at equal physical arclength')
    return {'branches':branch_count,'rotation_endpoint_errors':errors,'rotation_refinement_ratios':ratios}


def check_contract(cases,result):
    defaults=result['defaults']
    expected={'FLOW_LIGHT_EXTINCTION':12,'FLOW_LIGHT_AMBIENT':.28,'FLOW_STREAMLINE_STEP':.01,
              'FLOW_STREAMLINE_MAX_LENGTH':1,'FLOW_STREAMLINE_MIN_SPEED':1e-4,'FLOW_STREAMLINE_SPEED_MAX':1}
    require(all(defaults.get(key)==value for key,value in expected.items()),'fixed display constants and physical speed scale')
    seeds=defaults.get('FLOW_STREAMLINE_SEEDS')
    require(seeds==[[.34,.14,.42],[.46,.14,.42],[.34,.24,.54],[.46,.24,.54],[.40,.19,.38],[.40,.19,.58]],'six fixed physical seeds')
    require(all(len(p)==3 and all(type(v) in (int,float) and math.isfinite(v) and 0<=v<=1 for v in p) for p in seeds),'physical seed positions')
    require(len(result['built'])==len(seeds),'complete default streamline set')
    speed=math.hypot(.25,.5,-.125)
    for curve,seed in zip(result['built'],seeds):
        require(curve['seed']==seed,'default seed identity')
        for direction,sign in (('forward',1),('backward',-1)):
            branch=curve[direction]
            tangent=[sign*v/speed for v in (.25,.5,-.125)]
            boundary=min(((1-p)/v if v>0 else -p/v) for p,v in zip(seed,tangent) if v)
            length=min(1.,boundary)
            require(branch['termination']==('boundary' if boundary<1 else 'length'),'default uniform curve termination')
            close(branch['length'],length,'default physical arclength')
            for actual,p,v in zip(branch['points'][-1],seed,tangent):close(actual,p+length*v,'default uniform endpoint')
            require(len(branch['points'])==len(branch['speeds']),'default point/speed correspondence')
            for value in branch['speeds']:close(value,speed,'default curves retain physical speed')
    require(result['inputUnchanged'] is True,'display operations never mutate native input fields')
    require(len(result['rejected'])==4 and all(row['rejected'] is True for row in result['rejected']),'invalid field/query/light inputs reject')


def check_results(cases,result):
    samples=check_sampling(cases,result)
    light=check_light(cases,result)
    curves=check_curves(cases,result)
    check_contract(cases,result)
    return {'mac_queries':samples,'sampler_paths':2,'optical_depth_values':light,
            'streamline_branches':curves['branches'],'rotation_endpoint_errors':curves['rotation_endpoint_errors'],
            'rotation_refinement_ratios':curves['rotation_refinement_ratios'],
            'physical_speed_scale':True,'input_fields_unchanged':True,'invalid_inputs_rejected':4}


def check_native_curves(state,renderer):
    """Reintegrate displayed branches from the actual accepted MAC tensor.

    Each velocity query uses the exact-rational independent interpolant above;
    the RK formula follows the declared frozen arclength ODE, not advection time.
    """
    require(renderer.get('sourceTime')==state['time'] and renderer.get('sourceGeneration')==state['generation'],
            'derived display belongs to the accepted state')
    require(renderer.get('streamlineGeometry')=='frozen-instantaneous-arclength-rk4' and
            renderer.get('streamlinePositionUnits')=='m' and renderer.get('streamlineSpeedUnits')=='m/s',
            'instantaneous geometry and physical units disclosed')
    require(renderer.get('streamlineStep')==.01 and renderer.get('streamlineMaxLength')==1 and
            renderer.get('streamlineMinSpeed')==1e-4 and renderer.get('streamlineSpeedMax')==1,
            'fixed curve controls and physical colour scale')
    curves=renderer.get('streamlineData')
    require(isinstance(curves,list) and len(curves)==6,'all native-field derived curves available')
    seeds=[[.34,.14,.42],[.46,.14,.42],[.34,.24,.54],[.46,.24,.54],[.40,.19,.38],[.40,.19,.58]]
    require(renderer.get('streamlineSeeds')==seeds,'native frame uses fixed physical seeds')
    lower=state['grid']['origin']
    upper=[o+n*h for o,n,h in zip(lower,state['grid']['counts'],state['grid']['spacing'])]
    vertices=segments=0
    for curve,seed in zip(curves,seeds):
        require(curve.get('seed')==seed,'native curve seed identity')
        for name,sign in (('forward',1),('backward',-1)):
            branch=curve[name]
            expected=[seed]
            arclength=0.
            reason='steps'
            def tangent(point):
                velocity=sample_mac_rational(state,point)
                speed=math.hypot(*velocity)
                return None if speed<1e-4 else [sign*v/speed for v in velocity]
            for iteration in range(100):
                ds=min(.01,1-arclength)
                if ds<=1e-12:
                    reason='length';break
                point=expected[-1]
                stages=[]
                for fraction,previous in ((0,None),(.5,0),(.5,1),(1,2)):
                    query=point if previous is None else [p+ds*fraction*v for p,v in zip(point,stages[previous])]
                    vector=tangent(query)
                    if vector is None:break
                    stages.append(vector)
                if len(stages)!=4:
                    reason='stagnation';break
                endpoint=[point[d]+ds*math.fsum(weight*stages[s][d] for s,weight in enumerate((1,2,2,1)))/6 for d in range(3)]
                if any(q<lo or q>hi for q,lo,hi in zip(endpoint,lower,upper)):
                    cuts=[1.]
                    for p,q,lo,hi in zip(point,endpoint,lower,upper):
                        if q<lo:cuts.append((lo-p)/(q-p))
                        elif q>hi:cuts.append((hi-p)/(q-p))
                    fraction=min(cuts)
                    if fraction>1e-12:
                        expected.append([max(lo,min(hi,p+fraction*(q-p))) for p,q,lo,hi in zip(point,endpoint,lower,upper)])
                        arclength+=ds*fraction
                    reason='boundary';break
                expected.append(endpoint);arclength+=ds
                if 1-arclength<=1e-12:
                    reason='length';break
            require(branch.get('termination')==reason,'native curve termination matches accepted velocity')
            close(branch.get('length'),arclength,'native curve physical arclength',tolerance=3e-10)
            require(len(branch['points'])==len(expected)==len(branch['speeds']),'all native curve vertices and speeds')
            for point,expected_point,speed in zip(branch['points'],expected,branch['speeds']):
                for actual,value in zip(point,expected_point):close(actual,value,'native RK4 vertex',tolerance=3e-10)
                close(speed,math.hypot(*sample_mac_rational(state,point)),'native stored speed at rendered vertex',tolerance=3e-10)
            vertices+=len(expected);segments+=len(expected)-1
    require(type(renderer.get('streamlineVertexCount')) is int and renderer['streamlineVertexCount']>=2*segments,
            'actual renderer contains the native-field line segments')
    return {'curves':6,'branches':12,'speed_values':vertices,'rk4_segments':segments,
            'accepted_state_identity':True,'physical_units':True,'independent_mac_reintegration':True}
