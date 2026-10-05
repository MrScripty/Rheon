"""Build actual native executable and independently replay 3 dt × 2 profiles."""
import hashlib,json,os
from pathlib import Path
import subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]).resolve();output.mkdir();commands=[];summaries=[]
def run(command,name,stdout):
    with stdout.open('w') as stream:code=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
    commands.append({'name':name,'command':command,'exit':code})
    (output/'receipt.json').write_text(json.dumps({'commands':commands,'status':'in progress'},indent=2)+'\n')
    if code:raise SystemExit(code)
for profile in ['debug','release']:
    command=['cargo','build','--locked','--example','fixed_bottom_ale']+(['--release'] if profile=='release' else [])
    run(command,profile+'-build',output/(profile+'-build.log'))
    binary=Path(os.environ['CARGO_TARGET_DIR'])/profile/'examples/fixed_bottom_ale'
    binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    for label,dt,count in [('coarse','.05','10'),('medium','.025','20'),('fine','.0125','40')]:
        name=profile+'-'+label;data=output/(name+'.json')
        run([str(binary),dt,count],name,data);commands[-1]['binary_sha256']=binary_sha
        for optimized in [False,True]:
            cmd=[sys.executable]+(['-O'] if optimized else [])+[str(ROOT/'evidence/fitted-ale-native/verify_native.py'),str(data),'--negative-self-test']
            run(cmd,name+('-optimized' if optimized else '-normal'),output/(name+('-optimized' if optimized else '-normal')+'.log'))
        from verify_native import verify
        summaries.append({'profile':profile,**verify(json.loads(data.read_text()))})
from reference import require
for label in ['coarse','medium','fine']:require((output/('debug-'+label+'.json')).read_bytes()==(output/('release-'+label+'.json')).read_bytes(),'debug/release exact replay '+label)
for profile in ['debug','release']:
    rows=[s for s in summaries if s['profile']==profile]
    ratios=[rows[i]['temporal_L2_error']/rows[i+1]['temporal_L2_error']for i in range(2)]
    require(all(1.8<r<2.2 for r in ratios),'actual native first-order temporal convergence')
(output/'receipt.json').write_text(json.dumps({'commands':commands,'status':'PASS','summaries':summaries,'debug_release_byte_equal':True,'scope':'preceding physical family on fixed-bottom ALE mesh: genuine nonzero relative transport and temporal study against nonautonomous semidiscrete ODE; no general pressure coupling or continuum spatial accuracy'},indent=2)+'\n')
print('PASS six native runs, twelve normal/optimized validators with nine corruption controls each; all dt debug/release byte equal')
