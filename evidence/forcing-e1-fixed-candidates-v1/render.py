"""Captured fixed-candidate arithmetic comparisons; no numerical executions."""
from pathlib import Path
import json,os,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-e1-fixed-matplotlib');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-e1-fixed-cache')
import matplotlib
matplotlib.use('Agg');matplotlib.rcParams['svg.hashsalt']='rheon-e1-fixed-candidates-v1'
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;out=json.loads((P/'analysis-80-normal.json').read_text());rows=[x for x in out['rows']if x['order']==16]
names=['native_sequential','exact_sum_of_rounded_contributions','exact_products_sums_stored_stable','exact_products_sums_stored_direct','combined_exact_force_transfer','actual_106_chart_inertia_only','exact_stored_known_chart_inertia_only','exact_continuous_known_chart_inertia_only','fixed_operator_chart_endpoint_substitution_continuous_known']
labels=['Native archived norm','Exact final accumulation','Exact stored stable equation','Exact stored direct equation','Exact force + transfer assembly','Actual 106-bit chart (inertia only)','Exact chart, rounded knowns (inertia only)','Continuous knowns (inertia only)','Continuous knowns, fixed-operator endpoint']
fig,axes=plt.subplots(1,2,figsize=(13,6),constrained_layout=True)
for axis,row in zip(axes,rows):
 values=[row['native_norm']if name=='native_sequential'else row['variants'][name]['norm']for name in names];colors=['#4b6c9e']*5+['#d39335']*3+['#7561a2'];axis.barh(range(len(names)),[x/1e-13 for x in values],color=colors);axis.invert_yaxis();axis.axvline(1.,color='darkred',linestyle='--',label='Original Newton target');axis.set_yticks(range(len(names)),labels,fontsize=8);axis.set(xlabel='Norm / original 1e-13 target',title=f"Case {row['index']}: {row['kind']} {row['load']}\nFrozen accepted version {row['accepted_version']}, order 16");axis.grid(axis='x',alpha=.25);axis.legend(fontsize=8)
fig.suptitle('Fixed failed E1 candidates: summation changes alone do not remove refusal\nOrange: isolated sensitivity; purple: counterfactual fixed-operator endpoint. No accepted steps.',fontsize=12)
target=Path(sys.argv[1])if len(sys.argv)>1 else P/'arithmetic.svg';fig.savefig(target,metadata={'Date':None,'Creator':'Rheon fixed-candidate diagnostic renderer'}if target.suffix=='.svg'else None)
