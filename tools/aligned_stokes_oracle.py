#!/usr/bin/env python3
"""Independent Fraction qualification of the isolated aligned-Stokes experiment.

Geometry assembly checks are separate from exact algebra on actual stored
binary64 E, W, mass, area, volume, pressure, and caller state values. No native
energy/defect diagnostic substitutes for recomputation of its certificate.
"""
from __future__ import annotations
from fractions import Fraction as F
from itertools import product
from pathlib import Path
import argparse
import hashlib
import json
import math
import random
import struct
import subprocess
import sys

from aligned_strain_oracle import (Geometry, Face, Row, Refusal, require, bits_value,
                                  coordinates, flatten, reference, close, bound, apply)

SCHEMA = 'rheon-aligned-stokes-native-v1'
PROBE_SCHEMA = 'rheon-outward-probe-v1'
EPS = F.from_float(sys.float_info.epsilon)


def hex_bits(value):
    return struct.pack('>d',float(value)).hex()


def scalar(token):
    require(type(token) is str,'binary64 token type')
    value = bits_value(token)
    require(value == 0 or abs(value) >= F.from_float(sys.float_info.min), 'subnormal stored input')
    return value


def interval(tokens,label):
    require(type(tokens) is list and len(tokens)==2,'interval shape '+label)
    lo,hi = map(scalar,tokens)
    require(lo<=hi,'reversed interval '+label)
    return lo,hi


def encloses(tokens,value,label):
    lo,hi=interval(tokens,label)
    require(lo<=value<=hi,'unsound enclosure '+label)
    return lo,hi


def no_duplicate_object(pairs):
    result={}
    for key,value in pairs:
        require(key not in result,'duplicate JSON field '+key)
        result[key]=value
    return result


def read_json(path):
    return json.loads(Path(path).read_text(),object_pairs_hook=no_duplicate_object)


def exact_operation(op,a,b=None):
    alo,ahi=interval(a,'probe a')
    if b is not None:blo,bhi=interval(b,'probe b')
    if op=='add':return alo+blo,ahi+bhi
    if op=='sub':return alo-bhi,ahi-blo
    if op=='mul':
        values=[x*y for x in (alo,ahi) for y in (blo,bhi)]
        return min(values),max(values)
    if op=='div':
        require(not blo<=0<=bhi,'probe zero denominator')
        values=[x/y for x in (alo,ahi) for y in (blo,bhi)]
        return min(values),max(values)
    if op=='neg':return -ahi,-alo
    if op=='abs':return (F(0) if alo<=0<=ahi else min(abs(alo),abs(ahi))),max(abs(alo),abs(ahi))
    if op=='square':
        low,high=exact_operation('abs',a)
        return low*low,high*high
    raise Refusal('unknown primitive operation')


def probe_cases(seed=20261008,count=350):
    cases=[]
    def add(name,op,a,b=None,expect='accept'):
        cases.append({'id':name,'op':op,'a':[hex_bits(x) for x in a],
                      **({'b':[hex_bits(x) for x in b]} if b is not None else {}),'expect':expect})
    p=lambda x:(x,x)
    for op in ('add','sub','mul','div','neg','abs','square'):
        add('zero_'+op,op,p(0),p(1) if op in ('add','sub','mul','div') else None)
    add('exact_cancel','add',p(3),p(-3))
    add('equal_points','sub',p(3),p(3))
    add('opposite_quotient','div',p(3),p(-3))
    add('lost_low_part','add',p(1e16),p(1))
    add('uncorrelated_cancel','sub',(-2,3),(-2,3))
    add('square_crosses_zero','square',(-2,3))
    add('abs_crosses_zero','abs',(-2,3))
    add('negative_divisor','div',(-3,5),(-5,-2))
    add('zero_denominator','div',p(1),(-1,1),'refuse')
    add('overflow','mul',p(2.0**900),p(2.0**900),'refuse')
    add('underflow','mul',p(2.0**-600),p(2.0**-600),'refuse')
    add('subnormal_result','mul',p(sys.float_info.min),p(.5),'refuse')
    add('subnormal_input','add',p(sys.float_info.min/2),p(0),'refuse')
    add('invalid_interval','add',(2,1),p(0),'refuse')
    add('infinite_input','abs',p(float('inf')),expect='refuse')
    add('nan_input','neg',p(float('nan')),expect='refuse')
    rng=random.Random(seed)
    operations=('add','sub','mul','div','neg','abs','square')
    for i in range(count):
        def endpoints():
            scale=math.ldexp(1,rng.randint(-200,200))
            values=sorted((rng.uniform(-2,2)*scale,rng.uniform(-2,2)*scale))
            return tuple(values)
        a=endpoints();op=operations[i%len(operations)];b=None
        if op in ('add','sub','mul','div'):
            b=endpoints()
            if op=='div':
                scale=math.ldexp(1,rng.randint(-200,200))
                b=(.5*scale,2*scale)
                if i%2:b=(-b[1],-b[0])
        add('random_'+str(i),op,a,b)
    return cases


def verify_probe_observation(case,observation):
    expected_fields={'schema','id','op','a'}|({'b'} if 'b' in case else set())
    require(set(observation) in (expected_fields|{'result'},expected_fields|{'error'}),'probe output field roster')
    require(observation.get('schema')==PROBE_SCHEMA and all(observation.get(k)==case[k]
            for k in ('id','op','a') if k in case) and observation.get('b')==case.get('b'), 'probe input echo binding')
    if 'error' in observation:
        require(type(observation['error']) is str and observation['error'],'probe explicit refusal')
        require(case['expect']!='accept','required primitive was refused '+case['id'])
        return 'refused'
    require(case['expect']!='refuse','unsafe primitive was accepted '+case['id'])
    expected_lo,expected_hi=exact_operation(case['op'],case['a'],case.get('b'))
    actual_lo,actual_hi=interval(observation['result'],'probe result')
    require(actual_lo<=expected_lo<=expected_hi<=actual_hi,'primitive enclosure misses exact extrema '+case['id'])
    return 'accepted'


def qualify_probe(executable,output_dir=None):
    executable=Path(executable).resolve()
    before=hashlib.sha256(executable.read_bytes()).hexdigest()
    cases=probe_cases()
    def launch(batch):
        inputs=''.join(' '.join([c['id'],c['op'],*c['a'],*c.get('b',[])])+'\n' for c in batch)
        result=subprocess.run([str(executable)],input=inputs,capture_output=True,text=True,timeout=60)
        require(result.returncode==0,'native primitive probe process failed '+result.stderr)
        observations=[json.loads(line,object_pairs_hook=no_duplicate_object) for line in result.stdout.splitlines()]
        require(len(observations)==len(batch),'native primitive probe dropped output')
        return inputs,result.stdout,observations
    inputs,outputs,observations=launch(cases)
    statuses=[verify_probe_observation(c,o) for c,o in zip(cases,observations)]
    first=observations[next(i for i,c in enumerate(cases) if c['id']=='lost_low_part')]['result']
    sequence={'id':'forced_cancellation_sequence','op':'sub','a':first,
              'b':[hex_bits(1e16)]*2,'expect':'accept'}
    more_inputs,more_outputs,more_observations=launch([sequence])
    statuses.append(verify_probe_observation(sequence,more_observations[0]))
    # Composition must contain (1e16+1)-1e16 = 1, not its rounded point-zero.
    encloses(more_observations[0]['result'],F(1),'forced cancellation sequence')
    require(hashlib.sha256(executable.read_bytes()).hexdigest()==before,'primitive binary changed during qualification')
    if output_dir is not None:
        output_dir=Path(output_dir)
        output_dir.mkdir(parents=True,exist_ok=True)
        for name,payload in [('primitive-inputs.txt',inputs+more_inputs),('primitive-outputs.jsonl',outputs+more_outputs)]:
            with (output_dir/name).open('x') as out:out.write(payload)
    return {'primitive_cases':len(statuses),'primitive_accepted':statuses.count('accepted'),
            'primitive_refused':statuses.count('refused'),'forced_cancellation_enclosed':True,
            'primitive_binary_sha256':before,'primitive_inputs_sha256':hashlib.sha256((inputs+more_inputs).encode()).hexdigest(),
            'primitive_outputs_sha256':hashlib.sha256((outputs+more_outputs).encode()).hexdigest()}


def gaussian(a,b):
    augmented=[list(row)+[value] for row,value in zip(a,b)]
    n=len(b)
    for j in range(n):
        pivot=next((i for i in range(j,n) if augmented[i][j]),None)
        require(pivot is not None,'singular independent pressure system')
        augmented[j],augmented[pivot]=augmented[pivot],augmented[j]
        factor=augmented[j][j];augmented[j]=[x/factor for x in augmented[j]]
        for i in range(n):
            if i!=j and augmented[i][j]:
                factor=augmented[i][j]
                augmented[i]=[x-factor*y for x,y in zip(augmented[i],augmented[j])]
    return [row[-1] for row in augmented]


def geometry_close(actual,expected,label):
    # Primitive assembly has only a few rounded operations. A relative 64-eps
    # budget permits those operations without a dimensionally unrelated floor.
    close(actual,expected,label,tolerance=64*EPS)


def roster(value,keys,label):
    require(type(value)is dict and set(value)==set(keys),'field roster '+label)


def stored_geometry(records):
    meta=records['geometry']
    roster(meta,('counts','spacing','origin','lower','upper','density','viscosity','volumes'),'geometry')
    for key in ('counts','spacing','origin','lower','upper'):
        require(type(meta[key])is list and len(meta[key])==3,'geometry triple '+key)
    g=Geometry(tuple(meta['counts']),tuple(map(scalar,meta['spacing'])),tuple(map(scalar,meta['origin'])),
               tuple(meta['lower']),tuple(meta['upper']),scalar(meta['density']),scalar(meta['viscosity']))
    expected_faces,expected_rows,_,_=reference(g)
    require(type(records['active'])is list and type(records['rows'])is list,'native primitive arrays')
    faces=[]
    for i,value in enumerate(records['active']):
        roster(value,('id','axis','face','coordinates','position','area','distance','mass','negative_cell','positive_cell'),'active')
        require(type(value['id'])is int and value['id']==i,'native active index')
        require(all(type(value[k])is int for k in ('axis','face','negative_cell','positive_cell')),'active integer metadata')
        for key in ('coordinates','position'):
            require(type(value[key])is list and len(value[key])==3,'active triple '+key)
        require(all(type(x)is int for x in value['coordinates']),'active integer coordinates')
        faces.append(Face(value['axis'],value['face'],tuple(value['coordinates']),
                          tuple(map(scalar,value['position'])),scalar(value['area']),
                          scalar(value['distance']),scalar(value['mass'])))
    require(len(faces)==len(expected_faces),'native pressure active count')
    for i,(actual,expected) in enumerate(zip(faces,expected_faces)):
        require((actual.axis,actual.flat,actual.p,actual.position)==(expected.axis,expected.flat,expected.p,expected.position),
                'native active geometry '+str(i))
        for key in ('area','distance','mass'):
            require(getattr(actual,key)>0,'nonpositive stored '+key)
            geometry_close(getattr(actual,key),getattr(expected,key),'independent geometry '+key)
        lo=list(actual.p);lo[actual.axis]-=1
        require(records['active'][i]['negative_cell']==flatten(lo,g.counts) and
                records['active'][i]['positive_cell']==flatten(actual.p,g.counts),'native pressure incidence')
    rows=[]
    for i,value in enumerate(records['rows']):
        roster(value,('id','axes','coordinates','quadrant','boundary','weight','terms'),'row')
        require(type(value['id'])is int and value['id']==i,'native row index')
        require(type(value['axes'])is list and len(value['axes'])==2 and type(value['coordinates'])is list and len(value['coordinates'])==3,'row coordinate shape')
        require(all(type(x)is int for x in value['axes']+value['coordinates']+[value['quadrant'],value['boundary']]),'row integer metadata')
        require(type(value['terms'])is list and all(type(t)is list and len(t)==2 for t in value['terms']),'row term shape')
        terms=tuple(sorted((face,scalar(coefficient)) for face,coefficient in value['terms']))
        require(len({i for i,_ in terms})==len(terms) and all(type(i)is int and 0<=i<len(faces) and a for i,a in terms),
                'native row support')
        rows.append(Row(tuple(value['axes']),tuple(value['coordinates']),value['quadrant'],scalar(value['weight']),terms,value['boundary']))
    expected={r.key:r for r in expected_rows};actual={r.key:r for r in rows}
    require(len(actual)==len(rows) and actual.keys()==expected.keys(),'native row roster including zeros')
    for key,row in actual.items():
        e=expected[key]
        require(row.closure==e.closure and row.weight>0,'native row closure/positive weight')
        geometry_close(row.weight,e.weight,'independent row weight')
        require(tuple(i for i,_ in row.terms)==tuple(i for i,_ in e.terms),'native row face support')
        for (_,a),(_,b) in zip(row.terms,e.terms):
            require(a*b>0,'native coefficient sign')
            geometry_close(a,b,'independent coefficient')
    volumes=list(map(scalar,meta['volumes']))
    require(len(volumes)==math.prod(g.counts),'native cell volume count')
    for p,volume in zip(coordinates(g.counts),volumes):
        if g.fluid(p):
            require(volume>0,'native wet volume')
            geometry_close(volume,g.volume(p),'independent wet volume')
        else:require(volume==0,'native dry volume')
    return g,faces,rows,volumes


def velocity(records,name,g,faces):
    full=records[name]
    require(type(full)is list and len(full)==3,'full MAC vector shape '+name)
    fields=[]
    for axis,values in enumerate(full):
        dims=list(g.counts);dims[axis]+=1
        require(len(values)==math.prod(dims),'full MAC component length '+name)
        fields.append(list(map(scalar,values)))
    active={(f.axis,f.flat) for f in faces}
    for axis,values in enumerate(fields):
        for face,value in enumerate(values):
            if (axis,face) not in active:require(value==0,'nonzero prescribed trace '+name)
    return [fields[f.axis][f.flat] for f in faces]


def energy(faces,field):
    return sum(f.mass*x*x/2 for f,x in zip(faces,field))


def flux(g,faces,field):
    result=[F(0)]*math.prod(g.counts)
    for face,value in zip(faces,field):
        lo=list(face.p);lo[face.axis]-=1
        result[flatten(lo,g.counts)]+=face.area*value
        result[flatten(face.p,g.counts)]-=face.area*value
    return result


def maximum_divergence(g,faces,field,volumes):
    return max((abs(q)/v for q,v in zip(flux(g,faces,field),volumes) if v),default=F(0))


def ideal_projection(g,faces,field,dt,volumes):
    wet=[i for i,v in enumerate(volumes) if v]
    gauge=max(wet,key=lambda i:(volumes[i],-i))
    nongauge=[i for i in wet if i!=gauge]
    C=[[F(0)]*len(faces) for _ in volumes]
    for j,face in enumerate(faces):
        lo=list(face.p);lo[face.axis]-=1
        C[flatten(lo,g.counts)][j]=face.area;C[flatten(face.p,g.counts)][j]=-face.area
    L=[[sum(C[i][f]*C[j][f]/faces[f].mass for f in range(len(faces))) for j in nongauge] for i in nongauge]
    q=flux(g,faces,field)
    p=[F(0)]*len(volumes)
    for i,value in zip(nongauge,gaussian(L,[-q[i]/dt for i in nongauge])):p[i]=value
    projected=[value+dt*sum(C[i][f]*p[i] for i in wet)/faces[f].mass for f,value in enumerate(field)]
    require(all(q==0 for q in flux(g,faces,projected)),'independent exact projection failed')
    return projected,p,gauge


def certificate(data,defect,scale,tolerance,label):
    require(set(data)=={'defect','scale','allowance'},'certificate field roster '+label)
    dlo,dhi=encloses(data['defect'],defect,label+' defect')
    slo,shi=encloses(data['scale'],scale,label+' scale')
    alo,ahi=encloses(data['allowance'],tolerance*scale,label+' allowance')
    require(slo>=0 and alo>=0,'negative scale/allowance '+label)
    require(max(abs(dlo),abs(dhi))<=alo,'certificate does not authorize equation '+label)


def verify(records_path,compute_projection=True):
    records=read_json(records_path)
    return verify_record(records,compute_projection,hashlib.sha256(Path(records_path).read_bytes()).hexdigest())


def verify_record(records,compute_projection=True,records_sha256=None):
    roster(records,('schema','status','geometry','settings','active','rows','initial','viscous','final','pressure','gauge_cells','report','state_before','state_after','allocation'),'accepted packet')
    require(records.get('schema')==SCHEMA,'native experiment schema')
    require(records.get('status')=='accepted','accepted native experiment required')
    g,faces,rows,volumes=stored_geometry(records)
    settings=records['settings']
    roster(settings,('dt','divergence_limit','relative_update_limit','pressure_relative_residual','pressure_absolute_residual','pressure_max_iterations'),'settings')
    for key in ('pressure_relative_residual','pressure_absolute_residual'):
        require(scalar(settings[key])>=0,'negative pressure setting')
    require(type(settings['pressure_max_iterations'])is int and settings['pressure_max_iterations']>0,'pressure iteration setting')
    dt=scalar(settings['dt']);tau=scalar(settings['relative_update_limit'])
    div_limit=scalar(settings['divergence_limit'])
    require(dt>0 and 0<=tau<=F(1,10**8) and div_limit>=0,'experiment settings domain')
    before=velocity(records,'initial',g,faces);viscous=velocity(records,'viscous',g,faces);final=velocity(records,'final',g,faces)
    pressure=list(map(scalar,records['pressure']))
    require(len(pressure)==len(volumes),'pressure shape')
    wet=[i for i,v in enumerate(volumes) if v]
    gauge=max(wet,key=lambda i:(volumes[i],-i))
    require(records['gauge_cells']==[gauge] and pressure[gauge]==0 and all(pressure[i]==0 for i,v in enumerate(volumes) if not v),
            'pressure gauge/dry cells')
    ku,loss=apply(rows,before)
    scatter=[F(0)]*len(faces)
    for row in rows:
        strain=sum(a*before[i] for i,a in row.terms)
        for i,a in row.terms:scatter[i]+=abs(a*row.weight*strain)
    exact_bound=bound(rows,faces)
    report=records['report']
    roster(report,('bound','step_product','viscous_energy_delta','pressure_energy_delta','initial_divergence','final_divergence','update','momentum'),'report')
    blo,bhi=encloses(report['bound'],exact_bound,'stored coefficient B')
    require(blo>=0,'negative B interval')
    _,product_hi=encloses(report['step_product'],dt*g.viscosity*bhi,'dt mu B_upper')
    require(product_hi<=2,'uncertified Euler restriction')
    energies=[energy(faces,u) for u in (before,viscous,final)]
    for key,change in (('viscous_energy_delta',energies[1]-energies[0]),('pressure_energy_delta',energies[2]-energies[1])):
        _,hi=encloses(report[key],change,key)
        require(hi<=0,'uncertified actual energy nonincrease '+key)
    for key,field in (('initial_divergence',before),('final_divergence',final)):
        lo,hi=encloses(report[key],maximum_divergence(g,faces,field,volumes),key)
        require(lo>=0 and hi<=div_limit,'uncertified cellwise divergence '+key)
    require(len(report['update'])==len(faces) and len(report['momentum'])==len(faces),'coordinate certificate counts')
    momentum_defects=[]
    for i,face in enumerate(faces):
        increment=face.mass*(viscous[i]-before[i]);action=dt*g.viscosity*ku[i]
        certificate(report['update'][i],increment+action,abs(increment)+dt*g.viscosity*scatter[i],tau,'Euler face '+str(i))
        lo=list(face.p);lo[face.axis]-=1
        jump=pressure[flatten(face.p,g.counts)]-pressure[flatten(lo,g.counts)]
        impulse=dt*face.area*jump;increment=face.mass*(final[i]-viscous[i])
        momentum_defects.append(increment+impulse)
        certificate(report['momentum'][i],increment+impulse,abs(increment)+abs(impulse),tau,'pressure face '+str(i))
    for state in ('state_before','state_after'):
        roster(records[state],('accepted_steps',),state)
        require(type(records[state]['accepted_steps'])is int and records[state]['accepted_steps']>=0,'accepted counter type')
    require(records['state_after']['accepted_steps']==records['state_before']['accepted_steps']+1,'accepted transaction counter')
    allocation=records['allocation']
    roster(allocation,('operator_bytes','workspace_bytes','combined_bytes','limit'),'allocation')
    require(all(type(x)is int and x>=0 for x in allocation.values()),'allocation type')
    require(allocation['combined_bytes']==allocation['operator_bytes']+allocation['workspace_bytes'] and allocation['combined_bytes']<=allocation['limit'],'allocation accounting')
    normal_loss=sum(r.weight*sum(a*before[i] for i,a in r.terms)**2 for r in rows if r.family[0]==r.family[1])
    result={'active_faces':len(faces),'rows':len(rows),'zero_rows':sum(not r.terms for r in rows),
            'B_exact_stored':str(exact_bound),'energy_before_exact':str(energies[0]),
            'energy_viscous_exact':str(energies[1]),'energy_final_exact':str(energies[2]),
            'strain_loss_exact':str(loss),'normal_strain_loss_exact':str(normal_loss),'shear_strain_loss_exact':str(loss-normal_loss),'viscous_max_flux_exact':str(max(map(abs,flux(g,faces,viscous)))),
            'final_divergence_exact':str(maximum_divergence(g,faces,final,volumes)),
            'pressure_metric_defect_max_exact':str(max(map(abs,momentum_defects))),
            'records_sha256':records_sha256}
    exact_euler=[x-dt*g.viscosity*k/f.mass for x,k,f in zip(before,ku,faces)]
    result.update(ideal_euler_energy_exact=str(energy(faces,exact_euler)),
                  euler_velocity_error_max_exact=str(max(abs(a-b) for a,b in zip(viscous,exact_euler))))
    if compute_projection:
        ideal,ideal_p,_=ideal_projection(g,faces,viscous,dt,volumes)
        # Exact matched projection remains a reference. The certificate gates,
        # not a unit-sized closeness threshold, qualify the actual stored field.
        result.update(ideal_projected_energy_exact=str(energy(faces,ideal)),
                      matched_projection_velocity_error_max_exact=str(max(abs(a-b) for a,b in zip(final,ideal))),
                      matched_projection_pressure_error_max_exact=str(max(abs(a-b) for a,b in zip(pressure,ideal_p))))
    return result


def verify_refusal(records):
    roster(records,('schema','case','status','error','caller_before','caller_after','accepted_before','accepted_after','state_before','state_after'),'refusal packet')
    require(records['schema']=='rheon-aligned-stokes-refusal-v1' and records['status']=='refused','refusal schema')
    require(records['state_before']==records['state_after'],'refusal changed accepted counter')
    roster(records['state_before'],('accepted_steps',),'refusal state')
    require(type(records['state_before']['accepted_steps'])is int and records['state_before']['accepted_steps']>=0,'refusal counter type')
    require(type(records['case'])is str and type(records['error'])is str and records['error'],'explicit refusal cause')
    require(records['caller_before']==records['caller_after'],'refusal changed caller bits')
    require(records['accepted_before']==records['accepted_after'],'refusal changed accepted getters')
    # Equality is bit-for-bit, including signed zero and any deliberately invalid
    # caller input. The snapshots are native observations, not numeric tolerance.
    witness=records['accepted_before']
    roster(witness,('initial','viscous','final','pressure','report'),'accepted refusal witness')
    for key in ('initial','viscous','final','pressure'):
        require(type(witness[key])is list and witness[key],'nonempty accepted refusal witness '+key)
        for token in witness[key]:scalar(token)
    roster(witness['report'],('bound','step_product','viscous_energy_delta','pressure_energy_delta','initial_divergence','final_divergence','update','momentum'),'accepted refusal report')
    require(type(records['caller_before'])is list and len(records['caller_before'])==3,'refusal full MAC snapshot')
    for values in records['caller_before']:
        require(type(values)is list,'refusal MAC component')
        for token in values:
            require(type(token)is str and len(token)==16 and all(c in '0123456789abcdef' for c in token),'refusal bit token')
    return {'refused':True,'case':records['case'],'atomic_caller_bits':True,'atomic_accepted_getters':True,'error':records['error']}


def verify_any(path):
    records=read_json(path)
    if records.get('schema')=='rheon-aligned-stokes-refusal-v1':return verify_refusal(records)
    if records.get('schema')=='rheon-aligned-stokes-sequence-v1':
        roster(records,('schema','frames'),'sequence')
        require(type(records['frames'])is list and len(records['frames'])>=2,'sequence frames')
        summaries=[]
        previous=None
        for frame in records['frames']:
            if previous is not None:
                require(frame['initial']==previous['final'],'sequence caller continuity')
                require(frame['state_before']==previous['state_after'],'sequence accepted counter continuity')
                require(all(frame[key]==previous[key] for key in ('geometry','active','rows')),'sequence immutable operator')
            summaries.append(verify_record(frame))
            previous=frame
        return {'accepted_sequence_frames':len(summaries),'frames':summaries}
    return verify(path)


NATIVE_CASES=tuple((name,'accepted') for name in ('unit-curl','unit-rest','anisotropic-curl','unit-tiny','unit-large','unit-cross','unit-reflected'))+tuple((name,'refused') for name in ('unit-tinystep','unit-boundary','unit-oversized','unit-underflow','unit-overflow'))


def specimen_initial(records,name):
    """Bind qualification controls to the declared specimen, independently of exporter."""
    g,faces,_,_=stored_geometry(records)
    require(g.counts==(3,3,3) and g.lower==(1,1,1) and g.upper==(2,2,2),'specimen shape')
    anisotropic=name=='anisotropic-curl'
    h=(.3,.7,1.1) if anisotropic else (1.,)*3
    origin=(1e8,-1e8,.1) if anisotropic else (0.,)*3
    require(g.spacing==tuple(map(F.from_float,h)) and g.origin==tuple(map(F.from_float,origin)),'specimen represented grid')
    require(g.density==F.from_float(3.7 if anisotropic else 1.) and g.viscosity==F.from_float(.375 if anisotropic else 1.),'specimen material')
    exponent={'unit-tiny':-200,'unit-large':200,'unit-underflow':-600,'unit-overflow':600}.get(name,0)
    amplitude=math.ldexp(1.,exponent)
    samples=(((0,(1,0,0),.5),(0,(1,0,1),-.5),(2,(0,0,1),-.5),(2,(1,0,1),.5)) if name=='unit-cross' else
             ((0,(2,1,2),.5),(0,(2,2,2),-.5),(1,(1,2,2),-.5),(1,(2,2,2),.5)) if name=='unit-reflected' else
             ((0,(1,0,0),.5),(0,(1,1,0),-.5),(1,(0,1,0),-.5),(1,(1,1,0),.5)))
    sample_map={(a,p):q for a,p,q in samples}
    expected=[F.from_float(amplitude*sample_map.get((f.axis,f.p),0.)/float(f.area)) for f in faces]
    if name=='unit-rest':expected=[F(0)]*len(faces)
    require(velocity(records,'initial',g,faces)==expected,'declared specimen initial field '+name)
    require(scalar(records['settings']['dt'])==F(1,256) if anisotropic else scalar(records['settings']['dt'])==F(1,32),'specimen timestep')
    return g,faces,expected


def copy_record_for_initial(records,initial):
    copied=dict(records);copied['initial']=initial
    return copied


def qualify_native(executable,directory,cases=NATIVE_CASES):
    executable=Path(executable).resolve();directory=Path(directory)
    before=hashlib.sha256(executable.read_bytes()).hexdigest()
    summaries={};commands=[]
    for name,status in cases:
        output=directory/(name+'.json')
        argv=[str(executable),name,str(output)]
        result=subprocess.run(argv,capture_output=True,text=True,timeout=90)
        require(result.returncode==0,'native example process failed '+name+' '+result.stderr)
        require(output.is_file(),'native example omitted output '+name)
        summary=verify_any(output)
        require(bool(summary.get('refused'))==(status=='refused'),'native case acceptance mismatch '+name)
        packet=read_json(output)
        if status=='accepted':specimen_initial(packet,name)
        else:
            require(packet['case']==name,'refusal specimen identity')
            expected_error={'unit-tinystep':'Equation { stage: ViscousEquation','unit-boundary':'StepBound','unit-oversized':'StepBound','unit-underflow':'Enclosure(OverflowOrUnderflow)','unit-overflow':'Enclosure(OverflowOrUnderflow)'}[name]
            require(packet['error'].startswith(expected_error),'unexpected refusal stage '+name)
            baseline=read_json(directory/'unit-curl.json')
            g,faces,_,_=stored_geometry(baseline)
            witness={key:velocity(baseline,key,g,faces) for key in ('initial','viscous','final')}
            for key,values in witness.items():
                require([scalar(x) for x in packet['accepted_before'][key]]==values,'refusal retained qualified witness '+key)
            require(packet['accepted_before']['pressure']==baseline['pressure'] and packet['accepted_before']['report']==baseline['report'],'refusal retained qualified pressure/report')
            # The refused caller is independently assembled from the declared
            # control on the same operator, while prior accepted getters remain.
            candidate=copy_record_for_initial(baseline,packet['caller_before'])
            specimen_initial(candidate,name)
        summaries[name]=summary
        commands.append({'argv':argv,'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr,
                         'output_sha256':hashlib.sha256(output.read_bytes()).hexdigest()})
    output=directory/'unit-curl-sequence.json'
    argv=[str(executable),'unit-curl','3',str(output)]
    result=subprocess.run(argv,capture_output=True,text=True,timeout=90)
    require(result.returncode==0 and output.is_file(),'native repeated sequence process failed '+result.stderr)
    summaries['unit-curl-sequence']=verify_any(output)
    commands.append({'argv':argv,'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr,
                     'output_sha256':hashlib.sha256(output.read_bytes()).hexdigest()})
    base=read_json(directory/'unit-curl.json')
    for name,exponent in (('unit-tiny',-200),('unit-large',200)):
        if name in summaries:
            scaled=read_json(directory/(name+'.json'));factor=F(2)**exponent
            for key in ('initial','viscous','final','pressure'):
                left=[x for component in scaled[key] for x in component] if key!='pressure' else scaled[key]
                right=[x for component in base[key] for x in component] if key!='pressure' else base[key]
                require(all(scalar(a)==factor*scalar(b) for a,b in zip(left,right)),'dyadic native amplitude covariance '+name+' '+key)
    require(hashlib.sha256(executable.read_bytes()).hexdigest()==before,'native binary changed during qualification')
    return {'native_binary_sha256':before,'native_cases':summaries,'native_commands':commands}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('records',type=Path,nargs='?')
    parser.add_argument('--native',type=Path)
    parser.add_argument('--probe',type=Path)
    parser.add_argument('--probe-artifacts',type=Path)
    parser.add_argument('--output',type=Path)
    args=parser.parse_args()
    require(args.records is not None or args.probe is not None or args.native is not None,'native records or primitive probe required')
    result={}
    if args.native is not None:
        require(args.output is not None,'aggregate qualification requires fresh output directory')
        args.output.mkdir(parents=True,exist_ok=False)
        result.update(qualify_native(args.native,args.output))
    if args.records is not None:result.update(verify_any(args.records))
    if args.probe is not None:result.update(qualify_probe(args.probe,args.output if args.native else args.probe_artifacts))
    payload=json.dumps(result,indent=2)+'\n'
    if args.output:
        destination=args.output/'qualification.json' if args.native else args.output
        with destination.open('x') as out:out.write(payload)
    print(payload,end='')


if __name__=='__main__':main()
