from generated_fixtures import fixture
import copy
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_column_momentum as gate

FIXTURE = None
class ColumnMomentumVerifier(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        global FIXTURE
        FIXTURE = fixture("column_momentum")
    def test_real_native_records(self):
        result=gate.verify(FIXTURE)
        self.assertEqual((result['cases'],result['intervals']),(13,127))
    def test_finite_overlap_geometry_ledgers_and_pixels_reject_tampering(self):
        variants=[]
        def mutate(label, fn): variants.append((label,fn))
        mutate('NaN',lambda x:x['steps'][0]['after'][0].__setitem__(0,float('nan')))
        mutate('Infledger',lambda x:x['steps'][0]['report'].__setitem__('kinetic_after',float('inf')))
        mutate('model',lambda x:x.__setitem__('pressure_published',True))
        mutate('time',lambda x:x.__setitem__('physical_time_claimed',True))
        mutate('density',lambda x:x.__setitem__('density',1))
        mutate('spacing',lambda x:x['spacing'].__setitem__(0,1))
        mutate('height',lambda x:x['steps'][0]['after_heights'].__setitem__(0,2.5))
        mutate('donor',lambda x:x['steps'][0]['donor'][1].__setitem__(0,2.0))
        def balanced_update(x):
            x['steps'][0]['after'][0][1] += .3
            x['steps'][0]['after'][0][6] -= .5
        mutate('balanced-wrong-update',balanced_update)
        mutate('dry',lambda x:x['steps'][0]['after'][0].__setitem__(3,1.0))
        mutate('carry',lambda x:x['steps'][1]['before'][0].__setitem__(1,1.0))
        mutate('stamp',lambda x:x['steps'][0]['report'].__setitem__('after_version',2))
        mutate('mass',lambda x:x['steps'][0]['report'].__setitem__('mass_after',3.0))
        mutate('momentum',lambda x:x['steps'][0]['report']['momentum_after'].__setitem__(0,0.0))
        mutate('mixing',lambda x:x['steps'][0]['report'].__setitem__('mixing_loss',0.0))
        mutate('rounding',lambda x:x['steps'][0]['report'].__setitem__('rounding_work',1e-5))
        mutate('inflated-budget',lambda x:x['steps'][0]['report'].__setitem__('energy_budget',1e-4))
        mutate('support',lambda x:x['steps'][0]['report'].__setitem__('activated_nodes',0))
        mutate('workspace',lambda x:x['steps'][0]['report'].__setitem__('workspace_bytes',1))
        mutate('missing-interval',lambda x:x['steps'].pop())
        original=json.loads((FIXTURE/'exchange-y/case.json').read_text())
        for label,fn in variants:
            with self.subTest(label=label), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);shutil.copytree(FIXTURE/'exchange-y',root/'case')
                data=copy.deepcopy(original);fn(data);(root/'case/case.json').write_text(json.dumps(data))
                with self.assertRaises(ValueError):gate.verify_case(root/'case','exchange-y')
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);shutil.copytree(FIXTURE/'exchange-y',root/'case')
            from PIL import Image
            with Image.open(root/'case/final.png') as img:
                img.load();img.putpixel((0,0),0);img.save(root/'case/final.png')
            with self.assertRaises(ValueError):gate.verify_case(root/'case','exchange-y')
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);shutil.copytree(FIXTURE,root/'packet');shutil.rmtree(root/'packet/identity-z')
            with self.assertRaises(ValueError):gate.verify(root/'packet')
if __name__=='__main__':unittest.main()
