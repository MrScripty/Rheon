from pathlib import Path
import json
import subprocess
import unittest
from engineering_oracle import verify

HERE=Path(__file__).resolve().parent
class EngineeringReference(unittest.TestCase):
    def test_complete_bounded_domains(self):
        code="""import {faceWork,checkerboard,affineStress} from './engineering_reference.js';
        let rows=[];
        for(const a of [.25,.5,1])for(const u of [-1,0,1])for(const b of [-.5,0,.5])for(const p of [0,1,2])rows.push(['faceflux',faceWork(a,u,b,p)]);
        for(const a of [0,.5,1])for(const r of [.25,.5,1])for(const p of ['constant','alternating'])rows.push(['checker',checkerboard(a,r,p)]);
        for(const s of [-1,0,1])for(const m of [0,.5,1])for(const k of ['rotation','shear','extension'])rows.push(['stressprobe',affineStress(s,m,k)]);
        console.log(JSON.stringify(rows));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],cwd=HERE,text=True))
        self.assertEqual(len(rows),126)
        for lab,q in rows:verify(q,lab)

    def test_invalid_controls_refuse(self):
        code="""import {faceWork,checkerboard,affineStress} from './engineering_reference.js';
        const bad=[()=>faceWork(0),()=>faceWork(.5,NaN),()=>faceWork(.5,1,.2),()=>faceWork(.5,1,0,3),()=>checkerboard(1,-1),()=>checkerboard(1,1,'random'),()=>affineStress(1,-1),()=>affineStress(Infinity),()=>affineStress(1,1,'turbulence')];
        for(const f of bad){let refused=false;try{f();}catch(e){if(!(e instanceof RangeError))throw e;refused=true;}if(!refused)throw Error('Accepted unsupported state');}"""
        subprocess.run(['node','--input-type=module','-e',code],cwd=HERE,check=True)

    def test_independent_oracle_rejects_wrong_boundary_or_stress(self):
        code="""import {faceWork,affineStress} from './engineering_reference.js';console.log(JSON.stringify([faceWork(),affineStress()]));"""
        flux,stress=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],cwd=HERE,text=True))
        flux['boundaryWork']=0
        with self.assertRaises(ValueError):verify(flux,'faceflux')
        stress['dissipation']=stress['gradientProxy']
        with self.assertRaises(ValueError):verify(stress,'stressprobe')

if __name__=='__main__':unittest.main()
