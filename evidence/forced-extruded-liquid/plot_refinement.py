"""Plot actual endpoint errors, including the failed original geometry band."""
import json,sys,os
os.environ.setdefault("MPLCONFIGDIR","/tmp/rheon-forced-render-cache")
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
p=Path(sys.argv[1]);out=Path(sys.argv[2]);d=json.loads(p.read_text())
if d['status']!='FAIL_ORIGINAL_TEMPORAL_BAND' or len(d['references'])!=16:raise ValueError('expected retained observed limitation')
fig,axes=plt.subplots(2,2,figsize=(11,8),constrained_layout=True)
for i,kind in enumerate(['initial','pressure_state']):
 for j,name in enumerate(['full_velocity_lumped_L2_error','geometry_max_error']):
  ax=axes[i,j]
  for field in ['nonconstant','constant']:
   for load in ['forward','reversed']:
    rows=[r for r in d['rows']if r['kind']==kind and r['field']==field and r['load']==load]
    ax.loglog([r['interval']for r in rows],[r[name]for r in rows],marker='o',linestyle='-'if load=='forward'else'--',label=field+' / '+load)
  ax.set_title(kind+' / '+('full velocity lumped L2'if j==0 else'geometry max error'));ax.set_xlabel('Actual step interval h');ax.set_ylabel('Error to independently regenerated DAE');ax.grid(True,which='both',alpha=.2);ax.legend(fontsize=8)
fig.suptitle('Actual forced extrusion: full velocity passes original temporal band\nReversed geometry coarse ratios fail the unchanged 1.7–2.3 gate',fontsize=12)
fig.savefig(out,dpi=160);plt.close(fig)
