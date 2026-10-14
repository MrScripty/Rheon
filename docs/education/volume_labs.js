import {sweptVolume} from './volume_reference.js';
export function createVolumeLab({clear,box,arrow,range,choice,meta,setMetrics,setReference}) {
  return function volume(){
    let f=.3,c=.5,side='left';
    meta('Where is the liquid inside the cell?',
      'Top: the donor unit cube before transfer. Teal is liquid; orange is the positive-x swept slab; dark overlap is transferred. Bottom: donor and initially empty receiver amounts after the shared transfer.',
      'Prescribed axis-aligned interface and one face, unit-metre cubes, 0 ≤ swept width ≤ 1. Bottom fill heights encode amounts only, not reconstructed post-step interfaces. No pressure, gravity, general PLIC, or native liquid timestep.',
      '09-liquid-surfaces');
    function draw(){
      clear();const q=sweptVolume(f,c,side);setReference(q);
      // Common geometric scale: one metre = 0.7 scene units in all axes.
      const s=.7;
      const slab=(a,b,y,color,opacity)=>{if(b>a)box([(b-a)*s,s,s],[(a+b-1)*s/2,y,0],color,opacity);};
      box([s,s,s],[0,.55,0],0x18363d,.025);
      slab(q.lo,q.hi,.55,0x067d91,.3);
      slab(1-c,1,.55,0xc18442,.16);
      slab(q.overlapLo,q.overlapHi,.55,0x18363d,.6);
      arrow([-.48,1,0],[.96,0,0],0xaa5735);
      for(const [x,amount,color] of [[-.4,q.donor,0x067d91],[.4,q.receiver,0x4e98bb]]){
        box([s,s,s],[x,-.55,0],0x18363d,.025);
        if(amount>0)box([s,amount*s,s],[x,-.55-(1-amount)*s/2,0],color,.4);
      }
      const n=x=>x.toFixed(6);
      setMetrics([['Liquid fraction f',n(f)],['Swept width c = u Δt / h',n(c)],
        ['Geometric transfer Q',`${n(q.q)} m³`],['Donor / receiver after',`${n(q.donor)} / ${n(q.receiver)} m³`],
        ['Shared-flux total change',`${n(q.donor+q.receiver-f)} m³`],
        ['Uniform-mixture estimate c f',`${n(q.mixed)} m³`],
        ['Incorrect full-slab donor / receiver',`${n(q.raw[0])} / ${n(q.raw[1])} m³`],
        ['After separate clamping',`${n(q.clamped[0])} / ${n(q.clamped[1])} m³`],
        ['Clamping volume change',`${n(q.clampDrift)} m³`]]);
    }
    choice('Liquid location', ['Left of interface','Right of interface'],0,v=>{side=v?'right':'left';draw();});
    range('Initial liquid fraction (%)',100,30,v=>{f=v/100;draw();});
    range('Swept width (% of cell)',100,50,v=>{c=v/100;draw();});
    draw();
  };
}
