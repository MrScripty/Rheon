"""Bind final Rust/example bytes by rebuilding and requiring exact qualified replay.

The independent validators already qualify the recorded JSON. This final rebuild
checks that cosmetic/example/test source cleanup did not change any output; it
records final binaries/source hashes and refuses any changed numerical payload.
"""
import hashlib,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];output=Path(sys.argv[1]).resolve();output.mkdir();commands=[]
source={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for folder in ['src','tests','examples']for p in sorted((ROOT/folder).rglob('*.rs'))}
for profile in ['debug','release']:
    command=['cargo','build','--locked','--example','fixed_bottom_ale']+(['--release']if profile=='release'else [])
    with (output/(profile+'-build.log')).open('w')as stream:code=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT).returncode
    commands.append({'command':command,'exit':code})
    if code:raise SystemExit(code)
    binary=Path(os.environ['CARGO_TARGET_DIR'])/profile/'examples/fixed_bottom_ale';bh=hashlib.sha256(binary.read_bytes()).hexdigest()
    for label,dt,count in [('coarse','.05','10'),('medium','.025','20'),('fine','.0125','40')]:
        command=[str(binary),dt,count];payload=subprocess.check_output(command,cwd=ROOT);expected=ROOT/'evidence/fitted-ale-native/native-qualification'/(profile+'-'+label+'.json')
        if payload!=expected.read_bytes():raise ValueError('final-source numerical replay changed '+profile+'-'+label)
        commands.append({'command':command,'exit':0,'binary_sha256':bh,'byte_equal_qualified_payload':str(expected.relative_to(ROOT)),'payload_sha256':hashlib.sha256(payload).hexdigest()})
(output/'receipt.json').write_text(json.dumps({'commands':commands,'final_Rust_source_sha256':source,'all_final_source_replays_byte_equal':True,'scope':'actual final-source rebuild; inherits independently validated byte-identical JSON without repeating those validators'},indent=2)+'\n')
print('PASS final-source debug/release rebuild and six exact numerical replays')
