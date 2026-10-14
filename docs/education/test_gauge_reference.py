import json,subprocess,unittest
from pathlib import Path
from gauge_oracle import verify
HERE=Path(__file__).resolve().parent
class GaugeReference(unittest.TestCase):
    def test_all_controls(self):
        code="""import {pressureGauge} from './gauge_reference.js';const rows=[];
        for(const i of ['compatible','global-only'])for(const g of ['uniform','unequal'])for(const m of ['fixed','mean','volume','projected'])for(const a of [-2,0,2])for(const b of [-2,0,2])for(const e of [0,.25])rows.push(pressureGauge(i,g,m,a,b,e));console.log(JSON.stringify(rows));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],cwd=HERE,text=True));self.assertEqual(len(rows),288)
        for q in rows:verify(q)
    def test_refuses_unsupported(self):
        subprocess.run(['node','--input-type=module','-e',"""import {pressureGauge as f} from './gauge_reference.js';for(const a of [['bad'],['compatible','bad'],['compatible','uniform','bad'],['compatible','uniform','fixed',NaN],['compatible','uniform','fixed',0,Infinity],['compatible','uniform','fixed',0,0,1]]){let ok=false;try{f(...a)}catch(e){if(!(e instanceof RangeError))throw e;ok=true}if(!ok)throw Error('Invalid input admitted')}"""],cwd=HERE,check=True)
    def test_rejects_silent_repair_and_dropped_gauge_row(self):
        code="""import {pressureGauge} from './gauge_reference.js';console.log(JSON.stringify([pressureGauge('global-only'),pressureGauge('global-only','unequal','projected')]));"""
        rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],cwd=HERE,text=True))
        rows[0]['fullResidual']=rows[0]['reducedResidual'];rows[1]['acceptedOriginal']=True
        for q in rows:
            with self.assertRaises(ValueError):verify(q)
if __name__=='__main__':unittest.main()
