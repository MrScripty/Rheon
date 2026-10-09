import {test} from 'node:test';
import assert from 'node:assert/strict';
import {adaptRecording, KENOMA_COMMIT} from '../adapters.js';
// Small structural inputs test refusal/conversion, never physical qualification.
const frame = time_s => ({time_s,center_of_mass:[0,0,0],orientation:[1,0,0,0],velocity_m_s:[1,2,3],angular_velocity_rad_s:[0,0,1],vertices:Array.from({length:8},(_,i)=>[i%2,Math.floor(i/2)%2,Math.floor(i/4)])});
const rigid = () => ({mode:'free',mass_kg:2,spherical_inertia_kg_m2:1,triangles:Array.from({length:12},()=>[0,1,2]),initial:frame(0),steps:[{stored:frame(.125),force_n:[1,2,3],torque_n_m:[0,1,0],kinetic_after_j:1,energy_defect_j:0}]});
test('recorded geometry, load provenance, cloning and immutable display',()=>{
  const raw=rigid(), before=JSON.stringify(raw), out=adaptRecording(raw,{name:'structural test'});
  assert.deepEqual(out.cases[0].frames[1].vertices,raw.steps[0].stored.vertices);
  assert.deepEqual(out.cases[0].frames[1].force.origin,raw.initial.center_of_mass);
  assert.equal(out.cases[0].frames[0].force,undefined);
  assert.equal(JSON.stringify(raw),before);assert(!Object.isFrozen(raw));
  assert(Object.isFrozen(out.cases[0].frames[1].vertices));
  assert.throws(()=>out.cases[0].frames[1].vertices[0][0]=9,TypeError);
});
test('unsupported versions, nonfinite, topology and time refuse',()=>{
  for(const change of [r=>r.schema='unknown-v2',r=>r.mode='live',r=>r.initial.vertices[0][0]=Infinity,r=>r.triangles[0][0]=8,r=>r.steps[0].stored.time_s=0,r=>r.steps[0].force_n[0]=NaN,r=>r.mass_kg=-1,r=>r.steps=Array(65).fill(r.steps[0])]){
    const raw=rigid();change(raw);assert.throws(()=>adaptRecording(raw,{}));
  }
});
test('sparse vectors, rosters and mesh arrays refuse',()=>{
  for(const change of [r=>delete r.initial.vertices[0],r=>delete r.initial.vertices[0][0],r=>delete r.steps[0].stored.vertices[0],r=>delete r.triangles[0],r=>delete r.steps[0]]){
    const raw=rigid();change(raw);assert.throws(()=>adaptRecording(raw,{}));
  }
});
test('documented initial-only rigid output is supported',()=>{
  const raw=rigid();raw.steps=[];const out=adaptRecording(raw,{});
  assert.equal(out.cases[0].frames.length,1);assert.deepEqual(out.cases[0].frames[0].vertices,raw.initial.vertices);
});
test('native column retains recorded values and ignores reference columns',()=>{
  const raw={cases:[{name:'stored',snapshots:[{time:0,profiles:[[.25,1,99],[.75,2,88]],energy:3,bulk:4}]}]};
  const out=adaptRecording(raw,{});assert.deepEqual(out.cases[0].frames[0].profile,[[.25,1],[.75,2]]);
});
test('versioned shear time is a display product of stored step and dt',()=>{
  const raw={schema:'rheon-obstacle-flow-records-v1',shear_cases:[{id:'saved',dt:.125,centers:[1.75],density:2,viscosity:1,frames:[{step:0,velocity:[0]},{step:1,velocity:[.0625],energy:[0,1,2,3,4,5,6,7]}]}]};
  const out=adaptRecording(raw,{});assert.equal(out.cases[0].frames[1].time,.125);assert.deepEqual(out.cases[0].frames[1].profile,[[1.75,.0625]]);
  raw.shear_cases[0].frames[1].velocity=[];assert.throws(()=>adaptRecording(raw,{}));
  assert.equal(KENOMA_COMMIT,'2eb92a5d0924b5f2cbf310597dbd7b399fab8692');
});
