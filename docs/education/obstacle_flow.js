// Playback of qualified native records only. No browser fluid solver.
const $ = id => document.getElementById(id);
const ns = 'http://www.w3.org/2000/svg';
const number = x => Number(x).toPrecision(6);
function svg(root, name, attrs, content) {
  const node = document.createElementNS(ns, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, String(value));
  if (content !== undefined) node.textContent = content;
  root.appendChild(node); return node;
}
function metrics(id, values) {
  $(id).replaceChildren();
  for (const [label, value] of values) {
    const dt = document.createElement('dt'); const dd = document.createElement('dd');
    dt.textContent = label; dd.textContent = value; $(id).append(dt, dd);
  }
}
function fill(id, cases) { for (const [i,c] of cases.entries()) { const o=document.createElement('option');o.value=String(i);o.textContent=c.id;$(id).append(o); } }
function pressure(records) {
  const c=records.pressure_cases[Number($('flow-pressure-case').value)];
  const z=Number($('flow-slice').value); const state=$('flow-state').value;const root=$('flow-pressure-view'); root.replaceChildren();
  const cell=100, x0=100, y0=340, max=Math.max(1,...c.pressure.map(Math.abs));
  for(let j=0;j<3;j++) for(let i=0;i<3;i++) {
    const index=i+3*(j+3*z), p=c.pressure[index], dry=c.volumes[index]===0;
    svg(root,'rect',{x:x0+i*cell,y:y0-(j+1)*cell,width:cell,height:cell,fill:dry?'#595e68':`hsl(${215-60*p/max} 62% ${85-30*Math.abs(p)/max}%)`,stroke:'#2f405a'});
    svg(root,'text',{x:x0+(i+.5)*cell,y:y0-(j+.58)*cell,'text-anchor':'middle',fill:dry?'white':'#14203c','font-size':14},dry?'Solid':`${number(p)} Pa`);
    svg(root,'text',{x:x0+(i+.5)*cell,y:y0-(j+.30)*cell,'text-anchor':'middle',fill:dry?'white':'#14203c','font-size':12},`V ${number(c.volumes[index])}`);
  }
  const arrow=(x,y,dx,dy)=>{ if(Math.hypot(dx,dy)<1e-8)return;svg(root,'line',{x1:x,y1:y,x2:x+dx,y2:y+dy,stroke:'#bd1d36','stroke-width':3});const norm=Math.hypot(dx,dy),ux=dx/norm,uy=dy/norm;svg(root,'path',{d:`M ${x+dx-7*ux+4*uy} ${y+dy-7*uy-4*ux} L ${x+dx} ${y+dy} L ${x+dx-7*ux-4*uy} ${y+dy-7*uy+4*ux}`,fill:'none',stroke:'#bd1d36','stroke-width':2});};
  for(let j=0;j<3;j++)for(let i=1;i<3;i++)arrow(x0+i*cell,y0-(j+.5)*cell,35*c[state][0][i+4*(j+3*z)],0);
  for(let j=1;j<3;j++)for(let i=0;i<3;i++)arrow(x0+(i+.5)*cell,y0-j*cell,0,-35*c[state][1][i+3*(j+4*z)]);
  svg(root,'text',{x:100,y:380,fill:'currentColor','font-size':13},`Z center ${z+.5} m · arrows: ${state} in-plane speeds; fixed scale`);
  const l=c.ledger;
  metrics('flow-pressure-metrics', [['Geometry identity',`${c.stamp[0]}:${c.stamp[1]}`],['Components / pressure gauges',`${c.components} / ${c.gauges.join(', ')}`],['dt / density',`${c.dt} s / ${c.density} kg/m³`],['True integrated residual max',`${number(c.residual[1])} m³/s²`],['Predicted / actual divergence max',`${number(c.residual[2])} / ${number(l[0])} s⁻¹`],['Face-area kinetic energy, before → after',`${number(l[1])} → ${number(l[2])} J`],['Correction energy / residual work',`${number(l[3])} / ${number(l[4])} J`],['Pressure iterations',String(c.iterations)]]);
  root.dataset.case=c.id;root.dataset.state=state;root.dataset.slice=String(z);
}
function shear(records) {
  const c=records.shear_cases[Number($('flow-shear-case').value)];const step=Number($('flow-step').value),f=c.frames[step],root=$('flow-shear-view');root.replaceChildren();
  const wall=c.upper[1], top=c.counts[1], max=Math.max(.01,...c.frames.flatMap(f=>f.velocity.map(Math.abs)))*1.1;
  const x=v=>90+420*v/max,y=h=>290-240*(h-wall)/(top-wall);
  svg(root,'rect',{x:60,y:290,width:490,height:30,fill:'#595e68'});svg(root,'line',{x1:60,y1:50,x2:550,y2:50,stroke:'#595e68','stroke-width':7});
  svg(root,'line',{x1:90,y1:50,x2:90,y2:290,stroke:'#526682'});
  const path=f.velocity.map((v,i)=>`${i?'L':'M'} ${x(v)} ${y(c.centers[i])}`).join(' ');
  if(path)svg(root,'path',{d:path,fill:'none',stroke:'#1565c0','stroke-width':3});
  f.velocity.forEach((v,i)=>{svg(root,'circle',{cx:x(v),cy:y(c.centers[i]),r:5,fill:'#1565c0'});svg(root,'text',{x:x(v)+9,y:y(c.centers[i])+4,'font-size':12,fill:'currentColor'},number(v));});
  svg(root,'text',{x:70,y:341,'font-size':13,fill:'currentColor'},`uₜ (m/s) → · lower wall y=${wall} m · saved t=${number(step*c.dt)} s`);
  const e=f.energy,m=f.momentum;
  metrics('flow-shear-metrics',[['Geometry identity',`${c.stamp[0]}:${c.stamp[1]}`],['Boundary model',c.beta===null?'Stationary no-slip':'Stationary Navier β = '+c.beta+' Pa s/m'],['Density / dynamic viscosity',`${c.density} kg/m³ / ${c.viscosity} Pa s`],['Force input',`${c.force_value} ${c.force_kind==='density'?'N/m³':'m/s²'}`],['Saved step / dt',`${step} / ${c.dt} s`],['Kinetic energy',e?number(e[1])+' J':'0 J'],['Body work / viscous dissipation',e?`${number(e[3])} / ${number(e[4])} J`:'Initial field'],['Body impulse / wall impulse',m?`${number(m[2])} / ${number(m[3])} N s`:'Initial field'],['True momentum residual max',f.residual_max===undefined?'Initial field':number(f.residual_max)+' N']]);
  root.dataset.case=c.id;root.dataset.step=String(step);
}
async function main() {
  const response=await fetch('obstacle-flow-records.json');if(!response.ok)throw new Error(`Native field HTTP ${response.status}`);const records=await response.json();
  if(records.schema!=='rheon-obstacle-flow-records-v1')throw new Error('Unsupported native record schema');
  fill('flow-pressure-case',records.pressure_cases);fill('flow-shear-case',records.shear_cases);
  for(const id of ['flow-pressure-case','flow-state','flow-slice'])$(id).addEventListener('change',()=>pressure(records));
  $('flow-shear-case').addEventListener('change',()=>shear(records));$('flow-step').addEventListener('input',()=>shear(records));
  pressure(records);shear(records);$('flow-status').textContent='4 native pressure projections and 48 reduced-shear updates. Independent rational controls qualified the displayed fields.';
  window.__obstacleFlow={records,pressure:()=>pressure(records),shear:()=>shear(records)};
}
main().catch(error=>{$('flow-status').textContent='Native fields unavailable: '+error.message;console.error(error);});
