from generated_fixtures import fixture
import csv
import fnmatch
import re
import json
from pathlib import Path
import shutil
import tempfile
import subprocess
import unittest
import textwrap
from PIL import Image
from verify_column_interface import ROOT,verify

def ci_path_ledger():
    # Use this checkout's tracked inventory, excluding another lane's examples
    # and any untracked probes. The same returned rows drive the actual test.
    examples=subprocess.check_output(['git','ls-files','--','examples/*.rs'],cwd=ROOT,text=True).splitlines()
    required=examples+['tools/verify_column_interface.py','tools/test_column_interface_verifier.py','tools/generated_fixtures.py']
    text=(ROOT/'.github/workflows/rust-rheon.yml').read_text();controls=[]
    for event in ('pull_request','push'):
        section=re.split(r'\n  [a-z_]+:',text.split('  '+event+':\n',1)[1],maxsplit=1)[0]
        paths=re.findall(r"^      - '([^']+)'$",section,re.MULTILINE)
        for path in required:
            controls.append(dict(event=event,path=path,matched_patterns=[pattern for pattern in paths if fnmatch.fnmatchcase(path,pattern)]))
    return dict(tracked_examples=examples,additional_paths=required[len(examples):],controls=controls)

def verify_split_ci_budget(frozen,current):
    """Retain exact former step bodies/pins while allowing the approved partition.

    This intentionally verifies this one layout, not general YAML. The original
    snapshot remains immutable. No new Python dependency is required by CI.
    """
    def require(ok,message):
        if not ok:raise ValueError(message)
    def steps(body):
        # Split at actual step starts, then normalize only their common indent
        # and action-version comments; all command/assertion bytes stay exact.
        starts=list(re.finditer(r'(?m)^([ ]*)- (?:uses|name):',body))
        result=[]
        for i,start in enumerate(starts):
            block=body[start.start():starts[i+1].start() if i+1<len(starts) else len(body)]
            block=textwrap.dedent(block).rstrip()+'\n'
            block=re.sub(r'(?m)^((?:- |  )uses: [^ #]+) #.*$',r'\1',block)
            result.append(block)
        return result
    old=steps(frozen.split('    steps:\n',1)[1])
    require(len(old)==8,'immutable former eight-step baseline')
    matches=list(re.finditer(r'^  ([a-z-]+):\n(.*?)(?=^  [a-z-]+:|\Z)',current.split('jobs:\n',1)[1],re.M|re.S))
    require(len(matches)==3,'exact job inventory without duplicates')
    parsed={m.group(1):m.group(2) for m in matches}
    expected_prefixes={
        'feature-contracts': (
            '    name: Feature contracts (${{ matrix.mode }})\n'
            '    runs-on: ubuntu-24.04\n'
            '    timeout-minutes: 30\n'
            '    env:\n'
            "      CARGO_BUILD_JOBS: '1'\n"
            '    strategy:\n'
            '      fail-fast: false\n'
            '      matrix:\n'
            '        include:\n'
            '        - mode: default\n'
            "          flags: ''\n"
            '        - mode: core-only\n'
            '          flags: --no-default-features\n'
            '        - mode: desktop\n'
            '          flags: --features desktop\n'
        ),
        'executable-contracts': (
            '    runs-on: ubuntu-24.04\n'
            '    timeout-minutes: 30\n'
            '    env:\n'
            "      CARGO_BUILD_JOBS: '1'\n"
        ),
        'core-and-executable': (
            '    name: core-and-executable\n'
            '    needs:\n'
            '    - feature-contracts\n'
            '    - executable-contracts\n'
            '    if: always()\n'
            '    runs-on: ubuntu-24.04\n'
            '    timeout-minutes: 30\n'
        ),
    }
    require(list(parsed)==list(expected_prefixes),'exact independent jobs and original required status')
    for name,prefix in expected_prefixes.items():
        require(parsed[name].split('    steps:\n',1)[0]==prefix,'runner/budget/matrix/dependencies: '+name)
    feature=[old[0],old[2],
        '- name: Strict lints for selected features\n  run: |\n    cargo clippy --locked ${{ matrix.flags }} --all-targets -- -D warnings\n',
        '- name: Contract tests for selected features\n  run: |\n    cargo test --locked ${{ matrix.flags }}\n',
        "- name: Inspect core-only dependency tree\n  if: matrix.mode == 'core-only'\n  run: |\n    cargo tree --locked --no-default-features\n"]
    common=old[:3]+['- name: Formatting\n  run: |\n    cargo fmt --all --check\n']+old[5:]
    gate=['- name: Require every existing qualification path\n  env:\n    FEATURE_RESULT: ${{ needs.feature-contracts.result }}\n    EXECUTABLE_RESULT: ${{ needs.executable-contracts.result }}\n  run: |\n    test "$FEATURE_RESULT" = success\n    test "$EXECUTABLE_RESULT" = success\n']
    for name,expected in [('feature-contracts',feature),('executable-contracts',common),('core-and-executable',gate)]:
        require(steps(parsed[name].split('    steps:\n',1)[1])==expected,'exact retained steps: '+name)
    # Explicitly link matrix expansion to every original lint/test/tree command.
    lint=old[3].split('  run: |\n',1)[1].splitlines()
    tests=old[4].split('  run: |\n',1)[1].splitlines()
    require([line.strip() for line in lint]==['cargo fmt --all --check']+[
        'cargo clippy --locked'+flags+' --all-targets -- -D warnings'
        for flags in ('',' --no-default-features',' --features desktop')], 'original lint inventory')
    require([line.strip() for line in tests]==[
        'cargo test --locked'+flags for flags in ('',' --no-default-features',' --features desktop')]+[
        'cargo tree --locked --no-default-features'],'original test/dependency inventory')

class ColumnVerifierTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = fixture("column_interface")
    def test_actual_consecutive_reconstruction_and_convergence(self):
        result=verify(self.fixture)
        self.assertEqual(len(result['results']),13)
        self.assertEqual(sum(row.get('coupled_intervals',0) for row in result['results']),64)
    def test_every_coupled_final_artifact_matches_last_accepted_frame(self):
        cases=[method+'-'+kind for method in ('jacobi-pcg-v1','sgs-pcg-v1') for kind in ('pulse','activation','mixed-rest')]
        changes=('nan_velocity','finite_velocity','noninteger_face','nan_geometry','finite_geometry','pixel','corrupt_png','missing_faces','missing_geometry','missing_png')
        for name in cases:
            for change in changes:
                with self.subTest(case=name,change=change),tempfile.TemporaryDirectory() as temporary:
                    demo=Path(temporary)/'demo';shutil.copytree(self.fixture,demo)
                    case=demo/name
                    if change.startswith('missing_'):
                        filename={'missing_faces':'final-faces.csv','missing_geometry':'final-geometry.csv','missing_png':'final.png'}[change]
                        (case/filename).unlink()
                    elif change=='corrupt_png':
                        (case/'final.png').write_bytes(b'not a PNG')
                    elif change=='pixel':
                        path=case/'final.png'
                        with Image.open(path) as original:image=original.copy()
                        image.putpixel((0,0),image.getpixel((0,0))^1);image.save(path)
                    else:
                        filename='final-geometry.csv' if change.endswith('geometry') else 'final-faces.csv'
                        field='top_fraction' if change.endswith('geometry') else 'axis' if change=='noninteger_face' else 'velocity'
                        path=case/filename
                        with path.open() as stream:
                            reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
                        rows[0][field]='nan' if change.startswith('nan_') else str(float(rows[0][field])+0.01)
                        with path.open('w',newline='') as stream:
                            writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)
                    with self.assertRaises((ValueError,OSError)):verify(demo)
    def test_rust_ci_triggers_for_each_example_and_column_verifier(self):
        self.path_ledger=ci_path_ledger()
        for row in self.path_ledger['controls']:
            with self.subTest(event=row['event'],path=row['path']):
                self.assertTrue(row['matched_patterns'])
    def test_ci_budget_preserves_every_existing_job_check(self):
        frozen=(ROOT/'evidence/column-interface-final-artifacts/trials/frozen-workflow.yml').read_text()
        current=(ROOT/'.github/workflows/rust-rheon.yml').read_text()
        verify_split_ci_budget(frozen,current)
    def test_split_ci_budget_rejects_removed_checks_and_early_success(self):
        frozen=(ROOT/'evidence/column-interface-final-artifacts/trials/frozen-workflow.yml').read_text()
        current=(ROOT/'.github/workflows/rust-rheon.yml').read_text()
        mutations={
            'desktop':("        - mode: desktop\n          flags: --features desktop\n",''),
            'lint':(' --all-targets -- -D warnings',''),
            'test':('cargo test --locked ${{ matrix.flags }}','cargo test --locked'),
            'dependency tree':('cargo tree --locked --no-default-features','cargo tree --locked'),
            'smoke gate':("assert result['last_divergence_max'] <= 1e-5","assert True"),
            'early success':('    - executable-contracts\n',''),
            'skip success':('test "$EXECUTABLE_RESULT" = success','true'),
            'credential persistence':('persist-credentials: false','persist-credentials: true'),
            'Python dependency':('Pillow==12.3.0','Pillow'),
            'time limit':('timeout-minutes: 30','timeout-minutes: 60'),
        }
        for name,(before,after) in mutations.items():
            with self.subTest(change=name):
                self.assertIn(before,current)
                with self.assertRaises(ValueError):
                    verify_split_ci_budget(frozen,current.replace(before,after))
    def test_adversarial_finite_fields_ledgers_geometry_and_pixels(self):
        changes=('nan_pressure','inf_velocity','nan_fraction','nan_geometry','before','inward','outward','balance','budget','carry','nan_ledger','reconstruction_change','end_geometry','held_geometry','pressure_version','air_pressure','pixel','missing_frame','advection_fraction','advection_shape_mass_preserved')
        for change in changes:
            with self.subTest(change=change),tempfile.TemporaryDirectory() as temporary:
                demo=Path(temporary)/'demo';shutil.copytree(self.fixture,demo)
                case=demo/'jacobi-pcg-v1-activation'
                if change=='pixel':
                    path=case/'frame-01.png'
                    with Image.open(path) as original: image=original.copy()
                    image.putpixel((0,0),image.getpixel((0,0))^1);image.save(path)
                elif change=='missing_frame': (case/'frame-01-cells.csv').unlink()
                elif change=='advection_shape_mass_preserved':
                    path=demo/'advection-n16-c025/final-cells.csv'
                    with path.open() as stream:
                        reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
                    rows[24]['fraction']=str(float(rows[24]['fraction'])+.01)
                    rows[25]['fraction']=str(float(rows[25]['fraction'])-.01)
                    with path.open('w',newline='') as stream:
                        writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)

                else:
                    filename,field,index,value={
                        'nan_pressure':('frame-01-cells.csv','pressure',0,'nan'),
                        'inf_velocity':('frame-01-faces.csv','velocity',0,'inf'),
                        'nan_fraction':('initial-cells.csv','fraction',0,'nan'),
                        'nan_geometry':('frame-01-geometry.csv','top_fraction',0,'nan'),
                        'before':('steps.csv','volume_before',0,'999'),
                        'inward':('steps.csv','inward',0,'999'),
                        'outward':('steps.csv','outward',0,'999'),
                        'balance':('steps.csv','balance',0,'999'),
                        'budget':('steps.csv','budget',0,'999'),
                        'carry':('steps.csv','volume_before',1,'999'),
                        'nan_ledger':('steps.csv','balance',0,'nan'),
                        'reconstruction_change':('steps.csv','reconstruction_change',0,'999'),
                        'end_geometry':('frame-01-geometry.csv','top_fraction',0,'.25'),
                        'held_geometry':('frame-01-geometry.csv','pressure_top_fraction',0,'.25'),
                        'pressure_version':('frame-01-geometry.csv','pressure_geometry_version',0,'999'),
                        'air_pressure':('frame-01-cells.csv','pressure',2,'.25'),
                        'advection_fraction':('final-cells.csv','fraction',33,'.75'),
                    }[change]
                    if change=='advection_fraction': case=demo/'advection-n16-c025'
                    path=case/filename
                    with path.open() as stream:
                        reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
                    rows[index][field]=value
                    with path.open('w',newline='') as stream:
                        writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)
                with self.assertRaises((ValueError,FileNotFoundError)): verify(demo)
if __name__=='__main__': unittest.main()
