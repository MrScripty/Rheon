import {sampleAffine,evolve} from './transport_reference.js';

// Geometry is a fixed-scale mathematical diagram. Moving the camera never
// changes samples, and controls synchronously recompute bounded reference cases.
export function createTransportLabs({clear,line,dot,arrow,range,choice,meta,setMetrics,panel,setReference}) {
  const fmt=x=>Number(x).toPrecision(7), teal=0x067d91,warm=0xaa5735,blue=0x4e98bb,ink=0x18363d;
  function sampler(){
    let component=1,wrong=0,displacement=50;
    meta('Sample on the right lattice','Eight donor nodes and their weights form one trilinear sample. Teal arrows encode weight on one fixed scale; orange is the requested point. Inspect the donor table below.',
      'A complete regular stencil and an affine field in arbitrary units. Binary64 recomputation of the companion reference; no obstacle extension, advection step or native Rust execution. Exact-real convex-weight proofs do not certify this JavaScript.', '06-transport');
    choice('Stored lattice',['Cell centre','x face','y face','z face'],1,v=>{component=v;draw();});
    choice('Sampling offset',['Match stored lattice','Use cell-centre offset (deliberate error)'],0,v=>{wrong=v;draw();});
    range('Sample x displacement',100,50,v=>{displacement=v;draw();});
    const detail=document.createElement('details');detail.className='donor-table';
    const summary=document.createElement('summary');summary.textContent='Eight donor positions, weights and values';
    const table=document.createElement('table');detail.append(summary,table);panel.append(detail);
    function draw(){
      clear();const q=sampleAffine(['cell','x','y','z'][component],!!wrong,[.67+(displacement-50)*.004,.84,1.12]);setReference(q);
      const map=p=>p.map((v,i)=>(v-[.67,.84,1.12][i])*2.5);
      for(let a=0;a<8;a++)for(let b=a+1;b<8;b++)if([1,2,4].includes(a^b))line([map(q.donors[a].position),map(q.donors[b].position)],0x9eafb6);
      q.donors.forEach(d=>{dot(map(d.position),teal,.024);arrow(map(d.position),[0,.7*d.weight,0],teal);});
      const centroid=[0,1,2].map(a=>q.donors.reduce((s,d)=>s+d.weight*d.position[a],0));
      line([map(q.position),map(centroid)],warm);dot(map(centroid),teal,.032);dot(map(q.position),warm,.039);
      setMetrics([['Sample position',q.position.map(v=>v.toFixed(3)).join(', ')],['Weight sum',fmt(q.weightSum)],
        ['Interpolated value',fmt(q.value)],['Direct affine value',fmt(q.exact)],['Signed error',fmt(q.error)],
        ['Donor range',`${fmt(q.minimum)} … ${fmt(q.maximum)}`],['Within donor range',String(q.value>=q.minimum&&q.value<=q.maximum)]]);
      table.replaceChildren();
      const heading=document.createElement('tr');for(const label of ['Position (x, y, z)','Weight','Value']){const th=document.createElement('th');th.textContent=label;heading.append(th);}table.append(heading);
      for(const d of q.donors){const row=document.createElement('tr');for(const value of [d.position.map(v=>v.toFixed(2)).join(', '),fmt(d.weight),fmt(d.value)]){const cell=document.createElement('td');cell.textContent=value;row.append(cell);}table.append(row);}
    }
    draw();
  }
  function transport(){
    let profile=0,size=1,courant=50,steps=1;
    meta('Correct transport, then inspect fallback','Three periodic scalar sample curves: teal linear, orange unlimited correction, blue donor-bound fallback. Dark curves show exact translation. Each lane uses identical x and value scales; depth only separates methods.',
      'Constant prescribed velocity on a periodic 1D grid; no velocity self-advection, pressure, liquid interface or native Rust call. The native midpoint/clamped transport API is a different implementation. Bounded fallback can change the sum.', '07-corrected-advection');
    const profileControl=choice('Scalar profile',['Smooth sine','Top hat','Six samples: symmetric edge','Six samples: asymmetric edge'],0,v=>{profile=v;draw();});
    const count=choice('Sample count',['32','64','128','256'],1,v=>{if(v>=0&&v<4)size=v;draw();});
    const fixedCount=new Option('6 (fixed fixture)','fixed');fixedCount.disabled=true;count.add(fixedCount);
    const courantControl=range('Courant × 100',100,50,v=>{courant=v;draw();});
    const stepsControl=range('Transport steps',512,1,v=>{steps=v;draw();});
    const period=document.createElement('button');period.id='one-period';period.textContent='Sine · one period at Courant 0.5';panel.append(period);
    period.onclick=()=>{
      for(const [control,value] of [[profileControl,0],[courantControl,50],[stepsControl,2*[32,64,128,256][size]]]){
        control.value=value;control.dispatchEvent(new Event(control.tagName==='SELECT'?'change':'input'));
      }
    };
    function draw(){
      clear();const q=evolve({profile:['sine','top-hat','symmetric','asymmetric'][profile],n:[32,64,128,256][size],courant:courant/100,steps});setReference(q);count.disabled=profile>=2;count.value=profile>=2?'fixed':String(size);
      const points=(values,z)=>[...values,values[0]].map((v,i)=>[-.8+1.6*i/q.n,.45*v,z]);
      for(const [i,name] of ['linear','raw','limited'].entries()){
        const z=[-.45,0,.45][i];line([[-.8,0,z],[.8,0,z]],0xb6c8cd);line(points(q.exact,z),ink);
        const curve=line(points(q[name],z),[teal,warm,blue][i]);curve.userData.series=name;
        if(q.n===6)points(q[name],z).slice(0,-1).forEach(p=>dot(p,[teal,warm,blue][i],.017));
      }
      const values=[['Grid / steps',`${q.n} samples / ${q.steps} steps`],['Courant / travel',`${q.courant.toFixed(2)} / ${fmt(q.courant*q.steps/q.n)} periods`],['Initial sum',fmt(q.initial.reduce((s,v)=>s+v,0))]];
      for(const [name,label] of [['linear','Linear · teal'],['raw','Raw · orange'],['limited','Limited · blue']]){
        const m=q[name+'Metrics'];values.push([`${label} range`,`${fmt(m.min)} … ${fmt(m.max)}`],[`${label} sum Δ`,fmt(m.amountChange)],[`${label} RMS`,fmt(m.rms)]);
      }
      values.push(['Fallback cells · last / total',`${q.lastFallback} / ${q.fallbackCount}`],['RMS reference',profile===0?'Analytic sine translation':'Translated initial piecewise-linear interpolant']);setMetrics(values);
    }
    draw();
  }
  return {sampler,transport};
}
