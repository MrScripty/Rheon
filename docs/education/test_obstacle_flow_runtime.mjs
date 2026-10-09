// DOM-level playback unit checks. This is not Chromium/layout qualification.
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname,join} from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const here=dirname(fileURLToPath(import.meta.url));
const records=JSON.parse(readFileSync(process.argv[2],'utf8'));
class Element {
  constructor(name){this.name=name;this.children=[];this.attributes={};this.dataset={};this.value='';this.textContent='';this.listeners={};}
  setAttribute(k,v){this.attributes[k]=v;}
  appendChild(n){this.children.push(n);if(this.name==='select'&&this.children.length===1)this.value=n.value;return n;}
  append(...nodes){for(const n of nodes)this.appendChild(n);}
  replaceChildren(){this.children=[];}
  addEventListener(k,f){(this.listeners[k]??=[]).push(f);}
  event(k){for(const f of this.listeners[k]??[])f();}
}
const ids={};for(const id of ['flow-pressure-case','flow-state','flow-slice','flow-shear-case'])ids[id]=new Element('select');
for(const id of ['flow-pressure-view','flow-shear-view'])ids[id]=new Element('svg');
for(const id of ['flow-pressure-metrics','flow-shear-metrics','flow-status','flow-step'])ids[id]=new Element('div');
ids['flow-state'].value='before';ids['flow-slice'].value='0';ids['flow-step'].value='8';
const errors=[];const context={document:{getElementById:id=>{assert.ok(ids[id],id);return ids[id];},createElement:name=>new Element(name),createElementNS:(_,name)=>new Element(name)},fetch:async()=>({ok:true,json:async()=>records}),window:{},console:{error:e=>errors.push(e)}};
vm.runInNewContext(readFileSync(join(here,'obstacle_flow.js'),'utf8'),context);
for(let i=0;i<10&&!context.window.__obstacleFlow;i++)await new Promise(r=>setImmediate(r));
assert.deepEqual(errors,[]);assert.ok(context.window.__obstacleFlow);
const select=(id,value,event='change')=>{ids[id].value=String(value);ids[id].event(event);};
let pressureViews=0,shearFrames=0;
for(const [i,c] of records.pressure_cases.entries()){
 select('flow-pressure-case',i);
 for(const state of ['before','after'])for(let z=0;z<3;z++){
  select('flow-state',state);select('flow-slice',z);const view=ids['flow-pressure-view'];
  assert.equal(view.dataset.case,c.id);assert.equal(view.dataset.state,state);assert.equal(view.dataset.slice,String(z));
  assert.equal(view.children.filter(n=>n.name==='rect').length,9);
  const text=view.children.filter(n=>n.name==='text').map(n=>n.textContent);
  for(let j=0;j<3;j++)for(let x=0;x<3;x++){const cell=x+3*(j+3*z);assert.equal(text[2*(x+3*j)+1],`V ${Number(c.volumes[cell]).toPrecision(6)}`);}
  if(state==='after')assert.equal(view.children.filter(n=>n.name==='line').length,0);
  pressureViews++;
 }
}
for(const [i,c] of records.shear_cases.entries()){
 select('flow-shear-case',i);
 for(const frame of c.frames){select('flow-step',frame.step,'input');const view=ids['flow-shear-view'];assert.equal(view.dataset.case,c.id);assert.equal(view.dataset.step,String(frame.step));assert.equal(view.children.filter(n=>n.name==='circle').length,frame.velocity.length);
  assert.deepEqual(view.children.filter(n=>n.name==='text').slice(0,frame.velocity.length).map(n=>n.textContent),frame.velocity.map(v=>Number(v).toPrecision(6)));shearFrames++;
 }
}
assert.deepEqual(errors,[]);console.log(JSON.stringify({scope:'DOM playback unit tests, not browser/layout',pressureViews,shearFrames}));
