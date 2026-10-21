"""Independent rational, closed-form answers for the bounded teaching domains."""
from fractions import Fraction as F
import math

def close(a,b):
    if not math.isfinite(a) or abs(a-float(b))>2e-12:
        raise ValueError(f'Independent mismatch: {a} != {b}')

def verify(q,lab):
    if lab=='faceflux':
        a,u,b=map(F,[q['area'],q['velocity'],q['boundaryFlux']]);p,r=[[1,3],[3,1],[1,1]][q['pair']]
        answers={'flux':a*u,'pressureWork':(p-r)*(a*u+b),'interiorWork':(p-r)*a*u,'boundaryWork':(p-r)*b}
        for k,v in answers.items():close(q[k],v)
        for actual,want in zip(q['divergence'],[a*u+b,-a*u-b]):close(actual,want)
        if q['pressure']!=[p,r]:raise ValueError('Pressure pair')
        return
    if lab=='checker':
        a,r=F(q['amplitude']),F(q['response']);alternating=q['pattern']=='alternating'
        for i in range(4):
            sign=1 if i%2==0 else -1
            close(q['pressure'][i],sign*a if alternating else a)
            for k in ['gradient','cellVelocity','naive']:close(q[k][i],0)
            close(q['flux'][i],sign*2*a*r if alternating else 0)
            close(q['divergence'][i],sign*4*a*r if alternating else 0)
        return
    rate,mu=F(q['rate']),F(q['mu']);kind=q['kind']
    # Analytic derivatives of (-s*y,s*x,0), (s*y,0,0), (s*x,-s*y,0).
    grad={'rotation':[[0,-rate,0],[rate,0,0],[0,0,0]],'shear':[[0,rate,0],[0,0,0],[0,0,0]],'extension':[[rate,0,0],[0,-rate,0],[0,0,0]]}[kind]
    stress={'rotation':[[0,0,0]]*3,'shear':[[0,mu*rate,0],[mu*rate,0,0],[0,0,0]],'extension':[[2*mu*rate,0,0],[0,-2*mu*rate,0],[0,0,0]]}[kind]
    strain={'rotation':[[0,0,0]]*3,'shear':[[0,rate/2,0],[rate/2,0,0],[0,0,0]],'extension':[[rate,0,0],[0,-rate,0],[0,0,0]]}[kind]
    for key,expected in [('gradient',grad),('strain',strain),('stress',stress)]:
        for actual,row in zip(q[key],expected):
            for a,b in zip(actual,row):close(a,b)
    power={'rotation':0,'shear':mu*rate**2,'extension':4*mu*rate**2}[kind]
    close(q['dissipation'],power);close(q['boundaryPower'],power)
    close(q['gradientProxy'],mu*rate**2*(1 if kind=='shear' else 2))
    positions=[[x,y,z] for x in [-.25,.25] for y in [-.25,.25] for z in [-.25,.25]]
    if [s['position'] for s in q['samples']]!=positions:raise ValueError('Sample geometry')
    face_positions=[]
    for axis in range(3):
        for sign in [-1,1]:
            pos=[0,0,0];pos[axis]=sign*.5;face_positions.append(pos)
    if [f['position'] for f in q['faces']]!=face_positions:raise ValueError('Face geometry')
    for sample in q['samples']:
        x,y,z=map(F,sample['position']);want={'rotation':[-rate*y,rate*x,0],'shear':[rate*y,0,0],'extension':[rate*x,-rate*y,0]}[kind]
        for a,b in zip(sample['velocity'],want):close(a,b)
    face_power={'rotation':[0]*6,'shear':[0,0,mu*rate**2/2,mu*rate**2/2,0,0],'extension':[mu*rate**2]*4+[0,0]}[kind]
    for face_index,f in enumerate(q['faces']):
        close(f['power'],face_power[face_index])
        for i in range(3):close(f['traction'][i],sum(stress[i][j]*F(f['position'][j])*2 for j in range(3)))

def glyphs(q,lab):
    """Physical glyph contract independent of Three.js and DOM readouts."""
    if lab=='faceflux':
        return [([0,0,0],[q['velocity']*.25,0,0],'internal')]+[( [x,0,0],[-q['boundaryFlux']*.25,0,0],'boundary') for x in [-.8,.8]]
    if lab=='checker':
        return [([(i-1)*.4,.3,0],[v*.1,0,0],'compact') for i,v in enumerate(q['flux'])]
    return [(s['position'],[v*.3 for v in s['velocity']],'velocity') for s in q['samples']]+[(f['position'],[v*.12 for v in f['traction']],'traction') for f in q['faces']]
