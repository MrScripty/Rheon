import {faceWork,checkerboard,affineStress} from './engineering_reference.js';
export function createEngineeringLabs({clear,box,arrow,choice,meta,setMetrics,setReference}){
  const n=x=>Number(x).toFixed(4);
  const glyph=(origin,vector,color,label)=>{const g=arrow(origin,vector,color);if(g)g.userData.decayMethod=label;};
  function faceflux(){
    let area=.5,velocity=1,boundaryFlux=.5,pair=0;
    meta('One face, two ledgers, boundary work','Two unit cells share the central opening. Teal arrows are prescribed face velocity, orange arrows are external normal velocity. The opening area changes flux, not arrow length.','Fixed geometry, constant density 1 kg/m³. Pressures are test values, not solved pressures. Balanced external flux is prescribed at this instant; no moving wall or timestep is computed.','08-boundary-geometry');
    function draw(){clear();const q=faceWork(area,velocity,boundaryFlux,pair);setReference(q);
      for(const x of [-.4,.4])box([.8,.8,.8],[x,0,0],0x18363d,.035);
      const width=.8*Math.sqrt(area);box([.012,width,width],[0,0,0],0x067d91,.5);
      glyph([0,0,0],[velocity*.25,0,0],0x067d91,'internal');
      for(const x of [-.8,.8])glyph([x,0,0],[-boundaryFlux*.25,0,0],0xaa5735,'boundary');
      setMetrics([['Shared area (m²)',n(area)],['Pressure test pair (Pa)',q.pressure.join(' / ')],['Shared volume flux (m³/s)',n(q.flux)],['Cell divergences (1/s)',q.divergence.map(n).join(' / ')],['Sum of cell volume rates (m³/s)',n(q.divergence[0]+q.divergence[1])],['Pressure-divergence pairing (W)',n(q.pressureWork)],['Interior term (W)',n(q.interiorWork)],['Boundary term (W)',n(q.boundaryWork)],['Identity defect (W)',n(q.pressureWork-q.interiorWork-q.boundaryWork)],['Geometry / velocity scale','0.8 scene units/m; arrows 0.25 scene units per m/s']]);}
    choice('Opening area (m²)',['0.25','0.5','1'],1,v=>{area=[.25,.5,1][v];draw();});
    choice('Shared face velocity (m/s)',['−1','0','1'],2,v=>{velocity=[-1,0,1][v];draw();});
    choice('Left external outward flux (m³/s)',['−0.5','0','0.5'],2,v=>{boundaryFlux=[-.5,0,.5][v];draw();});
    choice('Pressure test pair (Pa)',['1 / 3','3 / 1','1 / 1'],0,v=>{pair=v;draw();});draw();
  }
  function checker(){
    let amplitude=1,response=1,pattern='alternating';
    meta('A pressure mode hidden from cell gradients','Four unit cells form a periodic strip. End faces are identified. Teal/orange cells show positive/negative kinematic pressure; arrows show the compact face response.','One-dimensional periodic algebra displayed as extruded cells, not a 3D fluid solution. Unit spacing, zero momentum source and constant diagonal response; no general Rhie–Chow, SIMPLE or convergence claim.','04-pressure-projection');
    function draw(){clear();const q=checkerboard(amplitude,response,pattern);setReference(q);
      q.pressure.forEach((p,i)=>box([.4,.4,.4],[(i-1.5)*.4,0,0],p>=0?0x067d91:0xaa5735,.12+.15*Math.abs(p)));
      q.flux.forEach((v,i)=>glyph([(i-1)*.4,.3,0],[v*.1,0,0],0x18363d,'compact'));
      setMetrics([['Kinematic pressure (m²/s²)',q.pressure.map(n).join(' / ')],['Cell central gradients (m/s²)',q.gradient.map(n).join(' / ')],['Naive face velocities (m/s)',q.naive.map(n).join(' / ')],['Compact face velocities (m/s)',q.flux.map(n).join(' / ')],['Compact divergences (1/s)',q.divergence.map(n).join(' / ')],['Periodic total divergence',n(q.divergence.reduce((s,v)=>s+v,0))],['Fixed arrow scale','0.1 scene units per m/s; last arrow belongs to periodic seam']]);}
    choice('Pressure pattern',['Constant','Alternating'],1,v=>{pattern=v?'alternating':'constant';draw();});
    choice('Pressure amplitude (m²/s²)',['0','0.5','1'],2,v=>{amplitude=[0,.5,1][v];draw();});
    choice('Momentum response (s)',['0.25','0.5','1'],2,v=>{response=[.25,.5,1][v];draw();});draw();
  }
  function stressprobe(){
    let rate=1,mu=1,kind='rotation';
    meta('Rotation, shear and the work through a cube','Teal sample arrows show prescribed affine velocity. Orange arrows at the six face centres show viscous traction. All fields are independent of z.','A centred unit cube samples an imposed field, not a no-slip enclosure or time evolution. Constant stress has zero divergence. Face work supplies shear/extension dissipation; pressure is set to zero for this constitutive test.','22-viscosity-and-stress');
    function draw(){clear();const q=affineStress(rate,mu,kind);setReference(q);box([1,1,1],[0,0,0],0x18363d,.025);
      for(const s of q.samples)glyph(s.position,s.velocity.map(v=>v*.3),0x067d91,'velocity');
      for(const f of q.faces)glyph(f.position,f.traction.map(v=>v*.12),0xaa5735,'traction');
      setMetrics([['Velocity gradient (1/s)',q.gradient.map(r=>r.map(n).join(', ')).join(' | ')],['Symmetric strain (1/s)',q.strain.map(r=>r.map(n).join(', ')).join(' | ')],['Viscous stress (Pa)',q.stress.map(r=>r.map(n).join(', ')).join(' | ')],['Integrated dissipation (W)',n(q.dissipation)],['Six-face boundary power (W)',n(q.boundaryPower)],['μ gradient:gradient (W)',n(q.gradientProxy)],['Power defect (W)',n(q.boundaryPower-q.dissipation)],['Different fixed glyph scales','Velocity × 0.3 s; traction × 0.12 m/Pa']]);}
    choice('Affine field',['Rigid rotation','Simple shear','Planar extension'],0,v=>{kind=['rotation','shear','extension'][v];draw();});
    choice('Rate (1/s)',['−1','0','1'],2,v=>{rate=[-1,0,1][v];draw();});
    choice('Dynamic viscosity (Pa s)',['0','0.5','1'],2,v=>{mu=[0,.5,1][v];draw();});draw();
  }
  return {faceflux,checker,stressprobe};
}
