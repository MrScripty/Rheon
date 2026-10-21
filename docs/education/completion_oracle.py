"""Independent closed-form, rational answers for authored completion lessons."""
from fractions import Fraction as F
from engineering_oracle import close
DOMAINS={'reconstruction':[[-.5,0,.5],[0,1],[0,1],[.5,1],['two-point','corrected']], 'coupling':[[.5,1,1.5],[0,1,2,4,8]]}
def verify(q):
 if q['kind']=='reconstruction':
  s,a,b,t,method=q['args'];s,a,b,t=map(F,[s,a,b,t]);two=(a+b*s)/(1+s*s)
  expected={'twoPoint':two,'correction':a-two,'normal':a if method=='corrected' else two,'normalError':0 if method=='corrected' else two-a,'faceLinear':2+b*s/2,'faceCorrected':2,'faceExact':2,'determinant':t}
  for k,v in expected.items():close(q[k],v)
  for x,y in zip(q['gradient'],[a,b]):close(x,y)
  for x,y in zip(q['values'],[2-a/2,2+a/2+b*s,2-a/2+b*t]):close(x,y)
  if q['points']!=[[-.5,0,0],[.5,float(s),0],[-.5,float(t),0]]:raise ValueError('Sample geometry')
 else:
  a,n=q['args'];a=F(a);rv=1-3*a/2;rp=1-a
  if len(q['history'])!=n+1:raise ValueError('History length')
  for j,state in enumerate(q['history']):
   u=(1-rv**j)/6;p=(1-rp**j)/2
   for actual,want in zip(state['u'],[u,-u]):close(actual,want)
   close(state['pressure'],p);close(state['continuity'],0)
   for actual,want in zip(state['momentum'],[1-3*u-p,3*u-p]):close(actual,want)
  u=(1-rv**n)/6;p=(1-rp**n)/2
  close(q['velocityError'],-rv**n/6);close(q['pressureError'],-rp**n/2);close(q['velocityFactor'],rv);close(q['pressureFactor'],rp)
  close(q['momentumNorm']**2,(1-3*u-p)**2+(3*u-p)**2)
  if any(q[k]!=q['history'][-1][k] for k in ['u','pressure','continuity','momentum']):raise ValueError('Final state')
def glyphs(q):
 if q['kind']=='reconstruction':return [([0,-.12,.28],[q['twoPoint']*.18,0,0],'two-point'),([0,.12,.28],[q['normal']*.18,0,0],'selected')]
 return [([0,(i-.5)*.45,.05],[q['u'][i]*.8,0,0],'path') for i in range(2)]
def boxes(q):
 if q['kind']=='reconstruction':return [([0,0,0],[.008,.5,.5])]
 return [([x,0,0],[.3,.75,.35]) for x in [-.5,.5]]
