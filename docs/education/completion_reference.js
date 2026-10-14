// Independently authored finite teaching mechanisms, not a CFD backend.
export const domains={
 reconstruction:[[-.5,0,.5],[0,1],[0,1],[.5,1],['two-point','corrected']],
 coupling:[[.5,1,1.5],[0,1,2,4,8]]
};
function admitted(name,args){if(args.length!==domains[name].length||args.some((x,i)=>!domains[name][i].includes(x)))throw new RangeError('Outside declared completion-lesson domain');}
export function reconstruction(skew=.5,gx=1,gy=1,height=1,method='two-point'){
 const args=[skew,gx,gy,height,method];admitted('reconstruction',args);
 const points=[[-.5,0,0],[.5,skew,0],[-.5,height,0]],values=points.map(p=>gx*p[0]+gy*p[1]+2);
 const gradientY=(values[2]-values[0])/height,gradientX=values[1]-values[0]-skew*gradientY;
 const twoPoint=(values[1]-values[0])/(1+skew*skew);
 const correction=gradientX*(1-1/(1+skew*skew))-gradientY*skew/(1+skew*skew);
 const normal=method==='corrected'?twoPoint+correction:twoPoint;
 const faceLinear=(values[0]+values[1])/2,faceCorrected=faceLinear-gradientY*skew/2;
 return {kind:'reconstruction',args,points,values,gradient:[gradientX,gradientY],twoPoint,correction,normal,normalError:normal-gx,faceLinear,faceCorrected,faceExact:2,determinant:height};
}
export function coupling(relaxation=1,steps=4){
 const args=[relaxation,steps];admitted('coupling',args);let u=[0,0],pressure=0;const history=[];
 const row=()=>({u:u.slice(),pressure,continuity:u[0]+u[1],momentum:[1-2*u[0]+u[1]-pressure,-2*u[1]+u[0]-pressure]});history.push(row());
 for(let i=0;i<steps;i++){
  const predictor=[(1+u[1]-pressure)/2,(u[0]-pressure)/2],correction=predictor[0]+predictor[1];
  const conservative=predictor.map(v=>v-correction/2);u=u.map((v,j)=>(1-relaxation)*v+relaxation*conservative[j]);pressure+=relaxation*correction;history.push(row());
 }
 const final=history.at(-1);return {kind:'coupling',args,history,...final,velocityError:u[0]-1/6,pressureError:pressure-.5,momentumNorm:Math.hypot(...final.momentum),velocityFactor:1-1.5*relaxation,pressureFactor:1-relaxation};
}
export const references={reconstruction,coupling};
