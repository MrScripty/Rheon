"""Actual finer attempts retain any unchanged-gate arithmetic refusals."""
import json,sys
from pathlib import Path
import taylor
step=taylor.temporal
out=Path(sys.argv[1]);records=[]
for kind in ['initial','pressure_state']:
 for h in [.0015625,.00078125]:
  trace=[];record=dict(kind=kind,interval=h)
  try:record.update(status='PASS',cell=step.solve(h,kind,diagnostics=False,trace=trace))
  except ValueError as error:record.update(status='REJECTED',reason=str(error))
  record['actual_Newton_trace']=trace;records.append(record);out.write_text(json.dumps(dict(scope='actual bounded original-gate finer cell attempts; refusal is preserved, not a changed real-equation limit',records=records),indent=2)+'\n');print(kind,h,record['status'],flush=True)
