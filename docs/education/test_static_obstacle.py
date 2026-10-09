"""Refuse corrupted native geometry records, including with Python assertions off."""
from pathlib import Path
import copy,json,os,unittest
from static_obstacle import controls

class GeometryRecords(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.records=json.loads((Path(os.environ['RHEON_OBSTACLE'])/'records.json').read_text())

    def test_native_roster_and_independent_oracles(self):
        result=controls(self.records)
        self.assertEqual(result['checks'],{'cases':9,'cells':240,'faces':962,'collision_probes':27,'flux_pairs':27,'topology_refusals':3})
        self.assertEqual(result['fluid_advances'],0)

    def test_cell_face_label_source_collision_and_flux_corruption_refuse(self):
        for change in [lambda r:r['cases'][0]['volumes'].__setitem__(0,1.0),
                       lambda r:r['cases'][0]['areas'][0].__setitem__(0,0.0),
                       lambda r:r['cases'][1]['labels'].__setitem__(0,1),
                       lambda r:r['cases'][0]['triangles'][0].reverse(),
                       lambda r:r['cases'][0]['probes'][0].__setitem__('t',0.5),
                       lambda r:r['cases'][0]['flux_controls'][0]['outward'].__setitem__(1,1.0)]:
            with self.subTest(change=change):
                records=copy.deepcopy(self.records);change(records)
                with self.assertRaises(ValueError):controls(records)

    def test_changed_fixture_roster_and_silently_accepted_baffle_refuse(self):
        r=copy.deepcopy(self.records);r['cases'].pop()
        with self.assertRaisesRegex(ValueError,'roster'):controls(r)
        r=copy.deepcopy(self.records);r['refusals'][0]['error']='success'
        with self.assertRaisesRegex(ValueError,'refusal'):controls(r)

if __name__=='__main__':unittest.main()
