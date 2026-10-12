// Educational binary64 mechanisms, independently compared with the book's
// companion Python equations. No native solver or pressure API is invoked.
const offsets={cell:[.5,.5,.5],x:[0,.5,.5],y:[.5,0,.5],z:[.5,.5,0]};
const spacing=[.2,.3,.4], origin=[-.1,.2,-.3];
const affine=p=>1.2+.7*p[0]-1.1*p[1]+.4*p[2];
export function sampleAffine(component='x',wrong=false,position=[.67,.84,1.12]) {
  if(!offsets[component]||position.length!==3||!position.every(Number.isFinite))throw new Error('Invalid affine sample');
  const stored=offsets[component], sampled=wrong?offsets.cell:stored;
  const grid=position.map((x,i)=>(x-origin[i])/spacing[i]-sampled[i]);
  const lo=grid.map(Math.floor), fraction=grid.map((x,i)=>x-lo[i]), donors=[];
  for(let i=0;i<2;i++)for(let j=0;j<2;j++)for(let k=0;k<2;k++){
    const corner=[i,j,k], p=corner.map((c,a)=>origin[a]+spacing[a]*(lo[a]+c+stored[a]));
    const weight=corner.reduce((w,c,a)=>w*(c?fraction[a]:1-fraction[a]),1);
    donors.push({position:p,weight,value:affine(p)});
  }
  const value=donors.reduce((s,d)=>s+d.weight*d.value,0),exact=affine(position);
  return {position:[...position],donors,value,exact,error:value-exact,
    weightSum:donors.reduce((s,d)=>s+d.weight,0),minimum:Math.min(...donors.map(d=>d.value)),
    maximum:Math.max(...donors.map(d=>d.value)),component,wrong};
}
const wrap=(i,n)=>((i%n)+n)%n;
export function sl(q,c){
  return q.map((_,i)=>{const departure=i-c,lo=Math.floor(departure),f=departure-lo;
    return (1-f)*q[wrap(lo,q.length)]+f*q[wrap(lo+1,q.length)];});
}
export function corrected(q,c){
  const forward=sl(q,c),backward=sl(forward,-c),candidate=forward.map((v,i)=>v+.5*(q[i]-backward[i]));
  let fallback=0;
  const limited=candidate.map((v,i)=>{const lo=Math.floor(i-c),a=q[wrap(lo,q.length)],b=q[wrap(lo+1,q.length)];
    if(v>=Math.min(a,b)&&v<=Math.max(a,b))return v;
    fallback++;return forward[i];});
  return {forward,candidate,limited,fallback};
}
export function evolve({profile='sine',n=64,courant=.5,steps=1}={}){
  if(!['sine','top-hat','symmetric','asymmetric'].includes(profile)||![32,64,128,256].includes(n)||
    !Number.isFinite(courant)||courant<0||courant>1||!Number.isInteger(steps)||steps<0||steps>512)throw new Error('Invalid transport control');
  const initial=profile==='symmetric'?[0,0,1,1,0,0]:profile==='asymmetric'?[0,.2,1,.4,0,0]:
    Array.from({length:n},(_,i)=>profile==='sine'?Math.sin(2*Math.PI*i/n):Number(i>=5*n/16&&i<9*n/16));
  n=initial.length;
  let linear=[...initial],raw=[...initial],limited=[...initial],fallbackCount=0,lastFallback=0;
  for(let step=0;step<steps;step++){
    linear=sl(linear,courant);raw=corrected(raw,courant).candidate;
    const result=corrected(limited,courant);limited=result.limited;lastFallback=result.fallback;fallbackCount+=lastFallback;
  }
  // Smooth sine has an analytic continuum translation. Other fixtures compare
  // with exact translation of the initial periodic piecewise-linear interpolant.
  const exact=profile==='sine'?initial.map((_,i)=>Math.sin(2*Math.PI*(i-courant*steps)/n)):sl(initial,courant*steps);
  const sum0=initial.reduce((s,v)=>s+v,0);
  const metrics=q=>({min:Math.min(...q),max:Math.max(...q),sum:q.reduce((s,v)=>s+v,0),
    amountChange:q.reduce((s,v)=>s+v,0)-sum0,rms:Math.sqrt(q.reduce((s,v,i)=>s+(v-exact[i])**2,0)/n)});
  return {profile,n,courant,steps,initial,linear,raw,limited,exact,
    linearMetrics:metrics(linear),rawMetrics:metrics(raw),limitedMetrics:metrics(limited),fallbackCount,lastFallback};
}
