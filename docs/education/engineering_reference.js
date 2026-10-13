// Authored bounded educational references; no native solver or imported mesh.
function member(x,values){if(!values.includes(x))throw new RangeError('Outside the declared teaching domain');}
export function faceWork(area=.5,velocity=1,boundaryFlux=.5,pair=0){
  member(area,[.25,.5,1]);member(velocity,[-1,0,1]);member(boundaryFlux,[-.5,0,.5]);member(pair,[0,1,2]);
  const pressure=[[1,3],[3,1],[1,1]][pair],flux=area*velocity;
  const divergence=[flux+boundaryFlux,-flux-boundaryFlux];
  return {area,velocity,boundaryFlux,pair,pressure,flux,divergence,
    pressureWork:pressure.reduce((s,p,i)=>s+p*divergence[i],0),
    interiorWork:-(pressure[1]-pressure[0])*flux,
    boundaryWork:(pressure[0]-pressure[1])*boundaryFlux};
}
export function checkerboard(amplitude=1,response=1,pattern='alternating'){
  member(amplitude,[0,.5,1]);member(response,[.25,.5,1]);member(pattern,['constant','alternating']);
  const pressure=Array.from({length:4},(_,i)=>amplitude*(pattern==='constant'||i%2===0?1:-1));
  const gradient=pressure.map((_,i)=>(pressure[(i+1)%4]-pressure[(i+3)%4])/2);
  const cellVelocity=gradient.map(g=>-response*g);
  const naive=cellVelocity.map((v,i)=>(v+cellVelocity[(i+1)%4])/2);
  const flux=naive.map((v,i)=>v+response*(gradient[i]+gradient[(i+1)%4])/2-response*(pressure[(i+1)%4]-pressure[i]));
  return {amplitude,response,pattern,pressure,gradient,cellVelocity,naive,flux,
    divergence:flux.map((v,i)=>v-flux[(i+3)%4])};
}
export function affineStress(rate=1,mu=1,kind='rotation'){
  member(rate,[-1,0,1]);member(mu,[0,.5,1]);member(kind,['rotation','shear','extension']);
  const gradient=kind==='rotation'?[[0,-rate,0],[rate,0,0],[0,0,0]]:kind==='shear'?[[0,rate,0],[0,0,0],[0,0,0]]:[[rate,0,0],[0,-rate,0],[0,0,0]];
  const strain=gradient.map((row,i)=>row.map((v,j)=>(v+gradient[j][i])/2));
  const stress=strain.map(row=>row.map(v=>2*mu*v));
  const velocity=x=>gradient.map(row=>row.reduce((s,v,j)=>s+v*x[j],0));
  const samples=[];
  for(const x of [-.25,.25])for(const y of [-.25,.25])for(const z of [-.25,.25])samples.push({position:[x,y,z],velocity:velocity([x,y,z])});
  const faces=[];
  for(let axis=0;axis<3;axis++)for(const sign of [-1,1]){
    const position=[0,0,0];position[axis]=sign*.5;
    const traction=stress.map(row=>row[axis]*sign),u=velocity(position);
    faces.push({position,traction,power:traction.reduce((s,t,j)=>s+t*u[j],0)});
  }
  return {rate,mu,kind,gradient,strain,stress,samples,faces,
    dissipation:2*mu*strain.flat().reduce((s,v)=>s+v*v,0),
    gradientProxy:mu*gradient.flat().reduce((s,v)=>s+v*v,0),
    boundaryPower:faces.reduce((s,f)=>s+f.power,0)};
}
