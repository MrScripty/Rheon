"""Independent rational mesh cubature and fixed-pose native velocity response.

Reuses the qualified load laboratory's degree-two rational integration, NOT the
production P1 reduction. Exact-real impulse/energy expectations are generated
fresh. Tolerance remains fixed absolute 1e-12; no IEEE or evolution theorem.
"""
import argparse
import copy
from fractions import Fraction as Q
import hashlib
import json
import math
from pathlib import Path
import random
import subprocess
import sys
import check_mesh_traction as mesh

ROOT = Path(__file__).resolve().parents[1]
TOLERANCE = mesh.TOLERANCE

def fixtures():
    cases = []
    for original in mesh.fixtures():
        f = copy.deepcopy(original)
        f.update(velocity=mesh.vector((1,2,3)), omega=mesh.vector(('1/4','1/2','3/4')),
                 mass=Q(2), inertia=mesh.vector((3,4,5)), duration=Q(1,2))
        if f['name']=='zero-twist-preserves-force-and-torque': f.update(velocity=mesh.ZERO,omega=mesh.ZERO)
        if f['name']=='rebased-reference-same-rigid-field': f['name']='shifted-declared-COM-body-response'
        cases.append(f)
    triangle = [(0,0,0),(2,0,0),(0,1,0)]
    for name, loads in [('pure-couple-negative-work',[(0,0,-3),(0,0,3),(0,0,0)]),
                        ('reversed-load-extracts-energy',[(0,0,0),(0,0,-3),(0,0,0)])]:
        f=mesh.case(name,'traction',triangle,[(0,1,2)],[loads],velocity=(1,2,3),omega=(4,5,6))
        f.update(mass=Q(2),inertia=mesh.vector((3,4,5)),duration=Q(1,2));cases.append(f)
    original=cases[0]
    for name, permutation, signs in [('axis-cycle',(1,2,0),(1,1,1)),
                                     ('reflection-x',(0,1,2),(-1,1,1)),
                                     ('reflection-z',(0,1,2),(1,1,-1))]:
        f=copy.deepcopy(original);f['name']=name
        polar=lambda v:tuple(v[permutation[i]]*signs[i] for i in range(3))
        determinant=math.prod(signs) # chosen cycle has positive determinant
        axial=lambda v:mesh.mul(polar(v),determinant)
        f['vertices']=[polar(v) for v in f['vertices']];f['reference']=polar(f['reference'])
        f['loads']=[[polar(v) for v in row] for row in f['loads']]
        f['velocity']=polar(f['velocity']);f['omega']=axial(f['omega'])
        f['inertia']=tuple(f['inertia'][i] for i in permutation);cases.append(f)
    for name, mass, inertia in [('mass-scaled',4,(3,4,5)),('inertia-scaled',2,(6,8,10)),
                                ('planar-inertia-equality',2,(1,1,2))]:
        f=copy.deepcopy(original);f.update(name=name,mass=Q(mass),inertia=mesh.vector(inertia));cases.append(f)
    return cases

def energy(mass,inertia,v,w):
    return (mass*mesh.dot(v,v)+sum(c*x*x for c,x in zip(inertia,w)))/2

def oracle(f):
    loads=mesh.oracle(f);J=mesh.mul(loads['force'],f['duration']);K=mesh.mul(loads['torque'],f['duration'])
    v=f['velocity'];w=f['omega'];vp=mesh.add(v,mesh.mul(J,1/f['mass']))
    wp=tuple(x+j/c for x,j,c in zip(w,K,f['inertia']))
    work=mesh.dot(J,mesh.mul(mesh.add(v,vp),Q(1,2)))+mesh.dot(K,mesh.mul(mesh.add(w,wp),Q(1,2)))
    old=energy(f['mass'],f['inertia'],v,w);new=energy(f['mass'],f['inertia'],vp,wp)
    if new-old != work: raise AssertionError('exact impulse work identity')
    return dict(force_n=loads['force'],torque_n_m=loads['torque'],impulse_n_s=J,
                angular_impulse_n_m_s=K,velocity_before_m_s=v,velocity_after_m_s=vp,
                angular_before_rad_s=w,angular_after_rad_s=wp,momentum_defect=mesh.ZERO,
                angular_momentum_defect=mesh.ZERO,kinetic_before_j=old,kinetic_after_j=new,
                impulse_work_j=work,update_work_j=work,energy_defect_j=Q(0),
                update_energy_defect_j=Q(0),generation_before=2,generation_after=3,
                equivalent_duration_s=f['duration'])

def input_text(f):
    tokens=mesh.input_text(f).split()
    at=3+3*len(f['vertices'])+3*len(f['triangles'])+9
    tokens[at:at]=[repr(float(f['mass']))]+[repr(float(x)) for x in f['inertia']]+[repr(float(f['duration']))]
    return ' '.join(tokens)+'\n'

def inertia_triples():
    smallest=math.ulp(0.0); largest=sys.float_info.max
    boundary=[smallest,math.nextafter(sys.float_info.min,0),sys.float_info.min,
              1.,math.nextafter(1.,math.inf),largest/2,largest]
    triples=[(a,b,c) for a in boundary for b in boundary for c in boundary]
    triples += [(1.,1.,2.),(1.,1.,3.),(math.nextafter(1.,math.inf),1.,math.ulp(1.)/2)]
    rng=random.Random(20261008)
    for _ in range(4096):
        a=math.ldexp(rng.uniform(1,2),rng.randrange(-1000,1000));b=math.ldexp(rng.uniform(1,2),rng.randrange(-1000,1000))
        c=math.ldexp(rng.uniform(1,2),rng.randrange(-1000,1000));triples.append((a,b,c))
        s=a+b
        if math.isfinite(s): triples.extend([(a,b,s),(a,b,math.nextafter(s,math.inf)),(a,b,math.nextafter(s,0.))])
    return triples

def inertia_oracle(executable,output):
    triples=inertia_triples(); records=[]
    for start in range(0,len(triples),256):
        batch=triples[start:start+256]
        text='inertia '+str(len(batch))+' '+' '.join(repr(x) for t in batch for x in t)+'\n'
        run=subprocess.run([str(executable)],input=text,text=True,capture_output=True,check=True,timeout=10)
        actual=json.loads(run.stdout);expected=[]
        for t in batch:
            q=tuple(map(Q,t)); expected.append(all(q[i]<=q[(i+1)%3]+q[(i+2)%3] for i in range(3)))
        if actual != expected or any(type(x) is not bool for x in actual): raise AssertionError('native physical inertia gate differs from exact dyadics')
        p=output/f'inertia-{start:05d}.json';p.write_text(json.dumps(dict(triples=batch,expected=expected,actual=actual,input=text),indent=2)+'\n')
        records.append(dict(path=p.name,sha256=mesh.sha(p),count=len(batch)))
    return dict(triples=len(triples),mismatches=0,records=records,semantics='exact represented binary64 rational triangle inequalities; finite corpus, no IEEE theorem')

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--executable',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    binary=args.executable.resolve(strict=True);output=args.output.resolve()
    if output.exists() or output.is_relative_to(ROOT):raise ValueError('fresh external output required')
    output.mkdir(parents=True);snapshot=mesh.source_snapshot(ROOT);binary_sha=mesh.sha(binary)
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    receipt=dict(schema='rheon.rigid-mesh-impulse.rational-comparison',source_head=head,source_sha256=snapshot,
                 executable_sha256=binary_sha,absolute_tolerance=TOLERANCE,command=sys.argv,fixtures=[],
                 physical_time_advances=0,pose_advances=0,held_campaigns=0,qualified=False)
    try:
        for f in fixtures():
            p=output/f['name'];text=input_text(f);expected=oracle(f)
            p.with_suffix('.fixture.json').write_text(json.dumps(mesh.exact_json(f),indent=2)+'\n')
            p.with_suffix('.input.txt').write_text(text);p.with_suffix('.expected.json').write_text(json.dumps(mesh.exact_json(expected),indent=2)+'\n')
            r=subprocess.run([str(binary)],input=text,text=True,capture_output=True,timeout=10)
            p.with_suffix('.stdout.json').write_text(r.stdout);p.with_suffix('.stderr.txt').write_text(r.stderr)
            if r.returncode:raise AssertionError(f['name']+': '+r.stderr)
            actual=json.loads(r.stdout,object_pairs_hook=mesh.strict_object,parse_constant=lambda x:(_ for _ in ()).throw(ValueError(x)))
            if set(actual)!=set(expected):raise AssertionError('native response schema differs')
            error=mesh.compare(actual,expected)
            receipt['fixtures'].append(dict(name=f['name'],maximum_absolute_error=error,stdout_sha256=mesh.sha(p.with_suffix('.stdout.json'))))
        receipt['inertia_gate']=inertia_oracle(binary,output)
        # Ensure the acceptance checker rejects nonnumeric and perturbed endpoints.
        expected=oracle(fixtures()[0]);actual=json.loads((output/(fixtures()[0]['name']+'.stdout.json')).read_text())
        for changed in [True,'3.25',float('nan'),actual['velocity_after_m_s'][2]+2*TOLERANCE]:
            mutated=copy.deepcopy(actual);mutated['velocity_after_m_s'][2]=changed
            try:mesh.compare(mutated,expected)
            except (ValueError,AssertionError):pass
            else:raise AssertionError('mutated native endpoint accepted')
        receipt['negative_checker_probes']=4
        if mesh.source_snapshot(ROOT)!=snapshot or mesh.sha(binary)!=binary_sha or subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()!=head:raise ValueError('source/executable drift')
        receipt['qualified']=True
    except Exception as e:receipt['failure']=str(e);raise
    finally:
        receipt['evidence_sha256']={p.name:mesh.sha(p) for p in sorted(output.iterdir()) if p.is_file()}
        (output/'qualification.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(qualified=True,fixtures=len(receipt['fixtures']),inertia_triples=receipt['inertia_gate']['triples'],maximum_absolute_error=max(r['maximum_absolute_error'] for r in receipt['fixtures']),output=str(output))))
if __name__=='__main__':main()
