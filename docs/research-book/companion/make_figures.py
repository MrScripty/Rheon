from pathlib import Path
import json
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle,FancyArrowPatch
from matplotlib.ticker import NullFormatter
R=Path(__file__).resolve().parents[1];O=R/'figures';O.mkdir(exist_ok=True)
D=json.loads((R/'companion/results.json').read_text())
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'axes.spines.top':False,
 'axes.spines.right':False,'axes.labelsize':10,'legend.fontsize':9,'svg.fonttype':'none'})
blue='#176B91';orange='#B86936';gray='#506272'
def save(fig,name):
 for ax in fig.axes:
  if ax.get_xscale()=='log':ax.xaxis.set_minor_formatter(NullFormatter())
 fig.tight_layout();fig.savefig(O/(name+'.png'),dpi=220,bbox_inches='tight')
 fig.savefig(O/(name+'.svg'),bbox_inches='tight');plt.close(fig)
fig,ax=plt.subplots(figsize=(6.2,3.7))
for x in range(4):ax.plot([x,x],[0,3],color='#c8d2da',lw=1)
for y in range(4):ax.plot([0,3],[y,y],color='#c8d2da',lw=1)
for i in range(3):
 for j in range(3):
  ax.scatter(i+.5,j+.5,c='black',s=18)
for i in range(4):
 for j in range(3):ax.arrow(i,j+.5,.20,0,head_width=.07,color=blue,length_includes_head=True)
for i in range(3):
 for j in range(4):ax.arrow(i+.5,j,0,.20,head_width=.07,color=orange,length_includes_head=True)
ax.text(3.2,2.55,'Black dots\ncell scalars',va='top');ax.text(3.2,1.65,'Blue arrows\nx velocity',color=blue)
ax.text(3.2,.7,'Copper arrows\ny velocity',color=orange)
ax.set(xlim=(-.2,4.6),ylim=(-.2,3.3),aspect='equal');ax.axis('off')
save(fig,'staggered-grid')
fig,ax=plt.subplots(figsize=(6.2,3.4))
for key,color,label in [('cg',blue,'Conjugate gradients'),('jacobi',orange,'Weighted Jacobi')]:
 y=D['projection']['histories'][key];ax.semilogy(np.arange(1,len(y)+1),y,color=color,label=label)
ax.set(xlabel='Iteration',ylabel='True reduced residual L2');ax.legend();ax.grid(alpha=.2)
save(fig,'pressure-residual')
fig,ax=plt.subplots(figsize=(6.2,3.5))
s=D['transport']['sine_period'];n=[x['n'] for x in s]
ax.loglog(n,[x['sl_l2'] for x in s],'o-',color=blue,label='Linear transport')
ax.loglog(n,[x['limited_corrected_l2'] for x in s],'s-',color=orange,label='Limited correction')
ax.set(xlabel='Cells in periodic interval',ylabel='RMS error after one period')
ax.set_xticks(n,labels=[str(x) for x in n]);ax.legend();ax.grid(alpha=.2,which='both')
save(fig,'transport-refinement')
fig,axs=plt.subplots(1,2,figsize=(6.2,2.8),gridspec_kw={'width_ratios':[1.3,1]})
W=np.array(D['transport']['nonconservative_W'])
im=axs[0].imshow(W,vmin=0,vmax=1,cmap='Blues')
for i in range(3):
 for j in range(3):axs[0].text(j,i,str(W[i,j]),ha='center',va='center')
axs[0].set(xlabel='Old sample',ylabel='New sample',xticks=[0,1,2],yticks=[0,1,2])
axs[1].bar(['Before','After'],[1,1.5],color=[blue,orange]);axs[1].set(ylabel='Total scalar',ylim=(0,1.7))
save(fig,'interpolation-mass')
fig,ax=plt.subplots(figsize=(6.2,3.4))
for key,color,label in [('layout_bytes',blue,'Named dense fields'),('apic_particle_bytes_8_per_cell',orange,'64-byte particles at 8 per cell')]:
 ax.plot([x['n'] for x in D['memory']],[x[key]/2**20 for x in D['memory']],'o-',color=color,label=label)
ax.set_yscale('log');ax.set(xlabel='Cells per axis',ylabel='Calculated memory MiB');ax.legend();ax.grid(alpha=.2)
save(fig,'memory-scaling')
fig,ax=plt.subplots(figsize=(6.2,3.2))
v=[D['projection']['residual_identity_linf_error_f64'],D['projection']['residual_identity_linf_error_f32']]
ax.bar(['Binary64\noperations','Binary32\nreplay'],v,color=[blue,orange])
ax.set_yscale('log');ax.set(ylabel='Maximum identity discrepancy')
for i,x in enumerate(v):ax.text(i,x*2,f'{x:.2e}',ha='center')
ax.set_ylim(1e-16,1e-4);ax.grid(axis='y',alpha=.2)
save(fig,'rounding-gap')
fig,ax=plt.subplots(figsize=(6.2,3.0))
x=np.arange(64)/64;q0=np.zeros(64);q0[20:36]=1
ax.step(x,q0,where='mid',color=gray,label='Initial box')
ax.plot(x,D['transport']['box_after'],color=blue,label='After one period')
ax.set(xlabel='Periodic coordinate',ylabel='Scalar',ylim=(-.05,1.1));ax.legend();ax.grid(alpha=.2)
save(fig,'box-diffusion')
fig,ax=plt.subplots(figsize=(6.2,3.1))
points=np.array([[0,0],[2,0],[1,1.7]])
for i,j in [(0,1),(1,2),(2,0)]:
 a=points[i];b=points[j]
 ax.add_patch(FancyArrowPatch(a+.14*(b-a),b-.14*(b-a),arrowstyle='-|>',mutation_scale=16,color=blue,lw=2))
for i,p in enumerate(points):ax.scatter(*p,s=450,c='#e7eff3',edgecolors=gray);ax.text(*p,str(i),ha='center',va='center')
ax.text(3,1.25,'Before  (3, 0, 0)',color=gray)
ax.text(3,.75,'After    (1, 1, 1)',color=blue)
ax.text(3,.15,'A cycle can retain\nzero-divergence flow')
ax.set(xlim=(-.5,5.7),ylim=(-.4,2.1),aspect='equal');ax.axis('off')
save(fig,'cycle-circulation')
print('Generated 8 original figures in PNG and SVG')
E=json.loads((R/'companion/depth-results.json').read_text())
fig,ax=plt.subplots(figsize=(6.2,3.3))
y=E['multigrid']['residual_history']
ax.semilogy(range(len(y)),y,'o-',color=blue)
ax.set(xlabel='Two-level cycles',ylabel='True residual L2');ax.grid(alpha=.2)
save(fig,'multigrid-mechanism')
fig,ax=plt.subplots(figsize=(6.2,3.3))
rows=E['rotation'];n=[v['steps'] for v in rows]
ax.loglog(n,[v['euler_error'] for v in rows],'o-',color=blue,label='Euler trace')
ax.loglog(n,[v['midpoint_error'] for v in rows],'s-',color=orange,label='Midpoint trace')
ax.set_xticks(n,labels=[str(x) for x in n]);ax.set(xlabel='Steps over unit time',ylabel='Departure position error')
ax.legend();ax.grid(alpha=.2,which='both');save(fig,'backtrace-refinement')
fig,ax=plt.subplots(figsize=(6.2,3.3))
rows=E['curvature'];h=[v['h'] for v in rows]
ax.loglog(h,[v['error'] for v in rows],'o-',color=blue)
ax.set(xlabel='Grid spacing h',ylabel='Absolute curvature error');ax.grid(alpha=.2,which='both')
save(fig,'curvature-refinement')
