"""Recorded native state viewer and scientific figures; no second simulator."""
import hashlib,json,os,sys
from pathlib import Path
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-third-render-cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation
import numpy as np
from PIL import Image
import replay
source=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir()
rows=[json.loads(x)for x in source.read_text().splitlines()];groups=replay.schema(rows)
def draw(ax,row):
 p=np.array(row['positions']);tri=np.array(row['triangles']);ids=np.array(row['periodic_indices']);u=np.array(row['velocity']);w=u[ids,2]
 ax.tripcolor(Triangulation(p[:,0],p[:,1],tri),facecolors=w[tri].mean(axis=1),vmin=-.25,vmax=.625,cmap='coolwarm',edgecolors='#263747',linewidth=.4)
 ax.set_aspect('equal');ax.set_xlim(-.02,1.15);ax.set_ylim(-.02,1.32);ax.set_xlabel('x (period 1)');ax.set_ylabel('y')
 ax.set_title(f"{row['kind']} / {row['field']}\nt={row['time']:.5f}, actual stamp={row['stamp']['version']}")
fig,axes=plt.subplots(2,2,figsize=(10,8),constrained_layout=True)
for ax,key in zip(axes.flat,[('initial','nonconstant',.003125),('pressure_state','nonconstant',.003125),('initial','constant',.003125),('pressure_state','constant',.003125)]):draw(ax,groups[key][-1])
fig.suptitle('Actual published periodic, z-invariant three-component extrusion\nTriangle mean third velocity w; material cap, weak natural traction; no general 3D reconstruction',fontsize=11)
fig.savefig(out/'published-third.png',dpi=170);fig.savefig(out/'published-third.pdf',metadata={'CreationDate':None,'ModDate':None});plt.close(fig)
frames=[]
for row in groups[('initial','nonconstant',.003125)]:
 fig,ax=plt.subplots(figsize=(7,4));draw(ax,row);fig.suptitle('Recorded native state; z periodic, all fields z independent',fontsize=11);fig.tight_layout();fig.canvas.draw();frames.append(Image.fromarray(np.asarray(fig.canvas.buffer_rgba()).copy()).convert('RGB'));plt.close(fig)
frames[0].save(out/'published-third.gif',save_all=True,append_images=frames[1:],duration=160,loop=0)
html='''<!doctype html><html><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Rheon recorded extrusion</title><style>body{margin:24px auto;max-width:1040px;padding:0 20px;font:16px system-ui;background:#eef2f5;color:#172a3b}h1{font-size:26px}canvas{width:100%;background:white;border:1px solid #bdcbd5;border-radius:8px}label{display:inline-block;margin:8px 16px 12px 0}select,button{font:inherit;padding:6px}input{width:60%}#info{font-variant-numeric:tabular-nums;padding:12px;background:white}p{line-height:1.5}</style><h1>Recorded native three-component extrusion</h1><p>x and z are periodic with period 1; all fields are independent of z. The bottom is impermeable with natural tangential traction. The cap is material with weak natural atmospheric-relative traction. Colors show triangle mean <b>w</b> from actual published nodes. This viewer plays recorded states.</p><label>Case <select id="case"></select></label><label>Step interval <select id="h"></select></label><button id="play">Play</button><canvas id="view" width="1000" height="540"></canvas><label>Published state <input id="frame" type="range" min="0" value="0"></label><div id="info"></div><script>const rows=DATA;const cases=['initial / nonconstant','pressure_state / nonconstant','initial / constant','pressure_state / constant'];const cs=document.getElementById('case'),hs=document.getElementById('h'),slider=document.getElementById('frame'),canvas=document.getElementById('view'),ctx=canvas.getContext('2d');cases.forEach(x=>cs.add(new Option(x,x)));[.05,.025,.0125,.00625,.003125].forEach(x=>hs.add(new Option(x,x)));hs.value='0.003125';let selected=[];function choose(){const [kind,field]=cs.value.split(' / ');selected=rows.filter(x=>x.kind===kind&&x.field===field&&x.h===Number(hs.value));slider.max=selected.length-1;slider.value=0;paint()}function color(w){const t=Math.max(0,Math.min(1,(w+.25)/.875));return `rgb(${Math.round(42+195*t)},${Math.round(112-34*t)},${Math.round(195-130*t)})`}function paint(){const s=selected[Number(slider.value)];ctx.clearRect(0,0,1000,540);const xy=p=>[70+720*p[0],485-340*p[1]];s.triangles.forEach(tri=>{let w=0;ctx.beginPath();tri.forEach((n,i)=>{const p=xy(s.positions[n]);i?ctx.lineTo(...p):ctx.moveTo(...p);w+=s.velocity[s.periodic_indices[n]][2]/3});ctx.closePath();ctx.fillStyle=color(w);ctx.fill();ctx.strokeStyle='#364859';ctx.lineWidth=.6;ctx.stroke()});ctx.fillStyle='#172a3b';ctx.font='16px system-ui';ctx.fillText('x →  (period 1)',370,525);ctx.fillText('y ↑',15,35);ctx.fillText('z periodic; no z variation',680,35);const energy=s.mass.reduce((a,m,i)=>a+.5*m*s.velocity[i].reduce((b,u)=>b+u*u,0),0);const momentum=s.mass.reduce((a,m,i)=>a+m*s.velocity[i][2],0);document.getElementById('info').textContent=`Time ${s.time.toFixed(5)} · actual stamp ${s.stamp.id}:${s.stamp.version} · published total energy ${energy.toPrecision(8)} · third momentum ${momentum.toPrecision(8)}`}cs.onchange=hs.onchange=choose;slider.oninput=paint;let timer;document.getElementById('play').onclick=function(){if(timer){clearInterval(timer);timer=null;this.textContent='Play'}else{this.textContent='Pause';timer=setInterval(()=>{slider.value=(Number(slider.value)+1)%selected.length;paint()},180)}};choose();</script></html>'''
(out/'published-third.html').write_text(html.replace('DATA',json.dumps(rows,separators=(',',':'))))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
(out/'receipt.json').write_text(json.dumps(dict(status='RENDERED_FROM_ACTUAL_SCHEMA_VERIFIED_PUBLICATIONS',actual_publications=len(rows),animated_states=len(frames),native_input_sha256=sha(source),renderer_sha256=sha(Path(__file__)),file_sha256={f.name:sha(f)for f in sorted(out.iterdir())},physics_replay_not_replaced_by_render=True,no_second_simulator=True),indent=2)+'\n')
print('PASS actual-state PNG/PDF/GIF and recorded-state HTML generated')
