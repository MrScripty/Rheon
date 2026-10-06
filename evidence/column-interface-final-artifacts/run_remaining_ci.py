"""Execute the two skipped workflow stages locally, preserving their commands.

Uses the selected environment's warm Cargo cache and installed pinned Python
packages. It does not rerun or claim qualification of hosted CI.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import PIL
import numpy

P=Path(__file__).resolve().parent;ROOT=P.parents[1]
OUT=P/'remaining-ci';OUT.mkdir()
workflow=(P/'trials/frozen-workflow.yml').read_text()
def stage(name):
    lines=workflow.splitlines();start=lines.index('      - name: '+name)+1
    if lines[start]!='        run: |':raise ValueError('original stage layout')
    code=[]
    for line in lines[start+1:]:
        if line and not line.startswith('          '):break
        code.append(line[10:]if line else '')
    return '\n'.join(code)+'\n'
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
if PIL.__version__!='12.3.0' or numpy.__version__!='2.3.5':raise ValueError('required CI Python package versions')
target=ROOT/'target';created=False
if not target.exists():
    target.symlink_to(Path(os.environ['CARGO_TARGET_DIR']),target_is_directory=True);created=True
records=[]
try:
    with tempfile.TemporaryDirectory(prefix='rheon-pr18-ci-')as temporary:
        runner=Path(temporary)
        subprocess.run([sys.executable,'-m','venv','--system-site-packages',str(runner/'rheon-tests')],check=True)
        env=dict(os.environ,RUNNER_TEMP=temporary,OPENBLAS_NUM_THREADS='1')
        for label,name in [('smoke','Execute real source-on source-off smoke export'),('lifecycle','Real comparison harness lifecycle tests')]:
            script=OUT/(label+'.sh');script.write_text(stage(name));start=time.monotonic()
            with (OUT/(label+'.log')).open('wb')as output,(OUT/(label+'-stderr.log')).open('wb')as error:
                answer=subprocess.run(['bash','-e',str(script)],cwd=ROOT,env=env,stdout=output,stderr=error)
            records.append(dict(name=name,exact_original_stage_script=str(script.relative_to(ROOT)),exit=answer.returncode,elapsed_seconds=time.monotonic()-start))
            if answer.returncode:raise ValueError('remaining stage failed: '+name)
        shutil.copytree(runner/'rheon-smoke',OUT/'smoke-native')
    receipt=dict(status='PASS_LOCAL_REMAINING_STAGES',qualified_source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),commands=records,warm_cache=True,hosted_CI_qualified=False,python_setup='Temporary venv uses existing system packages at the exact CI-pinned versions; no package download or toolchain install repeated',python_version=sys.version,package_versions=dict(Pillow=PIL.__version__,numpy=numpy.__version__),binary_sha256=sha(target/'release/rheon'),file_sha256={str(f.relative_to(ROOT)):sha(f)for f in sorted(OUT.rglob('*'))if f.is_file()},original_cancelled_run=37424773368)
    (OUT/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:v for k,v in receipt.items()if k!='file_sha256'},indent=2))
finally:
    if created:target.unlink()
