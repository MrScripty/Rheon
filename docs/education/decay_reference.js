// Analytic periodic shear mode: L=1 m, rho=1 kg/m^3, U0=1 m/s,
// nu=1/(4*pi^2) m^2/s gives lambda=1/s. No spatial/native solver.
export const decaySteps = [1,2,4,8,16,32,64];
export function modeDecay(duration=2, steps=1, index=steps) {
  if (![1,2,3].includes(duration) || !decaySteps.includes(steps) ||
      !Number.isInteger(index) || index<0 || index>steps)
    throw new RangeError('Require T in {1,2,3}, a listed step count, and 0 <= index <= steps.');
  const dt=duration/steps, time=index*dt;
  // Actual repeated updates; the independent oracle uses exact powers.
  let explicit=1,implicit=1;
  for(let n=0;n<index;n++){explicit*=1-dt;implicit/=1+dt;}
  const exact=Math.exp(-time);
  const amplitudes={explicit,implicit,exact};
  return {duration,steps,index,dt,time,lambda:1,nu:1/(4*Math.PI*Math.PI),
    factors:{explicit:1-dt,implicit:1/(1+dt)},amplitudes,
    energy:Object.fromEntries(Object.entries(amplitudes).map(([k,a])=>[k,a*a/4])),
    error:{explicit:Math.abs(explicit-exact),implicit:Math.abs(implicit-exact)},
    explicitEnergyStable:dt<=2,explicitSignPreserving:dt<=1};
}
// Physical sample locations; they are display samples, not a numerical mesh.
export function shearSamples(amplitude) {
  if (!Number.isFinite(amplitude) || Math.abs(amplitude)>2+1e-12)
    throw new RangeError('Amplitude exceeds the bounded lesson domain.');
  const samples=[];
  for(const x of [.25,.75])for(const z of [.2,.5,.8])for(let j=0;j<8;j++){
    const y=(j+.5)/8;
    samples.push({position:[x,y,z],velocity:[amplitude*Math.sin(2*Math.PI*y),0,0]});
  }
  return samples;
}
