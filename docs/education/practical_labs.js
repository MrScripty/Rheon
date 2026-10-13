import {references,domains} from './practical_reference.js';
export const practicalDescriptions={
 wallload:{title:'Wall forces close a power ledger',chapter:'02-continuum-equations',text:'A one-metre plate gap carries steady Newtonian shear under a uniform body force. Teal arrows show sampled velocity; orange arrows show wall traction on the fluid. Both wall and body work enter the balance.',limit:'Analytical unidirectional steady solution, unit wall area. No moving geometry, transient response, nonlinear constitutive law or two-way solid dynamics.',labels:['Viscosity (Pa s)','Upper-wall speed (m/s)','Body force (N/m³)'],defaults:[1,1,2]},
 jacobi:{title:'Run the iteration and inspect the true residual',chapter:'05-linear-solvers',text:'Each selected iteration executes weighted Jacobi on K = [[2, −1], [−1, 2]]. Arrows encode the two components of b − Kx. A positive matrix still needs an appropriate relaxation.',limit:'Two-unknown Dirichlet system with dimensionless quantities. No general CG/multigrid convergence or production timing claim.',labels:['Relaxation','Iterations','Right-hand-side mode'],defaults:[1,3,2]},
 memory:{title:'Count cells, faces and sparse storage',chapter:'13-memory-and-parallelism',text:'The cube shows the declared n × n × n allocation shape. Three face grids show axis-aligned face indexing. Count fields, CSR values/indices and five solver vectors separately.',limit:'Explicit array-byte model with 32-bit CSR indices, excluding allocator padding, ghosts, object overhead, particles, GPU copies and measured peak memory.',labels:['Cells per axis','Cell scalar fields','Bytes per scalar'],defaults:[1,1,0]},
 roundoff:{title:'Reduction order can discard a unit',chapter:'15-floating-point',text:'Accumulate [2^e, 1, −2^e] or cancel the large pair first. Binary32 rounds every addition with Math.fround; binary64 uses JavaScript Number. The two bars are the exact final sum and executed final sum.',limit:'Finite specified addition sequences, not a proof of Rust, SIMD, GPU, compiler reassociation or a complete IEEE solver.',labels:['Large-term exponent','Order','Arithmetic'],defaults:[1,0,0]},
 particletransfer:{title:'Transfer preserves momentum while removing variation',chapter:'11-particle-methods',text:'Two particles feed one mass-weighted grid sample. PIC replaces both velocities by the updated sample; FLIP adds the grid increment to each old particle. Blend to inspect the unresolved velocity mode.',limit:'One grid sample with weights one, fixed positions and masses. No advection, force solve, APIC affine moments, angular momentum or SPH/PBF dynamics.',labels:['Second mass (kg)','Common speed (m/s)','Counterflow speed (m/s)','Grid increment (m/s)','FLIP fraction'],defaults:[0,0,1,0,0]},
 sstprobe:{title:'Inspect a specified SST-2003 constitutive limiter',chapter:'22-viscosity-and-stress',text:'Supplied k, omega, strain magnitude and F2 determine eddy viscosity and clipped production. Bars are scalar readouts: viscosity, raw production, limited production. No turbulence evolution is animated.',limit:'Density 1 kg/m³; supplied fields only. No k/omega transport, wall-distance or blending solve, separation prediction or model validation. The laminar zero-k limit is explicit.',labels:['k (m²/s²)','Omega (1/s)','Strain magnitude (1/s)','Supplied F2'],defaults:[1,0,2,1]},
 refinement:{title:'A converged grid can retain model bias',chapter:'18-validation-program',text:'Midpoint quadrature integrates y² over [0,1]. Compute three successive grids, observed order, Richardson limit and an absolute GCI estimate. Add a constant bias to see why spatial refinement cannot remove it.',limit:'Smooth authored quadrature fixture with exact answer 1/3, ratio two and safety factor 1.25. GCI is an asymptotic discretization estimate, not a probability interval or physical validation.',labels:['Coarse cells','Constant model bias'],defaults:[1,0]},
 proposal:{title:'Accept a pressure proposal only after its gate',chapter:'01-purpose-and-model',text:'A two-cell Neumann candidate has a gauge shift and a pressure-jump error. Check finite values, declared units and the full residual. Rejection uses the exact fallback; both candidate and used flux are shown.',limit:'Synthetic proposal, not a model inference or learned accuracy result. Tolerance admits bounded nonzero divergence; zero tolerance is an exact finite fixture condition.',labels:['Pressure-jump error','Gauge offset','Residual tolerance','Candidate contract'],defaults:[2,1,1,0]}
};
export function createPracticalLabs({clear,box,arrow,dot,line,choice,meta,setMetrics,setReference}){
 const f=x=>x===null?'unavailable':Number(x).toPrecision(6),pair=a=>a.map(f).join(' / ');
 const glyph=(o,v,label,color=0x067d91)=>{const g=arrow(o,v,color);if(g)g.userData.decayMethod=label;};
 const bar=(x,v,scale,label,color=0x067d91)=>{if(v===0){dot([x,-.4,0],color,.012);return;}box([.12,Math.abs(v)*scale,.2],[x,-.4+v*scale/2,0],color,.7);};
 function draw(name,q){
  const a=q.args;
  if(name==='wallload'){
   for(const y of [-.5,.5])box([1,.04,.5],[0,y,0],0x18363d,.18);
   for(const s of q.samples)glyph([-.3,s.y-.5,.1],[s.velocity*.18,0,0],'velocity');
   for(const [y,t] of [[-.5,q.bottom],[.5,q.top]])glyph([.25,y,.25],[t*.1,0,0],'traction',0xaa5735);
   setMetrics([['Bottom / top fluid traction (Pa)',pair([q.bottom,q.top])],['Mean velocity (m/s)',f(q.mean)],['Body + wall forces (N)',f(q.forceBalance)],['Wall / body power (W)',pair([q.wallPower,q.bodyPower])],['Dissipation (W)',f(q.dissipation)],['Power balance error (W)',f(q.powerBalance)],['Scale','1 unit/m; velocity arrows 0.18 units/(m/s), traction arrows 0.1 units/Pa']]);
  }else if(name==='jacobi'){
   for(let i=0;i<2;i++){box([.35,.5,.35],[(i-.5)*.6,0,0],0x18363d,.06);glyph([(i-.5)*.6,.1,.25],[q.residual[i]*.06,0,0],'residual');}
   setMetrics([['Executed x',pair(q.x)],['Exact x',pair(q.exact)],['True residual',pair(q.residual)],['Residual norm',f(q.residualNorm)],['Error energy eᵀKe',f(q.energy)],['Smooth / alternating factors',pair([q.smoothFactor,q.alternatingFactor])],['Scale','Dimensionless diagram; residual arrows 0.06 units per residual unit']]);
  }else if(name==='memory'){
   const n=a[0],L=n*.035;box([L,L,L],[0,0,0],0x18363d,.04);
   for(let i=0;i<=n;i++){const t=i*.035-L/2;line([[-L/2,t,L/2],[L/2,t,L/2]],0x4e98bb);line([[L/2,-L/2,t],[L/2,L/2,t]],0x067d91);line([[t,L/2,-L/2],[t,L/2,L/2]],0xaa5735);line([[t,-L/2,L/2],[t,L/2,L/2]],0x4e98bb);line([[L/2,t,-L/2],[L/2,t,L/2]],0x067d91);line([[-L/2,L/2,t],[L/2,L/2,t]],0xaa5735);}
   setMetrics([['Cells / all axis faces',pair([q.cells,q.faces])],['Seven-point CSR nonzeros',f(q.nnz)],['Field / CSR / workspace bytes',pair([q.fieldBytes,q.csrBytes,q.workspaceBytes])],['Included total (MiB)',f(q.totalBytes/1048576)],['Scale','0.035 display units per cell; face grids show indexing only']]);
  }else if(name==='roundoff'){
   bar(-.25,1,.5,'exact',0x18363d);bar(.25,q.total,.5,'executed');
   setMetrics([['Terms',pair(q.terms)],['Accumulation trace',pair(q.trace)],['Exact / executed sum',pair([q.exact,q.total])],['Signed error',f(q.error)],['Scale','Both final-sum bars: 0.5 display units per sum unit; a dot means zero']]);
  }else if(name==='particletransfer'){
   for(let i=0;i<2;i++){const x=(i-.5)*.7;dot([x,0,0],0x18363d,.05*Math.cbrt(q.masses[i]));glyph([x,.22,.1],[q.input[i]*.18,0,0],'input',0xaa5735);glyph([x,-.22,.1],[q.output[i]*.18,0,0],'output');}
   setMetrics([['Input / output velocities',pair(q.input)+' → '+pair(q.output)],['Old / updated grid speed',pair([q.mean,q.grid])],['Expected / actual momentum (kg m/s)',pair([q.expectedMomentum,q.momentumAfter])],['Unresolved energy before / after (J)',pair([q.varianceBefore,q.varianceAfter])],['Scale','Schematic positions; velocity arrows 0.18 units/(m/s), particle volume proportional to mass']]);
  }else if(name==='sstprobe'){
   bar(-.3,q.viscosity,.12,'viscosity',0x18363d);bar(0,q.raw,.04,'raw',0xaa5735);bar(.3,q.limited,.04,'limited');
   setMetrics([['Positive denominator (1/s)',f(q.denominator)],['Eddy viscosity (Pa s)',f(q.viscosity)],['Raw / cap / limited production (W/m³)',pair([q.raw,q.cap,q.limited])],['Zero-k laminar case',q.laminar?'yes':'no'],['Scale','Viscosity 0.12 units/(Pa s); both production bars 0.04 units/(W/m³); compare production heights on their common scale']]);
  }else if(name==='refinement'){
   for(const s of q.samples){const h=s.value*.65;box([.9/a[0],h,.15],[.9*(s.y-.5),h/2-.4,0],0x067d91,.4);dot([.9*(s.y-.5),.65*s.y*s.y-.4,.14],0xaa5735,.015);}
   line(Array.from({length:33},(_,i)=>[.9*(i/32-.5),.65*(i/32)**2-.4,.14]),0xaa5735);
   setMetrics([['Three integrated values',pair(q.values)],['Observed order',f(q.order)],['Richardson limit / true answer',pair([q.extrapolated,q.exact])],['Absolute GCI estimate',f(q.absoluteGci)],['Fine-grid true error',f(q.trueError)],['Model bias',f(q.modelBias)],['Scale','x 0.9 units per coordinate, value 0.65 units per value; dots/curve show exact y²']]);
  }else{
   for(const x of [-.35,.35])box([.35,.5,.35],[x,0,0],0x18363d,.06);
   if(q.candidateFlux!==null)glyph([-.1,.18,.25],[q.candidateFlux*.28,0,0],'candidate',0xaa5735);
   glyph([-.1,-.18,.25],[q.usedFlux*.28,0,0],'used');
   setMetrics([['Candidate pressure',q.candidate?pair(q.candidate):'nonfinite proposal refused'],['Full candidate residual',q.residual?pair(q.residual):'unavailable'],['Finite / units',String(q.finite)+' / '+q.units],['Decision',q.accepted?'candidate accepted':'fallback: '+q.reason],['Candidate / used flux',pair([q.candidateFlux,q.usedFlux])],['Used full residual',pair(q.usedResidual)],['Scale','Two-cell dimensionless graph; flux arrows 0.28 display units per flux unit']]);
  }
 }
 return Object.fromEntries(Object.entries(practicalDescriptions).map(([name,d])=>[name,()=>{
  const args=d.defaults.map((j,i)=>domains[name][i][j]);meta(d.title,d.text,d.limit,d.chapter);const update=()=>{clear();const q=references[name](...args);setReference(q);draw(name,q);};
  d.labels.forEach((label,i)=>choice(label,domains[name][i].map(String),d.defaults[i],j=>{args[i]=domains[name][i][j];update();}));update();
 }]));
}
