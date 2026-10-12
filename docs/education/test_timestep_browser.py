"""Independent oracle and adversarial checks for the native timestep lesson."""
from copy import deepcopy
import os
from pathlib import Path
import shutil
import unittest

from timestep_browser import (FIXTURE, FIELDS, GRID, METHODS, check_native_reference,
                              check_face_table_layout, check_outcome, check_report, check_state, cycle_oracle,
                              field_diagnostics, float_bits, PACKET_FILES, qualify,
                              source_hashes, time_bits, validate_receipt, wait_ready)


def state_fixture(method=METHODS[0], accepted=True, generation=1):
    """Synthetic check input only; never a native packet or publication record."""
    reference = cycle_oracle(accepted)
    fields = {name: [float(value) for value in reference[name]] for name in FIELDS}
    time = .25 if accepted else 0.0
    return {'schema': 1, 'fixture': FIXTURE, 'method': method, 'grid': deepcopy(GRID),
            **fields, 'bits': {name: [float_bits(value) for value in fields[name]] for name in FIELDS},
            'time': time, 'time_bits': time_bits(time), 'generation': generation,
            'generation_u64': str(generation), 'accepted_since_reset': accepted}


def outcome_fixture(state, code='accepted'):
    report = None
    if code == 'accepted':
        derived = field_diagnostics(state)
        report = {'dt': .25, 'time': state['time'], 'generation': state['generation'],
                  **{key: derived[key] for key in ('actual_divergence_max', 'courant',
                                                   'tracer_integral', 'kinetic_energy')},
                  'forces': {'kinetic_energy_before': 0, 'kinetic_energy_after': .125, 'applied_work': .125},
                  'pressure': {'iterations': 3, 'true_residual_l2': 0,
                               'true_residual_max': 0, 'predicted_divergence_max': 0}}
    return {'status': 'accepted' if code == 'accepted' else 'rejected', 'code': code,
            'detail': None if code == 'accepted' else 'Unit-test adversarial input; not a native solver result',
            'state': deepcopy(state), 'report': report}


def packet_fixture():
    runs = []
    for method in METHODS:
        baseline = state_fixture(method, False, 0)
        accepted = state_fixture(method, True, 1)
        runs.append({'method': method, 'baseline': baseline,
                     'invalid_dt': outcome_fixture(baseline, 'invalid_time_step'),
                     'cancelled_rest': outcome_fixture(baseline, 'cancelled_before_commit'),
                     'accepted': outcome_fixture(accepted),
                     'cancelled_nonzero': outcome_fixture(accepted, 'cancelled_before_commit'),
                     'second_accept': outcome_fixture(accepted, 'reset_required'),
                     'reset': state_fixture(method, False, 2),
                     'after_reset': outcome_fixture(state_fixture(method, True, 3))})
    return {'schema': 1, 'fixture': FIXTURE, 'dt': .25, 'runs': runs}


class CycleOracleTests(unittest.TestCase):
    def test_phone_table_checks_inner_scroll_and_every_reference_column(self):
        layout = {'headers': ['Face', 'Accepted', 'Reference'], 'scrollWidth': 250, 'clientWidth': 250,
                  'scrollLeft': 0, 'left': 35, 'right': 285, 'top': 200, 'bottom': 330,
                  'headerBottom': 58, 'viewportWidth': 320, 'viewportHeight': 844,
                  'cells': [{'left': 35+i*250/3, 'right': 35+(i+1)*250/3,
                             'scrollWidth': 82, 'clientWidth': 82, 'visible': True}
                            for _ in range(5) for i in range(3)]}
        check_face_table_layout(layout)
        for change in ('wide_table', 'clipped_reference', 'hidden_column', 'clipped_value'):
            with self.subTest(change=change):
                damaged = deepcopy(layout)
                if change == 'wide_table':
                    damaged['scrollWidth'] = 390
                elif change == 'clipped_reference':
                    damaged['cells'][2]['right'] = 425
                elif change == 'hidden_column':
                    damaged['cells'][2]['visible'] = False
                else:
                    damaged['cells'][5]['scrollWidth'] = 150
                with self.assertRaisesRegex(RuntimeError, 'native face table'):
                    check_face_table_layout(damaged)

    def test_exact_circulation_force_work_and_post_projection_energy(self):
        expected = cycle_oracle()
        self.assertEqual(expected['x'], [0, .125, 0, 0, -.125, 0])
        self.assertEqual(expected['y'], [0, 0, -.125, .125, 0, 0])
        self.assertEqual(expected['z'], [0]*8)
        self.assertEqual(expected['tracer'], [0]*4)
        self.assertEqual(expected['divergence'], [0]*4)
        self.assertEqual(expected['force_work'], .125)
        self.assertEqual(expected['kinetic_energy'], .03125)
        self.assertEqual(expected['courant'], .0625)
        for method in METHODS:
            for accepted, generation in ((False, 0), (True, 1), (False, 2), (True, 3)):
                check_state(state_fixture(method, accepted, generation), method, accepted, generation)

    def test_complete_transaction_contract(self):
        packet = packet_fixture()
        self.assertEqual(tuple(check_native_reference(packet)), METHODS)

    def test_changed_face_even_with_coherent_diagnostics_is_rejected(self):
        state = state_fixture()
        state['x'][1] = .25
        state['bits']['x'][1] = float_bits(.25)
        # Recompute the diagnostics from the bad field: the independent cycle
        # oracle must still reject a coherent but numerically wrong report.
        outcome = outcome_fixture(state)
        with self.assertRaisesRegex(RuntimeError, 'cycle x'):
            check_outcome(outcome, METHODS[0], True, 1)

    def test_walls_tracer_and_binary32_bit_mirrors_are_checked(self):
        for name, offset, value in [('x', 0, .125), ('z', 4, .125), ('tracer', 0, .125)]:
            with self.subTest(name=name):
                state = state_fixture()
                state[name][offset] = value
                state['bits'][name][offset] = float_bits(value)
                with self.assertRaisesRegex(RuntimeError, 'cycle '+name):
                    check_state(state, METHODS[0], True, 1)
        state = state_fixture()
        state['bits']['y'][2] ^= 1
        with self.assertRaisesRegex(RuntimeError, 'bit mirror'):
            check_state(state, METHODS[0], True, 1)
        state = state_fixture()
        state['x'][1] += 1e-10
        with self.assertRaisesRegex(RuntimeError, 'exactly binary32'):
            check_state(state, METHODS[0], True, 1)

    def test_false_diagnostics_and_pressure_acceptance_are_rejected(self):
        state = state_fixture()
        for key, value in [('actual_divergence_max', .01), ('courant', .125),
                           ('kinetic_energy', .125), ('dt', .5), ('tracer_integral', 1),
                           ('time', .5), ('generation', 0)]:
            with self.subTest(key=key):
                report = outcome_fixture(state)['report']
                report[key] = value
                with self.assertRaisesRegex(RuntimeError, 'reported '+key):
                    check_report(report, state)
        for key, value in [('iterations', 0), ('true_residual_l2', float('nan')),
                           ('true_residual_max', 1), ('predicted_divergence_max', 1)]:
            with self.subTest(key=key):
                report = outcome_fixture(state)['report']
                report['pressure'][key] = value
                with self.assertRaisesRegex(RuntimeError, 'pressure'):
                    check_report(report, state)
        report = outcome_fixture(state)['report']
        report['forces']['applied_work'] = .03125
        with self.assertRaisesRegex(RuntimeError, 'force applied_work'):
            check_report(report, state)

    def test_missing_rejection_and_hidden_state_change_are_rejected(self):
        packet = packet_fixture()
        del packet['runs'][0]['cancelled_rest']
        with self.assertRaisesRegex(RuntimeError, 'missing native step outcome'):
            check_native_reference(packet)
        packet = packet_fixture()
        # IEEE negative zero passes the value oracle; transactional preservation
        # must additionally detect even this otherwise invisible bit change.
        changed = packet['runs'][0]['cancelled_nonzero']['state']
        changed['z'][0] = -0.0
        changed['bits']['z'][0] = float_bits(-0.0)
        with self.assertRaisesRegex(RuntimeError, 'changed accepted state or bits'):
            check_native_reference(packet)

    def test_rejection_cannot_publish_report_or_claim_acceptance(self):
        for accepted, generation in ((False, 0), (True, 1)):
            baseline = state_fixture(accepted=accepted, generation=generation)
            outcome = outcome_fixture(baseline, 'cancelled_before_commit')
            outcome['report'] = outcome_fixture(state_fixture())['report']
            with self.assertRaisesRegex(RuntimeError, 'no accepted report'):
                check_outcome(outcome, METHODS[0], accepted, generation, 'cancelled_before_commit', baseline)
            outcome = outcome_fixture(baseline, 'accepted')
            with self.assertRaisesRegex(RuntimeError, 'outcome code'):
                check_outcome(outcome, METHODS[0], accepted, generation, 'cancelled_before_commit', baseline)

    def test_reset_and_retry_generation_are_checked(self):
        for name in ('reset', 'after_reset'):
            packet = packet_fixture()
            state = packet['runs'][0][name]
            if name == 'after_reset':
                state = state['state']
            state['generation'] -= 1
            state['generation_u64'] = str(state['generation'])
            with self.assertRaisesRegex(RuntimeError, 'generation identity'):
                check_native_reference(packet)

    def test_unavailable_receipt_cannot_claim_execution_or_skip_controls(self):
        original = {'schema': 'rheon-timestep-browser-v1', 'included': False,
                    'fluid_advances': 0, 'rejected_candidates': 0, 'numerical_states': 0,
                    'unavailable_controls_disabled': True, 'no_synthetic_state': True,
                    'source_sha256': source_hashes(), 'packet_hashes': {}, 'screenshots': []}
        validate_receipt(original, False)
        for key, value in [('fluid_advances', 1), ('numerical_states', 1), ('included', True),
                           ('unavailable_controls_disabled', False), ('no_synthetic_state', False)]:
            with self.subTest(key=key):
                altered = deepcopy(original)
                altered[key] = value
                with self.assertRaises(ValueError):
                    validate_receipt(altered, False)

    def test_included_receipt_requires_numerical_transaction_and_mobile_coverage(self):
        original = {'schema': 'rheon-timestep-browser-v1', 'included': True, 'fixture': FIXTURE,
                    'source_sha256': source_hashes(), 'packet_hashes': {name: '0'*64 for name in PACKET_FILES},
                    'methods': list(METHODS), 'native_reference_runs': 2, 'numerical_states': 72,
                    'fluid_advances': 7, 'rejected_candidates': 6, 'native_resets': 2,
                    'desktop_scroll_stops': 3, 'chapter_scroll_stops': 2, 'desktop_direct_links': 5,
                    'full_field_and_bit_comparison': True, 'independent_cycle_projection': True,
                    'independent_cell_divergence': True, 'force_work_and_kinetic_energy': True,
                    'fresh_host_native_comparison': True, 'actual_wasm_worker': True,
                    'fixed_scale_velocity_arrows': True, 'rejection_preserves_accepted_bits': True,
                    'camera_and_reading_do_not_advance': True, 'manual_commands_pause_follow': True,
                    'mobile': [{'width': width, 'modes': ['text', 'split', 'example'], 'readable_text': True,
                                'no_horizontal_overflow': True, 'mode_preserves_lesson_and_scroll': True,
                                'layout_and_scroll_do_not_advance': True, 'rejection_then_acceptance': True,
                                'chapter_direct_link': True, 'all_face_table_columns_visible': True}
                               for width in (390, 320)], 'screenshots': []}
        validate_receipt(original, True)
        for key in ('source_sha256', 'packet_hashes', 'methods', 'actual_wasm_worker', 'rejection_preserves_accepted_bits', 'numerical_states',
                    'fresh_host_native_comparison', 'camera_and_reading_do_not_advance', 'mobile'):
            with self.subTest(missing=key):
                altered = deepcopy(original)
                del altered[key]
                with self.assertRaises(ValueError):
                    validate_receipt(altered, True)
        for key, value in [('methods', [METHODS[0]]), ('fluid_advances', 0), ('rejected_candidates', 0)]:
            with self.subTest(changed=key):
                altered = deepcopy(original)
                altered[key] = value
                with self.assertRaises(ValueError):
                    validate_receipt(altered, True)
        altered = deepcopy(original)
        altered['mobile'][1]['rejection_then_acceptance'] = False
        with self.assertRaisesRegex(ValueError, 'native phone qualification check'):
            validate_receipt(altered, True)
        altered = deepcopy(original)
        del altered['mobile'][1]['all_face_table_columns_visible']
        with self.assertRaisesRegex(ValueError, 'native phone qualification check'):
            validate_receipt(altered, True)


@unittest.skipUnless(os.environ.get('RHEON_TIMESTEP_EDITION'), 'requires an edition with a compiled solver packet')
class ActualBrowserRejectionTests(unittest.TestCase):
    def run_altered(self, alteration):
        from playwright.sync_api import sync_playwright
        from verify_browser import serve_site
        edition = Path(os.environ['RHEON_TIMESTEP_EDITION'])
        with serve_site(edition) as url, sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=os.environ.get('RHEON_CHROMIUM_EXECUTABLE') or shutil.which('chromium'), headless=True,
                                                  chromium_sandbox=True, args=['--use-gl=angle', '--use-angle=swiftshader',
                                                        '--enable-unsafe-swiftshader'])
            page = browser.new_page(viewport={'width': 1440, 'height': 1050})
            mobile = browser.new_page(viewport={'width': 390, 'height': 844}, is_mobile=True, has_touch=True)
            errors = []
            for target in (page, mobile):
                target.on('pageerror', lambda error: errors.append(str(error)))
            if alteration == 'missing_wasm':
                page.context.route('**/solver/rheon_solver_bg.wasm', lambda route: route.fulfill(status=404, body='missing'))
            if alteration == 'unresponsive_step':
                page.add_init_script('''document.addEventListener('click', event => {
                    if(event.target.id === 'timestep-step') event.stopImmediatePropagation();
                }, true);''')
            if alteration == 'delayed_native_response':
                page.add_init_script('''const NativeWorker = window.Worker;
                    window.Worker = class extends NativeWorker {
                      set onmessage(handler) {
                        super.onmessage = event => setTimeout(() => handler.call(this, event), 750);
                      }
                    };''')
            def load(target, path):
                # The missing-WASM probe deliberately prevents normal network
                # completion. Wait for the actual UI/native error or readiness
                # in qualify(), rather than making network idleness the oracle.
                target.goto(url+'/'+path, wait_until='domcontentloaded')
            def check_errors():
                if errors:
                    raise RuntimeError('Timestep Chromium errors '+str(errors))
            try:
                if alteration == 'delayed_native_response':
                    load(page, 'timestep-lab.html')
                    observed = wait_ready(page)
                    self.assertEqual(observed['reference']['runtime'], 'wasm-worker')
                    self.assertEqual(observed['reference']['state']['generation'], 0)
                    self.assertEqual(observed['reference']['requestCount'], 1)
                    self.assertFalse(observed['reference']['busy'])
                    check_errors()
                    return
                qualify(page, mobile, load, check_errors)
            finally:
                browser.close()

    def test_missing_actual_wasm_asset_refuses(self):
        with self.assertRaisesRegex(RuntimeError, 'worker failed|Chromium errors'):
            self.run_altered('missing_wasm')

    def test_unresponsive_native_step_button_refuses(self):
        with self.assertRaisesRegex(RuntimeError, 'one worker request per explicit command'):
            self.run_altered('unresponsive_step')

    def test_delayed_native_response_waits_for_real_state(self):
        self.run_altered('delayed_native_response')


if __name__ == '__main__':
    unittest.main()
