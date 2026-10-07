"""One scalar/layout-only preflight. Never invokes either proposed observation."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,os,re,subprocess,time,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
def sha(data):return hashlib.sha256(data).hexdigest()
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def run():
    require(not (P/'preflight-receipt.json').exists() and not (P/'preflight-commands.json').exists(),'exclusive native guard preflight; no rerun')
    policy=json.loads((P/'execution-protocol.json').read_text())
    for f,h in policy['sha256'].items():require(sha((ROOT/f).read_bytes())==h,'frozen preflight '+f)
    b=json.loads((P/'compile-binding.json').read_text());require(sha(Path(b['binary']).read_bytes())==b['binary_sha256']==policy['binary_sha256'],'exact fresh ELF')
    source= subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip();tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT,text=True).strip()
    conformance=load('forecast_source_conformance',P/'source_conformance.py').run()
    env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',RUST_MIN_STACK=str(policy['limits']['stack_bytes']))
    for key in list(env):
        if key.startswith('RHEON_'):env.pop(key)
    commands=[]
    tests=[('scalar-native','research_public_scalar_preflight'),('type-layout','research_public_type_layout'),('affine-native','scalar_affine_preflight'),('paired-layout','case41_paired_layout_preflight'),('forecast-input-layout','case41_forecast_input_layout_preflight')]
    for label,test in tests:
        argv=[b['binary'],'--exact','research_public_call::'+test,'--nocapture'];start=time.monotonic()
        done=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,timeout=60)
        (P/(label+'.log')).write_bytes(done.stdout);(P/(label+'-stderr.log')).write_bytes(done.stderr)
        commands.append(dict(argv=argv,exit=done.returncode,seconds=time.monotonic()-start,stdout_sha256=sha(done.stdout),stderr_sha256=sha(done.stderr),numerical_allow_variables_present=False,numerical_observations=0))
        (P/'preflight-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n')
        require(done.returncode==0 and not done.stderr,'native scalar/layout guard '+label)
        require(b'1 passed; 0 failed' in done.stdout and len(done.stdout)<policy['limits']['stdout_bytes'],'closed successful guard '+label)
    scalar=load('forecast_scalar',P/'check_scalar.py').run(P/'scalar-native.log');affine=load('forecast_affine',P/'check_affine.py').run()
    def records(path):
        result=[]
        for line in path.read_text().splitlines():
            pos=line.find('{')
            if pos>=0:result.append(json.loads(re.sub(r'\((\d+), (\d+)\)',r'[\1, \2]',line[pos:])))
        return result
    layout=records(P/'paired-layout.log')[0];inputs=records(P/'forecast-input-layout.log')[0]
    require(layout['candidate'][1:]==layout['reference'][1:] and layout['candidate'][-1]==176,'same return/point and fixed buffer layouts')
    declared=json.loads((P/'forecast-inputs.json').read_text());require([format(x,'016x') for x in inputs['forecast_bits']]==declared['forecast_unknown_bits'] and inputs['unknown_buffer_bytes']==176,'native literal input bits')
    mem=[]
    for mode in [[],['-O']]:
        done=subprocess.run([sys.executable,'-B',*mode,str(P/'audit_observation.py')],cwd=ROOT,capture_output=True)
        label='frames-optimized' if mode else 'frames-normal'
        (P/(label+'.json')).write_bytes(done.stdout);(P/(label+'-stderr.log')).write_bytes(done.stderr)
        require(done.returncode==0 and not done.stderr,'read-only frame observations');mem.append(done.stdout)
    require(mem[0]==mem[1],'normal/-O frame observations');frames=json.loads(mem[0]);require(frames['complete_memory_bound'] is None and not frames['unknown_costs_zeroed'],'honest incomplete static costs')
    geometry=[dict(requested_capacity=n,element_bytes=size,requested_payload_bytes=n*size) for n,size in layout['geometry']]
    host={'cgroup_memory_max':Path('/sys/fs/cgroup/memory.max').read_text().strip(),'cgroup_memory_current':Path('/sys/fs/cgroup/memory.current').read_text().strip(),'MemAvailable':next(line for line in Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:'))}
    (P/'host-preflight.json').write_text(json.dumps(host,indent=2,sort_keys=True)+'\n')
    require(host['cgroup_memory_max']=='max' or int(host['cgroup_memory_max'])-int(host['cgroup_memory_current'])>=2*policy['limits']['address_space_bytes'],'host resource headroom')
    # Prove absence of authority is fail-closed before marker, PID, or ELF call.
    denied=subprocess.run([sys.executable,'-B',str(P/'run_after_review_v2.py')],cwd=ROOT,capture_output=True)
    (P/'authorization-absent-stdout.log').write_bytes(denied.stdout);(P/'authorization-absent-stderr.log').write_bytes(denied.stderr)
    require(denied.returncode!=0 and b'authorization missing; native launch forbidden' in denied.stderr and not (P/'capture-before.json').exists() and not (P/'native.log').exists(),'missing authorization launches zero observations')
    result=dict(status='PASS_TWO_OBSERVATION_PREPARATION_ONLY',source=source,source_tree=tree,compiled_source=b['source'],compiled_source_tree=b['source_tree'],binary=b['binary'],binary_sha256=b['binary_sha256'],protocol_sha256=sha((P/'execution-protocol.json').read_bytes()),source_conformance=conformance,scalar=scalar,affine=affine,paired_layout=layout,forecast_literal_layout=inputs,geometry_allocations=geometry,common_geometry_requested_payload_bytes=sum(x['requested_payload_bytes'] for x in geometry),fixed_frame_observations_sha256=sha(mem[0]),fixed_frame_bound_scope='Actual complete frames retained; unknown external costs and callbacks not zeroed; no complete static certificate.',native_guard_invocations=5,native_equations=0,baseline_observations=0,forecast_observations=0,controller_corrections=0,owner_advances=0,execution_allowed=False,independent_review_accepted=False,historical_memory_qualified=False,historical_known_subtotal=66368,historical_cap=67584,complete_memory_bound=None,proposed_operational_limits=policy['limits'],absent_authorization_guard_passed=True)
    (P/'preflight-receipt.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');return result
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
