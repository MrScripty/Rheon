"""Render actual native accepted geometry, velocity and physical pressure exports."""
import io,json,os
from pathlib import Path
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.tri as tri
import numpy as np
from PIL import Image
P=Path(__file__).resolve().parent
q=P/'native-qualification';out=P/'renders';out.mkdir(exist_ok=True)
rows=[json.loads(x)for x in(q/'native-default.jsonl').read_text().splitlines()]
result=json.loads((q/'normal-replay.json').read_text());
if result['status']!='PASS' or len(rows)!=124:raise ValueError('complete actual native qualification')
fig,axes=plt.subplots(1,3,figsize=(12,3.8),layout='constrained')
for kind,label in [('initial','initial shear'),('pressure_state','pressure state')]:
 rr=[r for r in result['rows']if r['kind']==kind];h=[r['interval']for r in rr]
 axes[0].loglog(h,[r['velocity_lumped_L2_error']for r in rr],'o-',label=label)
 axes[1].loglog(h,[r['geometry_max_error']for r in rr],'o-',label=label)
 for name,sym in [('discrete_residual_work','o'),('energy_ledger_error','x')]:
  ratios=[max(abs(r['report'][name])/r['report']['fixed_work_allowance']for r in rows if r['kind']==kind and r['h']==interval)for interval in h];axes[2].plot(h,ratios,sym+'-',label=label+' '+('work'if sym=='o'else'ledger'))
for a,title in zip(axes,['Native velocity vs same spatial DAE','Native geometry vs same spatial DAE','Native defect / unchanged allowance']):a.set_title(title,fontsize=9);a.set_xlabel('h');a.grid(alpha=.3);a.legend(fontsize=7)
axes[2].axhline(1,color='red',ls=':');axes[2].set_ylim(0,1.05)
fig.suptitle('Actual native endpoint-donor step • bounded xy slice • first-order temporal evidence');fig.savefig(out/'native-refinement.png',dpi=160);fig.savefig(out/'native-refinement.pdf');plt.close(fig)
# Actual native raw-node to periodic-velocity layout, frozen topology verified in replay.
ids=np.array([0,1,0,2,3,2,4,5,6,7,8,9,10,11,12,13,9,14,15])
fine={kind:[r for r in rows if r['kind']==kind and r['h']==.003125]for kind in ['initial','pressure_state']}
pressure_bound=max(np.max(np.abs(r['physical_pressure']))for rr in fine.values()for r in rr)
frames=[];records=[]
for i in range(32):
 fig,axes=plt.subplots(1,2,figsize=(9,3.8),layout='constrained');record=[]
 for a,kind in zip(axes,['initial','pressure_state']):
  r=fine[kind][i];pos=np.array(r['positions']);vel=np.array(r['velocity'])[ids];top=np.array(r['triangles']);pressure=np.array(r['physical_pressure'])
  mesh=tri.Triangulation(pos[:,0],pos[:,1],top);artist=a.tripcolor(mesh,facecolors=pressure,cmap='coolwarm',vmin=-pressure_bound,vmax=pressure_bound,edgecolors='#75818b',linewidth=.35)
  a.quiver(pos[:,0],pos[:,1],vel[:,0],vel[:,1],color='#24364a',angles='xy',scale_units='xy',scale=5,width=.003)
  a.set(xlim=(-.03,1.16),ylim=(-.03,1.35),aspect='equal',title=kind.replace('_',' '));record.append(dict(kind=kind,stamp=r['step'],time=r['time'],q=r['end_q'],physical_pressure=r['physical_pressure'],mass=r['mass']))
 fig.colorbar(artist,ax=list(axes),label='native relative pressure',shrink=.7);fig.suptitle(f'Native accepted frame + velocity + pressure • stamp {i+1} • t={fine["initial"][i]["time"]:.6f}')
 buf=io.BytesIO();fig.savefig(buf,format='png',dpi=100);buf.seek(0);frames.append(Image.open(buf).convert('RGB'));plt.close(fig);records.append(record)
frames[-1].save(out/'native-endpoints.png');frames[0].save(out/'native-endpoints.gif',save_all=True,append_images=frames[1:],duration=120,loop=0)
(out/'receipt.json').write_text(json.dumps(dict(scope='64 actual native accepted endpoint records, reconstructed surface not claimed; no continuum validation',source='native-qualification/native-default.jsonl',frames=records),indent=2)+'\n');print('PASS actual native accepted geometry/velocity/pressure render and temporal plots')
