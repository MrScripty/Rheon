"""Independent rational rounding of actual scalar-only affine preflight probes."""
from pathlib import Path
import importlib.util,json
from fractions import Fraction as Q
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('primitive_oracle',P/'check_scalar.py');o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)
def run():
 rows=[json.loads(x)for x in(P/'affine-native.log').read_text().splitlines()if x.startswith('{')];o.require(len(rows)==6 and [r['case']for r in rows]==list(range(6)),'closed affine scalar-only roster')
 results=[]
 for row in rows:
  product=o.rounded(Q(row['time'])*Q(row['alpha']));expected=None if product is None else o.rounded(Q(row['eta'])+o.value(product));o.require(row['refused']==(expected is None),'every intermediate range guard')
  if expected is not None:o.require(row['parts']==expected and Q(row['stored'])==Q(float(o.value(expected))),'actual two-operation 106-bit affine parts and endpoint conversion')
  results.append({'case':row['case'],'expected_parts':expected,'refused':expected is None})
 return {'status':'PASS_SIX_SCALAR_AFFINE_PROBES','native_candidate_equations':0,'rows':results}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
