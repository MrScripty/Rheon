"""Actual recorded quadratic research candidates and their fixed work rejection."""
import io
import json
import os
from pathlib import Path
import sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl')
os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation
import numpy as np
from PIL import Image
import model as m

packet=Path(__file__).resolve().parent
study=json.loads(Path(sys.argv[1]).read_text());cells=study['cells'];coarse=cells[0]
plt.rcParams.update({'font.size':10,'figure.dpi':150})


def pressure_panel(ax,row,limit):
    pos=np.array(row['nodes']);tri=Triangulation(pos[:,0],pos[:,1],m.TRI)
    color=ax.tripcolor(tri,facecolors=row['reconstructed_pressure'],edgecolors='#334155',linewidth=.45,cmap='coolwarm',vmin=-limit,vmax=limit)
    initial=np.array(coarse['samples'][0]['nodes'])
    ax.plot(initial[3:6,0],initial[3:6,1],'k--',linewidth=1,label='initial cap')
    u=np.array(row['velocity']);raw=u[m.IDS]
    ax.quiver(pos[3:5,0],pos[3:5,1],raw[3:5,0],raw[3:5,1],angles='xy',scale_units='xy',scale=12,color='#0f172a')
    ax.set_aspect('equal');ax.set_xlim(-.03,1.1);ax.set_ylim(-.03,1.32)
    ax.set_xlabel('x [m]');ax.set_ylabel('y [m]');ax.legend(loc='lower right',fontsize=8)
    return color


limit=max(abs(p)for row in coarse['samples']for p in row['reconstructed_pressure'])
fig,axes=plt.subplots(1,3,figsize=(14,5),layout='constrained')
color=pressure_panel(axes[0],coarse['samples'][-1],limit)
axes[0].set_title('Solved pressure on the moving candidate')
fig.colorbar(color,ax=axes[0],label='relative pressure [Pa]')
h=[cell['interval']for cell in cells]
axes[1].loglog(h,[abs(cell['actual_momentum_residual_work'])for cell in cells],'o-',color='#b91c1c',label='actual residual work')
axes[1].loglog(h,[cell['fixed_work_allowance']for cell in cells],'--',color='#64748b',label='fixed work allowance')
axes[1].set_xlabel('candidate interval [s]');axes[1].set_ylabel('energy [J]')
axes[1].set_title('All three work gates fail');axes[1].grid(alpha=.25,which='both');axes[1].legend(fontsize=8)
gcl_ratio=max(abs(g)/allow for g,allow in zip(coarse['actual_local_GCL_defect'],coarse['local_GCL_allowance']))
ratios=[coarse['integrated_momentum_norm']/1e-11,gcl_ratio,coarse['max_pointwise_momentum_norm']/1e-11,abs(coarse['actual_momentum_residual_work'])/coarse['fixed_work_allowance']]
labels=['integrated\nmomentum','finite\nGCL','pointwise\nmomentum','weighted\nwork']
axes[2].bar(labels,ratios,color=['#15803d','#15803d','#b91c1c','#b91c1c'])
axes[2].set_yscale('log');axes[2].axhline(1,color='#64748b',linestyle='--')
axes[2].set_ylabel('ratio to fixed gate; above 1 fails');axes[2].set_title('Separate physical acceptance gates')
axes[2].tick_params(axis='x',labelsize=8);axes[2].grid(axis='y',alpha=.25)
fig.suptitle('Coupled quadratic research cell: moving constraints and physical GCL pass; weighted work fails\nTwo columns · all 22 integrated xy momentum equations · no accepted advancing state',fontsize=12)
fig.savefig(packet/'coupled-work-rejection.png');fig.savefig(packet/'coupled-work-rejection.pdf');plt.close(fig)
frames=[]
for row in coarse['samples']:
    fig,ax=plt.subplots(figsize=(6.5,5.5),layout='constrained')
    color=pressure_panel(ax,row,limit);fig.colorbar(color,ax=ax,label='relative pressure [Pa]')
    ax.set_title(f"Actual quadratic candidate parameter {row['time']:.4f} s")
    fig.suptitle('REJECTED — fixed weighted-work gate fails',color='#b91c1c',fontsize=12)
    fig.text(.05,.01,'Geometry/velocity/pressure replay is a research candidate; no simulation state is published.',fontsize=8)
    stream=io.BytesIO();fig.savefig(stream,format='png');stream.seek(0);frames.append(Image.open(stream).convert('RGB'));plt.close(fig)
frames[0].save(packet/'rejected-coupled-replay.gif',save_all=True,append_images=frames[1:],duration=1000,loop=0)
print('Rendered actual coupled quadratic research candidates with their work rejection')
