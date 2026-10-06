"""Signed error/h of existing endpoints; no simulation or reference integration."""
from pathlib import Path
import json,os,sys
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-e1-geometry-matplotlib');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-e1-geometry-cache')
import matplotlib
matplotlib.use('Agg');matplotlib.rcParams['svg.hashsalt']='rheon-e1-signed-geometry-v1'
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;out=json.loads((P/'outcome-v3-normal.json').read_text());labels=out['coordinate_labels'];fig,axes=plt.subplots(2,2,figsize=(12,8),constrained_layout=True)
for i,kind in enumerate(['initial','pressure_state']):
 for j,load in enumerate(['forward','reversed']):
  family=next(x for x in out['families']if x['kind']==kind and x['load']==load and x['field']=='nonconstant');rows=[x for x in family['errors']if x['status']=='COMPLETE'];axis=axes[i,j]
  for k,label in enumerate(labels):axis.semilogx([x['h']for x in rows],[x['signed_error_over_h'][k]for x in rows],'o-',label=label.replace('_',' '),markersize=4)
  axis.axhline(0.,color='gray',linewidth=.8);axis.axvline(.003125,color='gray',linestyle=':',label='Original finest coarse h');axis.set(title=kind+' '+load,xlabel='Original h',ylabel='Signed component error / h');axis.grid(alpha=.25);axis.legend(fontsize=7)
  refused=[x for x in family['errors']if x['status']!='COMPLETE']
  if refused:axis.text(.03,.05,'Missing final endpoint at h=0.00078125\nRefused prefix excluded from endpoint errors.',transform=axis.transAxes,fontsize=8,bbox=dict(facecolor='white',alpha=.8,edgecolor='gray'))
fig.suptitle('Existing common-endpoint geometry: signed error/h varies with h\nNonconstant third fields displayed; all eight original family/band failures retained in data.',fontsize=12)
target=Path(sys.argv[1])if len(sys.argv)>1 else P/'signed-components.svg';fig.savefig(target,metadata={'Date':None,'Creator':'Rheon existing-endpoint geometry renderer'}if target.suffix=='.svg'else None)
