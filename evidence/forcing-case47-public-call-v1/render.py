"""Render actual one-call publications and work gates; no new computation."""
from pathlib import Path
import json,sys
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent
rows=[json.loads(x)for x in(P/'E2-native.log').read_text().splitlines()if x.startswith('{')and '"model"'in x];old,new=rows[-2:];out=json.loads((P/'analysis-normal.json').read_text());plt.rcParams.update({'svg.hashsalt':'case47-public-one-call-v1','font.size':10});fig,axes=plt.subplots(1,3,figsize=(13,4),layout='constrained')
a=axes[0];pos=np.array(new['positions']);ids=np.array(new['periodic_indices']);u=np.array(new['velocity']);a.triplot(pos[:,0],pos[:,1],np.array(new['triangles']),color='#aaaaaa',linewidth=.7);dots=a.scatter(pos[:,0],pos[:,1],c=u[ids,2],cmap='viridis',s=30,zorder=3);fig.colorbar(dots,ax=a,label='Published third velocity');a.set(xlabel='x',ylabel='height',title='E2 actual version26 geometry/velocity');a.set_aspect('equal')
a=axes[1];names=['Planar','Third','Combined'];report=new['report'];ratios=[max(abs(report[k]['energy_ledger_error']),abs(report[k]['discrete_residual_work']))/report[k]['fixed_work_allowance']for k in ['planar','third','total']];a.bar(names,ratios,color=['#395b8c','#527b70','#b26a49']);a.axhline(1,color='#9f3c3c',linestyle='--',label='Original work bound');a.set(yscale='log',ylabel='Maximum reported work error / allowance',title='Full native work gates');a.set_ylim(1e-4,1.3)
for i,value in enumerate(ratios):a.text(i,value*1.2,f'{value:.3g}',ha='center',fontsize=9)
a.legend(fontsize=8)
a=axes[2];trace=[json.loads(x)for x in(P/'E2-native-stderr.log').read_text().splitlines()if x.startswith('{')];checks=[x for x in trace if x['event']=='newton_check'and x['accepted_version']==25];e1=json.loads((P/'analysis-normal.json').read_text())['jobs']['E1'];a.semilogy([x['calls']for x in checks],[x['rate_norm']for x in checks],'o-',color='#395b8c',label='E2 actual checks');a.plot([50],[1.3144896847477506e-13],'x',color='#b26a49',label='E1 refused terminal');a.axhline(1e-13,color='#9f3c3c',linestyle='--',label='Unchanged Newton target');a.set(xlabel='Counted equation calls',ylabel='Planar momentum-rate norm',title='One comparison call per job');a.legend(fontsize=8)
fig.suptitle('Case47 • exact E1 prefix25 • one E2 publication • no trajectory extension',fontsize=12);fig.savefig(sys.argv[1]if len(sys.argv)>1 else P/'public-call.svg',metadata={'Date':None});plt.close(fig)
