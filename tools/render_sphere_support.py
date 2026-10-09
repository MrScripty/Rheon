"""Plot retained Rust stationary frames and force records; no reconstructed motion."""
import hashlib
import json
from pathlib import Path
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
root,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=False)
a=json.loads((root/'gravity.json').read_text());rows=a['records']
fig,axes=plt.subplots(1,2,figsize=(10,4),layout='constrained')
axes[0].plot([r['after']['time_s'] for r in rows],[r['after']['center'][2] for r in rows],label='actual COM height')
axes[0].set(xlabel='Stored time (s)',ylabel='Height (m)',title='1,000 held native intervals',ylim=(0,.5));axes[0].grid(alpha=.2)
axes[1].plot([r['after']['time_s'] for r in rows],[r['gravity_force'][2] for r in rows],label='gravity')
axes[1].plot([r['after']['time_s'] for r in rows],[r['support_force'][2] for r in rows],label='support')
axes[1].plot([r['after']['time_s'] for r in rows],[r['net_force'][2] for r in rows],label='stored net force')
axes[1].set(xlabel='Stored time (s)',ylabel='Force (N)',title='Retained force proposal');axes[1].legend();axes[1].grid(alpha=.2)
fig.suptitle('Explicit stationary one-face equilibrium — constant load, zero full twist')
fig.savefig(out/'single-face-support.jpg',dpi=140);plt.close(fig)

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
receipt=dict(raw_input_sha256=sha(root/'gravity.input'),raw_output_sha256=sha(root/'gravity.json'),
             oracle_summary_sha256=sha(root/'oracle-summary.json'),
             executable_sha256=json.loads((root/'oracle-summary.json').read_text())['executable_sha256'],
             renderer_sha256=sha(Path(__file__)),image_sha256=sha(out/'single-face-support.jpg'),
             records=len(rows),renderer_integrates_physics=False)
(out/'render-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
