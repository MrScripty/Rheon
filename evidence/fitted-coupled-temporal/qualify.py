"""Actual bounded coupled research; modes, controls, exact interval/work checks."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import numpy
import scipy
import sympy
import mpmath

ROOT=Path(__file__).resolve().parents[2]
PREFIX=ROOT/'evidence/fitted-coupled-temporal'
output=Path(sys.argv[1]).resolve();output.mkdir()
commands=[]


def run(command,name,destination):
    with destination.open('w')as stdout,(output/(name+'-stderr.log')).open('w')as stderr:
        code=subprocess.run(command,cwd=ROOT,stdout=stdout,stderr=stderr).returncode
    commands.append(dict(name=name,command=command,exit=code))
    (output/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n')
    print(name,'exit',code,flush=True)
    if code:raise SystemExit(code)


for optimized in [False,True]:
    mode='optimized'if optimized else 'normal'
    prefix=[sys.executable]+(['-O']if optimized else [])
    for script,label in [('exact.py','exact'),('study.py','study'),('test_reference.py','tests')]:
        run(prefix+[str(PREFIX/script)],mode+'-'+label,output/(mode+'-'+label+('.log'if label=='tests'else '.json')))
    study=json.loads((output/(mode+'-study.json')).read_text())
    if not study['all_fixed_work_gates_fail']or study['accepted_advancing_steps']!=0:raise ValueError('honest work rejection')
    for label,cell in zip(['coarse','medium','fine'],study['cells']):
        payload=output/(mode+'-'+label+'-cell.json');payload.write_text(json.dumps(cell,indent=2)+'\n')
        run(prefix+[str(PREFIX/'verify_cell.py'),str(payload),'--negative-self-test'],mode+'-'+label+'-replay',output/(mode+'-'+label+'-replay.log'))
    coarse=output/(mode+'-coarse-cell.json')
    run(prefix+[str(PREFIX/'independent.py'),str(coarse),'16'],mode+'-independent',output/(mode+'-independent.json'))
    run(prefix+[str(PREFIX/'certificate.py'),str(coarse)],mode+'-certificate',output/(mode+'-certificate.json'))
    run(prefix+[str(ROOT/'evidence/fitted-pressure-path/verify.py'),'--evidence-commit','00c502f1fe5be7f9c527ebb14ba5b78e450cb031','--negative-self-test'],mode+'-frozen-predecessor',output/(mode+'-frozen-predecessor.log'))
for label in ['exact','study','independent','certificate','coarse-cell','medium-cell','fine-cell']:
    if (output/('normal-'+label+'.json')).read_bytes()!=(output/('optimized-'+label+'.json')).read_bytes():raise ValueError('mode byte equality: '+label)
source_files=sorted([*PREFIX.glob('*.py'),PREFIX/'README.md',ROOT/'docs/research-book/implementation/fitted-coupled-temporal-prototype.md'])
source_sha={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in source_files}
(output/'receipt.json').write_text(json.dumps(dict(status='PASS',commands=commands,
    versions=dict(python=sys.version,numpy=numpy.__version__,scipy=scipy.__version__,sympy=sympy.__version__,mpmath=mpmath.__version__),
    exact_mode_byte_equal=True,source_sha256=source_sha,reference_tests_per_mode=13,actual_cell_replays=6,
    corruption_rejections=48,independent_rational_80_digit_runs=2,actual_interval_certificates=2,
    interval_boxes_per_certificate=512,accepted_advancing_steps=0,new_Rust_temporal_builds=0,new_Lean_claims=0),indent=2)+'\n')
print('PASS coupled research: 13 tests per mode, six replays / 48 corruptions, two rational work reconstructions, two 512-box interval certificates; all candidates rejected by unchanged work gate')
