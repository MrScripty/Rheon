"""Recorded host research endpoints and separate first-order losses, no native claim."""
import json
import sys
from pathlib import Path
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.collections import PolyCollection
from PIL import Image
import io
import step
m=step.m
root=Path(__file__).parent
study=json.load(open(sys.argv[1]));rows=study['rows'];cell=json.load(open(sys.argv[2]))
fig,axes=plt.subplots(1,3,figsize=(15,4.8),layout='constrained')
fig.suptitle('Coupled first-order endpoint-donor HOST research: three completed cases pass work gates; .00625 refinement rejected; no public step')
h=np.array([r['interval']for r in rows]);err=np.array([r['velocity_lumped_L2_error']for r in rows])
axes[0].loglog(h,err,'o-',label='velocity vs refined coupled ODE');axes[0].loglog(h,err[-1]*h/h[-1],'--',label='first-order guide');axes[0].set(xlabel='interval [s]',ylabel='lumped velocity error',title='Same spatial model, time refinement');axes[0].legend();axes[0].grid(True,which='both',alpha=.3)
loss=np.array([r['loss_sums']for r in rows]);bottom=np.zeros(len(rows))
for k,label in enumerate(['backward-Euler increment','donor mixing','physical viscosity']):axes[1].bar(range(len(rows)),loss[:,k],bottom=bottom,label=label);bottom+=loss[:,k]
axes[1].set_xticks(range(len(rows)),[str(v)for v in h]);axes[1].set(xlabel='interval [s]',ylabel='dissipation to t=.1 [J]',title='Separate work identity terms');axes[1].legend()
axes[2].semilogx(h,[r['max_work_allowance_ratio']for r in rows],'o-',label='weighted momentum work');axes[2].semilogx(h,[r['max_GCL_allowance_ratio']for r in rows],'s-',label='local mass balance');axes[2].axhline(1,color='red',ls='--',label='fixed acceptance gate');axes[2].set(xlabel='interval [s]',ylabel='largest ratio to existing allowance',title='Actual repeated-step gates');axes[2].legend();axes[2].grid(alpha=.3)
fig.savefig(root/'discrete-work-refinement.png',dpi=150);fig.savefig(root/'discrete-work-refinement.pdf');plt.close(fig)
frames=[]
for recorded in cell['samples']:
 nodes=np.array(recorded['nodes']);velocity=np.array(recorded['velocity']);speed=np.linalg.norm(velocity[m.IDS[m.TRI]],axis=2).mean(axis=1)
 fig,ax=plt.subplots(figsize=(7,5),layout='constrained')
 collection=PolyCollection(nodes[m.TRI],array=speed,cmap='viridis',edgecolors='#334155',linewidths=.6);collection.set_clim(0,1.3);ax.add_collection(collection)
 ax.quiver(nodes[[3,4,5],0],nodes[[3,4,5],1],velocity[m.IDS[[3,4,5]],0],velocity[m.IDS[[3,4,5]],1],scale=15,color='black')
 ax.set(xlim=(-.03,1.16),ylim=(-.03,1.36),xlabel='x [m]',ylabel='y [m]',title=f"Single-cell material candidate path · s={recorded['time']:.5f}\nEndpoint discrete work passes; continuous momentum fails; no public step")
 fig.colorbar(collection,ax=ax,label='recorded candidate speed [m/s]')
 buf=io.BytesIO();fig.savefig(buf,format='png',dpi=100);plt.close(fig);buf.seek(0);frames.append(Image.open(buf).convert('RGB'))
frames[0].save(root/'host-discrete-work-replay.gif',save_all=True,append_images=frames[1:],duration=350,loop=0)
print('rendered',len(frames),'recorded single-cell candidate snapshots; three completed host refinement cases and fourth rejection; no new physics solve')
