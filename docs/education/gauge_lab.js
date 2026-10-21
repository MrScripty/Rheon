import {pressureGauge} from './gauge_reference.js';
export function createGaugeLab({clear,box,arrow,choice,meta,setMetrics,setReference}){
  return function gauges(){
    let input='compatible',geometry='unequal',mode='fixed',shiftA=0,shiftB=0,error=0;
    const n=x=>Number(x).toFixed(4),list=a=>a.map(n).join(' / ');
    meta('Compatibility comes before a pressure gauge','Two separated pairs are disconnected all-Neumann regions. Cell widths show volume. Bars along the display depth encode scalar pressure; dark face arrows show pressure-induced velocity correction. Shift either region and compare the unchanged face arrows.','Prescribed finite pressure system, density 1 kg/m³ and dt 1 s, zero initial internal flux. Bars are scalar markers, not force vectors. Incompatible boundary flux remains rejected; projected RHS explicitly changes the source. No production solver or advancing fluid.','05-linear-solvers');
    function draw(){clear();const q=pressureGauge(input,geometry,mode,shiftA,shiftB,error);setReference(q);
      for(let c=0;c<2;c++){
        const i=2*c,y=[-.45,.45][c];
        for(const [j,x] of [[i,-.15*q.volumes[i]],[i+1,.15*q.volumes[i+1]]]){
          box([.3*q.volumes[j],.3,.3],[x,y,0],0x18363d,.04);
          if(q.pressure[j]!==0)box([.06,.06,.06*Math.abs(q.pressure[j])],[x,y,.03*q.pressure[j]],q.pressure[j]>0?0x067d91:0xaa5735,.75);
        }
        const a=arrow([0,y,.06],[.08*q.flux[c],0,0],0x18363d);if(a)a.userData.decayMethod='correction';
      }
      setMetrics([['Original-system status',q.acceptedOriginal?'Accepted exact finite balance':'REJECTED: original balance not satisfied'],['Component sums of b (m³/s²)',list(q.sums)],['Original b (m³/s²)',list(q.b)],['Changed source, used − original',list(q.sourceChange)],['Pressure representatives (Pa)',list(q.pressure)],['Face pressure jumps (Pa)',list(q.jumps)],['Face velocity corrections (m/s)',list(q.flux)],['Full original residual (m³/s²)',list(q.residual)],['Retained-row residual maximum',n(q.reducedResidual)],['Full residual maximum',n(q.fullResidual)],['Actual divergences (1/s)',list(q.divergence)],['Maximum absolute divergence (1/s)',n(q.maxDivergence)],['Arithmetic pressure means (Pa)',list(q.means)],['Volume pressure means (Pa)',list(q.volumeMeans)],['Interior / boundary work (W)',list([q.interiorWork,q.boundaryWork])],['Boundary-work change from offsets (W)',n(q.boundaryShift)],['Pressure-divergence pairing (W)',n(q.pairing)],['Fixed display scales','Geometry 0.3 units/m; scalar bars 0.06 units/Pa; face arrows 0.08 units per m/s']]);
    }
    choice('Original boundary data',['Compatible per region','Global sum only'],0,v=>{input=v?'global-only':'compatible';draw();});
    choice('Cell volumes (m³)',['1 / 1 / 1 / 1','1 / 3 / 2 / 2'],1,v=>{geometry=v?'unequal':'uniform';draw();});
    choice('Gauge / explicit source treatment',['Fix left cell per region','Arithmetic mean zero','Volume mean zero','Project b, then fix left cells'],0,v=>{mode=['fixed','mean','volume','projected'][v];draw();});
    choice('Region A pressure offset (Pa)',['−2','0','2'],1,v=>{shiftA=[-2,0,2][v];draw();});
    choice('Region B pressure offset (Pa)',['−2','0','2'],1,v=>{shiftB=[-2,0,2][v];draw();});
    choice('Right-cell pressure perturbation in A (Pa)',['0','0.25'],0,v=>{error=[0,.25][v];draw();});draw();
  };
}
