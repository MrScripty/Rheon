"""Render captured endpoints, original bands and native margins only."""
from pathlib import Path
import json,os,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-e1-trajectories-matplotlib');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-e1-trajectories-cache')
import matplotlib
matplotlib.use('Agg');matplotlib.rcParams['svg.hashsalt']='rheon-e1-original-trajectories-v2'
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;n=json.loads((P/'native-outcome.json').read_text());c=json.loads((P/'convergence-outcome.json').read_text())
fig,axes=plt.subplots(2,2,figsize=(13,9),constrained_layout=True)
for f in c['families']:
 if f['field']!='nonconstant':continue
 label=f["kind"]+' '+f['load'];complete=[x for x in f['errors']if x['status']=='COMPLETE'];axes[0,0].loglog([x['h']for x in complete],[x['full_velocity_lumped_L2_error']for x in complete],'o-',label=label)
 checks=[x for x in f['checks']if x['quantity']=='geometry_max_error'and 'ratio'in x];axes[0,1].semilogx([x['finer_h']for x in checks],[x['ratio']for x in checks],'o-',label=label)
axes[0,0].set(title='Actual final-time velocity error (nonconstant third)',xlabel='Original step size h',ylabel='Lumped full velocity L2 error');axes[0,0].legend(fontsize=8);axes[0,0].grid(alpha=.3)
axes[0,0].text(.03,.48,'h=0.00078125 refusals:\ninitial forward at step 2;\npressure reversed at step 26.\nNo final-time errors for these prefixes.',transform=axes[0,0].transAxes,fontsize=8,bbox=dict(facecolor='white',alpha=.8,edgecolor='gray'))
axes[0,1].axhspan(1.7,2.3,color='green',alpha=.12,label='Original strict geometry band');axes[0,1].axvline(.003125,color='gray',linestyle=':',label='Original finest coarse h');axes[0,1].set(title='Geometry ratios retain original band failures',xlabel='Finer step size of halving pair',ylabel='Coarser error / finer error');axes[0,1].legend(fontsize=7);axes[0,1].grid(alpha=.3)
ordinary=[x for x in n['metrics']if x['mode']=='trajectory'];axes[1,0].semilogy(range(len(ordinary)),[x['finite_full_norm']for x in ordinary],'.',label='Finite full momentum norm');axes[1,0].semilogy(range(len(ordinary)),[x['direct_full_norm']for x in ordinary],'.',alpha=.5,label='Direct full momentum norm');axes[1,0].axhline(1e-11,color='darkred',linestyle='--',label='Original native physical gate');axes[1,0].set(title='All captured accepted steps: original native gates',xlabel='Accepted step in frozen roster order',ylabel='Reported combined momentum rate norm');axes[1,0].legend(fontsize=7);axes[1,0].grid(alpha=.3)
for index in [3,8,23,28]:
 pubs=[json.loads(l)for l in(P/f'trajectory-{index}-native.log').read_text().splitlines()if l.startswith('{')and '"model"'in l];axes[1,1].plot([x['time']for x in pubs],[x['end_q'][2]-1 for x in pubs],'o-',markersize=3,label=pubs[0]['kind']+' '+pubs[0]['load'])
axes[1,1].set(title='Published geometry over actual repeated clocks',xlabel='Actual accepted time',ylabel='Right cap displacement from original height');axes[1,1].legend(fontsize=7);axes[1,1].grid(alpha=.3)
fig.suptitle(f"E1 same-owner research trajectories: {n['ordinary_completed']}/48 complete; {n['actual_repeated_cancellations']} preserved interruptions\nIndependent replay and original temporal bands are separately qualified",fontsize=12)
output=Path(sys.argv[1])if len(sys.argv)>1 else P/'replay.svg';fig.savefig(output,metadata={'Date':None,'Creator':'Rheon captured trajectory evidence renderer'}if output.suffix=='.svg'else None)
