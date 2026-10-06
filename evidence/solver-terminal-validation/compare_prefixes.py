"""Compare raw original native checks/publications before newly continued states."""
from pathlib import Path
import json
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def checks(path):
 result={};key=None;retry=False
 for line in path.read_bytes().splitlines():
  r=json.loads(line)
  if r['event']=='case':key=(r['kind'],r['load'],r['h']);retry=False
  elif r['event']=='retry_begin':retry=True
  elif r['event']=='retry_end':retry=False
  elif r['event']=='newton_check'and not retry:result[(key,r['accepted_version'],r['iteration'])]=line
 return result
a=checks(ROOT/'evidence/forcing-finer-refusals/native-trace-default-stderr.log');b=checks(P/'trace-finer-stderr.log')
if not all(line==b.get(key)for key,line in a.items()):raise ValueError('changed original actual Newton arithmetic/check record')
def publications(path):
 result={}
 for line in path.read_bytes().splitlines():
  r=json.loads(line)
  if 'model'in r:result[(r['kind'],r['load'],r['h'],r['step'])]=line
 return result
a_public=publications(ROOT/'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl');b_public=publications(P/'forcing_temporal_probe-default.jsonl')
if not all(line==b_public.get(key)for key,line in a_public.items()):raise ValueError('changed original accepted finer prefix')
print(json.dumps({'original_newton_check_records_compared':len(a),'original_newton_check_records_byte_identical':True,'original_actual_finer_publications_compared':len(a_public),'original_actual_finer_publications_byte_identical':True,'new_actual_finer_publications':len(b_public),'comparison_excludes_frozen_refusal_markers':True,'original_refusal_markers_preserved_in_original_packet':True},indent=2))
