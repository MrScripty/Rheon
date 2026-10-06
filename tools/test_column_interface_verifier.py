import csv
import fnmatch
import re
import json
from pathlib import Path
import shutil
import tempfile
import subprocess
import unittest
from PIL import Image
from verify_column_interface import ROOT,verify

def ci_path_ledger():
    # Use this checkout's tracked inventory, excluding another lane's examples
    # and any untracked probes. The same returned rows drive the actual test.
    examples=subprocess.check_output(['git','ls-files','--','examples/*.rs'],cwd=ROOT,text=True).splitlines()
    required=examples+['tools/verify_column_interface.py','tools/test_column_interface_verifier.py','evidence/column-interface/demo/jacobi-pcg-v1-activation/final.png']
    text=(ROOT/'.github/workflows/rust-rheon.yml').read_text();controls=[]
    for event in ('pull_request','push'):
        section=re.split(r'\n  [a-z_]+:',text.split('  '+event+':\n',1)[1],maxsplit=1)[0]
        paths=re.findall(r"^      - '([^']+)'$",section,re.MULTILINE)
        for path in required:
            controls.append(dict(event=event,path=path,matched_patterns=[pattern for pattern in paths if fnmatch.fnmatchcase(path,pattern)]))
    return dict(tracked_examples=examples,additional_paths=required[len(examples):],controls=controls)

class ColumnVerifierTest(unittest.TestCase):
    def test_actual_consecutive_reconstruction_and_convergence(self):
        result=verify(ROOT/'evidence/column-interface/demo')
        self.assertEqual(len(result['results']),13)
        self.assertEqual(sum(row.get('coupled_intervals',0) for row in result['results']),64)
    def test_every_coupled_final_artifact_matches_last_accepted_frame(self):
        cases=[method+'-'+kind for method in ('jacobi-pcg-v1','sgs-pcg-v1') for kind in ('pulse','activation','mixed-rest')]
        changes=('nan_velocity','finite_velocity','noninteger_face','nan_geometry','finite_geometry','pixel','corrupt_png','missing_faces','missing_geometry','missing_png')
        for name in cases:
            for change in changes:
                with self.subTest(case=name,change=change),tempfile.TemporaryDirectory() as temporary:
                    demo=Path(temporary)/'demo';shutil.copytree(ROOT/'evidence/column-interface/demo',demo)
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
        self.assertEqual(current.split('jobs:\n',1)[1].replace('timeout-minutes: 30','timeout-minutes: 15'),frozen.split('jobs:\n',1)[1])
    def test_adversarial_finite_fields_ledgers_geometry_and_pixels(self):
        changes=('nan_pressure','inf_velocity','nan_fraction','nan_geometry','before','inward','outward','balance','budget','carry','nan_ledger','reconstruction_change','end_geometry','held_geometry','pressure_version','air_pressure','pixel','missing_frame','advection_fraction','advection_shape_mass_preserved')
        for change in changes:
            with self.subTest(change=change),tempfile.TemporaryDirectory() as temporary:
                demo=Path(temporary)/'demo';shutil.copytree(ROOT/'evidence/column-interface/demo',demo)
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
