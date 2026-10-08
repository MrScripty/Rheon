"""Requalify the immutable PR36 proof inputs locally, outside every checkout."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess

ROOT=Path(__file__).resolve().parents[2]
PIN='d31cf735645579357f9f58dcc55958e23f77af59'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def qualify(output, lean_bin, dependencies):
    output,lean_bin,dependencies=map(lambda p:Path(p).resolve(),(output,lean_bin,dependencies))
    probe=output
    while not probe.exists():probe=probe.parent
    if output.exists() or subprocess.run(['git','-C',str(probe),'rev-parse','--is-inside-work-tree'],capture_output=True).returncode==0:
        raise ValueError('Fresh output outside every Git checkout required')
    inventory=ROOT/'proofs/source-inventory.json'
    pinned=subprocess.check_output(['git','show',PIN+':proofs/source-inventory.json'],cwd=ROOT)
    if inventory.read_bytes()!=pinned:
        raise ValueError('Proof inputs require a reviewed decision beyond PR36')
    sources=json.loads(pinned)
    for name,digest in sources.items():
        if sha(ROOT/'proofs'/name)!=digest:
            raise ValueError('PR36 proof source changed: '+name)
    dependency_pins={}
    for p in json.loads((ROOT/'proofs/lake-manifest.json').read_text())['packages']:
        actual=subprocess.check_output(['git','rev-parse','HEAD'],cwd=dependencies/p['name'],text=True).strip()
        if actual!=p['rev']:raise ValueError('Dependency pin changed: '+p['name'])
        dependency_pins[p['name']]=actual
    version=subprocess.check_output([lean_bin/'lean','--version'],text=True).strip()
    if 'version 4.19.0,' not in version:raise ValueError('Pinned Lean 4.19.0 required')
    output.mkdir()
    work=output/'proof-work'
    shutil.copytree(ROOT/'proofs',work,ignore=shutil.ignore_patterns('.lake','__pycache__','*.pyc'))
    (work/'.lake').mkdir()
    (work/'.lake/packages').symlink_to(dependencies,target_is_directory=True)
    env=dict(os.environ,PATH=str(lean_bin)+os.pathsep+os.environ['PATH'],
             PYTHONDONTWRITEBYTECODE='1',XDG_CACHE_HOME=str(output/'cache'))
    receipt={'schema':'rheon-pr36-release-lean-qualification-v1','source_head':PIN,
             'source_tree':subprocess.check_output(['git','rev-parse',PIN+'^{tree}'],cwd=ROOT,text=True).strip(),
             'status':'running','lean_checked':False,'source_clean':True,
             'qualification_scope':'Unchanged PR36 proof copy, pinned normal lake build and rejection/source gates; no hosted CI',
             'lean_version':version,'proof_inventory_sha256':sha(inventory),
             'proof_source_sha256':sources,'dependency_pins':dependency_pins,
             'allowed_axioms':['propext','Classical.choice','Quot.sound'],
             'driver_sha256':sha(Path(__file__)),'commands':[],'held_campaigns':0,'public_actions':0}
    def run(args,name,cwd=work):
        r=subprocess.run(args,cwd=cwd,env=env,capture_output=True,text=True,timeout=1800)
        path=output/name;path.write_text(r.stdout+r.stderr)
        receipt['commands'].append({'command':args,'returncode':r.returncode,'log':name,'log_sha256':sha(path)})
        if r.returncode:raise RuntimeError('Qualification failed: '+name)
    try:
        run(['python3','scripts/check_sources.py'],'lean-source-gate.log')
        run(['lake','build'],'lean-build.log')
        run(['lake','env','lean','AxiomAudit.lean'],'lean-audit.log')
        for flags,label in [([],''),(['-O'],'-optimized')]:
            run(['python3',*flags,'scripts/test_audit.py'],'lean-audit-negative'+label+'.log')
            run(['python3',*flags,'scripts/test_check_sources.py'],'lean-source-tests'+label+'.log')
        audited=re.search(r'Axiom audit passed for (\d+) declarations',(output/'lean-audit.log').read_text())
        if audited is None:raise ValueError('Kernel audit did not report its declaration count')
        audit=(work/'AxiomAudit.lean').read_text()
        expected=re.search(r'let expected : Array Name := #\[(.*?)\]',audit,re.S).group(1)
        receipt.update(public_theorems=sum(len(re.findall(r'^theorem ',(work/name).read_text(),re.M))
                                          for name in sources if name.startswith('Rheon/')),
                       explicit_expected_declarations=len(re.findall(r'`Rheon\.',expected)),
                       audited_declarations=int(audited[1]))
        if any(sha(ROOT/'proofs'/name)!=digest or sha(work/name)!=digest for name,digest in sources.items()):
            raise ValueError('Proof sources changed during qualification')
        receipt.update(status='passed',lean_checked=True)
    except Exception as error:
        receipt.update(status='failed',failure=str(error))
        raise
    finally:
        (output/'qualification.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--lean-bin',type=Path,required=True,help='Pinned Lean bin directory')
    parser.add_argument('--lean-dependencies',type=Path,required=True)
    args=parser.parse_args()
    receipt=qualify(args.output,args.lean_bin,args.lean_dependencies)
    print(json.dumps({k:receipt[k] for k in ['status','source_head','public_theorems','explicit_expected_declarations','audited_declarations']}))
