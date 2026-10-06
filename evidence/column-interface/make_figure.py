"""Plot actual accepted native guidance/geometry and imposed-flow error data."""
import csv,json,math
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from PIL import Image
HERE=Path(__file__).resolve().parent
case=HERE/'demo/jacobi-pcg-v1-activation'
fig,axes=plt.subplots(1,4,figsize=(14,4.2),gridspec_kw={'width_ratios':[1,1,1.5,1.7]})
for ax,stem,title in zip(axes[:2],['initial','final'],['Initial: mixed top cells','Accepted: 8 coupled intervals']):
 with Image.open(case/(stem+'.png')) as image: ax.imshow(image,cmap='Blues',vmin=0,vmax=255,interpolation='nearest',extent=(0,2,0,4))
 with (case/(stem+'-geometry.csv')).open() as stream: geometry=list(csv.DictReader(stream))
 end=[float(r['full_layers'])+float(r['top_fraction']) for r in geometry]
 held=[float(r['pressure_full_layers'])+float(r['pressure_top_fraction']) for r in geometry]
 ax.step([0,1,2],[*end,end[-1]],where='post',color='#da662e',lw=2,label='End column height')
 ax.step([0,1,2],[*held,held[-1]],where='post',color='#293c54',ls='--',lw=1.2,label='Held pressure geometry')
 ax.set_title(title,fontsize=11);ax.set_xticks([.5,1.5],labels=['left','right']);ax.set_yticks([0,1,2,3,4]);ax.set_xlabel('Native +Z fraction integral')
axes[1].legend(fontsize=7,loc='upper left')
summary=json.loads((HERE/'numerical-summary.json').read_text())
conv=sorted([r for r in summary['results'] if r.get('courant')==.25],key=lambda r:r['x_cells'])
axes[2].loglog([r['x_cells'] for r in conv],[r['height_l1_error'] for r in conv],'o-',color='#217b8d',label='C=1/4')
extra=next(r for r in summary['results'] if r.get('courant')==.125)
axes[2].scatter([64],[extra['height_l1_error']],marker='s',color='#c16a34',label='C=1/8, more donor diffusion')
axes[2].set_xticks([16,32,64,128],labels=['16','32','64','128']);axes[2].minorticks_off();axes[2].set_xlabel('X cells, fixed final time 1/2');axes[2].set_ylabel('L1 height error');axes[2].set_title('Imposed X-flow refinement',fontsize=11);axes[2].grid(alpha=.2);axes[2].legend(fontsize=7)
profile=HERE/'demo/advection-n128-c025'
with (profile/'final-cells.csv').open() as stream: rows=[r for r in csv.DictReader(stream) if r['j']=='1' and r['k']=='0']
x=[(int(r['i'])+.5)/128 for r in rows]
actual=[.25*(1+float(r['fraction'])) for r in rows]
def primitive(t): return t if t<=.25 else .5 if t>=.75 else .25+.5*(t-.25)+math.sin(2*math.pi*(t-.25))/(4*math.pi)
exact=[.25*(1+128*(primitive((int(r['i'])+1)/128-.125)-primitive(int(r['i'])/128-.125))) for r in rows]
axes[3].plot(x,exact,color='#313c55',lw=2,label='Analytic cell-average height')
axes[3].plot(x,actual,color='#268e9b',ls='--',label='Accepted 128-cell height')
axes[3].set_xlabel('X position');axes[3].set_ylabel('Height');axes[3].set_title('Analytic prescribed-flow height',fontsize=11);axes[3].grid(alpha=.2);axes[3].legend(fontsize=7)
fig.suptitle('Consecutive mixed-interface column intervals',fontsize=15)
fig.text(.5,.02,'Coupled scene: volume 3 retained; a new pressure center activates. Imposed advection: convergence approaches first order.\nBounded bottom-attached geometry closure; no arbitrary VOF topology, oblique cut-cell momentum or validated general liquid accuracy.',ha='center',fontsize=9)
fig.tight_layout(rect=(0,.12,1,.92))
fig.savefig(HERE/'continuation-and-convergence.png',dpi=160,metadata={'Software':'Rheon accepted native PNG/CSV and independent analytic oracle'})
print(json.dumps({'native_inputs':['demo/jacobi-pcg-v1-activation/initial.png','demo/jacobi-pcg-v1-activation/final.png','demo/jacobi-pcg-v1-activation/initial-geometry.csv','demo/jacobi-pcg-v1-activation/final-geometry.csv'],'numeric_inputs':['numerical-summary.json','demo/advection-n128-c025/final-cells.csv'],'output':'continuation-and-convergence.png'}))
