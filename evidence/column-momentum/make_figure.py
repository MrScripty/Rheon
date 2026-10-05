import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
HERE=Path(__file__).resolve().parent
case=json.loads((HERE/'demo/exchange-y/case.json').read_text())
results=json.loads((HERE/'numerical-results.json').read_text())['results']
fig,axes=plt.subplots(1,3,figsize=(13,4),layout='constrained')
for index,(tag,step,key) in enumerate([('initial',case['steps'][0],'before'),('final',case['steps'][-1],'after')]):
    heights=step[key+'_heights'];fields=step[key]
    for c,height in enumerate(heights):
        wet=int(height)+(height%1>.5);y=[j+.5 if j+1<wet else .5*(j+height) for j in range(wet)]
        u=[fields[0][j+4*c] for j in range(wet)]
        axes[0].plot(u,y,'o-',label=f'{tag} column {c}')
axes[0].set(xlabel='tangential parcel velocity (m/s)',ylabel='dual slab midpoint (m)',title='Prescribed cap exchange')
axes[0].legend(fontsize=8)
steps=case['steps'];energy=[steps[0]['report']['kinetic_before']]+[s['report']['kinetic_after'] for s in steps]
axes[1].plot(range(len(energy)),energy,'o-',markersize=3);axes[1].set(xlabel='remap index (no physical clock)',ylabel='lumped kinetic energy (J)',title='Mixing loss; explicit cap ledgers close')
a=[r for r in results if r['name'].startswith('affine')];axes[2].loglog([8,16,32,64],[r['affine_weighted_rms'] for r in a],'o-',label='native f32 overlap remap')
axes[2].set(xlabel='full layers n; h=1/(n+1/4)',ylabel='liquid-weighted RMS error (m/s)',title='Affine parcel means; local cap change')
axes[2].legend(fontsize=8)
fig.suptitle('Column-profile remap prerequisite — no pressure or viscous traction update')
fig.savefig(HERE/'cap-exchange-and-remap-refinement.png',dpi=160)
