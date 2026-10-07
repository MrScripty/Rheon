"""Packet hashes, precision parity and in-memory primitive identity controls."""
from pathlib import Path
import copy,hashlib,importlib.util,json,math,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
inv=json.loads((P/'result-inventory.json').read_text())
for f,h in inv['sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'result '+f)
policy=json.loads((P/'protocol.json').read_text())
for f,h in policy['sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'input '+f)
models=[]
for d in [80,120]:
    data=(P/f'analysis-{d}-normal.json').read_bytes();require(data==(P/f'analysis-{d}-optimized.json').read_bytes(),'normal/-O parity')
    x=json.loads(data);require(x.pop('precision_digits')==d,'precision');models.append(x)
    for mode in ['normal','optimized']:require(not (P/f'analysis-{d}-{mode}-stderr.log').read_bytes(),'clean stderr')
require(models[0]==models[1],'80/120 display parity')
x=models[0];require(x['primitive_exact_above_Newton_gate'] and not x['primitive_latent_counterfactual_above_Newton_gate'],'actual/counterfactual classification')
require(x['new_native_equations']==x['controller_corrections']==x['owner_advances']==0 and not x['historical_memory_qualified'],'zero scope')
spec=importlib.util.spec_from_file_location('binding_controls',P/'analyze.py');reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)
rows=reader.records(ROOT/'evidence/forcing-e2-fixed-candidates-v1/candidate-fixed.log');sel=[r for _,_,r in rows if r.get('index')==41 and r.get('order')==16]
eq=next(r for r in sel if r['event']=='fixed_equation');points=[r for r in sel if r['event']=='fixed_point'];start=next(r for r in points if r['label']=='start');end=next(r for r in points if r['label']=='end');quad=[r for r in points if r['label']=='quadrature']
e=next(r for _,_,r in reader.records(ROOT/'evidence/case41-exploratory-seven-v1/native.log','test research_public_call::case41_paired_candidate_roster ... ') if r['event']=='fd_equation' and r['column']==-1)
c=json.loads((ROOT/'evidence/case41-exploratory-seven-v1/fixture.json').read_text())['original_case']
base=[e,eq,start,end,quad,c];require(reader.bindings(*base)['start_end_bit_identical'],'positive actual binding')
controls=[]
for name in ['archive-equation','endpoint-force','sample-count','donor-topology']:
    args=copy.deepcopy(base)
    if name=='archive-equation':args[1]['rate'][0]=math.nextafter(args[1]['rate'][0],math.inf)
    if name=='endpoint-force':args[3]['force'][0][0]=math.nextafter(args[3]['force'][0][0],math.inf)
    if name=='sample-count':args[4].pop()
    if name=='donor-topology':args[4][0]['pairs'][0][0]+=1
    try:reader.bindings(*args)
    except ValueError as error:controls.append({'name':name,'rejected':True,'reason':str(error)})
    else:raise ValueError('binding control not rejected '+name)
changes=subprocess.check_output(['git','diff','--name-only','025ae46ad728fc09d1bafcb516e474e6ea52641a','HEAD'],cwd=ROOT,text=True).splitlines()
require(all(f.startswith('evidence/case41-stored-primitive-attribution-v1/') for f in changes),'new packet only')
print(json.dumps({'status':'PASS_FROZEN_STORED_PRIMITIVE_ATTRIBUTION','source':inv['analysis_source'],'source_tree':inv['analysis_tree'],'normal_optimized_equal':True,'precision_80_120_equal':True,'binding_negative_controls':controls,'new_native_equations':0,'historical_memory_qualified':False},indent=2,sort_keys=True))
