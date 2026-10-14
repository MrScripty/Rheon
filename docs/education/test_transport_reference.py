"""Bounded independent checks of the teaching sampler and transport kernel."""
from copy import deepcopy
from fractions import Fraction as F
import json
from pathlib import Path
import subprocess
import unittest

from transport_browser import (FIXTURES, check_reference_lines, check_sampler, check_transport,
                               corrected_step, linear_step, sampling_expectation,
                               transport_expectation, vector)

HERE = Path(__file__).resolve().parent


def floats(value):
    if isinstance(value,F):
        return float(value)
    if isinstance(value,dict):
        return {key:floats(item) for key,item in value.items()}
    if isinstance(value,(list,tuple)):
        return [floats(item) for item in value]
    return value


def kernel(cases):
    """Execute the real pure ES module; no DOM, copied JS, or solver invocation."""
    script = '''
import {readFileSync} from 'node:fs';
const reference = await import(process.argv[1]);
const cases = JSON.parse(readFileSync(0,'utf8'));
const result = cases.map(({operation,args}) => {
  const before = JSON.stringify(args);
  const value = reference[operation](...args);
  if (JSON.stringify(args) !== before) throw Error('Reference kernel mutated its inputs');
  return value;
});
process.stdout.write(JSON.stringify(result));
'''
    result = subprocess.run(['node','--input-type=module','-e',script,(HERE/'transport_reference.js').as_uri()],
                            input=json.dumps(floats(cases)),text=True,capture_output=True,check=True)
    return json.loads(result.stdout)


class ExactContracts(unittest.TestCase):
    def test_exact_overshoot_reversion_and_asymmetric_total(self):
        symmetric = corrected_step(FIXTURES['symmetric'],F(1,2))
        self.assertEqual(min(symmetric['candidate']),-F(1,8))
        self.assertEqual(max(symmetric['candidate']),F(9,8))
        self.assertEqual(symmetric['fallback'],[False,True,False,True,False,False])
        self.assertEqual(symmetric['limited'],[0,0,F(5,8),1,F(3,8),0])
        asymmetric = corrected_step(FIXTURES['asymmetric'],F(1,4))
        self.assertEqual(asymmetric['fallback'],[True,False,False,False,False,False])
        self.assertEqual(min(asymmetric['candidate']),-F(3,160))
        self.assertEqual(sum(FIXTURES['asymmetric']),F(8,5))
        self.assertEqual(sum(asymmetric['limited']),F(259,160))
        self.assertEqual(sum(asymmetric['limited'])-sum(FIXTURES['asymmetric']),F(3,160))
        self.assertTrue(all(0<=value<=1 for value in asymmetric['limited']))

    def test_wrong_affine_offset_remains_bounded_but_biased(self):
        wrong = sampling_expectation('x',True)
        self.assertEqual(wrong['error'],-F(7,100))
        self.assertEqual(wrong['weightSum'],1)
        self.assertTrue(wrong['minimum']<=wrong['value']<=wrong['maximum'])
        for component in ('cell','x','y','z'):
            self.assertEqual(sampling_expectation(component)['error'],0)

    def test_false_conservation_changed_samples_and_counts_are_rejected(self):
        original = floats(transport_expectation('asymmetric',64,.25,1))
        check_transport(original,'asymmetric',64,.25,1)
        false_total = deepcopy(original)
        false_total['limitedMetrics']['sum'] = 1.6
        with self.assertRaisesRegex(RuntimeError,'limitedMetrics sum'):
            check_transport(false_total,'asymmetric',64,.25,1)
        changed = floats(transport_expectation('symmetric',64,.5,1))
        changed['limited'][2] = .5
        with self.assertRaisesRegex(RuntimeError,'transport limited'):
            check_transport(changed,'symmetric',64,.5,1)
        false_count = deepcopy(original)
        false_count['lastFallback'] = 0
        with self.assertRaisesRegex(RuntimeError,'fallback count'):
            check_transport(false_count,'asymmetric',64,.25,1)

    def test_wrong_donor_coordinates_and_plot_scales_are_rejected(self):
        sampler = floats(sampling_expectation())
        sampler['donors'][0]['position'][0] += .1
        with self.assertRaisesRegex(RuntimeError,'sampler donor'):
            check_sampler(sampler)
        reference = transport_expectation('asymmetric',64,.25,1)
        lines = []
        for series,z in zip(('linear','raw','limited'),(-.45,0,.45)):
            values = reference[series]
            points = [[-.8+1.6*i/len(values),.45*float(value),z] for i,value in enumerate(values)]
            points.append([.8,.45*float(values[0]),z])
            lines.append({'series':series,'points':points})
        check_reference_lines(lines,reference)
        lines[1]['points'][2][1] *= 1.1
        with self.assertRaisesRegex(RuntimeError,'rendered raw'):
            check_reference_lines(lines,reference)


class ActualKernel(unittest.TestCase):
    def test_affine_offsets_and_displaced_queries(self):
        cases,parameters = [],[]
        for component in ('cell','x','y','z'):
            for wrong in (False,True):
                for x in (.47,.67,.87):
                    args = [component,wrong,[x,.84,1.12]]
                    cases.append({'operation':'sampleAffine','args':args})
                    parameters.append(args)
        results = kernel(cases)
        self.assertEqual(len(results),24)
        for actual,args in zip(results,parameters):
            with self.subTest(args=args):
                check_sampler(actual,*args)

    def test_periodic_forward_backward_and_exact_limiter_decisions(self):
        cases,expected = [],[]
        # This additional near-peak stencil separates reversion from clamping:
        # its forward value lies strictly inside the old donor interval.
        for values in [*FIXTURES.values(),[F(0),F(9,10),F(1),F(0),F(0),F(0)]]:
            for courant in (F(-1,2),F(0),F(1,4),F(1,2),F(1)):
                cases += [{'operation':'sl','args':[values,courant]},
                          {'operation':'corrected','args':[values,courant]}]
                expected += [linear_step(values,courant),corrected_step(values,courant)]
        for actual,reference in zip(kernel(cases),expected):
            if isinstance(reference,list):
                vector(actual,reference,'kernel periodic linear')
            else:
                for key in ('forward','candidate','limited'):
                    vector(actual[key],reference[key],'kernel '+key)
                self.assertEqual(actual['fallback'],sum(reference['fallback']))

    def test_evolution_arrays_diagnostics_and_executed_refinement(self):
        cases = [{'profile':profile,'n':64,'courant':.5,'steps':1}
                 for profile in ('sine','top-hat','symmetric','asymmetric')]
        cases += [{'profile':'asymmetric','n':256,'courant':.25,'steps':1},
                  {'profile':'top-hat','n':64,'courant':.5,'steps':128},
                  {'profile':'sine','n':32,'courant':.5,'steps':0}]
        cases += [{'profile':'sine','n':n,'courant':.5,'steps':2*n} for n in (32,64,128,256)]
        results = kernel([{'operation':'evolve','args':[case]} for case in cases])
        for case,actual in zip(cases,results):
            with self.subTest(case=case):
                check_transport(actual,**case)
        # These rounded values are the chapter/companion's executed one-period
        # refinement, not a claimed asymptotic convergence order.
        historical_linear = (.187922,.101090,.052478,.026743)
        historical_limited = (.037630,.009022,.003211,.001022)
        for actual,linear,limited in zip(results[-4:],historical_linear,historical_limited):
            self.assertAlmostEqual(actual['linearMetrics']['rms'],linear,delta=5e-7)
            self.assertAlmostEqual(actual['limitedMetrics']['rms'],limited,delta=5e-7)
            self.assertLess(actual['limitedMetrics']['rms'],actual['linearMetrics']['rms'])


if __name__ == '__main__':
    unittest.main()
