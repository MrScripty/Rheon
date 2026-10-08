"""Mutation checks for native pressure/shear controls; valid under Python -O."""
from pathlib import Path
import copy,json,os,unittest,tempfile
from unittest.mock import patch
from obstacle_flow import controls, gaussian, F, source_hashes, validate_packet, publish, verify_published, sha
class FlowRecords(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.records=json.loads((Path(os.environ['RHEON_FLOW'])/'records.json').read_text())
    def test_all_native_fields_and_ledgers(self):
        c=controls(self.records);self.assertEqual(c['pressure_projections'],4);self.assertEqual(c['rational_linear_solves'],48)
    def test_pressure_corruptions_refuse(self):
        for change in [lambda c:c['pressure'].__setitem__(1,999),lambda c:c['volumes'].__setitem__(0,0),lambda c:c['areas'][0].__setitem__(1,0),lambda c:c['labels'].__setitem__(0,None),lambda c:c['after'][0].__setitem__(1,.01),lambda c:c['residual'].__setitem__(2,.01),lambda c:c['ledger'].__setitem__(1,999),lambda c:c['gauges'].__setitem__(0,1),lambda c:c['vertices'][0].__setitem__(0,999)]:
            with self.subTest(change=change):
                r=copy.deepcopy(self.records);change(r['pressure_cases'][0])
                with self.assertRaises((ValueError,TypeError,IndexError)):controls(r)
    def test_shear_corruptions_refuse(self):
        for change in [lambda c:c.__setitem__('beta',0),lambda c:c.__setitem__('density',8),lambda c:c['centers'].__setitem__(0,0),lambda c:c['layer_volumes'].__setitem__(0,1),lambda c:c['wall_conductances'].__setitem__(0,99),lambda c:c['frames'][2]['velocity'].__setitem__(0,1),lambda c:c['frames'][2]['energy'].__setitem__(3,999),lambda c:c['frames'][2]['momentum'].__setitem__(3,999),lambda c:c['frames'].pop()]:
            with self.subTest(change=change):
                r=copy.deepcopy(self.records);change(r['shear_cases'][0])
                with self.assertRaises((ValueError,TypeError,IndexError)):controls(r)
    def test_roster_and_scope_refuse(self):
        r=copy.deepcopy(self.records);r['pressure_cases'].pop()
        with self.assertRaisesRegex(ValueError,'roster'):controls(r)
        r=copy.deepcopy(self.records);r['schema']='general-fluid-solver'
        with self.assertRaisesRegex(ValueError,'schema'):controls(r)
    def test_review_counterexample_actual_field_equation_refuses(self):
        r=copy.deepcopy(self.records)
        r['shear_cases'][0]['frames'][1]['velocity'][0]+=8e-13
        with self.assertRaisesRegex(ValueError,'actual shear momentum equation'):
            controls(r)
    def test_review_budget_and_nonnegative_metadata_counterexamples_refuse(self):
        budget_targets=[lambda r:r['pressure_cases'][0]['ledger'],
                        lambda r:r['shear_cases'][0]['frames'][1]['energy'],
                        lambda r:r['shear_cases'][0]['frames'][1]['momentum']]
        for target,index in zip(budget_targets,[6,7,5]):
            for value in [1e300,float('inf'),float('-inf'),float('nan'),-1e-13]:
                with self.subTest(index=index,value=value):
                    r=copy.deepcopy(self.records);target(r)[index]=value
                    with self.assertRaises(ValueError):controls(r)
        for value in [-1e-13,float('inf'),float('-inf'),float('nan')]:
            with self.subTest(residual=value):
                r=copy.deepcopy(self.records);r['shear_cases'][0]['frames'][1]['residual_max']=value
                with self.assertRaises(ValueError):controls(r)
        for index in range(3):
            r=copy.deepcopy(self.records);r['pressure_cases'][0]['residual'][index]=-1e-13
            with self.assertRaises(ValueError):controls(r)
    def test_packet_and_published_source_record_renderer_bindings(self):
        repo=Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);packet=root/'packet';packet.mkdir();output=root/'output';output.mkdir()
            (packet/'records.json').write_text(json.dumps(self.records))
            receipt=dict(controls(self.records),schema='rheon-obstacle-flow-qualification-v1',clean=True,sources=source_hashes(repo),records_sha256=sha(packet/'records.json'),source_head='fixture-head',source_tree='fixture-tree',elf_sha256='fixture-elf')
            def identity(args,**kwargs):return 'fixture-tree' if args[-1]=='HEAD^{tree}' else 'fixture-head'
            def write():(packet/'qualification.json').write_text(json.dumps(receipt))
            write()
            with patch('obstacle_flow.subprocess.check_output',side_effect=identity):
                self.assertEqual(validate_packet(repo,packet),receipt)
                body=publish(repo,output,packet);self.assertIn('Saved native binary64 fields',body)
                self.assertEqual(verify_published(repo,output),receipt)
                renderer=output/'obstacle_flow.js';renderer.write_text(renderer.read_text()+'\nchanged')
                with self.assertRaisesRegex(ValueError,'renderer binding'):verify_published(repo,output)
                original=copy.deepcopy(receipt)
                for change in [lambda r:r.__setitem__('clean',False),lambda r:r.__setitem__('source_head','changed'),lambda r:r.__setitem__('source_tree','changed'),lambda r:r.__setitem__('records_sha256','changed'),lambda r:r['sources'].__setitem__('src/lib.rs','changed'),lambda r:r.__setitem__('pressure_projections',99)]:
                    receipt=copy.deepcopy(original);change(receipt);write()
                    with self.assertRaises(ValueError):validate_packet(repo,packet)
                receipt=original;write();(packet/'unexpected').write_text('unreviewed')
                with self.assertRaisesRegex(ValueError,'regular files'):validate_packet(repo,packet)
    def test_source_only_flow_states_absence(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo=Path(__file__).resolve().parents[2];output=Path(tmp)
            self.assertIn('fields are absent',publish(repo,output))
            self.assertIsNone(verify_published(repo,output))
            self.assertFalse((output/'obstacle-flow-records.json').exists())
    def test_dense_oracle_has_independent_hand_answer(self):
        self.assertEqual(gaussian([[F(4),F(-1),F(0)],[F(-1),F(3),F(-1)],[F(0),F(-1),F(4)]],[F(1)]*3),[F(2,5),F(3,5),F(2,5)])
if __name__=='__main__':unittest.main()
