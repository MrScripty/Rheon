"""Compare every slider pair against an independent exact piecewise oracle."""
from fractions import Fraction as F
from pathlib import Path
import json
import subprocess
import unittest

HERE=Path(__file__).resolve().parent
class VolumeReference(unittest.TestCase):
    def test_all_slider_states(self):
        source="""import {sweptVolume} from './volume_reference.js';
        const rows=[];
        for(const side of ['left','right'])for(let i=0;i<=100;i++)for(let j=0;j<=100;j++){
          const q=sweptVolume(i/100,j/100,side);
          rows.push([q.q,q.donor,q.receiver,q.mixed,q.clampDrift]);
        } console.log(JSON.stringify(rows));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',source],cwd=HERE,text=True))
        n=0
        for side in ('left','right'):
            for i in range(101):
                for j in range(101):
                    f,c=F(i,100),F(j,100)
                    # Piecewise integration of the indicator over the exiting slab.
                    if side=='left':q=F(0) if f<=1-c else f-(1-c)
                    else:q=f if f<=c else c
                    expected=[q,f-q,q,f*c,max(F(0),c-f)]
                    for actual,want in zip(rows[n],expected):
                        self.assertAlmostEqual(actual,float(want),delta=5e-15)
                    self.assertGreaterEqual(rows[n][1],-5e-15)
                    self.assertLessEqual(rows[n][2],1+5e-15)
                    n+=1
        self.assertEqual(n,20402)
    def test_invalid_inputs_rejected(self):
        source="""import {sweptVolume} from './volume_reference.js';
        for(const args of [[-1,0],[0,1.1],[NaN,0],[0,Infinity],[0,0,'front']]){
          let rejected=false;try{sweptVolume(...args);}catch(e){rejected=e instanceof RangeError;}
          if(!rejected)throw Error('Invalid input admitted');
        }"""
        subprocess.run(['node','--input-type=module','-e',source],cwd=HERE,check=True)
if __name__=='__main__':unittest.main()
