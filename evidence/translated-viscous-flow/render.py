"""Render recorded native accepted states; no synthesized simulation frames."""
import io,json,os
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
from pathlib import Path
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from PIL import Image
ROOT=Path(__file__).resolve().parent
packet=json.loads((ROOT/'native-qualification/debug-medium.json').read_text());states=[packet['states'][0]]+[x['state']for x in packet['states'][1:]];tri=np.array(packet['triangles']);ids=np.array(packet['nodes'])[:,2].astype(int);frames=[]
for index,s in enumerate(states):
    p=np.array(s['physical_nodes']);u=np.array(s['velocity']);fig,ax=plt.subplots(figsize=(7,4.4),layout='constrained')
    color=ax.tripcolor(p[:,0],p[:,1],tri,u[ids,2],shading='gouraud',vmin=0,vmax=2.25,cmap='viridis');fig.colorbar(color,ax=ax,label='third velocity w (m/s)')
    ax.plot(p[5:10,0],p[5:10,1],color='#c2410c',lw=2,label='accepted material cap')
    ax.plot(p[:5,0],p[:5,1],color='#374151',lw=2);ax.set(xlim=(-.04,1.18),ylim=(-.08,1.6),xlabel='physical x (m)',ylabel='height y (m)',title=f"Native translating viscous family · t={s['time']:.3f}s\nuₓ=0.25 m/s · relative pressure=0 · version {s['version']}")
    ax.legend(loc='lower right');buf=io.BytesIO();fig.savefig(buf,format='png',dpi=110);buf.seek(0);frames.append(Image.open(buf).convert('RGB').copy())
    if index==len(states)-1:fig.savefig(ROOT/'accepted-final.png',dpi=150);fig.savefig(ROOT/'accepted-final.pdf')
    plt.close(fig)
frames[0].save(ROOT/'accepted-replay.gif',save_all=True,append_images=frames[1:],duration=100,loop=0)
rows=json.loads((ROOT/'native-qualification/receipt.json').read_text())['summaries'];r=[s for s in rows if s['profile']=='debug'];fig,ax=plt.subplots(figsize=(5.5,4),layout='constrained');ax.loglog([x['dt']for x in r],[x['temporal_L2_error']for x in r],'o-',label='native vs semidiscrete exponential');ax.set(xlabel='dt (s)',ylabel='mass L² temporal error',title='Fixed spatial mesh: first-order temporal behavior');ax.grid(True,which='both',alpha=.3);ax.legend();fig.savefig(ROOT/'temporal-convergence.png',dpi=150);fig.savefig(ROOT/'temporal-convergence.pdf');plt.close(fig)
print('Rendered 21 recorded native states, final PNG/PDF and temporal study; no continuum accuracy claim')
