"""Draw the actual native geometry and instantaneous mass rates, not evolution."""
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.collections import PolyCollection
here=Path(__file__).resolve().parent
d=json.loads((here/'native-qualification/release.json').read_text())
s=d['samples'][0]
fig,ax=plt.subplots(figsize=(9,5))
polygons=[[d['nodes'][i][:2] for i in t] for t in d['triangles']]
rates=[sum(s['mass_rates'][d['nodes'][i][2]] for i in t)/3 for t in d['triangles']]
collection=PolyCollection(polygons,array=rates,cmap='coolwarm',edgecolor='#263647',linewidth=.65)
ax.add_collection(collection);ax.autoscale();ax.set_aspect('equal')
ax.set(xlabel='periodic x (m)',ylabel='height (m)',title='Corrected periodic fitted mesh: instantaneous mass rates\nNo physical time advancement or advancing liquid solve')
fig.colorbar(collection,ax=ax,label='mean nodal mass rate per triangle (kg/s)')
fig.tight_layout();fig.savefig(here/'native-instantaneous.png',dpi=160)
