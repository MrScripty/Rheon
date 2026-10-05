"""Plot verified native static publications and instantaneous transfer refinement."""
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
ROOT=Path(__file__).resolve().parent
pulse=json.loads((ROOT/'demo/pulse-y-0.25-jacobi/case.json').read_text())
summary=json.loads((ROOT/'numerical-results.json').read_text())
fig,axes=plt.subplots(1,3,figsize=(13,3.8),layout='constrained')
steps=pulse['steps'];x=np.arange(1,len(steps)+1)
axes[0].plot(x,[s['publication']['transfer']['kinetic_before'] for s in steps],'o-',label='Prescribed parcels')
axes[0].plot(x,[s['publication']['transfer']['kinetic_after'] for s in steps],'o-',label='Lifted MAC')
axes[0].plot(x,[s['publication']['projection']['kinetic_after'] for s in steps],'o-',label='Accepted MAC')
axes[0].set(xlabel='Static publication number (time = 0)',ylabel='Declared kinetic energy (J)',title='Pulse: prescription, walls, averaging, pressure')
axes[0].set_yscale('log');axes[0].legend(fontsize=8)
rows=[r for r in summary['results'] if r['name'].startswith('curl')];n=np.array([8,16,32,64]);err=np.array([r['curl_weighted_rms'] for r in rows])
axes[1].loglog(n,err,'o-',label='Accepted stored-f32 field')
axes[1].loglog(n,err[0]*(8/n)**2,'--',label='n⁻² reference')
axes[1].set(xlabel='Tangential cells per side',ylabel='Mass-weighted velocity RMS (m/s)',title='Instantaneous circulation transfer')
axes[1].legend(fontsize=8)
axes[2].loglog(n,[r['max_stored_divergence'] for r in rows],'o-',label='Actual stored-f32 divergence')
axes[2].axhline(1e-5,color='black',ls='--',label='Unchanged acceptance limit')
axes[2].set(xlabel='Tangential cells per side',ylabel='Maximum divergence (1/s)',title='Derivative roundoff grows with resolution')
axes[2].legend(fontsize=8)
for ax in axes:ax.grid(True,alpha=.25)
fig.suptitle('Fixed flat columns: static owner materialization, no moving-interface dynamics',fontsize=12)
fig.savefig(ROOT/'static-transfer-and-projection.png',dpi=160)
