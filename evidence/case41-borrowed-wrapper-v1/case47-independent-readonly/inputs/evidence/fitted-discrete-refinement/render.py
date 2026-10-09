"""Render only completed accepted host replays; no native/continuum claim."""
import io,json,os
from pathlib import Path
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl')
os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from PIL import Image
import temporal
P=Path(__file__).resolve().parent
out=P/'renders';out.mkdir(exist_ok=True)
records=[json.loads((P/('trials/repaired-'+label+'-trajectory.json')).read_text())for label in ['initial','pressure']]
for record in records:
 if record['status']!='PASS' or len(record['rows'])!=5:raise ValueError('completed replay required')
fig,axes=plt.subplots(2,2,figsize=(10,7),layout='constrained')
for r,label in zip(records,['initial shear','pressure state']):
 rows=r['rows'];h=[v['interval']for v in rows]
 axes[0,0].loglog(h,[v['velocity_lumped_L2_error']for v in rows],'o-',label=label)
 axes[0,1].loglog(h,[v['geometry_max_error']for v in rows],'o-',label=label)
 axes[1,0].plot(h,[v['max_work_allowance_ratio']for v in rows],'o-',label=label+' work')
 axes[1,0].plot(h,[v['max_GCL_allowance_ratio']for v in rows],'x--',label=label+' GCL')
 for j,name in enumerate(['BE','mixing','viscous']):axes[1,1].loglog(h,[v['loss_sums'][j]for v in rows],'o-',label=label+' '+name)
for a,title in zip(axes.flat,['Velocity error vs same spatial DAE','Geometry error vs same spatial DAE','Actual defect / unchanged allowance','Separate accumulated discrete losses']):
 a.set_title(title,fontsize=10);a.set_xlabel('step interval h');a.grid(alpha=.3);a.legend(fontsize=7)
axes[1,0].axhline(1,color='red',ls=':');axes[1,0].set_ylim(0,1.05)
fig.suptitle('Accepted HOST endpoint-donor replay: first-order temporal qualification')
fig.savefig(out/'refinement.png',dpi=160);fig.savefig(out/'refinement.pdf');plt.close(fig)
frames=[];physical=[]
for i in range(32):
 fig,axes=plt.subplots(1,2,figsize=(9,3.8),layout='constrained')
 row=[]
 for a,r,label in zip(axes,records,['initial shear','pressure state']):
  c=r['replays'][-1]['steps'][i]
  v=temporal.point(np.array(c['initial_q']),np.array(c['initial_eta']),np.array(c['unknowns']),c['interval'])
  s=v['spatial'];pos=s['pos'];vel=np.einsum('ndj,j->nd',temporal.m.R_NODE,v['z'])[:,:2];tri=temporal.m.TRI
  a.triplot(pos[:,0],pos[:,1],tri,lw=.45,color='#8ca1b2')
  rawvel=vel[temporal.m.IDS]
  a.quiver(pos[:,0],pos[:,1],rawvel[:,0],rawvel[:,1],color='#176ea0',angles='xy',scale_units='xy',scale=5,width=.003)
  q=c['end_q'];cap=np.array([[q[0],q[2]],[q[1],2.25-q[2]],[q[0]+1,q[2]]])
  a.plot(cap[:,0],cap[:,1],color='#ab4b16',lw=2);a.set(xlim=(-.03,1.16),ylim=(-.03,1.35),aspect='equal',title=label)
  row.append(dict(label=label,stamp=c['step'],q=c['end_q'],pressure=c['end_pressure'],energy=c['energy_new']))
 fig.suptitle(f'HOST accepted endpoints • h=.003125 • stamp {i+1} • t={(i+1)*.003125:.6f}')
 buf=io.BytesIO();fig.savefig(buf,format='png',dpi=100);buf.seek(0);frames.append(Image.open(buf).convert('RGB'));plt.close(fig);physical.append(row)
frames[-1].save(out/'accepted-endpoints.png');frames[0].save(out/'accepted-endpoints.gif',save_all=True,append_images=frames[1:],duration=120,loop=0)
(out/'receipt.json').write_text(json.dumps(dict(scope='32 actual accepted host endpoints per field; no native or continuum claim',source_inputs=['trials/repaired-initial-trajectory.json','trials/repaired-pressure-trajectory.json'],frames=physical),indent=2)+'\n')
print('PASS actual 64 accepted endpoint render records and temporal/loss/gate plots')
