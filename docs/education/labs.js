import * as THREE from 'three';
import {RecordedView} from './shared-viewer/view.js';
import {createTransportLabs} from './transport_labs.js';
const data=await (await fetch(new URL('./reference-data.json', import.meta.url))).json();
const host=document.querySelector('#scene'), selector=document.querySelector('#lab');
const panel=document.querySelector('#controls'), metrics=document.querySelector('#metrics');
let view, renderer, orbit, reference=null;
let scene=new THREE.Scene(), camera=new THREE.PerspectiveCamera(42,1,.01,100), world=new THREE.Group();
try {
  view=new RecordedView(host); ({scene,camera,renderer}=view); orbit=view.controls; world=view.group;
  // Page scrolling always belongs to the reader. Explicit controls own zoom.
  orbit.enableZoom=false; orbit.enablePan=false;
  document.querySelector('#render-status').textContent='Drag / arrows to rotate';
} catch(error) {
  document.querySelector('#render-status').textContent='3D unavailable; reference values remain usable.';
  host.textContent='WebGL unavailable. Use the chapter figures and numerical comparisons.';
}
function reset(){camera.position.set(1.65,1.25,2.1);camera.lookAt(0,0,0);if(orbit){orbit.target.set(0,0,0);orbit.update();}}
reset();document.querySelector('#reset-view').onclick=reset;
host.addEventListener('keydown',e=>{if(!orbit||!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(e.key))return;e.preventDefault();const a=(e.key==='ArrowLeft'||e.key==='ArrowUp'?.08:-.08);const axis=e.key.includes('Left')||e.key.includes('Right')?new THREE.Vector3(0,1,0):new THREE.Vector3(1,0,0);camera.position.applyAxisAngle(axis,a);orbit.update();});
for (const [id,scale] of [['zoom-in',.85],['zoom-out',1.18]]) document.querySelector('#'+id)?.addEventListener('click',()=>{
  if(!orbit)return;
  const offset=camera.position.clone().sub(orbit.target);offset.setLength(Math.max(.7,Math.min(12,offset.length()*scale)));
  camera.position.copy(orbit.target).add(offset);orbit.update();
});
const colors={teal:0x067d91,warm:0xaa5735,blue:0x4e98bb,ink:0x18363d};
function clear(){for(const c of [...world.children]){world.remove(c);c.traverse(o=>{o.geometry?.dispose();if(o.material){for(const m of(Array.isArray(o.material)?o.material:[o.material]))m.dispose();}});}}
function line(points,color=colors.ink){const geometry=new THREE.BufferGeometry().setFromPoints(points.map(p=>new THREE.Vector3(...p)));const object=new THREE.Line(geometry,new THREE.LineBasicMaterial({color}));world.add(object);return object;}
function dot(p,color,r=.025){const mesh=new THREE.Mesh(new THREE.SphereGeometry(r,12,8),new THREE.MeshStandardMaterial({color}));mesh.position.set(...p);world.add(mesh);}
function arrow(p,v,color=colors.teal){const length=Math.hypot(...v);if(length<1e-8)return;world.add(new THREE.ArrowHelper(new THREE.Vector3(...v).normalize(),new THREE.Vector3(...p),length,color,Math.min(.07,length*.35),Math.min(.04,length*.2)));}
function box(size,p,color,opacity=.22){const geometry=new THREE.BoxGeometry(...size);const mesh=new THREE.Mesh(geometry,new THREE.MeshStandardMaterial({color,transparent:true,opacity,depthWrite:false}));mesh.position.set(...p);world.add(mesh);const edge=new THREE.LineSegments(new THREE.EdgesGeometry(geometry),new THREE.LineBasicMaterial({color}));edge.position.set(...p);world.add(edge);}
function floor(){const grid=new THREE.GridHelper(2.2,12,0xb3c7c4,0xd1ddda);grid.position.y=-.55;world.add(grid);}
function setMetrics(values){metrics.replaceChildren();for(const [name,value] of values){const dt=document.createElement('dt'),dd=document.createElement('dd');dt.textContent=name;dd.textContent=value;metrics.append(dt,dd);}}
function range(name,max,value,onchange){const label=document.createElement('label');const input=document.createElement('input');input.type='range';input.min=0;input.max=max;input.step=1;input.value=value;input.setAttribute('aria-label',name);label.textContent=name;panel.append(label,input);input.oninput=()=>{onchange(Number(input.value));manualChange();};return input;}
function choice(name,labels,index,onchange){const label=document.createElement('label');label.textContent=name;const select=document.createElement('select');select.setAttribute('aria-label',name);labels.forEach((label,i)=>select.add(new Option(label,i)));select.value=index;panel.append(label,select);select.onchange=()=>{onchange(Number(select.value));manualChange();};return select;}
function meta(title,description,limit,chapter){document.querySelector('#lab-title').textContent=title;document.querySelector('#lab-description').textContent=description;document.querySelector('#lab-limit').textContent=limit;document.querySelector('#chapter-target').href=new URL(`chapters/${chapter}.html`,import.meta.url).href;document.querySelector('#lab-expected').textContent=expectations[selector.value];}
const fmt=(x,n=5)=>Number(x).toPrecision(n);
function projection(){const p=data.projection;meta('1 · A finite MAC projection','Blend the original and pressure-corrected saved face fields. This slider is an algebraic correction fraction, not a solver iteration count.','Unit-weight finite graph reference. Intermediate blends illustrate the correction; they are not accepted simulation steps.','04-pressure-projection');function draw(a){clear();floor();box([1.15,1.15,1.15],[0,0,0],colors.teal,.05);const u=p.before.map((v,i)=>v*(1-a)+p.after[i]*a);p.cells.forEach((q,i)=>dot(q,Math.abs((1-a)*p.divergence_before[i]+a*p.divergence_after[i])>.1?colors.warm:colors.teal));p.positions.forEach((q,i)=>{const v=[0,0,0];v[p.axes[i]]=u[i]*.16;arrow(q,v,u[i]>0?colors.teal:colors.warm);});setMetrics([['Correction fraction',`${Math.round(a*100)}%`],['Max incidence',fmt(Math.max(...p.divergence_before.map((v,i)=>Math.abs(v*(1-a)+p.divergence_after[i]*a))))],['Graph kinetic norm',fmt(.5*u.reduce((s,v)=>s+v*v,0))],['Incidence / initial',fmt(Math.max(...p.divergence_before.map((v,i)=>Math.abs(v*(1-a)+p.divergence_after[i]*a)))/Math.max(...p.divergence_before.map(Math.abs))) ],['Full solve residual',fmt(p.residual_max)],['Pressure gauge','cell 0 = 0']]);}range('Pressure correction',100,0,v=>draw(v/100));draw(0);}
function collision(){const d=data.collision;meta('2 · The path hits before the endpoint','A segment can cross a closed mesh and finish outside. The selected record uses the earliest triangle hit.','Static mesh poses, query tolerance 1e-12; the wall-speed arrow is metadata, not a swept moving-boundary solve. The separate Lean first-hit/clipping proof covers strict crossings of an infinite stationary plane; it does not certify these mesh queries or cut cells.','20-collision-mesh-pipeline');function draw(i){clear();floor();const q=d.cases[i],vertices=d.vertices.map(p=>[p[0]+q.shift,p[1],p[2]]);const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(d.faces.flatMap(f=>f.flatMap(j=>vertices[j])),3));g.computeVertexNormals();world.add(new THREE.Mesh(g,new THREE.MeshStandardMaterial({color:colors.teal,transparent:true,opacity:.45,side:THREE.DoubleSide})));world.add(new THREE.LineSegments(new THREE.EdgesGeometry(g),new THREE.LineBasicMaterial({color:colors.teal})));line([q.start,q.end],colors.warm);dot(q.start,colors.ink);dot(q.end,colors.ink);if(q.hit){dot(q.hit,colors.warm,.045);arrow(q.hit,q.wall_velocity,colors.teal);}setMetrics([['Trace',q.label],['Mesh translation',`${q.shift} m`],['Earliest hit t',q.t===null?'no intersection':fmt(q.t)],['Wall-speed annotation','0.2 m/s in +x'],['Query tolerance','1e−12'],['Start / end outside',q.label==='inside'?'starts inside':'both outside']]);}choice('Path and boundary pose',d.cases.map(q=>`${q.label} · shift ${q.shift} m`),3,draw);draw(3);}
function hydrostatic(){const d=data.hydrostatic;meta('3 · Density changes pressure','Two half-metre layers at rest under gravity. Bars encode pressure at stored heights; the surface pressure is zero.','Piecewise analytic hydrostatic oracle with independently checked discrete face increments. No two-phase transport or stratification solve.','21-density-and-liquid-volume');function draw(i){clear();floor();const q=d.sets[i];box([.7,.5,.7],[0,-.25,0],colors.teal,.3);box([.7,.5,.7],[0,.25,0],colors.blue,.18);q.y.forEach((y,j)=>{if(j%2===0)arrow([.42,y-.5,.2],[q.pressure[j]/Math.max(...d.sets.flatMap(s=>s.pressure))*.5,0,0],colors.warm);});setMetrics([['Lower density',`${q.lower_density} kg/m³`],['Upper density',`${q.upper_density} kg/m³`],['Bottom gauge pressure',`${fmt(q.pressure[0])} Pa`],['Gravity',`${d.gravity} m/s²`],['Analytic bottom pressure',`${fmt(d.gravity*.5*(q.lower_density+q.upper_density))} Pa`],['Face balance error',fmt(q.balance_error)]]);}choice('Layer densities',d.sets.map(q=>`${q.lower_density} / ${q.upper_density} kg/m³`),0,draw);draw(0);}
function viscous(){const d=data.viscous;let selected=0,step=0;meta('4 · Viscosity dissipates the mode','Stored dense implicit solves agree with the discrete sine eigenmode. Compare viscosity and time without changing the mesh.','Periodic scalar shear-mode oracle extruded in 3D. The finite-strain Lean bridge derives unforced dissipation from exact update equations; it does not certify this stencil or a full free-surface solver.','22-viscosity-and-stress');function draw(){clear();floor();const q=d.sets[selected].states[step];for(let z of [-.35,0,.35])d.x.forEach((y,i)=>arrow([-.25,y-.5,z],[q.velocity[i]*.4,0,0],q.velocity[i]>0?colors.teal:colors.warm));setMetrics([['Kinematic viscosity',`${d.sets[selected].nu} m²/s`],['Time',`${q.time.toFixed(2)} s`],['Discrete amplitude',fmt(q.amplitude)],['Continuum amplitude',fmt(q.exact_continuum_amplitude)],['Kinetic energy',fmt(q.energy)],['Amplitude gap',fmt(Math.abs(q.amplitude-q.exact_continuum_amplitude))]]);}choice('Kinematic viscosity',d.sets.map(q=>`${q.nu} m²/s`),0,v=>{selected=v;draw();});range('Stored reference time step',100,0,v=>{step=v;draw();});draw();}
function slip(){const d=data.slip;meta('5 · Slip changes traction','The upper wall moves at 1 m/s; the lower wall stays stationary. Change the lower Navier slip length.','Analytic steady Couette oracle with one slipping and one no-slip wall. Illustrative lengths are not material calibration.','24-slip-and-wall-friction');function draw(i){clear();floor();const q=d.sets[i];box([1.4,.03,.8],[0,-.5,0],colors.ink,.6);box([1.4,.03,.8],[0,.5,0],colors.ink,.6);for(let z of [-.3,0,.3])d.y.forEach((y,j)=>{if(j%2===0)arrow([-.3,y-.5,z],[q.velocity[j]*.5,0,0],colors.teal);});setMetrics([['Slip length',`${q.length} m`],['Fluid slip speed',`${fmt(q.velocity[0])} m/s`],['Lower-wall speed','0 m/s'],['Wall shear',`${fmt(q.shear)} Pa`],['Wall dissipation',`${fmt(q.relative_wall_dissipation)} W/m²`],['Wetting angle','independent']]);}choice('Lower-wall slip length',d.sets.map(q=>`${q.length} m${q.length===0?' · no-slip':''}`),0,draw);draw(0);}
function cap(){const d=data.cap;let angle=12,tension=1;meta('6 · Wetting shape and capillary pressure','Change contact angle at fixed volume, then change tension. The tension changes pressure and adhesion work, while the zero-gravity cap geometry stays fixed.','Analytic equilibrium spherical cap, negligible gravity. No dynamic contact line, hysteresis, relaxation or liquid simulation is claimed.','23-wetting-and-adhesion');function draw(){clear();floor();const q=d.sets[angle],s=q.tensions[tension],scale=50;const geometry=new THREE.LatheGeometry(q.profile.map(p=>new THREE.Vector2(p[0]*scale,p[1]*scale)),64);const mesh=new THREE.Mesh(geometry,new THREE.MeshPhysicalMaterial({color:0x239bb5,roughness:.22,metalness:0,transparent:true,opacity:.72,side:THREE.DoubleSide}));mesh.position.y=-.5;world.add(mesh);const circle=new THREE.Mesh(new THREE.CircleGeometry(q.contact_radius*scale,64),new THREE.MeshStandardMaterial({color:colors.teal,transparent:true,opacity:.6,side:THREE.DoubleSide}));circle.rotation.x=-Math.PI/2;circle.position.y=-.499;world.add(circle);box([2,.02,2],[0,-.52,0],colors.ink,.25);setMetrics([['Contact angle',`${q.theta}°`],['Liquid volume',`${fmt(q.volume*1e6)} mL`],['Contact radius',`${fmt(q.contact_radius*1000)} mm`],['Surface tension',`${s.sigma} N/m`],['Pressure jump',`${fmt(s.pressure_jump)} Pa`],['Adhesion work',`${fmt(s.adhesion_work)} J/m²`],['Volume integral error',`${fmt(Math.abs(q.integrated_volume-q.volume))} m³`]]);}range('Equilibrium contact angle (30–150 degrees)',24,12,v=>{angle=v;draw();});choice('Surface tension',d.sets[0].tensions.map(q=>`${q.sigma} N/m`),1,v=>{tension=v;draw();});draw();}
const expectations={
 projection:'Compare 0%, 50% and 100%: incidence falls linearly with the correction; the graph kinetic norm falls. The blend is not a solver iteration.',
 collision:'A path may have both endpoints outside and still hit. Compare crossing with near miss; a miss has no hit marker.',
 hydrostatic:'Increasing either layer density increases bottom pressure. All arrow lengths use one fixed scale; compare the formula and face-balance error.',
 viscous:'Increasing saved time damps the arrows and energy. The amplitude gap measures the difference from continuum decay at this fixed resolution.',
 slip:'Larger lower slip length increases fluid slip speed and reduces shear. The stationary lower wall itself remains at 0 m/s.',
 cap:'At fixed angle, changing tension changes pressure but not radius or volume. Changing angle changes shape; inspect the volume integral error.',
 sampler:'Matching offsets reproduce the affine value to roundoff. For x-face data at the fixture point, using the cell-centre offset gives a −0.07 bias despite nonnegative weights that sum to one.',
 transport:'The symmetric six-sample case at Courant 0.5 overshoots to −0.125 and 1.125 before two fallbacks. The asymmetric case at 0.25 restores bounds but changes the sum from 1.6 to 1.61875.'
};
const labs={projection,collision,hydrostatic,viscous,slip,cap,...createTransportLabs({clear,line,dot,arrow,range,choice,meta,setMetrics,panel,setReference:q=>{reference=q;}})};
let applying=false;
function manualChange(){if(!applying)host.dispatchEvent(new CustomEvent('rheon-manual-change',{bubbles:true}));}
export function selectLab(lab=selector.value){
  if(!labs[lab]||![...selector.options].some(o=>o.value===lab))return;selector.value=lab;panel.replaceChildren();reference=null;labs[lab]();
  const recomputed=lab==='sampler'||lab==='transport';
  const legend=document.querySelector('#plot-legend');
  if(legend){
    legend.hidden=!recomputed;legend.replaceChildren();
    for(const [label,color] of lab==='transport'?[['Linear','#067d91'],['Raw','#aa5735'],['Limited','#4e98bb'],['Exact','#18363d']]:[['Query','#aa5735'],['Weighted point / weights','#067d91']]){
      const item=document.createElement('span');item.textContent=label;item.style.borderColor=color;legend.append(item);
    }
  }
  document.querySelector('.example-kind').textContent=recomputed?'Browser recomputation · educational reference':'Reference model · no fluid timesteps';
  document.querySelector('#reference-sources').hidden=recomputed;
  document.querySelector('#transport-sources').hidden=!recomputed;
}
const presets={
 projection:{before:[0],half:[50],after:[100]},
 collision:{crossing:[3],miss:[4]},
 hydrostatic:{light:[0],heavy:[1]},
 viscous:{initial:[2,0],late:[2,100]},
 slip:{'no-slip':[0],slip:[2]},
 cap:{neutral:[12,1],spread:[0,1],tension:[12,2]},
 sampler:{correct:[1,0,50],wrong:[1,1,50]},
 transport:{symmetric:[2,1,50,1],asymmetric:[3,1,25,1],smooth:[0,1,50,128]}
};
export function setLesson(lab,preset){
 applying=true;
 try{selectLab(lab);const values=presets[lab]?.[preset]||[];
  panel.querySelectorAll('select,input').forEach((control,i)=>{if(values[i]===undefined)return;control.value=values[i];control.dispatchEvent(new Event(control.tagName==='SELECT'?'change':'input'));});
 }finally{applying=false;}
}
// Read-only inspection for the independent browser comparisons.
export function snapshot(){return {lab:selector.value,camera:camera.position.toArray(),target:orbit?.target.toArray(),webgl:!!renderer,reference,referenceLines:world.children.filter(o=>o.userData.series).map(o=>({series:o.userData.series,points:Array.from({length:o.geometry.attributes.position.count},(_,i)=>Array.from(o.geometry.attributes.position.array.slice(i*3,i*3+3)))})),arrows:world.children.filter(o=>o.type==='ArrowHelper').map(o=>({origin:o.position.toArray(),length:o.line.scale.y+o.cone.scale.y,direction:new THREE.Vector3(0,1,0).applyQuaternion(o.quaternion).toArray()}))};}
selector.addEventListener('change',()=>{selectLab();manualChange();});
selectLab([...selector.options].some(o=>o.value===location.hash.slice(1))?location.hash.slice(1):selector.options[0].value);
