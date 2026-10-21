"""Balanced interleaved frozen-binary controls: baseline candidate output.json."""
import subprocess,json,random,sys,statistics,time
from pathlib import Path
baseline,candidate,output=sys.argv[1:]
random.seed(581)
rows=[]
for steps in [5,50]:
 orders=[['baseline','candidate'],['candidate','baseline']]*10
 random.shuffle(orders)
 for trial,order in enumerate(orders):
  pair={}
  for label in order:
   cmd=['taskset','-c','0',(baseline if label == 'baseline' else candidate),'16','1',str(steps)]
   p=subprocess.run(cmd,text=True,capture_output=True,check=True)
   fields=dict(i.split('=',1) for i in p.stdout.splitlines()[0].split(' samples=')[0].split())
   pair[label]=fields
   rows.append(dict(steps=steps,trial=trial,order=order,version=label,command=cmd,fields=fields,stdout=p.stdout))
   Path(output).write_text(json.dumps(rows,indent=2)+'\n')
  for key in ['particles','steps','accepted_time','managed_bytes','fingerprint']:
   if pair['baseline'][key]!=pair['candidate'][key]:raise RuntimeError(key)
  print(steps,trial,*(pair[x]['frame_seconds'] for x in ['baseline','candidate']),flush=True)
