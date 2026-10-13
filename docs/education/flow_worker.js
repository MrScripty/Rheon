// One native owner; every command completes before the next command starts.
// A step is synchronous inside this worker. Pause is a controller operation
// that stops sending further steps; it cannot interrupt an in-flight step.
import init, {FlowSimulation} from './solver/rheon_solver.js';

const ready=init();
let simulation=null, lastState=null, queue=Promise.resolve();

function readState(value){
  const state=typeof value==='string'?JSON.parse(value):value;
  if(state?.schema!==1||state.fixture!=='force-plume-24-v1'||
      state.grid?.counts?.join(',')!=='24,24,24'||
      !Number.isInteger(state.accepted_steps)||state.accepted_steps<0||
      state.accepted_steps>80||state.max_steps!==80||
      !Number.isFinite(state.time)||typeof state.generation_u64!=='string'){
    throw Error('Unexpected native flow snapshot.');
  }
  for(const [name,length] of [['x',14400],['y',14400],['z',14400],['tracer',13824]]){
    if(!Array.isArray(state[name])||state[name].length!==length||
        state[name].some(value=>!Number.isFinite(value))){
      throw Error('Invalid native flow field: '+name+'.');
    }
  }
  return state;
}

async function handle(message){
  const {id,action,method='jacobi-pcg-v1'}=message||{};
  try{
    if(!Number.isSafeInteger(id)||id<1)throw Error('Invalid flow request identity.');
    await ready;
    let outcome=null, state;
    if(action==='configure'){
      // Do not destroy an existing owner before its replacement is ready.
      const next=new FlowSimulation(method);
      try{state=readState(next.snapshot());}
      catch(error){next.free();throw error;}
      simulation?.free();simulation=next;
    }else{
      if(!simulation)throw Error('Create the bounded flow scene first.');
      if(action==='snapshot')state=readState(simulation.snapshot());
      else if(action==='reset')state=readState(simulation.reset());
      else if(action==='step'||action==='reject'){
        outcome=JSON.parse(simulation.step(action==='reject'));
        if(!['accepted','rejected'].includes(outcome?.status))throw Error('Invalid native flow outcome.');
        state=readState(outcome.state);
      }else throw Error('Unknown flow command.');
    }
    lastState=state;
    self.postMessage({id,state,outcome,runtime:'wasm-worker'});
  }catch(error){
    self.postMessage({id,error:String(error),state:lastState,runtime:'wasm-worker'});
  }
}

self.onmessage=({data})=>{queue=queue.then(()=>handle(data));};
