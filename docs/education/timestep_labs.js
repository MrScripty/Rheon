// A bounded UI adapter, not a JavaScript solver. All state comes from Rust/WASM.
export function createTimestepLab({clear,line,dot,arrow,box,meta,setMetrics,panel,setReference,isActive,manualChange}){
  let worker, current=null, lastReport=null, busy=false, failure=null, included=null, starting=false;
  let serial=0, acceptedCalls=0, rejectedCalls=0, requestCount=0, method='jacobi-pcg-v1';
  const pending=new Map();
  let buttons=[], status, select, table;
  const positions=[[1,.5,.5],[1,1.5,.5],[.5,1,.5],[1.5,1,.5]];
  const expected=[.125,-.125,-.125,.125];
  const faceValues=s=>[s.x[1],s.x[4],s.y[2],s.y[3]];
  const point=p=>p.map((v,i)=>(v-[1,1,.5][i])*.6);
  const fmt=v=>Number(v).toPrecision(6);
  function inspection(){return {included,busy,runtime:current?.runtime||null,state:current?.state||null,
    outcome:current?.outcome||null,lastReport,requestCount,acceptedCalls,rejectedCalls,error:failure};}
  function draw(){
    if(!isActive())return;
    clear();
    box([1.2,1.2,.6],[0,0,0],0x18363d,.035);
    box([1.2,.6,.6],[0,-.3,0],0xaa5735,.08);
    for(const z of [-.3,.3]){
      line([[0,-.6,z],[0,.6,z]],0x9fb6ba);
      line([[-.6,0,z],[.6,0,z]],0x9fb6ba);
    }
    const s=current?.state;
    if(s){
      faceValues(s).forEach((v,i)=>{dot(point(positions[i]),0x18363d,.018);const vector=[0,0,0];vector[i<2?0:1]=v*1.6;arrow(point(positions[i]),vector,0x067d91);});
      const values=faceValues(s), accepted=s.accepted_since_reset;
      setMetrics([['Accepted time',fmt(s.time)+' s'],['State generation',s.generation_u64],
        ['Last operation',current.outcome?.code||'Ready at rest'],
        ['Accepted dt',lastReport?fmt(lastReport.dt)+' s':'—'],
        ['Accepted kinetic energy',lastReport?fmt(lastReport.kinetic_energy)+' J':'0 J'],
        ['Stored divergence max',lastReport?fmt(lastReport.actual_divergence_max)+' s⁻¹':'0 s⁻¹'],
        ['Pressure iterations',lastReport?String(lastReport.pressure.iterations):'No accepted solve yet'],
        ['Pressure true residual max',lastReport?fmt(lastReport.pressure.true_residual_max):'—'],
        ['Force-stage work',lastReport?fmt(lastReport.forces.applied_work)+' J':'—'],
        ['Accepted Courant',lastReport?fmt(lastReport.courant):'—'],
        ['Error against c/8',accepted?fmt(Math.max(...values.map((v,i)=>Math.abs(v-expected[i]))))+' m/s':'Run one step to compare']]);
      table.replaceChildren();
      const heading=document.createElement('p');heading.textContent='Interior face speeds (m/s)';table.append(heading);
      const rows=document.createElement('table');rows.innerHTML='<thead><tr><th>Face</th><th>Accepted</th><th>Reference</th></tr></thead>';
      const body=document.createElement('tbody');
      ['x lower','x upper','y left','y right'].forEach((label,i)=>{
        const row=document.createElement('tr');
        for(const text of [label,fmt(values[i]),accepted?fmt(expected[i]):'after step']){const cell=document.createElement('td');cell.textContent=text;row.append(cell);}
        body.append(row);
      });rows.append(body);table.append(rows);
    } else setMetrics([['Runtime',included===false?'Build not included':failure?'Unavailable':'Loading WebAssembly…']]);
    status.textContent=failure?'Solver unavailable: '+failure:included===false?'This edition has no compiled solver packet. No timestep can run.':busy?'Computing in the solver worker…':current?.outcome?.code==='cancelled_before_commit'?'Rejected at BeforeCommit. Accepted fields, time and generation are unchanged.':current?.outcome?.status==='rejected'?'Step rejected: '+current.outcome.code+'. '+(current.outcome.detail||''):s?.accepted_since_reset?'One step accepted. Inspect the reference, reject a candidate, or reset.':'At rest. Predict the circulation, then run one step.';
    buttons.forEach(button=>button.disabled=busy||!!failure||included!==true||!s||(button.id==='timestep-step'&&s.accepted_since_reset));
    select.disabled=busy||!!failure||included!==true;
    setReference(inspection());
  }
  function fail(error){failure=String(error);busy=false;worker?.terminate();worker=null;draw();}
  async function command(action){
    if(busy||failure||included!==true)return;
    busy=true;requestCount++;draw();
    try{
      const id=++serial;
      const result=await new Promise((resolve,reject)=>{
        const timer=setTimeout(()=>{pending.delete(id);reject(Error('Solver worker did not respond. Reload to start a new fixture.'));},30000);
        pending.set(id,{resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}});
        worker.postMessage({id,action,method});
      });
      current=result;
      if(result.outcome?.status==='accepted'){acceptedCalls++;lastReport=result.outcome.report;}
      else if(result.outcome?.status==='rejected')rejectedCalls++;
      else lastReport=null;
      busy=false;draw();
    }catch(error){fail(error);}
  }
  async function start(){
    try{
      const response=await fetch(new URL('./solver-presentation.json',import.meta.url));
      if(!response.ok)throw Error('Missing solver build declaration ('+response.status+').');
      const presentation=await response.json();included=presentation.included===true;
      if(!included){draw();return;}
      worker=new Worker(new URL('./timestep_worker.js',import.meta.url),{type:'module'});
      worker.onmessage=({data})=>{const request=pending.get(data.id);if(!request)return;pending.delete(data.id);data.error?request.reject(Error(data.error)):request.resolve(data);};
      worker.onerror=error=>fail(error.message||'Worker failed.');
      await command('configure');
    }catch(error){fail(error);}
  }
  return function timestep(){
    meta('9 · One accepted Rust timestep','A localized x-acceleration drives a four-cell sealed box. WebAssembly executes the existing force/projection step; arrows show only accepted velocities.','Fixed 2 × 2 × 1 grid, unit spacing and density, dt = 0.25 s. Initial advection acts on rest; tracer stays zero. No liquid interface, viscosity, buoyancy or contact physics. The rejection button schedules a native BeforeCommit probe; it is not an asynchronous interrupt.','17-rust-implementation');
    const label=document.createElement('label');label.textContent='Pressure method · changing it starts a new run';
    select=document.createElement('select');select.id='timestep-method';select.setAttribute('aria-label','Pressure method');
    select.add(new Option('Jacobi PCG','jacobi-pcg-v1'));select.add(new Option('Symmetric Gauss–Seidel PCG','sgs-pcg-v1'));select.value=method;
    select.onchange=()=>{manualChange();method=select.value;command('configure');};panel.append(label,select);
    const actions=document.createElement('div');actions.className='timestep-actions';buttons=[];
    for(const [id,text,action] of [['timestep-step','Run one step','step'],['timestep-reject','Reject before commit','reject'],['timestep-reset','Reset state','reset']]){
      const button=document.createElement('button');button.id=id;button.textContent=text;button.onclick=()=>{manualChange();command(action);};buttons.push(button);actions.append(button);
    }
    status=document.createElement('p');status.id='timestep-status';status.setAttribute('role','status');
    table=document.createElement('div');table.className='donor-table';table.id='timestep-faces';panel.append(actions,status,table);
    draw();if(!starting){starting=true;start();}
  };
}
