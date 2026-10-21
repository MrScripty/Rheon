// One owned native fixture per worker. Commands never overlap a synchronous step.
import init, {CycleSimulation} from './solver/rheon_solver.js';
let simulation;
const ready=init();
self.onmessage=async ({data:{id,action,method}})=>{
  try {
    await ready;
    let outcome=null, state;
    if(action==='configure'){
      simulation?.free();
      simulation=new CycleSimulation(method);
      state=JSON.parse(simulation.snapshot());
    } else {
      if(!simulation)throw Error('Create the fixed fixture first.');
      if(action==='reset')state=JSON.parse(simulation.reset());
      else if(action==='step'||action==='reject'){
        outcome=JSON.parse(simulation.step(.25,action==='reject'));
        state=outcome.state;
      } else throw Error('Unknown fixture command.');
    }
    self.postMessage({id,state,outcome,runtime:'wasm-worker'});
  } catch(error){self.postMessage({id,error:String(error)});}
};
