"""Closed-form rational oracle for two disconnected Neumann pairs."""
from fractions import Fraction as F
from engineering_oracle import close

def verify(q):
    b=[1,-1,2,-2] if q['input']=='compatible' else [1,0,-1,0]
    v=[1,1,1,1] if q['geometry']=='uniform' else [1,3,2,2]
    p=[];used=[];expected_flux=[];r=[];base=[]
    for c in range(2):
        i=c*2;s=b[i]+b[i+1];length=F(v[i]+v[i+1],2)
        br=F(b[i+1])- (F(s,2) if q['mode']=='projected' else 0)
        used.extend([F(b[i])-(F(s,2) if q['mode']=='projected' else 0),br])
        delta=br*length;offset=F(0)
        if q['mode']=='mean':offset=-delta/2
        if q['mode']=='volume':offset=-F(v[i+1],v[i]+v[i+1])*delta
        e=F(q['error']) if c==0 else F(0);shift=F(q['shiftA'] if c==0 else q['shiftB'])
        base.extend([offset,offset+delta+e]);p.extend([offset+shift,offset+delta+e+shift]);f=-br-e/length
        expected_flux.append(f);r.extend([b[i]-f,b[i+1]+f])
    answers={'volumes':v,'b':b,'used':used,'sourceChange':[used[i]-b[i] for i in range(4)],'sums':[b[0]+b[1],b[2]+b[3]],'pressure':p,'flux':expected_flux,'residual':r,'boundary':[-x for x in b],'divergence':[-r[i]/v[i] for i in range(4)],'usedResidual':[used[i]-b[i]+r[i] for i in range(4)],'jumps':[p[1]-p[0],p[3]-p[2]],'kp':[expected_flux[0],-expected_flux[0],expected_flux[1],-expected_flux[1]],'weights':[F(2,v[0]+v[1]),F(2,v[2]+v[3])],'means':[(p[0]+p[1])/2,(p[2]+p[3])/2],'volumeMeans':[(v[i]*p[i]+v[i+1]*p[i+1])/(v[i]+v[i+1]) for i in [0,2]]}
    for k,want in answers.items():
        if len(q[k])!=len(want):raise ValueError('Shape '+k)
        for a,bv in zip(q[k],want):close(a,bv)
    scalars={'reducedResidual':max(abs(r[1]),abs(r[3])),'fullResidual':max(map(abs,r)),'maxDivergence':max(abs(r[i]/v[i]) for i in range(4)),'boundaryWork':-sum(p[i]*b[i] for i in range(4)),'boundaryShift':-F(q['shiftA'])*(b[0]+b[1])-F(q['shiftB'])*(b[2]+b[3]),'interiorWork':sum(expected_flux[c]**2/F(answers['weights'][c]) for c in range(2)),'pairing':-sum(p[i]*r[i] for i in range(4))}
    for k,want in scalars.items():close(q[k],want)
    if q['compatible']!=(q['input']=='compatible') or q['acceptedOriginal']!=(all(x==0 for x in r)):raise ValueError('Acceptance misreported')
    close(q['pairing'],q['interiorWork']+q['boundaryWork'])

def glyphs(q):
    return [([0,y,.06],[.08*q['flux'][c],0,0],'correction') for c,y in enumerate([-.45,.45])]

def boxes(q):
    result=[]
    for c,y in enumerate([-.45,.45]):
        i=2*c
        for j,x in [(i,-.15*q['volumes'][i]),(i+1,.15*q['volumes'][i+1])]:
            result.append(([x,y,0],[.3*q['volumes'][j],.3,.3],0x18363d))
            if q['pressure'][j]!=0:
                result.append(([x,y,.03*q['pressure'][j]],[.06,.06,.06*abs(q['pressure'][j])],0x067d91 if q['pressure'][j]>0 else 0xaa5735))
    return result
