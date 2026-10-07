from generated_fixtures import fixture
import copy
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_column_mac as gate

FIXTURE = None
NAME = 'pulse-y-0.25-jacobi'
class ColumnMacVerifier(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        global FIXTURE
        FIXTURE = fixture("column_mac")
    def test_real_native_owner_records(self):
        result=gate.verify(FIXTURE)
        self.assertEqual((result['cases'],result['publications']),(22,76))
    def test_mass_pressure_epochs_ledgers_and_pixels_reject_tampering(self):
        variants=[]
        def mutate(label,fn):variants.append((label,fn))
        mutate('NaN',lambda x:x['steps'][0]['after_velocity'][0].__setitem__(1,float('nan')))
        mutate('Inf',lambda x:x['steps'][0]['publication']['projection'].__setitem__('kinetic_after',float('inf')))
        mutate('model',lambda x:x.__setitem__('model','liquid_dynamics'))
        mutate('moving',lambda x:x.__setitem__('moving_surface_dynamics',True))
        mutate('time',lambda x:x['steps'][0].__setitem__('physical_time',.125))
        mutate('spacing',lambda x:x['spacing'].__setitem__(1,.5))
        mutate('phase',lambda x:x['initial_fractions'].__setitem__(4,.5))
        mutate('phase-flag',lambda x:x['steps'][0].__setitem__('phase_unchanged',False))
        mutate('lift',lambda x:x['steps'][0]['lifted_velocity'][0].__setitem__(1,.2))
        mutate('wall',lambda x:x['steps'][0]['after_velocity'][0].__setitem__(0,.1))
        mutate('normal-correction',lambda x:x['steps'][0]['after_velocity'][1].__setitem__(2,.1))
        mutate('pressure',lambda x:x['steps'][0]['pressure'].__setitem__(0,-.5))
        mutate('export',lambda x:x['steps'][0]['exported_profiles'][0].__setitem__(0,.1))
        mutate('carry',lambda x:x['steps'][1]['before_velocity'][0].__setitem__(1,.1))
        mutate('carrier-id',lambda x:x['steps'][0]['publication'].__setitem__('after_carrier_id',18))
        mutate('volume-id',lambda x:x['steps'][0]['publication'].__setitem__('after_volume_id',18))
        mutate('geometry-id',lambda x:x['steps'][0]['publication']['transfer'].__setitem__('geometry_id',18))
        mutate('pressure-epoch',lambda x:x['steps'][0].__setitem__('pressure_geometry_version',12))
        mutate('export-epoch',lambda x:x['steps'][0].__setitem__('export_carrier_version',0))
        mutate('mass',lambda x:x['steps'][0]['publication']['transfer']['mac_mass'].__setitem__(0,.6))
        mutate('phase-mass',lambda x:x['steps'][0]['publication'].__setitem__('represented_phase_mass',.6))
        mutate('wall-impulse',lambda x:x['steps'][0]['publication']['transfer']['wall_impulse'].__setitem__(0,0))
        mutate('mixing',lambda x:x['steps'][0]['publication']['transfer'].__setitem__('mixing_loss',0))
        mutate('pressure-impulse',lambda x:x['steps'][0]['publication']['projection']['pressure_impulse'].__setitem__(0,0))
        mutate('omitted-normal',lambda x:x['steps'][0]['export'].__setitem__('omitted_normal_energy',0))
        mutate('prescribed-energy',lambda x:x['steps'][1]['publication'].__setitem__('prescribed_energy_change',0))
        mutate('inflated-budget',lambda x:x['steps'][0]['publication']['projection'].__setitem__('energy_budget',1e-3))
        mutate('inflated-phase-budget',lambda x:x['steps'][0]['publication'].__setitem__('phase_mass_budget',1e-3))
        mutate('memory',lambda x:x['steps'][0]['publication'].__setitem__('total_array_bytes',0))
        mutate('missing-publication',lambda x:x['steps'].pop())
        original=json.loads((FIXTURE/NAME/'case.json').read_text())
        for label,fn in variants:
            with self.subTest(label=label),tempfile.TemporaryDirectory() as directory:
                dest=Path(directory)/'case';shutil.copytree(FIXTURE/NAME,dest)
                data=copy.deepcopy(original);fn(data);(dest/'case.json').write_text(json.dumps(data))
                with self.assertRaises(ValueError):gate.verify_case(dest,NAME)
        with tempfile.TemporaryDirectory() as directory:
            dest=Path(directory)/'case';shutil.copytree(FIXTURE/NAME,dest)
            from PIL import Image
            with Image.open(dest/'final.png') as img:
                img.load();img.putpixel((0,0),0);img.save(dest/'final.png')
            with self.assertRaises(ValueError):gate.verify_case(dest,NAME)
        with tempfile.TemporaryDirectory() as directory:
            dest=Path(directory)/'packet';shutil.copytree(FIXTURE,dest);shutil.rmtree(dest/'curl-n64')
            with self.assertRaises(ValueError):gate.verify(dest)
if __name__=='__main__':unittest.main()
