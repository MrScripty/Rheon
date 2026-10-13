import {references} from './completion_reference.js';
export function createCompletionLabs({clear,box,arrow,dot,line,choice,meta,setMetrics,setReference}){
 const number=x=>Number(x).toFixed(5),list=a=>a.map(number).join(' / ');
 const glyph=(o,v,label,color=0x067d91)=>{const g=arrow(o,v,color);if(g)g.userData.decayMethod=label;};
 const descriptions={
 reconstruction:{title:'A skew face needs a reconstructed gradient',text:'Three pressure samples determine an affine planar gradient. The thin patch is the face at x=0; orange and teal arrows compare the two-point normal gradient with the selected reconstruction. The third sample supplies information absent from two centres alone.',limit:'Sample geometry and a finite affine field, not a mesh parser or general nonorthogonal solver. Fixed arrows encode pressure gradient, not fluid velocity. No accuracy claim for nonlinear fields.',chapter:'08-boundary-geometry',controls:[['Skew offset (m)',['−0.5','0','0.5'],[-.5,0,.5],2],['True x slope (Pa/m)',['0','1'],[0,1],1],['True y slope (Pa/m)',['0','1'],[0,1],1],['Third-sample height (m)',['0.5','1'],[.5,1],1],['Normal reconstruction',['Two-point only','Affine corrected'],['two-point','corrected'],0]]},
 coupling:{title:'Continuity can pass while momentum is wrong',text:'Two parallel paths connect the same regions. Signed arrows are nondimensional path velocities. Every corrected iterate has zero net flux; inspect both momentum residuals as you change iteration count and relaxation.',limit:'An explicitly stated two-path linear momentum model, not collocated Navier–Stokes or a full SIMPLE/Rhie–Chow implementation. Iteration count is real recurrence work but is not runtime or physical time.',chapter:'04-pressure-projection',controls:[['Relaxation',['0.5','1','1.5'],[.5,1,1.5],1],['Iterations',['0','1','2','4','8'],[0,1,2,4,8],3]]}
 };
 function draw(name,q){
  if(name==='reconstruction'){
   box([.008,.5,.5],[0,0,0],0x18363d,.15);for(const p of q.points)dot(p.map(v=>v*.55),0x18363d,.03);line(q.points.slice(0,2).map(p=>p.map(v=>v*.55)),0x4e98bb);
   glyph([0,-.12,.28],[q.twoPoint*.18,0,0],'two-point',0xaa5735);glyph([0,.12,.28],[q.normal*.18,0,0],'selected');
   setMetrics([['Reconstructed gradient (Pa/m)',list(q.gradient)],['Two-point normal (Pa/m)',number(q.twoPoint)],['Selected normal (Pa/m)',number(q.normal)],['Normal error (Pa/m)',number(q.normalError)],['Linear / corrected face pressure (Pa)',list([q.faceLinear,q.faceCorrected])],['Sample determinant (m²)',number(q.determinant)],['Scale','Geometry 0.55 units/m; arrows 0.18 units per Pa/m']]);
  }else{
   box([.3,.75,.35],[-.5,0,0],0x18363d,.04);box([.3,.75,.35],[.5,0,0],0x18363d,.04);
   for(let i=0;i<2;i++){line([[-.35,(i-.5)*.45,0],[.35,(i-.5)*.45,0]],0x4e98bb);glyph([0,(i-.5)*.45,.05],[q.u[i]*.8,0,0],'path');}
   setMetrics([['Path velocities',list(q.u)],['Pressure difference',number(q.pressure)],['Continuity residual',number(q.continuity)],['Momentum residuals',list(q.momentum)],['Momentum residual norm',number(q.momentumNorm)],['Velocity / pressure errors',list([q.velocityError,q.pressureError])],['Velocity / pressure error factors',list([q.velocityFactor,q.pressureFactor])],['Exact solution','u = (1/6, −1/6), pressure = 1/2'],['Scale','Nondimensional network; arrows 0.8 display units per velocity unit']]);
  }
 }
 return Object.fromEntries(Object.entries(descriptions).map(([name,d])=>[name,()=>{
  const args=d.controls.map(c=>c[2][c[3]]);meta(d.title,d.text,d.limit,d.chapter);
  const update=()=>{clear();const q=references[name](...args);setReference(q);draw(name,q);};
  d.controls.forEach(([label,labels,values,index],i)=>choice(label,labels,index,j=>{args[i]=values[j];update();}));update();
 }]));
}
