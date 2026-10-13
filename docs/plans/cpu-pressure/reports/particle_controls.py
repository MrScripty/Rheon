"""Balanced interleaved frozen-binary controls: baseline candidate output.json."""
import subprocess,json,random,sys
from pathlib import Path
baseline,candidate,output=sys.argv[1:]
random.seed(582)
rows=[]
cases=[('unbound-short',16,1,5,None,20,False),('unbound-long',16,1,50,None,20,False),('cpu2-long',16,1,50,'2',20,False),('same-binary-long',16,1,50,None,20,True)]
cases += [(f'matrix-{n}-{w}',n,w,20,None,8,False) for n in [8,16,32] for w in [1,2,4] if (n,w)!=(16,1)]
for case,n,w,steps,cpu,trials,same in cases:
 orders=[['baseline','candidate'],['candidate','baseline']]*(trials//2)
 random.shuffle(orders)
 for trial,order in enumerate(orders):
  pair={}
  for label in order:
   binary=(baseline if same or label == 'baseline' else candidate)
   cmd=(['taskset','-c',cpu] if cpu else [])+[binary,str(n),str(w),str(steps)]
   p=subprocess.run(cmd,text=True,capture_output=True,check=True)
   fields=dict(i.split('=',1) for i in p.stdout.splitlines()[0].split(' samples=')[0].split())
   pair[label]=fields
   rows.append(dict(case=case,steps=steps,trial=trial,order=order,version=label,command=cmd,fields=fields,stdout=p.stdout))
   Path(output).write_text(json.dumps(rows,indent=2)+'\n')
  for key in ['particles','steps','accepted_time','managed_bytes','fingerprint']:
   if pair['baseline'][key]!=pair['candidate'][key]:raise RuntimeError((case,key))
  print(case,trial,*(pair[x]['frame_seconds'] for x in ['baseline','candidate']),flush=True)
