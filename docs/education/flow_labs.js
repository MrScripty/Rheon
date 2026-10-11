// Controls and accepted-state presentation only; all fluid steps run in Rust.
import {FlowVolume} from './flow_volume.js';
import {FLOW_LIGHT_EXTINCTION,FLOW_LIGHT_AMBIENT} from './flow_field.js';

export function createFlowLab({clear,meta,setMetrics,panel,setReference,isActive,manualChange,view}){
  let worker=null, volume=null, renderedState=null, current=null, lastReport=null;
  let included=null, starting=false, mounted=false, busy=false, running=false, disposed=false;
  let failure=null, renderFailure=null, method='jacobi-pcg-v1', sourceOffIntegral=null;
  let lighting='lit', showStreamlines=true;
  let serial=0, requestCount=0, acceptedCalls=0, rejectedCalls=0, scheduled=null;
  let runButton, resetButton, status, time, progress, diagnostics, methodSelect;
  let legendHome=null;
  const borrowed=[];
  const pending=new Map();
  const format=value=>Number.isFinite(value)?Number(value).toPrecision(5):'—';
  const sourceSteps=()=>current?.state?.source_steps??30;

  function inspect({derived=false}={}){
    return {included,busy,running,mounted,runtime:current?.runtime||null,
      state:current?.state||null,outcome:current?.outcome||null,lastReport,
      requestCount,acceptedCalls,rejectedCalls,error:failure,renderError:renderFailure,
      sourceOffIntegral,renderer:volume?.snapshot?.({derived})||null};
  }

  function diagnosticRows(){
    const state=current?.state, report=lastReport;
    const integral=report?.tracer_integral;
    const loss=sourceOffIntegral>0&&Number.isFinite(integral)?
      100*(integral/sourceOffIntegral-1):null;
    const native=state?.metrics;
    return [
      ['Accepted dt',report?format(report.dt)+' s':'—'],
      ['State generation',state?.generation_u64??'—'],
      ['Stored divergence max',report?format(report.actual_divergence_max)+' s⁻¹':'—'],
      ['Courant number',report?format(report.courant):'—'],
      ['Kinetic energy',report?format(report.kinetic_energy)+' J':'—'],
      ['Force-stage work',report?.forces?format(report.forces.applied_work)+' J':'—'],
      ['Pressure iterations',report?String(report.pressure.iterations):'—'],
      ['Pressure residual max',report?format(report.pressure.true_residual_max):'—'],
      ['Tracer integral',report?format(integral)+' m³':'0 m³'],
      ['Change since source stopped',loss===null?'Available after the source pulse':(loss>=0?'+':'')+format(loss)+'%'],
      ...(native?[
        ['Tracer range',[native.tracer_min,native.tracer_max].map(format).join(' … ')],
        ['Maximum |u / v / w|',native.max_u_v_w.map(format).join(' / ')+' m/s'],
      ]:[]),
      ['Opacity transfer','Fixed extinction 32 m⁻¹ per concentration'],
      ['Depth lighting',lighting==='lit'?'+Y · extinction '+FLOW_LIGHT_EXTINCTION+' m⁻¹ per concentration · ambient '+Math.round(100*FLOW_LIGHT_AMBIENT)+'%':'Off'],
      ['Velocity curves',showStreamlines?'Instantaneous · fixed 0–1 m/s colour scale':'Off'],
      ['Numerical model','Sealed box · passive tracer · no internal obstacle'],
    ];
  }

  function draw(){
    if(!mounted||!isActive())return;
    const state=current?.state;
    if(volume&&state!==renderedState&&state){
      try{volume.update(state);renderedState=state;}
      catch(error){renderFailure=String(error);pause(false);}
    }
    const count=state?.accepted_steps??0, limit=state?.max_steps??80;
    time.textContent=(state?state.time.toFixed(3):'0.000')+' s';
    progress.textContent='24³ cells · '+count+'/'+limit;
    progress.setAttribute('aria-label','24 by 24 by 24 native cells; '+count+' of '+limit+' accepted steps');
    progress.title=count+' of '+limit+' accepted steps';
    runButton.textContent=running?'Pause':count>=limit?'Complete':'Run';
    runButton.setAttribute('aria-pressed',String(running));
    runButton.disabled=!!failure||included!==true||!state||!!renderFailure||
      (!running&&(busy||count>=limit));
    resetButton.disabled=busy||running||!!failure||included!==true||!state;
    methodSelect.disabled=busy||running||!!failure||included!==true;
    methodSelect.value=state?.method||method;
    const velocityLegend=document.querySelector('#flow-velocity-legend');
    if(velocityLegend)velocityLegend.hidden=!showStreamlines;
    if(failure)status.textContent='Flow unavailable: '+failure+' Last received accepted state retained.';
    else if(renderFailure)status.textContent='Volume display unavailable: '+renderFailure;
    else if(included===false)status.textContent='This edition has no compiled flow solver. No fluid steps can run.';
    else if(!state)status.textContent='Loading the Rust / WebAssembly flow scene…';
    else if(busy&&!running)status.textContent='Finishing the pending operation; its accepted state will be shown.';
    else if(current.outcome?.status==='rejected'&&current.outcome.code!=='step_limit'){
      status.textContent='Step rejected: '+current.outcome.code+'. '+(current.outcome.detail||'Accepted state is unchanged.');
    }else if(count>=limit)status.textContent=limit+' accepted steps complete · source off';
    else if(running)status.textContent=count<sourceSteps()?
      'Running · source active':
      'Running · source off; force continues';
    else if(count===0)status.textContent='At rest · '+sourceSteps()+'-step source pulse';
    else status.textContent='Paused · '+(count<sourceSteps()?'source active':'source off; force continues');
    diagnostics.replaceChildren();
    for(const [label,value] of diagnosticRows()){
      const dt=document.createElement('dt'), dd=document.createElement('dd');
      dt.textContent=label;dd.textContent=value;diagnostics.append(dt,dd);
    }
    setReference(inspect());
  }

  function pause(redraw=true){
    running=false;
    if(scheduled!==null){clearTimeout(scheduled);scheduled=null;}
    if(redraw)draw();
  }

  function fail(error){
    pause(false);failure=String(error);busy=false;
    worker?.terminate();worker=null;
    const requests=[...pending.values()];pending.clear();
    for(const request of requests)request.reject(Error(failure));
    draw();
  }

  async function command(action){
    if(busy||failure||included!==true||!worker)return;
    busy=true;requestCount++;draw();
    try{
      const id=++serial;
      const result=await new Promise((resolve,reject)=>{
        const timer=setTimeout(()=>{
          pending.delete(id);reject(Error('Flow worker did not respond. Reload to start a new owner.'));
        },30000);
        pending.set(id,{
          resolve:value=>{clearTimeout(timer);resolve(value);},
          reject:error=>{clearTimeout(timer);reject(error);},
        });
        worker.postMessage({id,action,method});
      });
      current=result;
      sourceOffIntegral=result.state.tracer_integral_at_source_cutoff??null;
      if(result.outcome?.status==='accepted'){
        acceptedCalls++;lastReport=result.outcome.report;
      }else if(result.outcome?.status==='rejected'){
        rejectedCalls++;pause(false);
      }else if(action==='configure'||action==='reset'){
        lastReport=null;sourceOffIntegral=null;
      }
      if(result.state.accepted_steps>=result.state.max_steps)pause(false);
      busy=false;draw();
      return result;
    }catch(error){fail(error);}
  }

  async function pump(){
    scheduled=null;
    if(!running||busy||failure||!mounted||!isActive())return;
    await command('step');
    if(running&&!busy&&!failure&&mounted&&isActive()){
      scheduled=setTimeout(pump,0);
    }
  }

  async function start(){
    try{
      const response=await fetch(new URL('./solver-presentation.json',import.meta.url));
      if(!response.ok)throw Error('Missing solver build declaration ('+response.status+').');
      const presentation=await response.json();included=presentation.included===true;
      if(disposed)return;
      if(!included){draw();return;}
      worker=new Worker(new URL('./flow_worker.js',import.meta.url),{type:'module'});
      worker.onmessage=({data})=>{
        const request=pending.get(data.id);if(!request)return;pending.delete(data.id);
        data.error?request.reject(Error(data.error)):request.resolve(data);
      };
      worker.onerror=event=>fail(event.message||'Flow worker failed.');
      worker.onmessageerror=()=>fail('Flow worker returned an unreadable message.');
      await command('configure');
    }catch(error){fail(error);}
  }

  function unmount(){
    pause(false);mounted=false;
    volume?.dispose();volume=null;renderedState=null;
    for(const {node,placeholder,open} of borrowed.splice(0)){
      if(node instanceof HTMLDetailsElement)node.open=open;
      node.classList.remove('flow-inspector-content');
      placeholder.replaceWith(node);
    }
    if(legendHome){
      legendHome.placeholder.replaceWith(legendHome.node);legendHome=null;
    }
  }

  function dispose(){
    unmount();disposed=true;failure='Flow controller disposed.';busy=false;
    worker?.terminate();worker=null;
    const requests=[...pending.values()];pending.clear();
    for(const request of requests)request.reject(Error(failure));
  }

  function mount(){
    if(disposed)throw Error('Flow controller has been disposed.');
    volume?.dispose();volume=null;renderedState=null;
    clear();mounted=true;renderFailure=null;
    const legend=document.querySelector('#plot-legend'),scene=document.querySelector('#scene');
    if(legend&&scene){
      const placeholder=document.createComment('flow legend home');
      legend.replaceWith(placeholder);scene.after(legend);legendHome={node:legend,placeholder};
    }
    if(view){
      try{volume=new FlowVolume(view,{lighting,streamlines:showStreamlines});}
      catch(error){renderFailure=String(error);}
    }else renderFailure='WebGL is unavailable.';
    meta('Computed 3D flow',
      'A source pulse in a 24³ sealed box. The volume shows accepted tracer concentration; the curves follow the velocity field at this one time.',
      'Coarse 3D appearance transport, with fixed concentration-to-opacity mapping. The tracer integral is not conserved by semi-Lagrangian sampling. No internal obstacle, free surface, buoyancy or resolved turbulence is simulated.',
      '14-guidance-rendering');
    setMetrics([]);
    const actions=document.createElement('div');actions.className='flow-actions';
    runButton=document.createElement('button');runButton.id='flow-run';
    runButton.onclick=()=>{
      manualChange();
      if(running)pause();
      else if(!busy&&!failure&&included===true&&current?.state&&
          current.state.accepted_steps<current.state.max_steps){
        running=true;draw();pump();
      }
    };
    resetButton=document.createElement('button');resetButton.id='flow-reset';resetButton.textContent='Reset';
    resetButton.onclick=()=>{manualChange();pause(false);command('reset');};
    const summary=document.createElement('div');summary.className='flow-summary';
    time=document.createElement('strong');time.id='flow-time';
    progress=document.createElement('span');progress.id='flow-progress';
    summary.append(time,progress);
    actions.append(runButton,resetButton,summary);
    status=document.createElement('p');status.id='flow-status';status.className='flow-status';status.setAttribute('role','status');
    const details=document.createElement('details');details.id='flow-diagnostics';
    const title=document.createElement('summary');title.textContent='Inspect · model, numerics & sources';
    const label=document.createElement('label');label.htmlFor='flow-method';label.textContent='Pressure method · starts a new run';
    methodSelect=document.createElement('select');methodSelect.id='flow-method';
    methodSelect.add(new Option('Jacobi PCG','jacobi-pcg-v1'));
    methodSelect.add(new Option('Symmetric Gauss–Seidel PCG','sgs-pcg-v1'));
    methodSelect.value=method;
    methodSelect.onchange=()=>{
      if(busy||running||failure||included!==true)return;
      manualChange();pause(false);method=methodSelect.value;command('configure');
    };
    diagnostics=document.createElement('dl');diagnostics.className='flow-diagnostic-values';
    details.append(title);
    // Keep the shared book evidence nodes and their identities. Restore them
    // before the next lesson clears its controls, including prior open states.
    function borrow(selector){
      const node=panel.closest('.lab-panel').querySelector(selector);
      if(!node)return;
      const placeholder=document.createComment('flow inspector: '+selector);
      const open=node instanceof HTMLDetailsElement?node.open:null;
      node.replaceWith(placeholder);
      if(node instanceof HTMLDetailsElement)node.open=true;
      node.classList.add('flow-inspector-content');
      borrowed.push({node,placeholder,open});details.append(node);
    }
    borrow('#lab-description');
    borrow('.example-notes');
    const display=document.createElement('div');display.className='flow-display-controls';
    for(const [id,text,checked,change] of [
      ['flow-lighting','Depth lighting',lighting==='lit',value=>{lighting=value?'lit':'unlit';}],
      ['flow-streamlines','Instantaneous velocity',showStreamlines,value=>{showStreamlines=value;}],
    ]){
      const label=document.createElement('label'),input=document.createElement('input');
      input.type='checkbox';input.id=id;input.checked=checked;
      input.onchange=()=>{
        manualChange();change(input.checked);
        volume?.setDisplay({lighting,streamlines:showStreamlines});draw();
      };
      label.append(input,document.createTextNode(text));display.append(label);
    }
    const displayLimit=document.createElement('p');displayLimit.className='flow-display-limit';
    displayLimit.textContent='Lighting uses the tracer above each sample as a depth cue; it does not change opacity. Curve overlays use six fixed seeds; colour saturates at 1 m/s. They are instantaneous streamlines, not particle paths. Display interpolation adds no simulation detail.';
    details.append(display,displayLimit);
    details.append(label,methodSelect,diagnostics);
    borrow('.example-sources');
    borrow('#chapter-target');
    panel.append(actions,status,details);
    draw();
    if(!starting){starting=true;start();}
  }

  Object.assign(mount,{inspect,pause,unmount,dispose});
  return mount;
}
