"""Render captured numerical evidence only; no simulation or reconstruction."""
from pathlib import Path
import json,os,struct,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-public-call-66k-matplotlib')
os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-public-call-66k-cache')
import matplotlib
matplotlib.use('Agg')
matplotlib.rcParams['svg.hashsalt']='rheon-fixed-public-call-66k'
import matplotlib.pyplot as plt
import matplotlib.tri as mtri
P=Path(__file__).resolve().parent
r=json.loads((P/'outcome.json').read_text());s=r['accepted_snapshot'];checks=r['main_newton_checks']
def f64(bits):return struct.unpack('<d',struct.pack('<Q',bits))[0]
points=[[f64(b) for b in p] for p in s['positions']]
x,y=zip(*points);third=[f64(s['velocity'][i][2]) for i in s['periodic']]
fig,ax=plt.subplots(1,2,figsize=(11,4),constrained_layout=True)
ax[0].semilogy([c['calls'] for c in checks],[c['rate_norm'] for c in checks],'o-',label='Recorded main Newton checks')
ax[0].axhline(1e-13,color='darkred',linestyle='--',label='Unchanged 1e-13 threshold')
ax[0].set(xlabel='Equation evaluations',ylabel='Momentum rate norm',title='One fixed call: 3 corrections, 4 checks')
ax[0].set_xticks([1,8,15,22]);ax[0].grid(alpha=.3);ax[0].legend(fontsize=8)
ax[1].triplot(mtri.Triangulation(x,y,s['triangles']),color='.6',linewidth=.7)
sc=ax[1].scatter(x,y,c=third,cmap='coolwarm',s=25,zorder=3)
fig.colorbar(sc,ax=ax[1],label='Stored third velocity component')
ax[1].set(xlabel='x',ylabel='y',title='Accepted 2.5D mesh: version 2, t=0.0015625',aspect='equal')
output=Path(sys.argv[1]) if len(sys.argv)>1 else P/'replay.svg'
metadata={'Date':None,'Creator':'Rheon captured evidence renderer'} if output.suffix=='.svg' else None
fig.savefig(output,metadata=metadata)
