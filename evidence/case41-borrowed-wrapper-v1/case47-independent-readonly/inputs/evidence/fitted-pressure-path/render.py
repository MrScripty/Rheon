"""Render actual native pressure and rejected candidate path, never simulation."""
import io
import json
import os
from pathlib import Path
import sys
os.environ.setdefault('MPLCONFIGDIR', '/tmp/rheon-mpl')
os.environ.setdefault('XDG_CACHE_HOME', '/tmp/rheon-font-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation
import numpy as np
from PIL import Image

packet=Path(__file__).resolve().parent
payload=json.loads(Path(sys.argv[1]).read_text())
samples=payload['samples']
plt.rcParams.update({'font.size':10, 'figure.dpi':150})


def mesh_panel(ax,row,values,label,limit):
    pos=np.array(row['nodes'])[:,:2]
    tri=Triangulation(pos[:,0],pos[:,1],np.array(row['triangles']))
    colors=ax.tripcolor(tri,facecolors=values,edgecolors='#334155',linewidth=0.45,cmap='coolwarm',vmin=-limit,vmax=limit)
    ax.set_aspect('equal');ax.set_xlabel('x [m]');ax.set_ylabel('y [m]')
    ax.set_title(label);ax.set_xlim(-0.03,1.1);ax.set_ylim(-0.03,1.32)
    return colors


fig,axes=plt.subplots(1,3,figsize=(14,4.8),layout='constrained')
pmax=max(abs(x)for x in samples[0]['pressure_values'])
c=mesh_panel(axes[0],samples[0],samples[0]['pressure_values'],'Solved pressure on the force mesh',pmax)
fig.colorbar(c,ax=axes[0],label='relative pressure [Pa]')
dmax=max(abs(x)for x in samples[-1]['divergence'])
c=mesh_panel(axes[1],samples[-1],samples[-1]['divergence'],'Rejected material path endpoint',dmax)
fig.colorbar(c,ax=axes[1],label='actual divergence [1/s]')
axes[2].plot([s['path_parameter']for s in samples],[s['divergence_max']for s in samples],'o-',color='#b91c1c',label='measured max divergence')
axes[2].axhline(1e-11,color='#64748b',linestyle='--',label='existing divergence gate')
axes[2].set_xlabel('candidate path parameter [s]');axes[2].set_ylabel('actual max divergence [1/s]')
axes[2].grid(alpha=.25);axes[2].legend(loc='upper left',fontsize=8)
fig.suptitle('Nonuniform xy pressure solve passes; its straight material path fails\nTwo columns · full symmetric strain · no accepted advancing state',fontsize=13)
fig.savefig(packet/'pressure-path-counterexample.png')
fig.savefig(packet/'pressure-path-counterexample.pdf')
plt.close(fig)
frames=[]
for i,row in enumerate(samples):
    fig,ax=plt.subplots(figsize=(6.5,5.5),layout='constrained')
    c=mesh_panel(ax,row,row['divergence'],f"Candidate path parameter {row['path_parameter']:.4f} s",dmax)
    fig.colorbar(c,ax=ax,label='actual divergence [1/s]')
    initial=np.array(samples[0]['nodes'])
    ax.plot(initial[3:6,0],initial[3:6,1],'k--',linewidth=1,label='initial cap')
    pos=np.array(row['nodes']);u=np.array(row['velocity'])
    ax.quiver(pos[3:5,0],pos[3:5,1],u[pos[3:5,2].astype(int),0],u[pos[3:5,2].astype(int),1],angles='xy',scale_units='xy',scale=12,color='#0f172a')
    ax.legend(loc='lower right',fontsize=8)
    fig.suptitle('REJECTED CANDIDATE PATH — no advancing state published',fontsize=10,color='#b91c1c')
    fig.text(.05,.01,f"max div = {row['divergence_max']:.6g}; local continuity defect = {row['continuity_defect_max']:.6g}",fontsize=9)
    stream=io.BytesIO();fig.savefig(stream,format='png');stream.seek(0);frames.append(Image.open(stream).convert('RGB'));plt.close(fig)
frames[0].save(packet/'rejected-candidate-replay.gif',save_all=True,append_images=frames[1:],duration=1000,loop=0)
print('Rendered actual native force pressure and five rejected candidate samples')
