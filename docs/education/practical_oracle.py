"""Rational/struct-based oracle, independent from browser recurrence execution."""
from fractions import Fraction as F
import struct
from engineering_oracle import close
DOMAINS={'wallload':[[.5,1,2],[0,1,2],[-2,0,2]],'jacobi':[[.5,1,1.5],[0,1,2,4,8],['smooth','alternating','mixed']],'memory':[[8,16,32],[1,4,8],[4,8]],'roundoff':[[23,24,25],['left','cancel'],['binary32','binary64']],'particletransfer':[[1,2],[0,1],[0,1],[0,.25],[0,.5,1]],'sstprobe':[[0,1,2],[1,2],[0,1,4],[0,1]],'refinement':[[2,4,8,16],[0,.0625]],'proposal':[[0,.0625,.25],[-2,0,2],[0,.125],['finite','nonfinite','wrong-units']]}
def compare(a,b):
 if isinstance(b,dict):
  for k,v in b.items():compare(a[k],v)
 elif isinstance(b,list):
  if not isinstance(a,list) or len(a)!=len(b):raise ValueError('Shape')
  for x,y in zip(a,b):compare(x,y)
 elif b is None or isinstance(b,(str,bool)):
  if a!=b:raise ValueError('Decision/string')
 else:close(a,b)
def answer(name,args):
 if name=='wallload':
  mu,U,G=map(F,args);bottom=-mu*U-G/2;top=mu*U-G/2;mean=U/2+G/(12*mu)
  return dict(samples=[dict(y=F(i,8),velocity=U*F(i,8)+G*F(i,8)*(1-F(i,8))/(2*mu),shear=mu*U+G*(1-2*F(i,8))/2) for i in range(9)],bottom=bottom,top=top,mean=mean,wallPower=U*top,bodyPower=G*mean,dissipation=mu*U**2+G**2/(12*mu),forceBalance=0,powerBalance=0)
 if name=='jacobi':
  a,n,mode=args;a=F(a);b={'smooth':[1,1],'alternating':[1,-1],'mixed':[1,0]}[mode];bp=F(b[0]+b[1],2);bm=F(b[0]-b[1],2);exact=[bp+bm/3,bp-bm/3];history=[]
  for j in range(n+1):
   ep=-bp*(1-a/2)**j;em=-bm/3*(1-3*a/2)**j;e=[ep+em,ep-em];x=[exact[i]+e[i] for i in range(2)];r=[-2*e[0]+e[1],-2*e[1]+e[0]];history.append(dict(x=x,residual=r,error=e,energy=2*(e[0]**2-e[0]*e[1]+e[1]**2)))
  return dict(b=b,exact=exact,history=history,**history[-1],smoothFactor=1-a/2,alternatingFactor=1-3*a/2)
 if name=='memory':
  n,c,w=args;cells=n*n*n;faces=sum(n*n*(n+1) for _ in range(3));nnz=cells+sum(2*n*n*(n-1) for _ in range(3));field=w*(c*cells+faces);csr=w*nnz+4*nnz+4*(cells+1);work=w*cells*5
  return dict(cells=cells,faces=faces,nnz=nnz,fieldBytes=field,csrBytes=csr,workspaceBytes=work,totalBytes=field+csr+work)
 if name=='roundoff':
  exponent,order,precision=args;A=2**exponent;terms=[A,1,-A] if order=='left' else [A,-A,1];rd=lambda x:struct.unpack('f',struct.pack('f',x))[0] if precision=='binary32' else float(x);s=rd(0);trace=[]
  for v in terms:s=rd(s+rd(v));trace.append(s)
  return dict(A=A,terms=terms,trace=trace,total=s,exact=1,error=s-1)
 if name=='particletransfer':
  m,v,d,du,a=map(F,args);vs=[v+d,v-d];mean=(vs[0]+m*vs[1])/(1+m);grid=mean+du;output=[grid+a*(x-mean) for x in vs];variance=((vs[0]-mean)**2+m*(vs[1]-mean)**2)/2;p=vs[0]+m*vs[1]
  return dict(masses=[1,m],input=vs,mean=mean,grid=grid,output=output,momentumBefore=p,momentumAfter=p+(1+m)*du,expectedMomentum=p+(1+m)*du,varianceBefore=variance,varianceAfter=a*a*variance)
 if name=='sstprobe':
  k,w,S,f=map(F,args);den=max(F(31,100)*w,S*f);mu=F(31,100)*k/den;raw=mu*S*S;cap=F(9,10)*w*k
  return dict(denominator=den,viscosity=mu,raw=raw,cap=cap,limited=min(raw,cap),laminar=k==0)
 if name=='refinement':
  n,bias=args;bias=F(bias);values=[F(1,3)-F(1,12*m*m)+bias for m in [n,2*n,4*n]];gci=F(5,4)*F(1,12*(4*n)**2)
  return dict(values=values,order=2,extrapolated=F(1,3)+bias,absoluteGci=gci,relativeGci=gci/abs(values[-1]),exact=F(1,3),trueError=values[-1]-F(1,3),modelBias=bias,samples=[dict(y=F(2*i+1,2*n),value=F(2*i+1,2*n)**2+bias) for i in range(n)])
 if name=='proposal':
  e,s,t,mode=args;e,s,t=map(F,[e,s,t]);accepted=mode=='finite' and abs(e)<=t;valid=mode!='nonfinite';return dict(candidate=[s,s-1+e] if valid else None,units='m/s' if mode=='wrong-units' else 'Pa',finite=valid,residual=[e,-e] if valid else None,maxResidual=abs(e) if valid else None,accepted=accepted,usedPressure=[0,-1+e] if accepted else [0,-1],candidateFlux=1-e if valid else None,usedFlux=1-e if accepted else 1,usedResidual=[e,-e] if accepted else [0,0],fallback=not accepted,reason='nonfinite' if not valid else 'unit contract' if mode=='wrong-units' else 'accepted' if accepted else 'full residual')
 raise ValueError('Unknown mechanism')
def verify(q):
 name=q['kind'];args=q['args']
 if name not in DOMAINS or len(args)!=len(DOMAINS[name]) or any(a not in d for a,d in zip(args,DOMAINS[name])):raise ValueError('Domain')
 compare(q,answer(name,args))
 if name=='jacobi':
  for row in q['history']:close(row['residualNorm']**2,sum(x*x for x in row['residual']))
  close(q['residualNorm']**2,sum(x*x for x in q['residual']))

def scene(q):
 """Independent expected displayed geometry from qualified quantities."""
 name=q['kind'];a=q['args'];boxes=[];arrows=[];spheres=[]
 def box(o,size):boxes.append((o,size))
 def arrow(o,v,label):
  if sum(x*x for x in v)**.5>=1e-8:arrows.append((o,v,label))
 def sphere(o,r):spheres.append((o,r))
 def bar(x,v,scale):
  if v==0:sphere([x,-.4,0],.012)
  else:box([x,-.4+v*scale/2,0],[.12,abs(v)*scale,.2])
 if name=='wallload':
  for y in [-.5,.5]:box([0,y,0],[1,.04,.5])
  for s in q['samples']:arrow([-.3,s['y']-.5,.1],[s['velocity']*.18,0,0],'velocity')
  for y,t in [[-.5,q['bottom']],[.5,q['top']]]:arrow([.25,y,.25],[t*.1,0,0],'traction')
 elif name=='jacobi':
  for i in range(2):box([(i-.5)*.6,0,0],[.35,.5,.35]);arrow([(i-.5)*.6,.1,.25],[q['residual'][i]*.06,0,0],'residual')
 elif name=='memory':L=a[0]*.035;box([0,0,0],[L,L,L])
 elif name=='roundoff':bar(-.25,1,.5);bar(.25,q['total'],.5)
 elif name=='particletransfer':
  for i in range(2):
   x=(i-.5)*.7;sphere([x,0,0],.05*q['masses'][i]**(1/3));arrow([x,.22,.1],[q['input'][i]*.18,0,0],'input');arrow([x,-.22,.1],[q['output'][i]*.18,0,0],'output')
 elif name=='sstprobe':bar(-.3,q['viscosity'],.12);bar(0,q['raw'],.012);bar(.3,q['limited'],.12)
 elif name=='refinement':
  for s in q['samples']:
   h=s['value']*.65;box([.9*(s['y']-.5),h/2-.4,0],[.9/a[0],h,.15]);sphere([.9*(s['y']-.5),.65*s['y']**2-.4,.14],.015)
 elif name=='proposal':
  for x in [-.35,.35]:box([x,0,0],[.35,.5,.35])
  if q['candidateFlux'] is not None:arrow([-.1,.18,.25],[q['candidateFlux']*.28,0,0],'candidate')
  arrow([-.1,-.18,.25],[q['usedFlux']*.28,0,0],'used')
 return boxes,arrows,spheres
