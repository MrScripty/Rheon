"""Execute the actual pure JavaScript display algorithms against exact oracles."""
from copy import deepcopy
import json
from pathlib import Path
import shutil
import subprocess
import unittest

from flow_field_oracles import (JS_PROBE,check_contract,check_curves,check_light,
                                check_results,check_sampling,fixture_cases)


class ActualJavaScriptFieldTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        node=shutil.which('node')
        if node is None:raise RuntimeError('Node is required for actual flow field qualification')
        cls.cases=fixture_cases()
        script='import fs from "node:fs"; const probe=('+JS_PROBE+'); console.log(JSON.stringify(await probe(JSON.parse(fs.readFileSync(0,"utf8")))));'
        payload={'url':(Path(__file__).with_name('flow_field.js')).as_uri(),'cases':cls.cases}
        run=subprocess.run([node,'--input-type=module','-e',script],input=json.dumps(payload),text=True,capture_output=True,check=True,timeout=30)
        cls.result=json.loads(run.stdout)

    def test_staggered_affine_donor_singleton_and_clamped_samples(self):
        self.assertEqual(check_sampling(self.cases,self.result),14)

    def test_optical_depth_integrates_physical_columns(self):
        self.assertEqual(check_light(self.cases,self.result),96)

    def test_frozen_streamline_geometry_speeds_termination_and_convergence(self):
        self.assertEqual(check_curves(self.cases,self.result)['branches'],20)

    def test_fixed_scales_input_immutability_and_invalid_inputs(self):
        check_contract(self.cases,self.result)

    def test_oracle_rejects_wrong_offset_depth_and_fictional_speed(self):
        mutations=[lambda r:r['samples'][0]['direct'][0].__setitem__(0,999),
                   lambda r:r['light'][0]['values'].__setitem__(0,.125),
                   lambda r:r['curves'][0]['forward']['speeds'].__setitem__(0,1.),
                   lambda r:r.__setitem__('inputUnchanged',False)]
        for mutate in mutations:
            result=deepcopy(self.result);mutate(result)
            with self.assertRaises(RuntimeError):check_results(self.cases,result)


if __name__=='__main__':unittest.main()
