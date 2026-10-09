"""Independent snapshot checks and real Chromium rejection probes."""
from copy import deepcopy
from pathlib import Path
import json
import os
import shutil
import tempfile
import unittest

from aligned_strain_browser import check_snapshot, qualify
from aligned_strain_packet import browser_expectation, ROOT
from aligned_strain_oracle import Refusal
from test_aligned_strain_packet import algebra_fixture


def snapshot_fixture(packet,preset='corner',amplitude=1,secondary=1,mu=1):
    expected=browser_expectation(packet,preset,amplitude,secondary,mu)
    snapshot={key:[float(x) for x in value] if isinstance(value,list) else float(value)
              for key,value in expected.items()}
    snapshot.update(selectedRow=0,selectedRowData=packet['rows'][0],controls={
        'preset':preset,'amplitude':amplitude,'secondary':secondary,'mu':mu,
        'cornerSelection':packet['defaultCornerSelection']})
    return snapshot


class SnapshotRejectionTest(unittest.TestCase):
    def test_independent_arrays_and_metrics(self):
        packet=algebra_fixture()
        for preset in packet['presets']:
            for amplitude,mu in ((1,1),(1e-6,2),(-2,0)):
                check_snapshot(packet,snapshot_fixture(packet,preset,amplitude,-amplitude,mu))

    def test_coherent_report_and_snapshot_corruptions_refuse(self):
        packet=algebra_fixture();original=snapshot_fixture(packet)
        changes=(lambda s:s['strains'].__setitem__(0,999),
                 lambda s:s['field'].__setitem__(0,999),
                 lambda s:s['action'].__setitem__(0,999),
                 lambda s:s['forces'].__setitem__(0,999),
                 lambda s:s.__setitem__('D',-1),
                 lambda s:s.__setitem__('B',18),
                 lambda s:s.__setitem__('selectedRow',-1),
                 lambda s:s.__setitem__('selectedRowData',packet['rows'][1]),
                 lambda s:s['controls'].__setitem__('mu',3),
                 lambda s:s['action'].__setitem__(0,float('nan')))
        for change in changes:
            with self.subTest(change=change):
                snapshot=deepcopy(original);change(snapshot)
                with self.assertRaises(Refusal):check_snapshot(packet,snapshot)

    def test_tiny_sign_reversal_has_no_absolute_tolerance_floor(self):
        packet=algebra_fixture();snapshot=snapshot_fixture(packet,amplitude=1e-6,secondary=1e-6)
        i=next(i for i,x in enumerate(snapshot['forces']) if x)
        snapshot['forces'][i]*=-1
        with self.assertRaisesRegex(Refusal,'independent mismatch'):
            check_snapshot(packet,snapshot)


@unittest.skipUnless(os.environ.get('RHEON_EDITION'),'qualification supplies a built edition')
class ChromiumRejectionTest(unittest.TestCase):
    def run_qualifier(self,alter=None):
        from playwright.sync_api import sync_playwright
        from verify_browser import serve_site
        edition=Path(os.environ['RHEON_EDITION'])
        with serve_site(edition) as url,sync_playwright() as p:
            browser=p.chromium.launch(executable_path=shutil.which('chromium'),headless=True,args=['--no-sandbox'])
            page=browser.new_page(viewport={'width':1440,'height':1050})
            mobile=browser.new_page(viewport={'width':390,'height':844},is_mobile=True,has_touch=True)
            errors=[]
            page.on('pageerror',lambda error:errors.append(str(error)))
            mobile.on('pageerror',lambda error:errors.append(str(error)))
            def load(target,path):
                target.goto(url+'/'+path);target.wait_for_load_state('networkidle')
                if alter and target==page:
                    target.wait_for_function('window.alignedStrainLab!==undefined');alter(target)
            def check_errors():
                if errors:raise RuntimeError('Chromium errors '+str(errors))
            try:
                return qualify(page,mobile,load,check_errors,edition/'aligned-strain-packet')
            finally:browser.close()

    def test_actual_chromium_independent_comparisons(self):
        result=self.run_qualifier()
        self.assertGreaterEqual(result['independent_rational_comparisons'],8)
        self.assertTrue(result['mobile_no_overflow'])

    def test_unresponsive_actual_controls_refuse(self):
        def alter(page):
            page.evaluate('document.querySelector("#strain-amplitude").addEventListener("input",e=>e.stopImmediatePropagation(),true)')
        with self.assertRaisesRegex(Refusal,'unresponsive'):
            self.run_qualifier(alter)

    def test_changed_actual_force_scatter_refuses(self):
        def alter(page):
            page.evaluate('''() => {
                const original=window.alignedStrainLab.snapshot;
                window.alignedStrainLab.snapshot=()=>{const state=original();state.forces[0]+=1;return state;};
            }''')
        with self.assertRaisesRegex(Refusal,'independent mismatch'):
            self.run_qualifier(alter)


if __name__ == '__main__':
    unittest.main()
