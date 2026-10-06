"""Verify copied final inputs before running the preserved figure source."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

P=Path(__file__).resolve().parent
ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from verify_column_interface import verify
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-column-final-render-cache')
out=Path(sys.argv[1]);out.mkdir()
original=ROOT/'evidence/column-interface'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
with tempfile.TemporaryDirectory()as temporary:
    staged=Path(temporary)/'column';staged.mkdir()
    shutil.copytree(original/'demo',staged/'demo')
    shutil.copyfile(original/'make_figure.py',staged/'make_figure.py')
    summary=verify(staged/'demo')
    (staged/'numerical-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    subprocess.run([sys.executable,str(staged/'make_figure.py')],check=True)
    for name in ['continuation-and-convergence.png','numerical-summary.json']:shutil.copyfile(staged/name,out/name)
    receipt=dict(status='RENDERED_AFTER_FULL_FINAL_ARTIFACT_VERIFICATION',scenarios=len(summary['results']),frozen_plot_source_sha256=sha(staged/'make_figure.py'),guard_source_sha256=sha(Path(__file__)),copied_native_file_sha256={str(f.relative_to(staged)):sha(f)for f in sorted((staged/'demo').rglob('*'))if f.is_file()},output_sha256={f.name:sha(f)for f in sorted(out.iterdir())},historical_artifacts_overwritten=False)
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
