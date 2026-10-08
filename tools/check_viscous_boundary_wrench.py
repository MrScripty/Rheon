#!/usr/bin/env python3
"""Independent geometry and Fraction check of the stationary viscous wrench.

The ideal lift is derived without native rows. A separate nearest-binary64
assembly model rounds each documented primitive (no FMA) and requires exact
stored E/W/mass/Cs/Co values. Only native action/work diagnostics receive an
explicit same-unit sum-of-absolute-contributions tolerance; it is not enclosure
or permission to advance physics. This is a variational generalized wrench,
not a point traction/facet load or a pressure wrench.
"""
from __future__ import annotations
from dataclasses import dataclass
from fractions import Fraction as F
from pathlib import Path
import argparse
import copy
import hashlib
import json
import math
import struct
import subprocess
import sys

from aligned_strain_oracle import (Geometry, Face, Row, Refusal, require, reference,
                                  bits_value, flatten, coordinates, close)

SCHEMA = 'rheon-viscous-boundary-wrench-v1'
EPS = F.from_float(sys.float_info.epsilon)
DIAGNOSTIC_TOLERANCE = 128*EPS


def bits(x):
    return struct.pack('>d',float(x)).hex()


def scalar(token):
    require(type(token)is str,'binary64 token type')
    return bits_value(token)


def rounded(value):
    try: result=float(value)
    except OverflowError as error: raise Refusal('independent binary64 overflow') from error
    require(math.isfinite(result),'independent binary64 nonfinite')
    return F.from_float(result)


def product64(values):
    result=F(1)
    for value in values: result=rounded(result*value)
    return result


def rigid_basis(axis,position,ref):
    dx,dy,dz=[p-r for p,r in zip(position,ref)]
    result=[F(0)]*6;result[axis]=F(1)
    result[3:]=((F(0),dz,-dy),(-dz,F(0),dx),(dy,-dx,F(0)))[axis]
    return result


def stored_basis(axis,position,ref):
    radius=[rounded(p-r) for p,r in zip(position,ref)]
    return rigid_basis(axis,radius,(F(0),)*3)


def stored_geometry(g):
    """Independent stored primitives; never read any observation to assemble."""
    ideal_faces,ideal_rows,_,_=reference(g)
    faces=[]
    for f in ideal_faces:
        area=product64([rounded(g.width(d,f.p[d])) for d in ((f.axis+1)%3,(f.axis+2)%3)])
        distance=rounded(f.distance)
        mass=product64((g.density,area,distance))
        faces.append(Face(f.axis,f.flat,f.p,f.position,area,distance,mass))
    # Native enumeration sorts each edge's actual quadrants 0,1,2,3. The
    # ideal geometry oracle enumerates signs independently; reorder only its
    # sector blocks, never infer the order from observed rows.
    ordered=[];groups={}
    for row in ideal_rows:
        if row.family[0]==row.family[1]:ordered.append(row)
        else:groups.setdefault((row.family,row.p),[]).append(row)
    for group in groups.values():ordered.extend(sorted(group,key=lambda r:r.quadrant))
    rows=[]
    for row in ordered:
        terms=tuple((i,rounded(F(1 if a>0 else -1)/rounded(F(1)/abs(a)))) for i,a in row.terms)
        a,b=row.family
        if a==b:
            volume=product64([rounded(g.width(d,row.p[d])) for d in range(3)])
            weight=rounded(2*volume)
        else:
            cell=list(row.p);cell[a]-=1-(row.quadrant&1);cell[b]-=1-((row.quadrant>>1)&1)
            c=3-a-b
            width_a=abs(rounded(g.center(a,cell[a])-g.plane(a,row.p[a])))
            width_b=abs(rounded(g.center(b,cell[b])-g.plane(b,row.p[b])))
            width_c=rounded(g.width(c,cell[c]))
            weight=product64((width_a,width_b,width_c))
        rows.append(Row(row.family,row.p,row.quadrant,weight,terms,row.closure))
    return faces,rows


@dataclass(frozen=True)
class Lift:
    solid: tuple[F,...]
    outer: tuple[F,...]


def lift_rows(g,ref,stored=False):
    faces,rows=stored_geometry(g) if stored else reference(g)[:2]
    lookup={(f.axis,f.p):i for i,f in enumerate(faces)}
    basis=stored_basis if stored else rigid_basis
    def add(out,coefficient,values):
        for k in range(6):
            term=coefficient*values[k]
            out[k]=rounded(out[k]+rounded(term)) if stored else out[k]+term
    lifts=[]
    for row in rows:
        solid=[F(0)]*6;outer=[F(0)]*6
        a,b=row.family
        if a==b:
            width=rounded(g.width(a,row.p[a])) if stored else g.width(a,row.p[a])
            inverse=rounded(F(1)/width) if stored else F(1)/width
            for side in (0,1):
                p=list(row.p);p[a]+=side;p=tuple(p)
                if (a,p) in lookup: continue
                coefficient=inverse if side else -inverse
                trace=basis(a,g.position(a,p),ref)
                target=outer if p[a] in (0,g.counts[a]) else solid
                # A padded retained box leaves every nonouter missing trace on
                # its actual obstacle face. Its location is never projected.
                add(target,coefficient,trace)
        elif row.closure in (1,2):
            for i,coefficient in row.terms:
                f=faces[i];add(solid,-coefficient,basis(f.axis,f.position,ref))
        lifts.append(Lift(tuple(solid),tuple(outer)))
    return faces,rows,lifts


def exact_metrics(g,faces,rows,lifts,ref,u,xi=(F(0),)*6,eta=(F(0),)*6):
    strains=[sum(a*u[i] for i,a in row.terms) for row in rows]
    force=[F(0)]*len(faces);force_scale=[F(0)]*len(faces)
    solid=[F(0)]*6;outer=[F(0)]*6;solid_scale=[F(0)]*6;outer_scale=[F(0)]*6
    loss=F(0);loss_scale=F(0);row_work=F(0);row_work_scale=F(0)
    for row,lift,strain in zip(rows,lifts,strains):
        load=-g.viscosity*row.weight*strain
        strain_scale=sum(abs(a*u[i]) for i,a in row.terms)
        load_scale=g.viscosity*row.weight*strain_scale
        loss+=g.viscosity*row.weight*strain**2
        loss_scale+=g.viscosity*row.weight*strain_scale**2
        for i,a in row.terms:
            force[i]+=a*load;force_scale[i]+=abs(a)*load_scale
        for k in range(6):
            solid[k]+=lift.solid[k]*load;solid_scale[k]+=abs(lift.solid[k])*load_scale
            outer[k]+=lift.outer[k]*load;outer_scale[k]+=abs(lift.outer[k])*load_scale
        virtual=sum(a*x for a,x in zip(lift.solid,xi))+sum(a*x for a,x in zip(lift.outer,eta))
        row_work+=load*virtual
        row_work_scale+=load_scale*(sum(abs(a*x) for a,x in zip(lift.solid,xi))+sum(abs(a*x) for a,x in zip(lift.outer,eta)))
    fluid=[F(0)]*6;fluid_scale=[F(0)]*6
    for f,value,scale in zip(faces,force,force_scale):
        for k,a in enumerate(stored_basis(f.axis,f.position,ref)):
            fluid[k]+=a*value;fluid_scale[k]+=abs(a)*scale
    work=sum(x*y for x,y in zip(u,force));work_scale=sum(abs(x)*s for x,s in zip(u,force_scale))
    wrench_work=sum(a*x for a,x in zip(solid,xi))+sum(a*x for a,x in zip(outer,eta))
    return {'strains':strains,'force':force,'solid_wrench':solid,'outer_wrench':outer,'fluid_wrench':fluid,
            'balance_defect':[s+o+f for s,o,f in zip(solid,outer,fluid)],'dissipation':loss,'force_work':work,'work_defect':loss+work,
            'wrench_work':wrench_work,'row_work':row_work,'defect':wrench_work-row_work,
            'scales':{'force':force_scale,'solid_wrench':solid_scale,'outer_wrench':outer_scale,'fluid_wrench':fluid_scale,
                      'balance_defect':[s+o+f for s,o,f in zip(solid_scale,outer_scale,fluid_scale)],
                      'dissipation':loss_scale,'force_work':work_scale,'work_defect':loss_scale+work_scale,
                      'wrench_work':row_work_scale,'row_work':row_work_scale,'defect':2*row_work_scale}}


def common_residual(faces,rows,lifts,ref):
    residual=[F(0)]*6;scales=[F(0)]*6
    for row,lift in zip(rows,lifts):
        for k in range(6):
            terms=[a*stored_basis(faces[i].axis,faces[i].position,ref)[k] for i,a in row.terms]
            terms.extend((lift.solid[k],lift.outer[k]))
            residual[k]=max(residual[k],abs(sum(terms)))
            scales[k]=max(scales[k],sum(map(abs,terms)))
    return residual,scales


FIXTURES={
 'unit-center':((3,3,3),(1.,)*3,(0.,)*3,(1,)*3,(2,)*3,1.,1.),
 'reference-shift':((3,3,3),(1.,)*3,(0.,)*3,(1,)*3,(2,)*3,1.,1.),
 'polynomial-coarse':((3,3,3),(1.,)*3,(0.,)*3,(1,)*3,(2,)*3,1.,1.),
 'polynomial-bounded':((6,6,6),(.5,)*3,(0.,)*3,(2,)*3,(4,)*3,1.,1.),
 'anisotropic':((4,5,3),(.5,.75,1.25),(0.,)*3,(1,2,1),(3,3,2),2.,.375),
 'translated-nonmidpoint':((3,3,3),(.3,.7,1.1),(1e8,-1e8,.1),(1,)*3,(2,)*3,3.7,.375),
 'translated-large':((3,3,3),(.3,.7,1.1),(1e12,-1e12,.1),(1,)*3,(2,)*3,3.7,.375),
}
SOLID_TWIST=(.25,-.5,.125,.75,-.25,.5)
OUTER_TWIST=(-.5,.25,.75,-.125,.5,-.25)


def fixture(case):
    require(case in FIXTURES,'unknown fixed fixture')
    n,h,o,lo,hi,rho,mu=FIXTURES[case]
    g=Geometry(n,tuple(map(F.from_float,h)),tuple(map(F.from_float,o)),lo,hi,F.from_float(rho),F.from_float(mu))
    ref=tuple(rounded(g.plane(d,lo[d])+rounded(F(1,2)*rounded(g.plane(d,hi[d])-g.plane(d,lo[d])))) for d in range(3))
    if case=='reference-shift':ref=tuple(map(F.from_float,(-.25,2.5,.75)))
    return g,ref


def expected_fields(g,faces,rows,ref):
    """Declared field controls, independently generated at represented samples."""
    matrix=((.25,.5,-.125),(-.25,-.5,.375),(.125,-.25,.5))
    fields={'rest':[0.]*len(faces),'translation':[1. if f.axis==0 else 0. for f in faces]}
    affine=[];rigid=[]
    for f in faces:
        x=[float(v) for v in f.position];r=[x[d]-float(ref[d]) for d in range(3)]
        total=0.
        for d in range(3):total=total+matrix[f.axis][d]*r[d]
        affine.append(total)
        rigid.append(0.+((-0.25*r[2]-.125*r[1]),(.125*r[0]-.5*r[2]),(.5*r[1]+.25*r[0]))[f.axis])
    fields['affine-strain']=affine;fields['rigid-sample']=rigid
    fields['crosscomponent']=[((i*17+3)%29-14)/16. for i in range(len(faces))]
    local=[0.]*len(faces);selected={(0,(1,0,0)),(0,(1,1,0)),(1,(0,1,0)),(1,(1,1,0))}
    for i,f in enumerate(faces):
        if (f.axis,f.p) in selected:
            local[i]=-(float(f.position[1])-float(g.plane(1,1))) if f.axis==0 else float(f.position[0])-float(g.plane(0,1))
    fields['compatible-local-rotation']=local
    low=[float(g.plane(d,g.lower[d])) for d in range(3)];high=[float(g.plane(d,g.upper[d])) for d in range(3)]
    poly=[];tilted=[];center=low[0]+.5*(high[0]-low[0])
    for f in faces:
        x=list(map(float,f.position));p=[(x[d]-low[d])*(x[d]-high[d]) for d in range(3)]
        tilt=1.+x[0]-center
        if f.axis==0:
            poly.append(2.*p[0]*p[0]*p[1]*(2.*x[1]-low[1]-high[1])*p[2]*p[2])
            tilted.append(225.*tilt*p[0]*p[0]*2.*p[1]*(2.*x[1]-low[1]-high[1])*p[2]*p[2])
        elif f.axis==1:
            poly.append(-2.*p[0]*(2.*x[0]-low[0]-high[0])*p[1]*p[1]*p[2]*p[2])
            tilted.append(-225.*(p[0]*p[0]+2.*tilt*p[0]*(2.*x[0]-low[0]-high[0]))*p[1]*p[1]*p[2]*p[2])
        else:poly.append(0.);tilted.append(0.)
    fields['polynomial-curl']=poly;fields['polynomial-tilted-curl']=tilted
    for a,b,c,name in ((1,2,0,'polynomial-tilted-curl-yz'),(2,0,1,'polynomial-tilted-curl-zx')):
        center_a=low[a]+.5*(high[a]-low[a]);values=[]
        for f in faces:
            x=list(map(float,f.position));p=[(x[d]-low[d])*(x[d]-high[d]) for d in range(3)]
            tilt=1.+(x[a]-center_a)
            if f.axis==a:values.append(225.*tilt*p[a]*p[a]*2.*p[b]*(2.*x[b]-low[b]-high[b])*p[c]*p[c])
            elif f.axis==b:values.append(-225.*(p[a]*p[a]+2.*tilt*p[a]*(2.*x[a]-low[a]-high[a]))*p[b]*p[b]*p[c]*p[c])
            else:values.append(0.)
        fields[name]=values
    seen=set()
    for row in rows:
        if row.closure!=2 or len(row.terms)!=2 or (row.family,row.p) in seen:continue
        seen.add((row.family,row.p));u=[0.]*len(faces)
        u[row.terms[0][0]]=.625;u[row.terms[1][0]]=-.375
        fields['corner-'+'-'.join(map(str,(*row.family,*row.p)))]=u
    return fields


def polynomial_integral(coefficients,lo,hi):
    return sum(c*(hi**(i+1)-lo**(i+1))/F(i+1) for i,c in enumerate(coefficients))


def continuum_polynomial_wrench(g,ref,alpha=F(1),beta=F(0),axes=(0,1,2)):
    """Separate exact facet traction integrals, not the discrete lift matrix.

    psi=alpha*(1+beta*(x-cx))*[Px Py Pz]^2; u=curl(psi ez).
    A smooth cutoff can leave the finite sampled points and obstacle vicinity
    untouched while enforcing the stationary outer boundary. Point continuum
    incompressibility does not imply incidence divergence of sampled MAC data.
    """
    low=[g.plane(d,g.lower[d]) for d in range(3)];high=[g.plane(d,g.upper[d]) for d in range(3)]
    require(axes in ((0,1,2),(1,2,0),(2,0,1)),'oriented cyclic polynomial axes')
    lengths=[high[d]-low[d] for d in axes];lx,ly,lz=lengths
    integrals=[polynomial_integral((F(0),F(0),L*L,-2*L,F(1)),F(0),L) for L in lengths]
    ix,iy,iz=integrals
    left=1-beta*lx/2;right=1+beta*lx/2
    force_y=2*g.viscosity*alpha*lx*lx*(left-right)*iy*iz
    torque_xfaces=-g.viscosity*alpha*lx**3*(left+right)*iy*iz
    # Odd (x-cx)*Px^2 integral vanishes; compute it rather than assume it.
    odd=polynomial_integral((F(0),F(0),-lx**3/2,2*lx*lx,-F(5)*lx/2,F(1)),F(0),lx)
    torque_yfaces=-2*g.viscosity*alpha*ly**3*(ix+beta*odd)*iz
    force=[F(0)]*3;force[axes[1]]=force_y
    center=[(a+b)/2 for a,b in zip(low,high)];offset=[c-r for c,r in zip(center,ref)]
    torque=[offset[1]*force[2]-offset[2]*force[1],offset[2]*force[0]-offset[0]*force[2],offset[0]*force[1]-offset[1]*force[0]]
    torque[axes[2]]+=torque_xfaces+torque_yfaces
    return force+torque

def pairs(pairs):
    result={}
    for key,value in pairs:
        require(key not in result,'duplicate JSON key '+key);result[key]=value
    return result


def read_json(path):
    with Path(path).open('rb') as stream:raw=stream.read(2*1024*1024+1)
    require(len(raw)<=2*1024*1024,'native JSON byte cap')
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda _: (_ for _ in ()).throw(Refusal('nonfinite JSON constant')))


def roster(value,keys,label):
    require(type(value)is dict and set(value)==set(keys),'field roster '+label)


def native_vector(value,count,label):
    require(type(value)is list and len(value)==count,'vector shape '+label)
    return [scalar(x) for x in value]


def diagnostic(actual,expected,scale,label):
    close(actual,expected,label,tolerance=DIAGNOSTIC_TOLERANCE,scale=scale)


def exact_stored(token,expected,label):
    require(token==bits(expected),'stored assembly bits '+label)


def verify_record(packet,case=None):
    roster(packet,('schema','meta','reference','active','rows','fields','report'),'packet')
    require(packet['schema']==SCHEMA,'wrench schema')
    meta=packet['meta'];roster(meta,('counts','spacing','origin','lo','hi','rho','mu'),'meta')
    for key in ('counts','spacing','origin','lo','hi'):
        require(type(meta[key])is list and len(meta[key])==3,'meta triple '+key)
    require(all(type(n)is int and 3<=n<=16 for n in meta['counts']),'bounded oracle grid counts')
    g=Geometry(tuple(meta['counts']),tuple(map(scalar,meta['spacing'])),tuple(map(scalar,meta['origin'])),
               tuple(meta['lo']),tuple(meta['hi']),scalar(meta['rho']),scalar(meta['mu']))
    ref=tuple(native_vector(packet['reference'],3,'reference'))
    if case is None:
        case=next((name for name in FIXTURES if fixture(name)==(g,ref)),None)
    require(case is not None and fixture(case)==(g,ref),'declared fixed fixture geometry/material/reference')
    faces,rows,lifts=lift_rows(g,ref,stored=True)
    prescribed_fields=expected_fields(g,faces,rows,ref)
    require(type(packet['active'])is list and len(packet['active'])==len(faces),'active count')
    for i,(actual,f) in enumerate(zip(packet['active'],faces)):
        roster(actual,('id','axis','face','coordinates','position','area','distance','mass'),'active')
        require(type(actual['id'])is int and actual['id']==i,'active id')
        require(type(actual['axis'])is int and actual['axis']==f.axis and type(actual['face'])is int and actual['face']==f.flat,'active topology')
        require(type(actual['coordinates'])is list and all(type(x)is int for x in actual['coordinates']) and actual['coordinates']==list(f.p),'active coordinates')
        native_vector(actual['position'],3,'active position')
        for key,value in zip(('position',), (f.position,)):
            require(actual[key]==[bits(x) for x in value],'stored active '+key)
        for key in ('area','distance','mass'):exact_stored(actual[key],getattr(f,key),'active '+key)
    require(type(packet['rows'])is list and len(packet['rows'])==len(rows),'row count including zeros')
    for i,(actual,row,lift) in enumerate(zip(packet['rows'],rows,lifts)):
        roster(actual,('id','axes','coordinates','quadrant','boundary','weight','terms','solid','outer'),'row')
        require(type(actual['id'])is int and actual['id']==i,'row id')
        for key,expected in (('axes',list(row.family)),('coordinates',list(row.p))):
            require(type(actual[key])is list and all(type(x)is int for x in actual[key]) and actual[key]==expected,'row metadata '+key)
        require(type(actual['quadrant'])is int and actual['quadrant']==row.quadrant,'row quadrant')
        require(actual['boundary']=={0:'interior',1:'flat',2:'corner',4:'normal'}[row.closure],'row boundary classification')
        exact_stored(actual['weight'],row.weight,'row weight')
        require(type(actual['terms'])is list and len(actual['terms'])==len(row.terms),'term count')
        for term,(j,a) in zip(actual['terms'],row.terms):
            roster(term,('active','coefficient'),'term')
            require(type(term['active'])is int and term['active']==j,'term active identity')
            exact_stored(term['coefficient'],a,'row coefficient')
        for key,value in (('solid',lift.solid),('outer',lift.outer)):
            native_vector(actual[key],6,'row '+key)
            require(actual[key]==[bits(x) for x in value],'stored lift '+key)
    roster(packet['report'],('common_rigid_residual_max','allocated_bytes','combined_operator_bytes'),'report')
    allocated=packet['report']['allocated_bytes'];combined=packet['report']['combined_operator_bytes']
    require(type(allocated)is int and type(combined)is int and len(rows)*96<=allocated<=combined<=16000000,'retained allocation accounting')
    residual,scale=common_residual(faces,rows,lifts,ref)
    for k,value in enumerate(native_vector(packet['report']['common_rigid_residual_max'],6,'common rigid residual')):
        require(value>=0,'negative rigid residual');diagnostic(value,residual[k],scale[k],'rigid residual '+str(k))
    require(type(packet['fields'])is list and len(packet['fields'])==len(prescribed_fields),'fixed field roster')
    names=set();summaries={}
    for field in packet['fields']:
        roster(field,('name','identity','values','force','solid_wrench','outer_wrench','fluid_wrench','balance_defect','dissipation','force_work','work_defect','virtual_work'),'field')
        require(type(field['name'])is str and field['name'] and field['name'] not in names,'unique named field');names.add(field['name'])
        identity=field['identity'];roster(identity,('surface_stamp','reference','density','viscosity'),'field identity')
        roster(identity['surface_stamp'],('id','version'),'surface stamp')
        require(identity['surface_stamp']=={'id':'73','version':'1'},'actual owner stamp')
        require(identity['reference']==packet['reference'] and identity['density']==meta['rho'] and identity['viscosity']==meta['mu'],'actual report reference/material identity')
        require(field['name'] in prescribed_fields,'unknown fixed field')
        require(field['values']==[bits(x) for x in prescribed_fields[field['name']]],'prescribed field bits '+field['name'])
        u=native_vector(field['values'],len(faces),'field values')
        virtual=field['virtual_work'];roster(virtual,('solid_twist','outer_twist','wrench_work','row_work','defect'),'virtual work')
        require(virtual['solid_twist']==list(map(bits,SOLID_TWIST)) and virtual['outer_twist']==list(map(bits,OUTER_TWIST)),'prescribed virtual twist controls')
        xi=native_vector(virtual['solid_twist'],6,'solid twist');eta=native_vector(virtual['outer_twist'],6,'outer twist')
        metrics=exact_metrics(g,faces,rows,lifts,ref,u,xi,eta)
        for key,count in (('force',len(faces)),('solid_wrench',6),('outer_wrench',6),('fluid_wrench',6),('balance_defect',6)):
            values=native_vector(field[key],count,key)
            for j,value in enumerate(values):diagnostic(value,metrics[key][j],metrics['scales'][key][j],key+' '+str(j))
        for key in ('dissipation','force_work','work_defect'):
            diagnostic(scalar(field[key]),metrics[key],metrics['scales'][key],key)
        require(scalar(field['dissipation'])>=0,'negative physical dissipation')
        for key in ('wrench_work','row_work','defect'):
            diagnostic(scalar(virtual[key]),metrics[key],metrics['scales'][key],'virtual '+key)
        summaries[field['name']]={'dissipation_exact':str(metrics['dissipation']),
             'solid_wrench_exact':list(map(str,metrics['solid_wrench'])),'outer_wrench_exact':list(map(str,metrics['outer_wrench'])),
             'balance_defect_exact':list(map(str,metrics['balance_defect']))}
    polynomial_comparisons={}
    for name,alpha,beta,axes in (('polynomial-curl',F(1),F(0),(0,1,2)),('polynomial-tilted-curl',F(225),F(1),(0,1,2)),('polynomial-tilted-curl-yz',F(225),F(1),(1,2,0)),('polynomial-tilted-curl-zx',F(225),F(1),(2,0,1))):
        analytic=continuum_polynomial_wrench(g,ref,alpha,beta,axes)
        discrete=list(map(F,summaries[name]['solid_wrench_exact']))
        observed=native_vector(next(f for f in packet['fields'] if f['name']==name)['solid_wrench'],6,'observed polynomial wrench')
        sign=lambda x:1 if x>0 else -1 if x<0 else 0
        polynomial_comparisons[name]={'continuum_wrench_exact':list(map(str,analytic)),'discrete_minus_continuum_exact':list(map(str,(a-b for a,b in zip(discrete,analytic)))),
            'observed_native_wrench_exact':list(map(str,observed)),'observed_minus_continuum_exact':list(map(str,(a-b for a,b in zip(observed,analytic)))),
            'sign_matches':[sign(a)==sign(b) for a,b in zip(observed,analytic)],'scope':'fixed specimen, not accuracy or refinement qualification'}
    return {'schema':SCHEMA,'fixture':case,'polynomial_physical_controls':polynomial_comparisons,'active_faces':len(faces),'rows':len(rows),'zero_rows':sum(not r.terms for r in rows),
            'fields':summaries,'assembly_comparison':'exact independent nearest-binary64 bits',
            'diagnostic_relative_contribution_tolerance':str(DIAGNOSTIC_TOLERANCE),
            'common_rigid_residual_exact':list(map(str,residual)),
            'scope':'stationary variational viscous generalized wrench; pressure disabled'}


def verify(path,case=None):
    result=verify_record(read_json(path),case);result['records_sha256']=hashlib.sha256(Path(path).read_bytes()).hexdigest();return result


def hostile_records(packet):
    cases=[]
    def add(name,mutate):
        bad=copy.deepcopy(packet);mutate(bad);cases.append((name,bad))
    nonzero=next(i for i,r in enumerate(packet['rows']) if r['terms'])
    zero=next(i for i,r in enumerate(packet['rows']) if not r['terms'])
    solid=next((i,k) for i,r in enumerate(packet['rows']) for k,x in enumerate(r['solid']) if scalar(x))
    translation=next(i for i,f in enumerate(packet['fields']) if f['name']=='translation')
    add('unknown-pressure-key',lambda p:p.__setitem__('pressure',[]))
    add('boolean-active-id',lambda p:p['active'][0].__setitem__('id',False))
    add('boolean-row-id',lambda p:p['rows'][0].__setitem__('id',False))
    add('boolean-term-active',lambda p:p['rows'][nonzero]['terms'][0].__setitem__('active',False))
    add('coefficient-sign',lambda p:p['rows'][nonzero]['terms'][0].__setitem__('coefficient',bits(-scalar(p['rows'][nonzero]['terms'][0]['coefficient']))))
    add('positive-mass-changed',lambda p:p['active'][0].__setitem__('mass',bits(2*scalar(p['active'][0]['mass']))))
    add('negative-weight',lambda p:p['rows'][0].__setitem__('weight',bits(-scalar(p['rows'][0]['weight']))))
    add('zero-row-deleted',lambda p:p['rows'].pop(zero))
    add('zero-row-made-active',lambda p:p['rows'][zero].__setitem__('terms',[{'active':0,'coefficient':bits(1)}]))
    add('lift-sign',lambda p:p['rows'][solid[0]]['solid'].__setitem__(solid[1],bits(-scalar(p['rows'][solid[0]]['solid'][solid[1]]))))
    add('lift-one-ulp',lambda p:p['rows'][solid[0]]['solid'].__setitem__(solid[1],bits(math.nextafter(float(scalar(p['rows'][solid[0]]['solid'][solid[1]])),math.inf))))
    add('all-lifts-zero',lambda p:[r.update(solid=[bits(0)]*6,outer=[bits(0)]*6) for r in p['rows']])
    add('field-values-zero',lambda p:p['fields'][translation].__setitem__('values',[bits(0)]*len(p['active'])))
    def consistent_zero(p):
        for f in p['fields']:
            f['values']=[bits(0)]*len(p['active']);f['force']=[bits(0)]*len(p['active'])
            for k in ('solid_wrench','outer_wrench','fluid_wrench','balance_defect'):f[k]=[bits(0)]*6
            for k in ('dissipation','force_work','work_defect'):f[k]=bits(0)
            for k in ('wrench_work','row_work','defect'):f['virtual_work'][k]=bits(0)
    add('selfconsistent-all-fields-zero',consistent_zero)
    add('native-force-forged',lambda p:p['fields'][translation]['force'].__setitem__(0,bits(1e6)))
    add('native-torque-forged',lambda p:p['fields'][translation]['solid_wrench'].__setitem__(5,bits(1e6)))
    add('physical-dissipation-negative',lambda p:p['fields'][translation].__setitem__('dissipation',bits(-1)))
    add('virtual-row-work-forged',lambda p:p['fields'][translation]['virtual_work'].__setitem__('row_work',bits(1e6)))
    add('twist-control-forged',lambda p:p['fields'][0]['virtual_work']['solid_twist'].__setitem__(0,bits(1)))
    add('unknown-field',lambda p:p['fields'][translation].__setitem__('name','unsupported'))
    add('duplicate-field',lambda p:p['fields'].append(copy.deepcopy(p['fields'][0])))
    add('meta-density-forged',lambda p:p['meta'].__setitem__('rho',bits(2)))
    add('reference-forged',lambda p:p['reference'].__setitem__(0,bits(0)))
    add('identity-stamp-version',lambda p:p['fields'][0]['identity']['surface_stamp'].__setitem__('version','2'))
    add('identity-boolean-stamp',lambda p:p['fields'][0]['identity']['surface_stamp'].__setitem__('id',True))
    add('identity-reference',lambda p:p['fields'][0]['identity']['reference'].__setitem__(0,bits(0)))
    add('identity-density',lambda p:p['fields'][0]['identity'].__setitem__('density',bits(2)))
    add('identity-viscosity',lambda p:p['fields'][0]['identity'].__setitem__('viscosity',bits(2)))
    add('nonfinite-binary64',lambda p:p['fields'][0]['force'].__setitem__(0,bits(float('inf'))))
    return cases


def mutation_suite(packet):
    refused=[]
    for name,bad in hostile_records(packet):
        try:verify_record(bad)
        except Refusal:refused.append(name)
        else:raise Refusal('hostile native record accepted '+name)
    return refused


def reference_covariance(first,second):
    ref0=list(map(scalar,first['reference']));ref1=list(map(scalar,second['reference']))
    shift=[b-a for a,b in zip(ref0,ref1)]
    meta=first['meta'];g=Geometry(tuple(meta['counts']),tuple(map(scalar,meta['spacing'])),tuple(map(scalar,meta['origin'])),tuple(meta['lo']),tuple(meta['hi']),scalar(meta['rho']),scalar(meta['mu']))
    f0,r0,l0=lift_rows(g,ref0,stored=True);f1,r1,l1=lift_rows(g,ref1,stored=True)
    fields1={f['name']:f for f in second['fields']};compared=[]
    for a in first['fields']:
        b=fields1[a['name']]
        if a['values']!=b['values']:continue
        u=list(map(scalar,a['values']));m0=exact_metrics(g,f0,r0,l0,ref0,u);m1=exact_metrics(g,f1,r1,l1,ref1,u)
        for key in ('solid_wrench','outer_wrench','fluid_wrench'):
            old=list(map(scalar,a[key]));new=list(map(scalar,b[key]))
            fx,fy,fz=old[:3];dx,dy,dz=shift
            cross=(dy*fz-dz*fy,dz*fx-dx*fz,dx*fy-dy*fx)
            s0=m0['scales'][key];s1=m1['scales'][key]
            for k in range(3):diagnostic(new[k],old[k],s0[k]+s1[k],'reference force covariance')
            crossed_scale=(abs(dy)*s0[2]+abs(dz)*s0[1],abs(dz)*s0[0]+abs(dx)*s0[2],abs(dx)*s0[1]+abs(dy)*s0[0])
            for k in range(3):diagnostic(new[k+3],old[k+3]-cross[k],s0[k+3]+s1[k+3]+crossed_scale[k],'reference torque covariance')
        compared.append(a['name'])
    require(compared,'reference-invariant field controls required')
    return {'compared_field_names':compared,'shift_exact':list(map(str,shift)),'relation':'torque(new)=torque(old)-shift cross force; diagnostics only'}

def source_snapshot(repo):
    tracked=subprocess.check_output(['git','ls-files','-z'],cwd=repo).decode().split('\0')
    required=('tools/check_viscous_boundary_wrench.py','tools/test_viscous_boundary_wrench_oracle.py',
              'src/viscous_boundary_wrench.rs','examples/viscous_boundary_wrench.rs')
    paths=sorted(set(p for p in tracked if p)|set(required))
    return {'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
            'tree':subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip(),
            'status':subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True),
            'sha256':{p:hashlib.sha256((repo/p).read_bytes()).hexdigest() for p in paths}}


def qualify(executable,directory):
    executable=Path(executable);require(executable.is_absolute(),'absolute executable path required')
    executable=executable.resolve();directory=Path(directory).resolve()
    require(not directory.exists(),'fresh output directory required')
    parent=directory.parent
    while not parent.exists():parent=parent.parent
    require(subprocess.run(['git','-C',str(parent),'rev-parse','--is-inside-work-tree'],capture_output=True).returncode!=0,'evidence must be outside Git checkouts')
    repo=Path(__file__).resolve().parents[1];source=source_snapshot(repo)
    binary=hashlib.sha256(executable.read_bytes()).hexdigest()
    directory.mkdir(parents=True,exist_ok=False)
    results={};commands=[]
    for case in FIXTURES:
        path=directory/(case+'.json');argv=[str(executable),case,str(path)]
        process=subprocess.run(argv,capture_output=True,text=True,timeout=90)
        require(process.returncode==0,'native fixture failed '+case+' '+process.stderr)
        result=verify(path,case);results[case]=result
        commands.append({'argv':argv,'returncode':process.returncode,'stdout':process.stdout,'stderr':process.stderr,'records_sha256':result['records_sha256']})
    rejected=mutation_suite(read_json(directory/'unit-center.json'))
    # Also exercise the raw parser instead of a Python dictionary normalization.
    raw=(directory/'unit-center.json').read_bytes();malformed=raw.replace(b'"schema":',b'"schema":"duplicate","schema":',1)
    duplicate=directory/'hostile-duplicate-key.json';duplicate.write_bytes(malformed)
    try:verify(duplicate)
    except Refusal:rejected.append('duplicate-raw-json-key')
    else:raise Refusal('duplicate raw JSON key accepted')
    require(hashlib.sha256(executable.read_bytes()).hexdigest()==binary,'native binary changed during qualification')
    require(source_snapshot(repo)==source,'source changed during qualification')
    covariance=reference_covariance(read_json(directory/'unit-center.json'),read_json(directory/'reference-shift.json'))
    physical={case:results[case]['polynomial_physical_controls'] for case in ('polynomial-coarse','polynomial-bounded')}
    result={'schema':'rheon-viscous-boundary-wrench-qualification-v1','source':source,'executable_sha256':binary,
            'commands':commands,'cases':results,'physical_controls':physical,'reference_covariance':covariance,'hostile_records_rejected':rejected,
            'scope':'fixed specimens; stationary variational generalized viscous wrench; pressure disabled; no accuracy/refinement qualification'}
    with (directory/'qualification.json').open('x') as stream:json.dump(result,stream,indent=2);stream.write('\n')
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('records',type=Path,nargs='?');parser.add_argument('--executable',type=Path);parser.add_argument('--output',type=Path)
    args=parser.parse_args()
    require((args.records is None)!=(args.executable is None),'choose native launch or one native record')
    if args.executable is not None:
        require(args.output is not None,'native qualification needs fresh evidence directory');result=qualify(args.executable,args.output)
    else:
        result=verify(args.records)
        if args.output:
            with args.output.open('x') as stream:json.dump(result,stream,indent=2);stream.write('\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
