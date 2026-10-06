"""Component errors and withheld predictions, without changing the max-norm gate."""
import json,os
from pathlib import Path
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-forcing-diagnosis-plots');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-forcing-diagnosis-cache')
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;d=json.loads((P/'first-reference-diagnosis.json').read_text());probe=json.loads((P/'first-probe-replay.json').read_text());fig,axes=plt.subplots(2,2,figsize=(11,8),constrained_layout=True)
for i,kind in enumerate(['initial','pressure_state']):
 row=next(x for x in d['rows']if x['kind']==kind and x['load']=='reversed');h=np.array([x['h']for x in row['errors']]);e=np.array([x['error_components']for x in row['errors']]);c=np.array(row['component_fit']['coefficients_c1_c2_c3']);dense=np.geomspace(.0015625,.05,100)
 ax=axes[i,0]
 for j,label in enumerate(['x0','H0','x1','H1']):ax.plot(h,e[:,j]/h,'o-',label=label);ax.plot(dense,c[0,j]+dense*c[1,j]+dense*dense*c[2,j],':',alpha=.6)
 ax.set_xscale('log');ax.set_title(kind+': signed component error / h');ax.set_xlabel('Step interval h');ax.set_ylabel('(native − tighter reference) / h');ax.legend(ncol=2);ax.grid(alpha=.2)
 ax=axes[i,1];maximum=np.max(abs(e),axis=1);ax.loglog(h,maximum,'o-',label='Actual max error');ax.loglog(dense,np.max(abs(dense[:,None]*c[0]+dense[:,None]**2*c[1]+dense[:,None]**3*c[2]),axis=1),':',label='Fit to three finest original points')
 completed=[x for x in probe['rows']if x['kind']==kind and x['load']=='reversed'and x['probe_status']=='COMPLETE']
 for x in completed:ax.loglog(x['h'],x['geometry_max_error'],'s',label='Actual additional native endpoint')
 ax.set_title(kind+': unchanged geometry max norm');ax.set_xlabel('Step interval h');ax.set_ylabel('Geometry max error');ax.legend(fontsize=8);ax.grid(alpha=.2)
fig.suptitle('Reversed forcing: competing horizontal cap errors and coarse cancellation\nOriginal five-interval band remains failed; dotted fit is diagnostic, not a qualification',fontsize=12)
fig.savefig(P/'component-regime.png',dpi=165);fig.savefig(P/'component-regime.pdf',metadata={'CreationDate':None,'ModDate':None});plt.close(fig)
