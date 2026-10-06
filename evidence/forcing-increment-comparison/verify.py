"""Freeze/replay the completed bounded experiment, never rerun native E1."""
from pathlib import Path
import copy,gzip,hashlib,importlib.util,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='78b7365513ccf317e554d36f968da36b55f04065';TREE='35770b5a8cbab53a24232298961eac4eeca5c801'
def require(ok,message):
    if not ok:raise ValueError(message)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def load_module(name):
    spec=importlib.util.spec_from_file_location('comparison_'+name,P/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def preserved():
    require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT).decode().strip()==TREE,'frozen staged source tree')
    entries=subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0');digest=hashlib.sha256();count=0
    for entry in entries:
        if not entry:continue
        metadata,name=entry.split(b'\t',1);mode,kind,oid=metadata.split();require(mode in (b'100644',b'100755') and kind==b'blob','frozen file modes')
        data=(ROOT/name.decode()).read_bytes();actual=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()
        require(actual==oid,'all frozen paths unchanged: '+name.decode());digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
    require(count==6996,'full prior source/evidence inventory')
    return dict(files=count,path_content_sha256=digest.hexdigest())
def checks():
    require((P/'comparison-normal.json').read_bytes()==(P/'comparison-optimized.json').read_bytes()==(P/'comparison-replay.json').read_bytes(),'normal/optimized comparison parity')
    require((P/'A0-normal.json').read_bytes()==(P/'A0-optimized.json').read_bytes(),'normal/optimized A0 parity')
    a=load_module('analyze');comparison=a.main(True);require(comparison==json.loads((P/'comparison-normal.json').read_text()),'actual full result reader replay')
    require(a.main(False)==json.loads((P/'A0-normal.json').read_text()),'actual A0 replay')
    scalar=load_module('check_scalar');s=scalar.run(P/'preflight-staged.log');require(s==json.loads((P/'scalar-oracle-staged.json').read_text()),'exact scalar replay')
    memory=load_module('check_memory_staged').run();require(memory==json.loads((P/'memory-preflight-staged.json').read_text()),'compiled machine-code workspace replay')
    for name in ['native-disassembly.txt','native-disassembly-staged.txt']:
        raw=gzip.decompress((P/(name+'.gz')).read_bytes());require(hashlib.sha256(raw).hexdigest()==(P/(name+'.sha256')).read_text().strip(),'lossless linked disassembly binding')
    for name in ['preflight-native.log','preflight-linked.log','preflight-staged.log','B0-native.log','E1-native.log']:
        require('1 passed; 0 failed' in (P/name).read_text(),'actual targeted Rust test passed')
    original=json.loads((P/'baseline-binding.json').read_text())
    for path,value in original['production_sha256'].items():require(sha(ROOT/path)==value,'production bytes restored')
    e=json.loads((P/'E1-command.json').read_text());b=json.loads((P/'B0-command.json').read_text())
    require(e['exit']==b['exit']==0 and e['preflight_sha256']==b['preflight_sha256']==sha(P/'memory-preflight-staged.json')
            and e['binary_sha256']==b['binary_sha256']==memory['binary_sha256'],'same qualified native binary/preflight')
    require(e['A0_sha256']==sha(P/'A0-normal.json'),'A0 completed before E1')
    require(json.loads((P/'memory-preflight-first.json').read_text())['status']=='MEMORY_PREFLIGHT_BLOCKED','first rejected memory reader retained')
    def valid(r):
        require(r['B0']['native_refusal'] is True and r['A0']['strict_exact_threshold_pass'] is False,'B0/A0 failures not promoted')
        require(r['owner_advances']==r['searches']==r['acceptance_promotions']==0 and r['original_native_refusals']==5
                and r['original_temporal_band']=='FAIL_ORIGINAL_TEMPORAL_BAND','no native search/adoption/trajectory promotion')
        require(r['E1']['scope'].startswith('changed chart') and r['E1']['endpoint_bit_changes']['end_z']==11
                and r['E1']['endpoint_bit_changes']['end_velocity']==11,'changed E1 fields explicit')
        require(r['E1']['unchanged_geometry_and_basis_bitwise'] is True and r['E1']['third_or_owner_step_qualified'] is False,'native geometry and full-step limitations retained')
    valid(comparison)
    mutations=[('false original A0 pass',lambda r:r['A0'].update(strict_exact_threshold_pass=True)),
        ('false original B0 pass',lambda r:r['B0'].update(native_refusal=False)),
        ('E1 mislabeled identical fields',lambda r:r['E1'].update(endpoint_bit_changes=dict(end_z=0,end_velocity=0))),
        ('alternate geometry silently accepted',lambda r:r['E1'].update(unchanged_geometry_and_basis_bitwise=False)),
        ('unrun third/public step promoted',lambda r:r['E1'].update(third_or_owner_step_qualified=True)),
        ('hidden candidate search',lambda r:r.update(searches=1)),
        ('invented owner advance',lambda r:r.update(owner_advances=1)),
        ('original temporal failure promoted',lambda r:r.update(original_temporal_band='PASS'))]
    rejected=[]
    for name,mutation in mutations:
        altered=copy.deepcopy(comparison);mutation(altered)
        try:valid(altered)
        except ValueError:rejected.append(name)
        else:raise ValueError('missed result corruption: '+name)
    # A bitwise binding must reject signed-zero corruption even though == does not.
    try:a.same_float([-0.0],[0.0])
    except ValueError:rejected.append('signed zero corrupted')
    else:raise ValueError('signed zero binding')
    return dict(status='PASS_BOUNDED_EXPERIMENT_BINDING_ONLY',B0_bitwise_float_checks=comparison['B0']['bitwise_float_checks'],
        scalar_assertion_groups=s['native_assertion_groups'],exact_scalar_probes=36,kernel_stack_bound=memory['compiled_kernel_stack_bound'],
        additional_live_bound=memory['compiled_additional_live_bound'],reserved_additional_bytes=memory['reserved_additional_bytes'],
        combined_reservation_bytes=memory['combined_reservation_bytes'],B0_norm=comparison['B0']['coarse_norm'],
        A0_norm=comparison['A0']['stable_norm_float'],E1_norm=comparison['E1']['coarse_native_norm'],
        E1_fine_norm=comparison['E1']['fine_native_norm'],isolated_planar_gates_pass=comparison['E1']['isolated_planar_gates']['pass'],
        result_corruptions_rejected=rejected,owner_advances=0,searches=0,production_changes=0)
def artifacts():
    return {str(f.relative_to(ROOT)):sha(f) for f in sorted(P.iterdir()) if f.is_file() and f.name!='receipt.json'
            and not f.name.startswith(('root-verification','post-git')) and f.name!='record_post_git.py'}
def verify(receipt):
    require(receipt['base']==BASE and receipt['base_tree']==TREE,'receipt source identity')
    require(receipt['preserved_inventory']==preserved(),'full preserved source/evidence binding')
    require(receipt['artifacts']==artifacts(),'closed completed artifact hashes')
    require(receipt['checks']==checks(),'complete actual result checks')
    return dict(**receipt['checks'],preserved_files=receipt['preserved_inventory']['files'],artifacts=len(receipt['artifacts']))
if __name__=='__main__':
    if '--freeze' in sys.argv:
        r=dict(base=BASE,base_tree=TREE,preserved_inventory=preserved(),checks=checks(),artifacts=artifacts())
        (P/'receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(status='FROZEN_BOUNDED_EXPERIMENT',receipt_sha256=sha(P/'receipt.json'),artifacts=len(r['artifacts']))))
    else:print(json.dumps(verify(json.loads((P/'receipt.json').read_text())),indent=2,sort_keys=True))
