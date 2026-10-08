import * as THREE from 'three';
import {OrbitControls} from 'three/addons/controls/OrbitControls.js';

const data=await (await fetch('obstacle-records.json')).json();
const host=document.querySelector('#obstacle-scene');
host.style.cssText='height:560px;min-width:0;touch-action:none';
const renderer=new THREE.WebGLRenderer({antialias:true});
renderer.setPixelRatio(Math.min(devicePixelRatio,2));host.append(renderer.domElement);
renderer.setClearColor(0xf0f4f6);
const scene=new THREE.Scene();
const camera=new THREE.PerspectiveCamera(42,1,0.01,1000);
const controls=new OrbitControls(camera,renderer.domElement);controls.enableDamping=true;
const group=new THREE.Group();scene.add(group);
scene.add(new THREE.AmbientLight(0xffffff,2));
const light=new THREE.DirectionalLight(0xffffff,3);light.position.set(5,7,8);scene.add(light);
const select=document.querySelector('#obstacle-case'),cell=document.querySelector('#obstacle-cell'),axis=document.querySelector('#obstacle-axis');
for(const c of data.cases)select.add(new Option(c.id.replaceAll('-',' '),c.id));
for(const r of data.refusals){const li=document.createElement('li');li.textContent=`${'XYZ'[r.axis]} separator ${r.lower[r.axis]}–${r.upper[r.axis]} m: unresolved cell topology; refused.`;document.querySelector('#obstacle-refusals').append(li);}
let current, center, extent;
const number=x=>x.toPrecision(7);
const position=(c,p)=>p.map((n,d)=>c.origin[d]+n*c.spacing[d]);
const coordinates=(c,i)=>[i%c.counts[0],Math.floor(i/c.counts[0])%c.counts[1],Math.floor(i/(c.counts[0]*c.counts[1]))];
function box(lo,hi,color,opacity=1,wire=false){
  const geometry=new THREE.BoxGeometry(...hi.map((x,d)=>x-lo[d]));
  const mesh=wire?new THREE.LineSegments(new THREE.EdgesGeometry(geometry),new THREE.LineBasicMaterial({color})):
    new THREE.Mesh(geometry,new THREE.MeshBasicMaterial({color,transparent:opacity<1,opacity,depthWrite:opacity===1}));
  if(wire)geometry.dispose();mesh.position.set(...lo.map((x,d)=>(x+hi[d])/2));group.add(mesh);
}
function line(a,b,color){const g=new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(...a),new THREE.Vector3(...b)]);group.add(new THREE.Line(g,new THREE.LineBasicMaterial({color})));}
function face(c,d){
  const p=[1,1,1],lo=position(c,p),hi=position(c,p.map((n,a)=>n+(a===d?0:1)));
  const tangents=[0,1,2].filter(a=>a!==d),corners=[];
  for(const bits of [[0,0],[1,0],[1,1],[0,1]]){const x=lo.slice();tangents.forEach((a,i)=>x[a]=bits[i]?hi[a]:lo[a]);corners.push(x);}
  const geometry=new THREE.BufferGeometry();geometry.setAttribute('position',new THREE.Float32BufferAttribute(corners.flat(),3));geometry.setIndex([0,1,2,0,2,3]);
  group.add(new THREE.Mesh(geometry,new THREE.MeshBasicMaterial({color:0x00a6b8,side:THREE.DoubleSide,transparent:true,opacity:.45,depthWrite:false})));
  for(let i=0;i<4;i++)line(corners[i],corners[(i+1)%4],0x007b8d);
}
function reset(){camera.position.copy(center).add(new THREE.Vector3(1.5,1.1,1.6).multiplyScalar(extent));camera.near=extent/1000;camera.far=extent*100;camera.updateProjectionMatrix();controls.target.copy(center);controls.update();}
function draw(){
  for(const child of [...group.children]){group.remove(child);child.geometry?.dispose();child.material?.dispose();}
  const c=current,d=+axis.value,index=+cell.value,p=coordinates(c,index),lo=position(c,p),hi=position(c,p.map(x=>x+1));
  for(let i=0;i<c.volumes.length;i++){
    const q=coordinates(c,i),a=position(c,q),b=position(c,q.map(x=>x+1));
    box(a,b,c.labels[i]===null?0xa6adb2:c.labels[i]===0?0x326dbe:0x1b9672,.12);
    box(a,b,0xa8bac5,1,true);
  }
  const geometry=new THREE.BufferGeometry();geometry.setAttribute('position',new THREE.Float32BufferAttribute(c.vertices.flat(),3));geometry.setIndex(c.triangles.flat());
  group.add(new THREE.Mesh(geometry,new THREE.MeshBasicMaterial({color:0xc53d45,side:THREE.DoubleSide,transparent:true,opacity:.55,depthWrite:false})));
  box(c.lower,c.upper,0x9c2638,1,true);box(lo,hi,0xe09b10,1,true);face(c,d);
  const probe=c.probes[d];line(probe.start,probe.end,0x512572);
  const dot=new THREE.Mesh(new THREE.SphereGeometry(extent*.017,16,12),new THREE.MeshBasicMaterial({color:0xf2b82b}));dot.position.set(...probe.position);group.add(dot);
  const full=hi.reduce((v,x,a)=>v*(x-lo[a]),1),flux=c.flux_controls[d];
  const metrics=[['Selected cell (i,j,k)',p.join(', ')],['Fluid volume / cell volume',`${number(c.volumes[index])} / ${number(full)} m³`],['Selected cell component',c.labels[index]===null?'dry':String(c.labels[index])],['Connected fluid components',String(c.components)],['Total fluid volume',`${number(c.volumes.reduce((a,b)=>a+b,0))} m³`],['Shared face at (1,1,1)',`${'XYZ'[d]} · ${number(flux.area)} m²`],['Native outward flux pair at 2 m/s',`${number(flux.outward[0])}, ${number(flux.outward[1])} m³/s`],['Pair sum',`${number(flux.outward[0]+flux.outward[1])} m³/s`],['Native first collision t',number(probe.t)],['Hit position',`${probe.position.map(number).join(', ')} m`],['Shared source stamp',c.stamp.join(' / ')]];
  const list=document.querySelector('#obstacle-metrics');list.replaceChildren();
  for(const [label,value]of metrics){const dt=document.createElement('dt'),dd=document.createElement('dd');dt.textContent=label;dd.textContent=value;list.append(dt,dd);}
  document.querySelector('#obstacle-status').textContent=`Recorded geometry: ${c.id}. No fluid advances. Cell boxes indicate connectivity; the red mesh shows the actual solid.`;
}
function choose(){
  current=data.cases.find(c=>c.id===select.value);cell.replaceChildren();
  for(let i=0;i<current.volumes.length;i++)cell.add(new Option(coordinates(current,i).join(', '),i));
  cell.value=String(Math.floor(current.volumes.length/2));
  const lo=current.origin.map((x,d)=>Math.min(x,current.lower[d]));
  const hi=position(current,current.counts).map((x,d)=>Math.max(x,current.upper[d]));
  center=new THREE.Vector3(...lo.map((x,d)=>(x+hi[d])/2));extent=Math.max(...hi.map((x,d)=>x-lo[d]));draw();reset();
}
select.addEventListener('change',choose);cell.addEventListener('change',draw);axis.addEventListener('change',draw);document.querySelector('#obstacle-reset').onclick=reset;
new ResizeObserver(()=>{const w=host.clientWidth,h=host.clientHeight;renderer.setSize(w,h);camera.aspect=w/h;camera.updateProjectionMatrix();}).observe(host);
choose();renderer.setAnimationLoop(()=>{controls.update();renderer.render(scene,camera);});
