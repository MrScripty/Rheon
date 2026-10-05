import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from PIL import Image
from verify_column_interface import ROOT,verify

class ColumnVerifierTest(unittest.TestCase):
    def test_actual_consecutive_reconstruction_and_convergence(self):
        result=verify(ROOT/'evidence/column-interface/demo')
        self.assertEqual(len(result['results']),13)
        self.assertEqual(sum(row.get('coupled_intervals',0) for row in result['results']),64)
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
