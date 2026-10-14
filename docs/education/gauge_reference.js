// Independent finite pressure teaching model; no native solver invocation.
const member=(x,a)=>{if(!a.includes(x))throw new RangeError('Outside pressure-gauge teaching domain');};
export function pressureGauge(input='compatible',geometry='unequal',mode='fixed',shiftA=0,shiftB=0,error=0){
  member(input,['compatible','global-only']);member(geometry,['uniform','unequal']);member(mode,['fixed','mean','volume','projected']);member(shiftA,[-2,0,2]);member(shiftB,[-2,0,2]);member(error,[0,.25]);
  const volumes=geometry==='uniform'?[1,1,1,1]:[1,3,2,2];
  const b=input==='compatible'?[1,-1,2,-2]:[1,0,-1,0];
  const sums=[b[0]+b[1],b[2]+b[3]],used=b.slice(),pressure=[],weights=[],jumps=[],flux=[],kp=[],base=[];
  for(let c=0;c<2;c++){
    const i=2*c,w=2/(volumes[i]+volumes[i+1]);weights.push(w);
    if(mode==='projected'){used[i]-=sums[c]/2;used[i+1]-=sums[c]/2;}
    const right=used[i+1]/w;
    let offset=mode==='mean'?-right/2:mode==='volume'?-volumes[i+1]*right/(volumes[i]+volumes[i+1]):0;
    base.push(offset,offset+right+(c===0?error:0));offset+=c===0?shiftA:shiftB;
    pressure.push(offset,offset+right+(c===0?error:0));
    const jump=pressure[i+1]-pressure[i];jumps.push(jump);flux.push(-w*jump);kp.push(-w*jump,w*jump);
  }
  const residual=b.map((v,i)=>v-kp[i]),usedResidual=used.map((v,i)=>v-kp[i]);
  const boundary=b.map(v=>-v),divergence=residual.map((r,i)=>-r/volumes[i]);
  const dot=(a,b)=>a.reduce((s,v,i)=>s+v*b[i],0);
  const boundaryWork=dot(pressure,boundary),baseBoundaryWork=dot(base,boundary);
  return {input,geometry,mode,shiftA,shiftB,error,volumes,b,used,sourceChange:used.map((v,i)=>v-b[i]),sums,weights,pressure,jumps,flux,kp,residual,usedResidual,boundary,divergence,
    compatible:sums.every(s=>s===0),acceptedOriginal:residual.every(r=>r===0),
    reducedResidual:Math.max(Math.abs(residual[1]),Math.abs(residual[3])),fullResidual:Math.max(...residual.map(Math.abs)),maxDivergence:Math.max(...divergence.map(Math.abs)),
    means:[(pressure[0]+pressure[1])/2,(pressure[2]+pressure[3])/2],volumeMeans:[0,2].map(i=>(volumes[i]*pressure[i]+volumes[i+1]*pressure[i+1])/(volumes[i]+volumes[i+1])),
    boundaryWork,boundaryShift:boundaryWork-baseBoundaryWork,interiorWork:-dot(jumps,flux),pairing:dot(pressure,residual.map(r=>-r))};
}
