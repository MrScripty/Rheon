import itertools,json,subprocess,unittest
from pathlib import Path
from completion_oracle import DOMAINS,verify
HERE=Path(__file__).resolve().parent
class CompletionReference(unittest.TestCase):
 def test_every_finite_state(self):
  cases=[[name,list(args)] for name,domain in DOMAINS.items() for args in itertools.product(*domain)]
  code="import {references} from './completion_reference.js';let s='';for await(const c of process.stdin)s+=c;console.log(JSON.stringify(JSON.parse(s).map(([n,a])=>references[n](...a))));"
  rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],input=json.dumps(cases),cwd=HERE,text=True));self.assertEqual(len(rows),len(cases))
  for q in rows:verify(q)
 def test_invalid_controls_refuse(self):
  code="""import {references,domains} from './completion_reference.js';for(const [name,domain] of Object.entries(domains))for(let i=0;i<domain.length;i++){let a=domain.map(d=>d[0]);a[i]=NaN;let refused=false;try{references[name](...a)}catch(e){if(!(e instanceof RangeError))throw e;refused=true}if(!refused)throw Error(name+' admitted invalid control')}"""
  subprocess.run(['node','--input-type=module','-e',code],cwd=HERE,check=True)
if __name__=='__main__':unittest.main()
