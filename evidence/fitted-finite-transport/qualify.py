"""Run actual new research scripts in normal/optimized modes, preserving logs."""
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];output=Path(sys.argv[1]).resolve();output.mkdir();rows=[]
for optimized in [False,True]:
    mode='optimized'if optimized else 'normal'
    for script,label in [('reference.py','geometry'),('numerical.py','numerical'),('test_reference.py','tests')]:
        command=[sys.executable]+(['-O']if optimized else [])+[str(ROOT/'evidence/fitted-finite-transport'/script)]
        with (output/(mode+'-'+label+('.json'if label!='tests'else '.log'))).open('w')as stdout:
            with (output/(mode+'-'+label+'-stderr.log')).open('w')as stderr:code=subprocess.run(command,cwd=ROOT,stdout=stdout,stderr=stderr).returncode
        rows.append({'command':command,'exit':code});(output/'receipt.json').write_text(json.dumps({'commands':rows},indent=2)+'\n');print(mode,label,'exit',code,flush=True)
        if code:raise SystemExit(code)
for label in ['geometry','numerical']:
    if (output/('normal-'+label+'.json')).read_bytes()!=(output/('optimized-'+label+'.json')).read_bytes():raise ValueError('normal/optimized byte replay '+label)
print('PASS new research: actual finite geometry, face integration, constrained solve, nine tests both modes and exact byte replay')
