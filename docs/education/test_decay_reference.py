"""Independent rational powers and high-precision exponential, not JS iteration."""
from decimal import Decimal, localcontext
from fractions import Fraction as F
from pathlib import Path
import json
import math
import subprocess
import unittest

HERE=Path(__file__).resolve().parent
STEPS=[1,2,4,8,16,32,64]
class DecayReference(unittest.TestCase):
    def test_all_control_states(self):
        source="""import {modeDecay,decaySteps} from './decay_reference.js';
        console.log(JSON.stringify([1,2,3].flatMap(t=>decaySteps.flatMap(n=>
          Array.from({length:n+1},(_,j)=>modeDecay(t,n,j))))));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',source],cwd=HERE,text=True))
        self.assertEqual(len(rows),402)
        for q in rows:
            t,n,j=q['duration'],q['steps'],q['index'];a=F(t,n)
            with localcontext() as ctx:
                ctx.prec=60
                exact=float((-Decimal(t*j)/Decimal(n)).exp())
            amplitudes={'explicit':float((1-a)**j),'implicit':float((1/(1+a))**j),'exact':exact}
            for method,want in amplitudes.items():
                self.assertAlmostEqual(q['amplitudes'][method],want,delta=5e-14)
                self.assertAlmostEqual(q['energy'][method],want*want/4,delta=5e-14)
            for method in ['explicit','implicit']:
                self.assertAlmostEqual(q['error'][method],abs(amplitudes[method]-exact),delta=5e-14)
            self.assertEqual(q['explicitEnergyStable'],a<=2)
            self.assertEqual(q['explicitSignPreserving'],a<=1)
        # Fixed final time: independently observed error decreases on refinement.
        errors=[q['error'] for q in rows if q['duration']==2 and q['steps']>=2 and q['index']==q['steps']]
        for method in ['explicit','implicit']:
            self.assertTrue(all(a[method]>b[method] for a,b in zip(errors,errors[1:])))
    def test_physical_samples_and_energy(self):
        source="""import {shearSamples} from './decay_reference.js';
        console.log(JSON.stringify([-2,-1,0,.5,1].map(a=>[a,shearSamples(a)])));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',source],cwd=HERE,text=True))
        for amplitude,samples in rows:
            self.assertEqual(len(samples),48)
            for sample in samples:
                x,y,z=sample['position'];vx,vy,vz=sample['velocity']
                self.assertEqual((vy,vz),(0,0))
                self.assertAlmostEqual(vx,amplitude*math.sin(2*math.pi*y),delta=1e-14)
            # Uniform sample quadrature is exact for this single trigonometric mode.
            self.assertAlmostEqual(sum(sum(v*v for v in s['velocity']) for s in samples)/96,amplitude**2/4,delta=1e-14)
    def test_invalid_inputs(self):
        source="""import {modeDecay,shearSamples} from './decay_reference.js';
        for(const f of [()=>modeDecay(0,1),()=>modeDecay(2,3),()=>modeDecay(2,1,-1),
          ()=>modeDecay(2,1,2),()=>modeDecay(2,1,.5),()=>modeDecay(NaN,1),
          ()=>shearSamples(Infinity),()=>shearSamples(3)]){
          let rejected=false;try{f();}catch(e){rejected=e instanceof RangeError;}
          if(!rejected)throw Error('Invalid input admitted');
        }"""
        subprocess.run(['node','--input-type=module','-e',source],cwd=HERE,check=True)
if __name__=='__main__':unittest.main()
