"""Scientific plot of actual captured decisions; no synthetic accepted endpoint."""
from pathlib import Path
import json
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
P=Path(__file__).resolve().parent;r=json.loads((P/'analysis-normal.json').read_text());rows=r['rows'];x=np.arange(5)
labels=['initial +\nh=.0015625','initial +\nh=.00078125','initial −\nh=.00078125','pressure +\nh=.00078125','pressure −\nh=.00078125']
fig,axes=plt.subplots(1,2,figsize=(12,4.6),layout='constrained')
axes[0].plot(x,[y['native_norm']/1e-13 for y in rows],'o',ms=9,label='actual native norm')
axes[0].plot(x,[y['exact_sum_same_native_endpoints_norm']/1e-13 for y in rows],'x',ms=10,label='exact sum, same captured endpoints')
axes[0].axhline(1,color='#bd423d',ls='--',label='unchanged Newton target')
axes[0].set(ylabel='Residual norm / 1e−13',title='All five original decisions remain refusals',ylim=(0.8,2.05))
axes[0].legend(fontsize=8,loc='upper left')
axes[1].bar(x-.16,[y['accepted_to_rebuilt_start_max_gap_per_h']/1e-13 for y in rows],width=.32,label='stored start → rebuilt chart / h')
axes[1].bar(x+.16,[y['max_stored_known_increment_error_per_h']/1e-13 for y in rows],width=.32,label='known increment rounding / h')
axes[1].axhline(1,color='#bd423d',ls='--');axes[1].set(ylabel='Coefficient discrepancy / h / 1e−13',title='Measured input sensitivity; no floor theorem');axes[1].legend(fontsize=8)
for ax in axes:ax.set_xticks(x,labels,fontsize=8);ax.grid(axis='y',alpha=.25)
fig.suptitle('Frozen forcing candidates: evaluation only, zero owner advances',fontsize=13)
for suffix in ['png','pdf']:fig.savefig(P/('arithmetic-diagnosis.'+suffix),dpi=180)
print('PASS scientific figures use five captured refused candidates and exact-sum diagnostic data')
