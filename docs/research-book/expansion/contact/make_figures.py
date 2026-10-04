"""Original analytical illustrations; no numerical fluid results or copied art."""
from pathlib import Path
import os
# Keep optional plotting caches inside this contribution directory.
os.environ.setdefault("MPLCONFIGDIR", str(Path(__file__).resolve().parent / "_plot_cache"))
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Arc

ROOT=Path(__file__).resolve().parent
OUT=ROOT/'figures'
OUT.mkdir(exist_ok=True)
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':11,'svg.fonttype':'none',
                     'axes.spines.top':False,'axes.spines.right':False})
colors=['#176b8a','#228b85','#628b47','#c08c2e','#b45944']

fig,axs=plt.subplots(1,5,figsize=(14,3.8),layout='constrained')
for ax,deg,color in zip(axs,[30,60,90,120,150],colors):
    theta=np.deg2rad(deg)
    r=(3/(np.pi*(2-3*np.cos(theta)+np.cos(theta)**3)))**(1/3)
    phi=np.linspace(-theta,theta,500)
    x,y=r*np.sin(phi),r*(np.cos(phi)-np.cos(theta))
    ax.fill(x,y,color=color,alpha=.22)
    ax.plot(x,y,color=color,lw=2.5)
    ax.axhline(0,color='#444444',lw=1.2)
    ax.set_xlim(-1.65,1.65); ax.set_ylim(-.12,1.55)
    ax.set_aspect('equal'); ax.set_title(f'{deg}°',weight='bold')
    ax.set_xticks([]); ax.set_yticks([])
    for spine in ax.spines.values(): spine.set_visible(False)
fig.suptitle('Equal volume spherical caps on a rigid flat substrate',fontsize=16,weight='bold')
fig.supxlabel('Analytical cross-sections at zero gravity • common scale • each cap has volume 1',fontsize=11)
fig.savefig(OUT/'equal-volume-sessile-caps.svg',bbox_inches='tight')
fig.savefig(OUT/'equal-volume-sessile-caps.png',dpi=160,bbox_inches='tight')
plt.close(fig)

fig,axs=plt.subplots(1,2,figsize=(11,4.5),layout='constrained')
y=np.linspace(0,1,200)
ratios=[0,.01,.1,1]
for q,color in zip(ratios,colors):
    axs[0].plot((y+q)/(1+2*q),y,lw=2.2,color=color,label=f'ℓ/H = {q:g}')
axs[0].set(xlabel='Fluid speed u/U',ylabel='Height y/H',xlim=(0,1),ylim=(0,1),title='Exact Couette profiles')
axs[0].legend(loc='upper left',fontsize=10); axs[0].grid(alpha=.18)
q=np.array(ratios); bulk=1/(1+2*q)**2; wall=2*q/(1+2*q)**2
xx=np.arange(len(q))
axs[1].bar(xx,bulk,color='#176b8a',label='Bulk viscous loss')
axs[1].bar(xx,wall,bottom=bulk,color='#c08c2e',label='Wall slip loss')
axs[1].scatter(xx,1/(1+2*q),color='#252525',marker='_',s=300,zorder=3,label='Wall input power')
axs[1].set(xticks=xx,xticklabels=[str(t) for t in ratios],xlabel='Slip length ℓ/H',ylabel='Power per area / (μ U²/H)',title='Exact steady work balance')
axs[1].legend(fontsize=9,loc='upper right'); axs[1].grid(axis='y',alpha=.18)
fig.savefig(OUT/'couette-slip-and-power.svg',bbox_inches='tight')
fig.savefig(OUT/'couette-slip-and-power.png',dpi=160,bbox_inches='tight')
plt.close(fig)

fig,ax=plt.subplots(figsize=(11,5),layout='constrained')
theta=np.pi/3; R=2; phi=np.linspace(-theta,theta,300)
x=R*np.sin(phi); y=R*(np.cos(phi)-np.cos(theta))
ax.fill(x,y,color='#cce8ef'); ax.plot(x,y,color='#176b8a',lw=3)
ax.plot([-3.7,3.7],[0,0],color='#555555',lw=4)
ax.fill_between([-3.7,3.7],-.22,0,color='#dddddd')
ax.text(-3.45,-.53,'Rigid impermeable solid',fontsize=12)
ax.annotate('Local wall velocity Uw',xy=(.4,-.35),xytext=(-1.5,-.35),
            arrowprops=dict(arrowstyle='->',color='#a56623',lw=2),va='center',color='#895119')
ax.annotate('Normal constraint\n(u − Uw) · n = 0',xy=(-.7,.03),xytext=(-3.5,1.3),
            arrowprops=dict(arrowstyle='->',color='#555555'),ha='left')
ax.annotate('Tangential friction\ntraction = −β slip',xy=(.6,.12),xytext=(1.8,.55),
            arrowprops=dict(arrowstyle='->',color='#555555'),ha='left')
ax.annotate('Surface tension γ\ncurvature pressure jump',xy=(.5,.935),xytext=(.7,1.75),
            arrowprops=dict(arrowstyle='->',color='#176b8a'),ha='left')
a=R*np.sin(theta)
ax.add_patch(Arc((a,0),.75,.75,angle=0,theta1=120,theta2=180,color='#b45944',lw=2))
ax.text(a-.52,.14,'θ',color='#b45944',fontsize=15)
ax.annotate('Equilibrium angle through liquid\nset by interfacial energy',xy=(a-.2,.18),xytext=(.8,-1.0),
            arrowprops=dict(arrowstyle='->',color='#b45944'),ha='center',color='#974230')
ax.set(xlim=(-3.8,3.8),ylim=(-1.15,2.15)); ax.set_aspect('equal'); ax.axis('off')
ax.set_title('Different boundary mechanisms need different controls',fontsize=16,weight='bold',pad=15)
fig.savefig(OUT/'boundary-mechanisms.svg',bbox_inches='tight')
fig.savefig(OUT/'boundary-mechanisms.png',dpi=160,bbox_inches='tight')
plt.close(fig)
print('Created three original SVG figures and PNG previews.')

