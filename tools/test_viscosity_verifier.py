from generated_fixtures import fixture
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from PIL import Image
from verify_viscosity import ROOT,verify

class ViscosityVerifierTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = fixture("viscosity")
    def test_native_shear_decay_refinement_and_coupled_box(self):
        result=verify(self.fixture)
        self.assertEqual(len(result['results']),13)
        self.assertEqual(sum(r['intervals'] for r in result['results'] if r['kind']=='coupled'),36)
        self.assertFalse(result['free_surface_viscosity_validated'])
    def test_adversarial_fields_energy_units_ledger_and_pixels(self):
        variants=('nan_velocity','inf_energy','nan_budget','negative_dissipation','stability','identity','inflated_budget','amplitude','mode_shape','wall_velocity','density','viscosity','boundary','phase_fraction','pressure','generation','time','carry','ledger_budget','workspace','pixel','missing_frame','missing_case')
        for variant in variants:
            with self.subTest(variant=variant),tempfile.TemporaryDirectory() as temporary:
                demo=Path(temporary)/'demo';shutil.copytree(self.fixture,demo)
                case=demo/'space-n16';coupled=demo/'jacobi-pcg-v1-coupled-mu0.125'
                if variant=='missing_case':shutil.rmtree(case)
                elif variant=='missing_frame':(coupled/'frame-01-cells.csv').unlink()
                elif variant=='pixel':
                    path=case/'final.png'
                    with Image.open(path) as original:image=original.copy()
                    image.putpixel((0,0),image.getpixel((0,0))^1);image.save(path)
                elif variant in ('density','viscosity','boundary'):
                    path=case/'case.json';meta=json.loads(path.read_text())
                    key,value={'density':('density',999),'viscosity':('dynamic_viscosity',999),'boundary':('wall','no-slip')}[variant]
                    meta[key]=value;path.write_text(json.dumps(meta))
                else:
                    name,key,index,value={
                        'nan_velocity':('final-faces.csv','velocity',1,'nan'),
                        'inf_energy':('steps.csv','kinetic_after',0,'inf'),
                        'nan_budget':('steps.csv','rounding_budget',0,'nan'),
                        'negative_dissipation':('steps.csv','dissipation_before',0,'-1'),
                        'stability':('steps.csv','stability',0,'.251'),
                        'identity':('steps.csv','identity_error',0,'999'),
                        'inflated_budget':('steps.csv','rounding_budget',0,'999'),
                        'amplitude':('steps.csv','amplitude',0,'.5'),
                        'mode_shape':('final-faces.csv','velocity',1,'.25'),
                        'wall_velocity':('final-faces.csv','velocity',0,'.25'),
                        'phase_fraction':('frame-01-cells.csv','fraction',0,'.9'),
                        'pressure':('frame-01-cells.csv','pressure',0,'.5'),
                        'generation':('liquid.csv','liquid_version',0,'999'),
                        'time':('liquid.csv','time',0,'999'),
                        'carry':('liquid.csv','volume_before',1,'3'),
                        'ledger_budget':('liquid.csv','budget',0,'999'),
                        'workspace':('liquid.csv','total_bytes',0,'999'),
                    }[variant]
                    if variant in ('phase_fraction','pressure','generation','time','carry','ledger_budget','workspace'):case=coupled
                    path=case/name
                    with path.open() as stream:reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
                    rows[index][key]=value
                    with path.open('w',newline='') as stream:writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)
                with self.assertRaises((ValueError,FileNotFoundError)):verify(demo)
if __name__=='__main__':unittest.main()
