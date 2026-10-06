"""Render five captured traces and physical-gate reports, without simulation."""
from pathlib import Path
import json,os,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-generalization-matplotlib')
os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-generalization-cache')
import matplotlib
matplotlib.use('Agg');matplotlib.rcParams['svg.hashsalt']='rheon-five-refusals-v2'
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;r=json.loads((P/'outcome.json').read_text())
fig,axes=plt.subplots(1,2,figsize=(12,4.8),constrained_layout=True)
labels=['Initial forward h=1/640','Initial forward h=1/1280','Initial reversed h=1/1280','Pressure forward control','Pressure reversed prefix 1']
for c,label in zip(r['cases'],labels):
 trace=c['main_checks'];axes[0].semilogy([x['calls'] for x in trace],[x['rate_norm'] for x in trace],'o-',label=label,markersize=4)
axes[0].axhline(1e-13,color='black',linestyle='--',label='Newton target: 1e-13')
axes[0].set(title='Five once-only calls: all converged',xlabel='Counted Newton equation evaluations',ylabel='Recorded main Newton rate norm');axes[0].grid(alpha=.3);axes[0].legend(fontsize=7)
for key,label,marker in [('finite_momentum_rate_norm','Planar finite','o'),('direct_momentum_rate_norm','Planar direct','s')]:
 axes[1].semilogy(range(5),[c['report']['step']['planar'][key] for c in r['cases']],marker+'-',label=label)
for key,label,marker in [('finite_momentum_rate_norm','Third finite','^'),('direct_momentum_rate_norm','Third direct','v')]:
 axes[1].semilogy(range(5),[c['report']['step']['third'][key] for c in r['cases']],marker+'-',label=label)
axes[1].axhline(1e-11,color='darkred',linestyle='--',label='Original physical gate: 1e-11')
axes[1].axhline(1e-13,color='black',linestyle=':',label='Newton target (reference only)')
axes[1].set(title='Final planar and third-component reports',xlabel='Frozen case index',ylabel='Reported momentum rate norm');axes[1].set_xticks(range(5));axes[1].grid(alpha=.3);axes[1].legend(fontsize=7)
output=Path(sys.argv[1]) if len(sys.argv)>1 else P/'replay.svg';metadata={'Date':None,'Creator':'Rheon fixed-roster evidence renderer'} if output.suffix=='.svg' else None
fig.savefig(output,metadata=metadata)
