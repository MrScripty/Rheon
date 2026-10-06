"""Regenerate a readable field figure from the unchanged actual publications."""
import hashlib,json,os,sys
from pathlib import Path
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-forced-render-cache')
os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-forced-font-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation
from matplotlib.colors import Normalize
from matplotlib.cm import ScalarMappable
import numpy as np
ROOT=Path(__file__).resolve().parents[2];P=ROOT/'evidence/forced-extruded-liquid';sys.path.insert(0,str(P));import replay
source=P/'native-trials/first-native.jsonl';rows=[json.loads(line)for line in source.read_text().splitlines()];groups=replay.schema(rows)
fig,axes=plt.subplots(2,2,figsize=(10,8),constrained_layout=True)
for ax,key in zip(axes.flat,[(k,'nonconstant',a,.003125)for k in ['initial','pressure_state']for a in ['forward','reversed']]):
 row=groups[key][-1];p=np.array(row['positions']);tri=np.array(row['triangles']);u=np.array(row['velocity']);ids=np.array(row['periodic_indices']);w=u[ids,2]
 ax.tripcolor(Triangulation(p[:,0],p[:,1],tri),facecolors=w[tri].mean(axis=1),vmin=-.25,vmax=.625,cmap='coolwarm',edgecolors='#263747',linewidth=.4)
 ax.set_aspect('equal');ax.set_xlim(-.02,1.15);ax.set_ylim(-.02,1.32);ax.set_xlabel('x (period 1)');ax.set_ylabel('y');ax.set_title(row['kind']+' / '+row['load']+' load\nactual t='+format(row['time'],'.5f')+', stamp='+str(row['stamp']['version']))
fig.colorbar(ScalarMappable(norm=Normalize(-.25,.625),cmap='coolwarm'),ax=axes.ravel().tolist(),label='Third velocity w (triangle mean)',shrink=.8)
fig.suptitle('Actual forced periodic, z-invariant three-component extrusion\nMaterial cap; weak natural traction; recorded native triangle means\nPhysical equations pass; reversed coarse geometry temporal gate fails',fontsize=11)
out=Path(__file__).parent/'readable-fields.png';fig.savefig(out,dpi=170);plt.close(fig)
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
(Path(__file__).parent/'readable-fields-receipt.json').write_text(json.dumps(dict(status='RENDERED_ACTUAL_PUBLICATIONS',native_input_sha256=sha(source),renderer_sha256=sha(Path(__file__)),output_sha256=sha(out),actual_plotted_publications=[groups[k][-1]['stamp']for k in [(x,'nonconstant',a,.003125)for x in ['initial','pressure_state']for a in ['forward','reversed']]],replaces_clipped_subplot_title_only=True,physical_equations_changed=False),indent=2)+'\n')
