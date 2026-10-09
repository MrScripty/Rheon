"""Plot analytic references, explicitly not native advancing/render evidence."""
from pathlib import Path
import math
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

here=Path(__file__).resolve().parent
x=[0,.25,.5,.75,1]
h=[1,1.25,1,1.5,1]
fig,axes=plt.subplots(1,2,figsize=(11,3.7))
axes[0].plot(x,h,'o-',label='H(x,0)')
# Periodic horizontal translation: analytic graph, not computed time stepping.
shift=.125
pairs=sorted([((v+shift)%1,w) for v,w in zip(x[:-1],h[:-1])])
lastx,lasth=pairs[-1]
firstx,firsth=pairs[0]
y0=lasth+(firsth-lasth)*(1-lastx)/(firstx+1-lastx)
axes[0].plot([0]+[v for v,_ in pairs]+[1],[y0]+[w for _,w in pairs]+[y0],'o--',label='analytic translation by 1/8')
axes[0].set(xlabel='periodic x',ylabel='height H',ylim=(0,1.7),title='Varying graph: prescribed analytic motion')
axes[0].legend(fontsize=8)
a=.1;mu=.5;k=math.pi;c=-k*k/(6+k*k)
xx=[i/100 for i in range(101)]
pressure=[-2*mu*a*k*math.cos(k*v)*(1+3*c) for v in xx]
normal=[-a*k*math.cos(k*v)*(1+c) for v in xx]
axes[1].plot(xx,pressure,label='required relative pressure at flat cap')
axes[1].plot(xx,normal,label='instantaneous normal velocity')
axes[1].set(xlabel='x',title='Manufactured instantaneous traction benchmark')
axes[1].legend(fontsize=8)
fig.suptitle('Research references only — no native advancing solver or temporal replay',fontsize=11)
fig.tight_layout()
fig.savefig(here/'analytic-reference.png',dpi=160)
