"""Standalone scientific render of actual native publications; no integration."""
from pathlib import Path
import json
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
P=Path(__file__).resolve().parent
def run():
 rows=[json.loads(x) for x in (P/'main-native.log').read_text().splitlines() if x.startswith('{') and '"model"' in x];a=json.loads((P/'analysis-normal.json').read_text());last=rows[-1]
 plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'rheon-case47-pure-e2-original128-v2','axes.spines.top':False,'axes.spines.right':False})
 fig,ax=plt.subplots(2,2,figsize=(12,8),layout='constrained');xy=np.asarray(last['positions']);tri=np.asarray(last['triangles']);colors=ax[0,0].tripcolor(xy[:,0],xy[:,1],tri,facecolors=last['physical_pressure'],edgecolors='#545e70',linewidth=.45,cmap='coolwarm');fig.colorbar(colors,ax=ax[0,0],label='Reconstructed pressure (fixture units)');ax[0,0].set_aspect('equal');ax[0,0].set(xlabel='x',ylabel='y',title=f'Actual publication {last["step"]}, t = {last["time"]:.17g}')
 time=[x['time'] for x in rows];q=np.asarray([x['end_q'] for x in rows])
 for j,label in enumerate(['Left cap x','Middle cap x','Left cap height']):ax[0,1].plot(time,q[:,j],label=label)
 ax[0,1].set(xlabel='Accepted time',ylabel='Cap coordinate',title='Pure E2 from the original constructor');ax[0,1].legend()
 norms=[x['Newton_norm'] for x in a['main_solver_details']];ax[1,0].semilogy(time[1:],norms,color='#2563aa',label='Final order16 Newton norm');ax[1,0].axhline(1e-13,color='#bf3a39',linestyle='--',label='Unchanged 1e-13 target');ax[1,0].set(xlabel='Accepted time',ylabel='Generalized force residual norm',title='128 accepted calls; no step retries');ax[1,0].legend()
 for key,label in [('planar','Planar'),('third','Third component'),('total','Combined')]:
  ratio=[abs(x['report'][key]['energy_ledger_error'])/x['report'][key]['fixed_work_allowance'] for x in rows[1:]];ax[1,1].semilogy(time[1:],np.maximum(ratio,np.finfo(float).tiny),label=label)
 ax[1,1].axhline(1,color='#bf3a39',linestyle='--',label='Original fixed allowance');ax[1,1].set(xlabel='Accepted time',ylabel='Absolute ledger error / allowance',title='Actual native work reports');ax[1,1].legend()
 fig.suptitle('Rheon case47: restricted periodic extrusion\nOriginal 128-step pure-E2 trajectory • 11 cancellation-preservation controls',fontsize=14)
 fig.savefig(P/'trajectory.svg',metadata={'Date':None});fig.savefig(P/'trajectory.png',dpi=150,metadata={'Software':'Rheon frozen evidence renderer'});plt.close(fig)
if __name__=='__main__':run()
