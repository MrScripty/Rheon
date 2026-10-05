"""Plot native velocity, isolated mode convergence and separately coupled decay."""
from pathlib import Path
import csv
import math
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.ticker import NullFormatter
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from verify_viscosity import verify,faces
HERE=Path(__file__).resolve().parent
result=verify(HERE/'demo');lookup={r['case']:r for r in result['results']}
plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
fig,axes=plt.subplots(2,3,figsize=(14,8.5),layout='constrained')
for ax,stem,title in [(axes[0,0],'initial','Initial manufactured velocity'),(axes[0,1],'final','After isolated viscosity, T = 0.1 s')]:
    v=faces(HERE/'demo/space-n16'/f'{stem}-faces.csv',[16,16,3]);xs=[];ys=[];us=[];vs=[]
    for j in range(0,16,2):
        for i in range(0,16,2):
            xs.append((i+.5)/16);ys.append((j+.5)/16)
            us.append(.5*(v[0,i,j,1]+v[0,i+1,j,1]));vs.append(.5*(v[1,i,j,1]+v[1,i,j+1,1]))
    ax.quiver(xs,ys,us,vs,scale=12,width=.008,color='#225b8b');ax.set(xlim=(0,1),ylim=(0,1),aspect='equal',xlabel='x (m)',ylabel='y (m)',title=title)
ax=axes[0,2]
for name,label in [('material-n16-nu0.01','ν = 0.01'),('space-n16','ν = 0.05'),('material-n16-nu0.1','ν = 0.1')]:
    rows=list(csv.DictReader((HERE/'demo'/name/'steps.csv').open()));e0=float(rows[0]['kinetic_before']);t=[float(r['step'])*float(r['dt']) for r in rows]
    ax.plot([0]+t,[1]+[float(r['kinetic_after'])/e0 for r in rows],label=label)
ax.set(xlabel='time (s)',ylabel='stage energy / initial energy',title='Declared kinematic coefficient (m²/s)');ax.legend()
ax=axes[1,0];n=[8,16,32,64];errors=[lookup[f'space-n{x}']['continuum_rms'] for x in n]
ax.loglog(n,errors,'o-',label='Native vs continuum');ax.loglog(n,[errors[0]*(8/x)**2 for x in n],'--',label='n⁻² reference')
limit=HERE/'refinement-limit/space-n64';v=faces(limit/'final-faces.csv',[64,64,3]);initial=faces(limit/'initial-faces.csv',[64,64,3]);exact=math.exp(-.05*.1*2*math.pi**2);error=math.sqrt(math.fsum((u-exact*initial[key])**2 for key,u in v.items())/len(v))
ax.scatter([64],[error],color='#ba4827',marker='x',s=70,label='2053 steps: failed modal oracle');ax.set(xlabel='cells per side n',ylabel='RMS velocity error (m/s)',title='Spatial study and retained f32 limit');ax.set_xticks(n,[str(x) for x in n]);ax.legend(fontsize=9)
ax=axes[1,1];steps=[32,64,128];errs=[lookup[f'time-n12-s{s}']['time_error'] for s in steps]
ax.loglog(steps,errs,'o-',label='Native vs discrete exponential');ax.loglog(steps,[errs[0]*32/s for s in steps],'--',label='steps⁻¹ reference');ax.set(xlabel='step count at fixed n = 12',ylabel='amplitude error',title='Time refinement');ax.set_xticks(steps,[str(s) for s in steps]);ax.legend(fontsize=9)
ax=axes[1,2]
for mu,color in [('0','#6c7a89'),('0.125','#225b8b')]:
    rows=list(csv.DictReader((HERE/'demo'/f'jacobi-pcg-v1-coupled-mu{mu}'/'liquid.csv').open()))
    ax.plot([float(r['time']) for r in rows],[float(r['accepted_energy']) for r in rows],'o-',label=f'μ = {mu} Pa s',color=color)
ax.set(xlabel='accepted coupled time (s)',ylabel='accepted kinetic energy (J)',title='2×2×1 filled box: advection also damps');ax.legend()
for ax in (axes[1,0],axes[1,1]): ax.xaxis.set_minor_formatter(NullFormatter())
fig.suptitle('Newtonian symmetric-strain update: sealed free-slip box\nAnalytic numerical benchmark; no free-surface or calibrated material claim',fontsize=15)
fig.savefig(HERE/'shear-decay-and-refinement.png',dpi=170)
