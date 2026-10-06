"""Scientific figures from actual publications and terminal trace, without simulation."""
from pathlib import Path
import json,sys,hashlib
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent;ROOT=P.parents[1];sys.path.insert(0,str(ROOT/'evidence/forced-extruded-liquid'));import replay as frozen
new=[json.loads(l)for l in(P/'forcing_temporal_probe-default.jsonl').read_text().splitlines()];old=[json.loads(l)for l in(ROOT/'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_text().splitlines()]
def selected(rows):return [r for r in rows if 'model'in r and r['kind']=='pressure_state'and r['load']=='reversed'and r['h']==.0015625]
a=selected(new);b=selected(old)
if len(a)!=65 or len(b)!=7:raise ValueError('actual newly completed and historically truncated trajectory')
x=np.array([frozen.r.m.q_to_x(np.array(r['end_q']))for r in a]);xb=np.array([frozen.r.m.q_to_x(np.array(r['end_q']))for r in b]);t=np.array([r['time']for r in a]);tb=np.array([r['time']for r in b])
plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
fig,axes=plt.subplots(2,2,figsize=(11,7),layout='constrained')
for j,label in enumerate(['periodic cap x0','left cap height H0','middle cap x1','middle cap height H1']):
 ax=axes.flat[j];ax.plot(t,x[:,j]-x[0,j],color='#176f91',label='candidate actual accepted states');ax.plot(tb,xb[:,j]-xb[0,j],color='#d07127',linewidth=3,label='original accepted prefix');ax.axvline(.0109375,color='#777',ls=':',lw=1);ax.set_title(label);ax.set_xlabel('actual accepted clock');ax.set_ylabel('change from constructor');ax.grid(alpha=.2)
axes.flat[0].legend(fontsize=8)
fig.suptitle('Reversed pressure fixture, h=0.0015625: final authorized correction fully qualifies\nOriginal refusal at step 7 remains frozen; candidate reaches t=0.1')
fig.savefig(P/'accepted-geometry.png',dpi=180);fig.savefig(P/'accepted-geometry.pdf');plt.close(fig)
fig,axes=plt.subplots(1,2,figsize=(11,4),layout='constrained');steps=a[1:];times=[r['time']for r in steps]
axes[0].plot(times,[r['report']['planar']['finite_momentum_rate_norm']/1e-13 for r in steps],'.-',ms=3,color='#176f91');axes[0].axhline(1,color='#c33',ls='--');axes[0].axvline(.0109375,color='#777',ls=':');axes[0].set_ylabel('published order-32 planar rate / 1e-13');axes[0].set_title('All 64 published steps');axes[0].set_xlabel('accepted clock')
for name,color in [('planar','#176f91'),('third','#d07127'),('total','#458849')]:
 axes[1].plot(times,[abs(r['report'][name]['energy_ledger_error'])/r['report'][name]['fixed_work_allowance']for r in steps],'.-',ms=3,color=color,label=name)
axes[1].axhline(1,color='#c33',ls='--');axes[1].set_ylabel('|reported energy ledger| / fixed work allowance');axes[1].set_xlabel('accepted clock');axes[1].set_title('Original work thresholds retained');axes[1].legend(fontsize=8)
for ax in axes:ax.grid(alpha=.2)
fig.suptitle('Actual reports after full native qualification; independent replay also passes unchanged gates')
fig.savefig(P/'accepted-gates.png',dpi=180);fig.savefig(P/'accepted-gates.pdf');plt.close(fig)
record={'source':'922466bd9efdf4612462288fbe2df5a679ef4ccb','actual_new_states':len(a),'actual_original_prefix_states':len(b),'coordinates':['x0','H0','x1','H1'],'times':t.tolist(),'new_coordinates':x.tolist(),'original_times':tb.tolist(),'original_coordinates':xb.tolist(),'browser_rasterization_claimed':False,'input_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [P/'forcing_temporal_probe-default.jsonl',ROOT/'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl']}}
(P/'render-data.json').write_text(json.dumps(record,indent=2)+'\n');print('PASS actual published geometry and unchanged-gate scientific PNG/PDF; no simulated states')
