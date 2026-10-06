"""Focused regressions for accepted malformed qualification substitutes."""
import copy,importlib.util,json,unittest
from pathlib import Path
P=Path(__file__).resolve().parent;R=P.parents[1]
def load(name,file):
 spec=importlib.util.spec_from_file_location(name,P/file);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
host=load('repair_host_oracle','verify_trajectory.py');native=load('repair_native_oracle','replay.py')
class Qualification(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.good=json.loads((R/'evidence/fitted-discrete-refinement/trials/repaired-initial-trajectory.json').read_text());cls.native=[json.loads(x)for x in(P/'trials/native-state-identity.jsonl').read_text().splitlines()]
 def test_distinct_schedule_cannot_be_five_coarse_duplicates(self):
  bad=copy.deepcopy(self.good);bad['replays']=[copy.deepcopy(bad['replays'][0])for _ in range(5)];bad['rows']=[copy.deepcopy(bad['rows'][0])for _ in range(5)]
  with self.assertRaisesRegex(ValueError,'distinct'):host.verify(bad)
 def test_common_endpoint_duration_is_required(self):
  bad=copy.deepcopy(self.good);bad['duration']=999
  with self.assertRaisesRegex(ValueError,'endpoint'):host.verify(bad)
 def test_reference_list_cannot_be_empty(self):
  bad=copy.deepcopy(self.good);bad['references']=[]
  with self.assertRaisesRegex(ValueError,'references'):host.verify(bad)
 def test_reference_must_be_related_to_same_initial_field(self):
  bad=copy.deepcopy(self.good);bad['references'][1]['final_z'][0]+=.001
  with self.assertRaisesRegex(ValueError,'reference'):host.verify(bad)
 def test_fabricated_error_summary_is_recomputed(self):
  bad=copy.deepcopy(self.good);bad['rows'][0]['velocity_lumped_L2_error']=999
  with self.assertRaisesRegex(ValueError,'velocity_lumped'):host.verify(bad)
 def test_fabricated_ratio_summary_is_recomputed(self):
  bad=copy.deepcopy(self.good);bad['velocity_refinement_ratios']=[999]*4
  with self.assertRaisesRegex(ValueError,'velocity ratios'):host.verify(bad)
 def test_incomplete_case_cannot_claim_common_end(self):
  bad=copy.deepcopy(self.good);bad['replays'][4]['steps'].pop()
  with self.assertRaisesRegex(ValueError,'count'):host.verify(bad)
 def test_required_native_publication_fields_cannot_be_omitted(self):
  for name in ['mass','positions','triangles','physical_pressure','stamp']:
   with self.subTest(field=name):
    bad=copy.deepcopy(self.native);del bad[0][name]
    with self.assertRaisesRegex(ValueError,'required'):native.replay(bad)
 def test_native_stamp_is_actual_owner_id_and_version(self):
  for component in ['id','version']:
   with self.subTest(component=component):
    bad=copy.deepcopy(self.native);bad[0]['stamp'][component]+=1
    with self.assertRaisesRegex(ValueError,'identity'):native.replay(bad)
 def test_native_case_order_is_distinct_and_canonical(self):
  bad=copy.deepcopy(self.native);bad[0]['h']=.03
  with self.assertRaisesRegex(ValueError,'canonical'):native.replay(bad)
if __name__=='__main__':unittest.main()
