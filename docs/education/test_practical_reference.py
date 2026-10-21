import itertools,json,subprocess,unittest,copy
from pathlib import Path
from practical_oracle import DOMAINS,verify
HERE=Path(__file__).resolve().parent
class PracticalReference(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cases=[[n,list(a)] for n,d in DOMAINS.items() for a in itertools.product(*d)]
  code="import {references} from './practical_reference.js';let s='';for await(const c of process.stdin)s+=c;console.log(JSON.stringify(JSON.parse(s).map(([n,a])=>references[n](...a))));"
  cls.rows=json.loads(subprocess.check_output(['node','--input-type=module','-e',code],input=json.dumps(cls.cases),cwd=HERE,text=True))
 def test_all_admitted_states(self):
  self.assertEqual(len(self.rows),len(self.cases))
  for q in self.rows:verify(q)
 def test_oracle_rejects_wrong_physics_and_decisions(self):
  for name,key in [('wallload','top'),('jacobi','alternatingFactor'),('memory','nnz'),('roundoff','total'),('particletransfer','varianceAfter'),('sstprobe','limited'),('refinement','extrapolated'),('proposal','accepted')]:
   q=copy.deepcopy(next(q for q in self.rows if q['kind']==name));q[key]=not q[key] if type(q[key]) is bool else q[key]+.125
   with self.assertRaises((ValueError,AssertionError)):verify(q)
 def test_invalid_arguments_refuse(self):
  code="import {references,domains} from './practical_reference.js';for(const [n,d] of Object.entries(domains))for(let i=0;i<d.length;i++){let a=d.map(x=>x[0]);a[i]=NaN;let ok=false;try{references[n](...a)}catch(e){if(!(e instanceof RangeError))throw e;ok=true}if(!ok)throw Error(n+' accepted NaN')}"
  subprocess.run(['node','--input-type=module','-e',code],cwd=HERE,check=True)
if __name__=='__main__':unittest.main()
