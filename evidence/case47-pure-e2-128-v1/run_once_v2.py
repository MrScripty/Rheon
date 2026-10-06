"""One original main trajectory; bounded controls only after complete/replayed main."""
from pathlib import Path
import hashlib,json,os,resource,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def rows(path):return [json.loads(x) for x in path.read_text().splitlines() if x.startswith('{')]
def bits(r):return {k:v for k,v in r.items() if k not in ['event','label','step']}
def inspect(label,expected_stage):
 data=rows(P/(label+'-native.log'));terminal=[r for r in data if r.get('event')=='pure_e2_terminal'];require(len(terminal)==1,'one native terminal')
 begin=[r for r in data if r.get('event')=='pure_e2_begin'];require(len(begin)==1 and begin[0]['cancel_stage']==expected_stage and begin[0]['E2_before_first_advance'] and begin[0]['retries']==0,'exact native job scope')
 result=[r for r in data if r.get('event')=='pure_e2_result'];state=[r for r in data if r.get('event')=='accepted_bits'];pubs=[r for r in data if 'model' in r]
 require([r['step'] for r in result]==list(range(1,len(result)+1)) and len(state)==1+2*len(result),'bounded complete state observations')
 for index,r in enumerate(result):
  before,after=state[1+2*index:3+2*index];require(before['label']=='before' and after['label']=='after' and before['step']==after['step']==index+1,'paired borrowed journals')
  if r['result'].startswith('Err('):require(bits(before)==bits(after),'refusal/cancellation preserves every accepted field')
  else:require(r['result'].startswith('Ok(') and after['stamp']==[131,index+1],'actual publication')
 require(len(pubs)==1+terminal[0]['accepted_steps'],'actual accepted prefix count')
 return {'terminal':terminal[0],'public_attempts':len(result),'publications':len(pubs),'retries':0},data
require(not(P/'main-before.json').exists(),'never repeat this experiment')
b=json.loads((P/'binary-binding.json').read_text());p=json.loads((P/'protocol.json').read_text());pre=json.loads((P/'preflight-receipt-v2.json').read_text());require(pre['status']=='PASS_PURE_E2_SCALAR_AND_FULL_PUBLIC_MEMORY' and pre['memory_bound']<=67584,'passing preflight')
for name,digest in p['source_sha256'].items():require(sha((ROOT/name).read_bytes())==digest,'frozen pure-E2 source')
amendment=json.loads((P/'reader-amendment.json').read_text())
for name,digest in amendment['new_reader_sha256'].items():require(sha((P/name).read_bytes())==digest,'frozen parser amendment')
for name,digest in pre['artifact_sha256'].items():require(sha((P/name).read_bytes())==digest,'passing preflight artifacts')
require(sha(Path(b['binary']).read_bytes())==b['binary_sha256']==pre['binary_sha256'],'exact ELF')
env=dict(os.environ,RHEON_CASE47_PURE_E2_ALLOW='case47-pure-E2-original128-first-visit-cancel-v1');env.pop('RHEON_CASE47_CANCEL_STAGE',None)
argv=[b['binary'],'--exact','research_public_call::case47_pure_e2_original_128_or_cancel','--nocapture'];jobs=[]
def job(label,stage):
 require(not(P/(label+'-before.json')).exists(),'one invocation per job')
 if stage is not None:env['RHEON_CASE47_CANCEL_STAGE']=str(stage)
 else:env.pop('RHEON_CASE47_CANCEL_STAGE',None)
 before={'argv':argv,'environment':{k:env[k] for k in ['RHEON_CASE47_PURE_E2_ALLOW','RHEON_CASE47_CANCEL_STAGE'] if k in env},'binary_sha256':b['binary_sha256'],'preflight_receipt_sha256':sha((P/'preflight-receipt-v2.json').read_bytes()),'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip(),'source_tree':subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip(),'stdout_cap':8*1024*1024,'stderr_cap':4*1024*1024,'retries':0}
 (P/(label+'-before.json')).write_text(json.dumps(before,indent=2,sort_keys=True)+'\n')
 def limits():resource.setrlimit(resource.RLIMIT_FSIZE,(8*1024*1024,8*1024*1024))
 start=time.monotonic()
 with (P/(label+'-native.log')).open('wb') as out,(P/(label+'-native-stderr.log')).open('wb') as err:r=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err,preexec_fn=limits)
 after={**before,'exit':r.returncode,'seconds':time.monotonic()-start,'stdout_sha256':sha((P/(label+'-native.log')).read_bytes()),'stderr_sha256':sha((P/(label+'-native-stderr.log')).read_bytes())};(P/(label+'-completed.json')).write_text(json.dumps(after,indent=2,sort_keys=True)+'\n')
 require((P/(label+'-native.log')).stat().st_size<=before['stdout_cap'] and (P/(label+'-native-stderr.log')).stat().st_size<=before['stderr_cap'],'bounded native journals');require(r.returncode==0,'native test failure: stop, retain evidence')
 summary,data=inspect(label,-1 if stage is None else stage);jobs.append({'label':label,**summary});(P/'jobs-completed.json').write_text(json.dumps(jobs,indent=2,sort_keys=True)+'\n');return summary,data
main,data=job('main',None)
physics=[]
for mode,flags in [('normal',[]),('optimized',['-O'])]:
 args=[sys.executable,*flags,str(P/'replay_physics.py')];out=P/f'physical-{mode}.json';err=P/f'physical-{mode}-stderr.log'
 with out.open('wb') as a,err.open('wb') as e:r=subprocess.run(args,cwd=ROOT,stdout=a,stderr=e,env=env)
 physics.append({'argv':args,'exit':r.returncode,'stdout':out.name,'stderr':err.name,'stdout_sha256':sha(out.read_bytes()),'stderr_sha256':sha(err.read_bytes())});(P/'physical-commands.json').write_text(json.dumps(physics,indent=2,sort_keys=True)+'\n');require(r.returncode==0,'physical reader error')
require((P/'physical-normal.json').read_bytes()==(P/'physical-optimized.json').read_bytes(),'physical mode parity')
physical=json.loads((P/'physical-normal.json').read_text())
if main['terminal']['status']=='COMPLETE' and main['terminal']['accepted_steps']==128 and physical['status']=='PASS_UNCHANGED_PHYSICAL_REPLAY':
 mainpub=[r for r in data if 'model' in r][:26];mainresult=[r for r in data if r.get('event')=='pure_e2_result'][:25];mainbits=[r for r in data if r.get('event')=='accepted_bits'][:51]
 for stage in range(11):
  summary,control=job(f'cancel-{stage}',stage)
  require([r for r in control if 'model' in r]==mainpub,'exact pure-E2 prefix reports/fields')
  require([r for r in control if r.get('event')=='pure_e2_result'][:25]==mainresult,'exact pure-E2 prefix results/barriers')
  require([r for r in control if r.get('event')=='accepted_bits'][:51]==mainbits,'exact pure-E2 prefix all accepted bits')
  require(summary['terminal']['status']=='CANCELLED' and summary['terminal']['attempted_step']==26 and summary['terminal']['accepted_steps']==25,'one intended cancellation; stop on any other refusal')
else:
 print('Main trajectory/replay did not qualify; all cancellation jobs withheld.',file=sys.stderr)
print(json.dumps({'status':'EXPERIMENT_COMPLETED_QUALIFICATION_READER_PENDING','jobs':jobs,'physical_status':physical['status'],'retries':0},indent=2,sort_keys=True))
