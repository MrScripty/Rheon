import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from PIL import Image
from verify_column_shear import ROOT,verify

class ColumnShearVerifierTest(unittest.TestCase):
    def test_native_liquid_masses_forces_energy_and_convergence(self):
        result=verify(ROOT/'evidence/column-shear/demo')
        self.assertEqual(len(result['results']),24)
        self.assertEqual(sum(r['intervals'] for r in result['results']),1298)
        self.assertTrue(result['prerequisite_only'])
        self.assertFalse(result['coupled_carrier_free_surface_viscosity_claimed'])
    def test_adversarial_geometry_force_energy_momentum_and_pixels(self):
        variants=('nan_field','inf_mass','nan_force','mass_full_box','dual_length','force_balanced_pair','update','carry','position','stability','energy','energy_budget','dissipation','momentum','momentum_budget','force_budget','volume','volume_budget','geometry_version','workspace','density','viscosity','boundary','traction','coupled_claim','pixel','missing_profile','missing_case')
        for variant in variants:
            with self.subTest(variant=variant),tempfile.TemporaryDirectory() as temporary:
                demo=Path(temporary)/'demo';shutil.copytree(ROOT/'evidence/column-shear/demo',demo)
                case=demo/'decay-a1-n8-f0.75'
                if variant=='missing_case':shutil.rmtree(case)
                elif variant=='missing_profile':(case/'profiles.csv').unlink()
                elif variant=='pixel':
                    path=case/'final.png'
                    with Image.open(path) as original:image=original.copy()
                    image.putpixel((0,0),image.getpixel((0,0))^1);image.save(path)
                elif variant in ('density','viscosity','boundary','traction','coupled_claim'):
                    path=case/'case.json';meta=json.loads(path.read_text())
                    key,value={'density':('density',99),'viscosity':('dynamic_viscosity',99),'boundary':('lateral_boundary','sealed'),'traction':('endpoint_shear_traction','wall-damping'),'coupled_claim':('coupled_carrier_step',True)}[variant]
                    meta[key]=value;path.write_text(json.dumps(meta))
                else:
                    name,key,index,value={
                        'nan_field':('final-faces.csv','velocity',0,'nan'),
                        'inf_mass':('profiles.csv','mass',0,'inf'),
                        'nan_force':('profiles.csv','force0',0,'nan'),
                        'mass_full_box':('profiles.csv','mass',8,str(3*.75/8.75)),
                        'dual_length':('profiles.csv','dual_length',8,'99'),
                        'force_balanced_pair':('profiles.csv','force0',0,'balanced'),
                        'update':('profiles.csv','after0',0,'.5'),
                        'carry':('profiles.csv','before0',9,'.5'),
                        'position':('profiles.csv','position',0,'99'),
                        'stability':('steps.csv','stability',0,'1.1'),
                        'energy':('steps.csv','kinetic_after',0,'99'),
                        'energy_budget':('steps.csv','energy_budget',0,'99'),
                        'dissipation':('steps.csv','dissipation_before',0,'-1'),
                        'momentum':('steps.csv','momentum_after0',0,'99'),
                        'momentum_budget':('steps.csv','momentum_budget0',0,'99'),
                        'force_budget':('steps.csv','force_budget0',0,'99'),
                        'volume':('steps.csv','volume',0,'99'),
                        'volume_budget':('steps.csv','mass_volume_budget',0,'99'),
                        'geometry_version':('steps.csv','geometry_version',0,'99'),
                        'workspace':('steps.csv','workspace_bytes',0,'99'),
                    }[variant]
                    path=case/name
                    with path.open() as stream:reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
                    if variant=='force_balanced_pair':
                        rows[0][key]=str(float(rows[0][key])+.01);rows[1][key]=str(float(rows[1][key])-.01)
                    else:rows[index][key]=value
                    with path.open('w',newline='') as stream:writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)
                with self.assertRaises((ValueError,FileNotFoundError)):verify(demo)
if __name__=='__main__':unittest.main()
