"""Render actual cut-basis weights and native traction-free decay evidence."""
from pathlib import Path
import csv
import sys
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.ticker import NullFormatter
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from verify_column_shear import verify
HERE=Path(__file__).resolve().parent
results=verify(HERE/'demo');lookup={r['case']:r for r in results['results']}
plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
fig,axes=plt.subplots(2,3,figsize=(14,8.5),layout='constrained')
ax=axes[0,0];y=np.linspace(0,2.25,300);n0=np.maximum(0,np.minimum(1,1.5-y));n1=1-n0
ax.plot(y,n0,label='N₀: integrated length = h');ax.plot(y,n1,label='N₁: integrated length = 1.25h');ax.axvspan(2,2.25,alpha=.15,color='#2675a2',label='partial cap');ax.axvline(2.25,color='#2675a2',ls='--');ax.set(xlabel='normal position y/h',ylabel='basis value',title='Liquid-only row-sum mass (top fraction ¼)');ax.legend(fontsize=9)
ax=axes[0,1];x=np.arange(3);ax.bar(x-.18,[1,1.25,0],width=.35,label='fraction ¼: two wet nodes');ax.bar(x+.18,[1,1,.75],width=.35,label='fraction ¾: three wet nodes');ax.set(xticks=x,xticklabels=['0','1','2'],xlabel='wet-node layer index',ylabel='dual liquid length / h',title='Center classification changes support');ax.legend(fontsize=9)
ax=axes[0,2];case=HERE/'demo/decay-a1-n16-f0.75';rows=list(csv.DictReader((case/'profiles.csv').open()));first=[r for r in rows if r['step']=='1'];last_step=rows[-1]['step'];last=[r for r in rows if r['step']==last_step];ys=[float(r['position']) for r in first]
ax.plot([float(r['before0']) for r in first],ys,'o-',label='Initial native samples');ax.plot([float(r['after0']) for r in last],ys,'o-',label='Native at 0.1 s');ax.plot(np.cos(np.pi*np.array(ys))*np.exp(-.05*.1*np.pi**2),ys,'--',label='Continuum decay');ax.axhline(1,ls=':',color='#2675a2');ax.set(xlabel='tangential velocity (m/s)',ylabel='normal position (m)',title='Fixed flat surface, periodic lateral patch');ax.legend(fontsize=9)
ax=axes[1,0];ns=[8,16,32,64]
for fraction in [.25,.5,.75]:ax.loglog(ns,[lookup[f'decay-a1-n{n}-f{fraction}']['continuum_rms'] for n in ns],'o-',label=f'top fraction {fraction}')
ax.set(xlabel='full layer count',ylabel='native liquid-weighted RMS error (m/s)',title='Spatial refinement vs analytic shear');ax.set_xticks(ns,[str(n) for n in ns]);ax.xaxis.set_minor_formatter(NullFormatter());ax.legend()
ax=axes[1,1];steps=[64,128,256];ax.loglog(steps,[lookup[f'time-a1-n12-f0.75-s{s}']['native_time_error'] for s in steps],'o-',label='Native vs exact discrete exponential');ax.loglog(steps,[lookup[f'time-a1-n12-f0.75-s{s}']['time_error'] for s in steps],'--',label='Double explicit reference');ax.set(xlabel='step count at fixed geometry',ylabel='liquid-weighted time error (m/s)',title='Separate time refinement');ax.set_xticks(steps,[str(s) for s in steps]);ax.xaxis.set_minor_formatter(NullFormatter());ax.legend(fontsize=9)
ax=axes[1,2];history=list(csv.DictReader((HERE/'demo/decay-a1-n32-f0.75/steps.csv').open()));e0=float(history[0]['kinetic_before']);ax.plot([int(r['step'])*float(r['dt']) for r in history],[float(r['kinetic_after'])/e0 for r in history],label='Actual native kinetic energy');ax.set(xlabel='time (s)',ylabel='energy / initial energy',title='Measured viscosity-only dissipation');ax.legend(fontsize=9)
fig.suptitle('Free-surface shear quadrature prerequisite\nFlat fixed geometry, constant density, zero endpoint shear traction; no coupled carrier step',fontsize=15)
fig.savefig(HERE/'cut-mass-and-shear-decay.png',dpi=170)
