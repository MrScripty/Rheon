"""Render archived fixed-equation comparisons; never invokes a solver."""
from pathlib import Path
import json,os,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-e2-fixed-mpl');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-e2-fixed-cache')
import matplotlib
matplotlib.use('Agg');matplotlib.rcParams['svg.hashsalt']='rheon-e2-native-known-fixed-v1'
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;out=json.loads((P/'analysis-80-normal.json').read_text());fig,axes=plt.subplots(1,2,figsize=(11,5),constrained_layout=True)
for axis,index in zip(axes,[41,47]):
 rows=[x for x in out['rows']if x['index']==index and x['status']=='NATIVE_FIXED_EQUATION_EVALUATED']
 if not rows:axis.text(.1,.5,'Native equation refusal; no residual fabricated.',transform=axis.transAxes);continue
 for k,(key,label,color)in enumerate([('original_E1_norm','Original E1','#888888'),('native_norm','E2 stored native','#3565a5'),('exact_stored_stable','E2 exact stored inputs','#2d8d72'),('latent_inertia_only_counterfactual','Latent inertia only (diagnostic)','#cb9c39')]):
  values=[r[key]['norm']if isinstance(r[key],dict)else r[key]for r in rows];axis.bar([j+(k-1.5)*.18 for j in range(len(rows))],[v/1e-13 for v in values],width=.18,label=label,color=color)
 axis.axhline(1.,color='darkred',linestyle='--',label='Original Newton target');axis.set_xticks(range(len(rows)),[f"Order {r['order']}"for r in rows]);axis.set(title=f'Frozen case {index}',ylabel='Norm / 1e-13',xlabel='Prescribed quadrature order');axis.grid(axis='y',alpha=.2);
 if index==41:legend_handles,legend_labels=axis.get_legend_handles_labels()
fig.legend(legend_handles,legend_labels,loc='outside lower center',ncol=3,fontsize=8)
fig.suptitle('Working affine-known rounding only; native public knowns and stored inertia retained\nFour fixed equations. No Newton correction, third solve, accepted state or trajectory.',fontsize=11)
target=Path(sys.argv[1])if len(sys.argv)>1 else P/'fixed-equations-v2.svg';fig.savefig(target,metadata={'Date':None,'Creator':'Rheon E2 fixed-equation evidence renderer'}if target.suffix=='.svg'else None)
