"""Select all five frozen terminal refusals and their actual accepted inputs."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[2];P=Path(__file__).resolve().parent;E=ROOT/'evidence/solver-terminal-validation'
events=[json.loads(x)for x in (E/'trace-finer-stderr.log').read_text().splitlines()]
public=[json.loads(x)for x in (E/'trace-finer.jsonl').read_text().splitlines()]
selected=[];case=None;retry=False;check=None;failed=False
for row in events:
 event=row['event']
 if event=='case':case=(row['kind'],row['load'],row['h']);retry=False;failed=False
 elif event=='retry_begin':retry=True
 elif event=='newton_check' and not retry:
  if row['iteration']==1:check=row;failed=False
 elif event=='refusal' and not retry:failed=True
 elif event=='post_window_observation' and failed and not retry and row['order']==16:
  kind,load,h=case
  previous=[r for r in public if r.get('model') and (r['kind'],r['load'],r['h'],r['step'])==(kind,load,h,check['accepted_version'])]
  if len(previous)!=1:raise ValueError('unique actual accepted state')
  old=previous[0]
  if old['end_q']!=check['q']or old['end_eta']!=check['eta']:raise ValueError('actual trace/publication input match')
  selected.append(dict(kind=kind,load=load,h=h,accepted_version=check['accepted_version'],q=check['q'],eta=check['eta'],unknown=row['unknown'],old=old['velocity'],mass=old['mass'],expected_rate=row['rate'],expected_norm=row['rate_norm'],stamp=old['stamp'],time=old['time']))
if len(selected)!=5:raise ValueError('all five remaining actual refusals')
(P/'inputs.json').write_text(json.dumps({'frozen_head':'dad53b4054034fe0c2ff6240464df7441ab4e6a9','input_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [E/'trace-finer-stderr.log',E/'trace-finer.jsonl']},'rows':selected},indent=2)+'\n')
def rust(x):
 if isinstance(x,list):return '['+','.join(rust(y)for y in x)+']'
 if isinstance(x,float):return repr(x)
 return str(x)
fields=['q','eta','unknown','old','mass','expected_rate','expected_norm','h']
source='const CASES: [Case; 5] = [\n'
for i,row in enumerate(selected):
 source+='Case { index:'+str(i)+','+','.join(k+':'+rust(row[k])for k in fields)+', acceleration:'+rust([.0625,-.125,.03125]if row['load']=='forward'else[-.0625,.125,-.03125])+' },\n'
source+='];\n';(P/'native_inputs.rs').write_text(source)
print('Bound all five refused candidates to frozen actual accepted publications')
