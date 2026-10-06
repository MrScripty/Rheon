"""Render actual accepted native geometry, velocity and changing nodal masses."""
import io,json,os
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
from pathlib import Path
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from PIL import Image
ROOT=Path(__file__).resolve().parent;data=json.loads((ROOT/'native-qualification/debug-medium.json').read_text());states=[data['states'][0]]+[e['state']for e in data['states'][1:]];tri=np.array(data['triangles']);ids=np.array(data['nodes'])[:,2].astype(int);frames=[]
for index,s in enumerate(states):
    p=np.array(s['physical_nodes']);v=np.array(s['velocity']);fig,axes=plt.subplots(1,2,figsize=(10,4.2),layout='constrained');c=axes[0].tripcolor(p[:,0],p[:,1],tri,v[ids,2],shading='gouraud',vmin=0,vmax=2.25,cmap='viridis');fig.colorbar(c,ax=axes[0],label='third velocity w (m/s)');axes[0].triplot(p[:,0],p[:,1],tri,color='white',alpha=.22,lw=.45);axes[0].plot(p[5:10,0],p[5:10,1],color='#c2410c',lw=2);axes[0].plot(p[:5,0],p[:5,1],color='#1f2937',lw=2);axes[0].set(xlim=(-.04,1.18),ylim=(-.04,1.6),xlabel='physical x (m)',ylabel='height y (m)',title='Actual bottom-fixed ALE mesh and velocity');m=np.array(s['mass']);old=np.array(states[0]['mass']);axes[1].bar(np.arange(32),m-old,color=np.where(m>=old,'#2563eb','#c2410c'));axes[1].axhline(0,color='black',lw=.6);axes[1].set(ylim=(-.036,.036),xlabel='periodic nodal dual cell',ylabel='mass change since accepted initial state (kg)',title='Conservative liquid redistribution');axes[1].text(.03,.95,f"total mass = {sum(m):.15g} kg\nrelative pressure = 0 (analytic)",transform=axes[1].transAxes,va='top',fontsize=9);fig.suptitle(f"Native nonzero relative transport + viscosity · t={s['time']:.3f}s · version {s['version']}");buf=io.BytesIO();fig.savefig(buf,format='png',dpi=100);buf.seek(0);frames.append(Image.open(buf).convert('RGB').copy())
    if index==len(states)-1:fig.savefig(ROOT/'accepted-final.png',dpi=150);fig.savefig(ROOT/'accepted-final.pdf')
    plt.close(fig)
frames[0].save(ROOT/'accepted-replay.gif',save_all=True,append_images=frames[1:],duration=100,loop=0)
receipt=json.loads((ROOT/'native-qualification/receipt.json').read_text());rows=[r for r in receipt['summaries']if r['profile']=='debug'];fig,ax=plt.subplots(figsize=(6,4),layout='constrained');ax.loglog([r['dt']for r in rows],[r['temporal_L2_error']for r in rows],'o-',label='native vs time-dependent ALE ODE');ax.grid(True,which='both',alpha=.3);ax.set(xlabel='nominal dt (s); actual final interval recorded',ylabel='mass L² temporal error',title='Native ALE first-order time refinement');ax.legend();fig.savefig(ROOT/'temporal-convergence.png',dpi=150);fig.savefig(ROOT/'temporal-convergence.pdf');plt.close(fig)
print('Rendered 21 actual native accepted states: changing geometry, velocity and masses; no manufactured frames or continuum accuracy claim')
