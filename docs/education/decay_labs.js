import {modeDecay,decaySteps,shearSamples} from './decay_reference.js';
export function createDecayLab({clear,box,arrow,range,choice,meta,setMetrics,setReference}) {
  return function decay(){
    let method='explicit',duration=2,steps=1,index=1,stepControl;
    meta('Stable energy can hide a reversed flow',
      'One periodic shear mode fills a unit cube: u = (A sin(2πy), 0, 0). Colored arrows show the selected method; dark arrows show exact decay at the same sample positions. Both use one fixed velocity scale. Cube edges mark a periodic domain, not walls.',
      'Independent modal time integration, λ = 1/s. No spatial discretization or native timestep. Samples repeat the same y-dependent field in x and z; camera and scroll never advance time. Energy is the analytic unit-cube integral, not a sum over display arrows.',
      '12-time-and-error');
    function draw(){
      clear();const q=modeDecay(duration,steps,index), samples=shearSamples(q.amplitudes[method]);
      const color={explicit:0xaa5735,implicit:0x067d91,exact:0x18363d}[method];
      box([1,1,1],[0,0,0],0x18363d,.02);
      const emit=(sample,tint)=>{
        const glyph=arrow(sample.position.map(v=>v-.5),sample.velocity.map(v=>v*.18),tint);
        if(glyph)glyph.userData.decayMethod=tint===0x18363d?'exact':method;
      };
      // Same origins and scale: exact remains dark when numerical arrows extend/reverse.
      for(const sample of samples)emit(sample,color);
      if(method!=='exact')for(const sample of shearSamples(q.amplitudes.exact))emit(sample,0x18363d);
      setReference({...q,method,displaySamples:samples,velocityScale:.18});
      const n=v=>v.toPrecision(6);
      setMetrics([['Physical model','L = 1 m, ρ = 1 kg/m³, λ = 1/s'],
        ['Target time / step count',`${duration} s / ${steps}`],['Δt / displayed time',`${n(q.dt)} s / ${n(q.time)} s`],
        ['Explicit factor / implicit factor',`${n(q.factors.explicit)} / ${n(q.factors.implicit)}`],
        ['Explicit / implicit / exact A (m/s)',[q.amplitudes.explicit,q.amplitudes.implicit,q.amplitudes.exact].map(n).join(' / ')],
        ['Explicit / implicit / exact energy (J)',[q.energy.explicit,q.energy.implicit,q.energy.exact].map(n).join(' / ')],
        ['Explicit / implicit amplitude error (m/s)',`${n(q.error.explicit)} / ${n(q.error.implicit)}`],
        ['Explicit energy / sign test',`${q.explicitEnergyStable?'nonincreasing':'unstable'} / ${q.explicitSignPreserving?'preserved':'reverses per step'}`],
        ['Arrow scale','Velocity × 0.18 s; no normalization']]);
    }
    choice('Displayed integrator',['Explicit Euler','Implicit Euler','Exact decay'],0,v=>{method=['explicit','implicit','exact'][v];draw();});
    choice('Fixed comparison duration',[1,2,3].map(t=>`${t} s`),1,v=>{duration=[1,2,3][v];draw();});
    choice('Steps across fixed duration',decaySteps.map(n=>String(n)),0,v=>{steps=decaySteps[v];index=steps;if(stepControl){stepControl.max=steps;stepControl.value=index;}draw();});
    stepControl=range('Displayed step',steps,index,v=>{index=v;draw();});
    draw();
  };
}
