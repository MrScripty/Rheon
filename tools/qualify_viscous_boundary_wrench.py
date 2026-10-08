"""Local, source-frozen opt-in qualification of the stationary viscous wrench.

This runs no held PDE campaign or hosted CI. All new outputs must be outside Git.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = '39bf717fd3dc550fd9437b9cd3da6f532375d5dd'
ADAPTATIONS = {'src/lib.rs', 'proofs/Rheon.lean', 'proofs/AxiomAudit.lean',
               'proofs/source-inventory.json', '.gitignore'}

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

def qualify(output, lean, dependencies):
    output, lean, dependencies = [Path(x).resolve() for x in (output, lean, dependencies)]
    worktrees = [Path(x.split(' ',1)[1]).resolve() for x in
                 git('worktree','list','--porcelain').splitlines() if x.startswith('worktree ')]
    if output.exists() or any(output.is_relative_to(p) for p in worktrees):
        raise ValueError('new external evidence directory required')
    if git('status','--porcelain','--untracked-files=all'):
        raise ValueError('clean committed source required')
    tracked = git('ls-files').splitlines()
    sources = {p: sha(ROOT/p) for p in tracked}
    baseline = set(git('ls-tree','-r','--name-only',BASE).splitlines())
    changed = set(git('diff',BASE,'--name-only').splitlines())
    if (changed & baseline)-ADAPTATIONS:
        raise ValueError('protected baseline source changed: '+str((changed & baseline)-ADAPTATIONS))
    oldlib = subprocess.check_output(['git','show',BASE+':src/lib.rs'],cwd=ROOT)
    export = b'mod viscous_boundary_wrench;\npub use viscous_boundary_wrench::{\n    AlignedViscousBoundaryWrench, ViscousBoundaryLift, ViscousBoundaryVirtualWork,\n    ViscousBoundaryWrenchReport,\n};\n'
    expectedlib = oldlib.replace(b'mod static_obstacle;\n', b'mod static_obstacle;\nmod viscous_boundary_wrench;\n').replace(b'mod motion;\n',export.removeprefix(b'mod viscous_boundary_wrench;\n')+b'mod motion;\n')
    if (ROOT/'src/lib.rs').read_bytes() != expectedlib:
        raise ValueError('lib adaptation exceeds exact wrench module/export insertion')
    manifest = json.loads((ROOT/'proofs/lake-manifest.json').read_text())
    pins = {}
    for package in manifest['packages']:
        actual = subprocess.check_output(['git','rev-parse','HEAD'],cwd=dependencies/package['name'],text=True).strip()
        if actual != package['rev']: raise ValueError('dependency pin changed '+package['name'])
        pins[package['name']] = actual
    target = Path(json.loads(subprocess.check_output(['cargo','metadata','--offline','--locked','--no-deps','--format-version','1'],cwd=ROOT))['target_directory']).resolve()
    if any(target.is_relative_to(p) for p in worktrees): raise ValueError('Cargo target must be external')
    output.mkdir(parents=True)
    receipt = {'qualified': False, 'base': BASE, 'source_head': git('rev-parse','HEAD'),
               'source_tree': git('rev-parse','HEAD^{tree}'), 'source_clean': True,
               'source_sha256': sources, 'protected_baseline_files': sorted(baseline-ADAPTATIONS),
               'changed_paths': sorted(changed), 'lean_dependency_pins': pins,
               'scope': 'stationary stored-coefficient variational viscous boundary wrench; no exact geometric traction accuracy, pressure coupling, stepping or moving geometry',
               'hosted_ci_requested': False, 'held_campaigns': 0, 'resource_limits_expanded': False,
               'commands': [], 'binaries': {}}

    def run(command, name, cwd=ROOT, env=None, expected_error=None):
        r = subprocess.run(list(map(str,command)),cwd=cwd,env=env,text=True,capture_output=True,timeout=180)
        log = output/name; log.write_text(r.stdout+r.stderr)
        receipt['commands'].append({'command':list(map(str,command)), 'cwd':str(cwd),
                                    'returncode':r.returncode,'log':name,'log_sha256':sha(log)})
        if expected_error is None:
            if r.returncode: raise RuntimeError('failed '+name)
        else:
            errors=[s.split(': error: ',1)[1] for s in (r.stdout+r.stderr).splitlines() if ': error: ' in s]
            if not r.returncode or errors != [expected_error]: raise RuntimeError('wrong negative failure '+name)
        return r.stdout

    try:
        receipt['toolchains'] = {'rust':run(['rustc','-Vv'],'rustc.log'),
                                 'cargo':run(['cargo','--version'],'cargo.log'),
                                 'lean':run([lean,'--version'],'lean-version.log')}
        tests = ['viscous_boundary_wrench_contract','aligned_strain_contract','obstacle_flow_contract',
                 'static_obstacle_contract','mesh_traction_contract','sphere_support_contract']
        testargs = sum((['--test',t] for t in tests),[])
        for profile in ['debug','release']:
            flags = ['--release'] if profile=='release' else []
            run(['cargo','test','--offline','--locked','--no-default-features',*flags,*testargs], 'native-'+profile+'.log')
            run(['cargo','build','--offline','--locked','--no-default-features',*flags,'--example','viscous_boundary_wrench'], 'build-'+profile+'.log')
            binary = target/profile/'examples/viscous_boundary_wrench'
            (output/'binaries').mkdir(exist_ok=True)
            frozen=output/'binaries'/('viscous_boundary_wrench-'+profile)
            shutil.copyfile(binary,frozen); frozen.chmod(0o700)
            receipt['binaries'][profile]={'path':str(frozen),'sha256':sha(frozen)}
            for optimize in ([False,True] if profile=='debug' else [False]):
                suffix = profile+('-optimized' if optimize else '')
                run([sys.executable,*(['-O'] if optimize else []),'tools/check_viscous_boundary_wrench.py',
                     '--executable',frozen,'--output',output/('oracle-'+suffix)],'oracle-'+suffix+'.log')
        for optimize in [False,True]:
            run([sys.executable,*(['-O'] if optimize else []),'-m','unittest','discover','-s','tools',
                 '-p','test_viscous_boundary_wrench_oracle.py'],'oracle-tests'+('-optimized' if optimize else '')+'.log')
        run(['cargo','clippy','--offline','--locked','--no-default-features','--lib','--example',
             'viscous_boundary_wrench','--test','viscous_boundary_wrench_contract','--','-D','warnings'],'clippy.log')
        run(['rustfmt','--edition','2024','--check','src/viscous_boundary_wrench.rs','src/lib.rs',
             'examples/viscous_boundary_wrench.rs','tests/viscous_boundary_wrench_contract.rs'],'rustfmt.log')
        run([sys.executable,'proofs/scripts/check_sources.py'],'proof-source-gate.log')
        proof_output = output/'lean-build'; (proof_output/'Rheon').mkdir(parents=True)
        paths = [str(proof_output)]+[str(p/'.lake/build/lib/lean') for p in sorted(dependencies.iterdir())
                                    if (p/'.lake/build/lib/lean').is_dir()]
        env = dict(os.environ,LEAN_PATH=':'.join(paths))
        modules = {'Rheon.'+p.stem:p for p in (ROOT/'proofs/Rheon').glob('*.lean')}
        completed = set()
        while len(completed)<len(modules):
            ready=[n for n,p in modules.items() if n not in completed and all(d not in modules or d in completed
                    for d in re.findall(r'^import (Rheon\.[A-Za-z]+)$',p.read_text(),re.M))]
            if not ready: raise ValueError('cyclic proof dependency')
            for name in sorted(ready):
                p=modules[name]
                run([lean,'-o',proof_output/'Rheon'/(p.stem+'.olean'),p.relative_to(ROOT/'proofs')],
                    'lean-'+p.stem+'.log',ROOT/'proofs',env)
                completed.add(name)
        run([lean,'-o',proof_output/'Rheon.olean','Rheon.lean'],'lean-root.log',ROOT/'proofs',env)
        run([lean,'AxiomAudit.lean'],'axiom-audit.log',ROOT/'proofs',env)
        audit = (ROOT/'proofs/AxiomAudit.lean').read_text()
        probes = [('axiom','axiom injected : False','Disallowed axiom Rheon.ViscousBoundaryWrench.injected in Rheon.ViscousBoundaryWrench.injected'),
                  ('sorry','theorem injected : False := by sorry','Disallowed axiom sorryAx in Rheon.ViscousBoundaryWrench.injected')]
        for label,declaration,error in probes:
            path=proof_output/('negative-'+label+'.lean')
            path.write_text(audit.replace('open Lean Elab Command','namespace Rheon.ViscousBoundaryWrench\n'+declaration+'\nend Rheon.ViscousBoundaryWrench\nopen Lean Elab Command'))
            run([lean,path],'negative-'+label+'.log',ROOT/'proofs',env,expected_error=error)
        if sources != {p:sha(ROOT/p) for p in tracked} or receipt['source_head'] != git('rev-parse','HEAD'):
            raise ValueError('source changed during qualification')
        if git('status','--porcelain','--untracked-files=all'): raise ValueError('source became dirty')
        for binary in receipt['binaries'].values():
            if sha(Path(binary['path'])) != binary['sha256']: raise ValueError('frozen binary changed')
        receipt['qualified']=True
    except Exception as error:
        receipt['failure']=f'{type(error).__name__}: {error}'
        raise
    finally:
        receipt['evidence_sha256']={str(p.relative_to(output)):sha(p) for p in sorted(output.rglob('*')) if p.is_file()}
        (output/'qualification.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return {'qualified':True,'head':receipt['source_head'],'output':str(output)}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--lean-bin',type=Path,required=True)
    parser.add_argument('--lean-dependencies',type=Path,required=True)
    args=parser.parse_args()
    print(json.dumps(qualify(args.output,args.lean_bin,args.lean_dependencies)))
