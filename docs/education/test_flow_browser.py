"""Independent diagnostic identities and adversarial 3D qualification gates.

Small synthetic tensors here test rejection/measurement logic only. They are
never a native trajectory, renderer screenshot, or publication baseline.
"""
from copy import deepcopy
import json
import math
import os
from pathlib import Path
import struct
import unittest

from flow_browser import (FIXTURE, FIELDS, GRID, METHODS, PARAMETERS, PACKET_FILES,
                          check_metrics, check_native_reference, check_report,
                          check_state, decode_frame, field_counts, field_diagnostics,
                          f32_bits, index, project_domain_corners, source_hashes, validate_receipt)


def rest_fixture():
    fields={name:[0.]*math.prod(counts) for name,counts in field_counts(GRID).items()}
    return {'schema':1,'fixture':FIXTURE,'method':METHODS[0],'grid':deepcopy(GRID),
            'parameters':deepcopy(PARAMETERS),
            'source':{'lower':[.30,.08,.38],'upper':[.50,.28,.58],'tracer_rate':2.,'vertical_acceleration':0.},
            **fields,'bits':{name:[0]*len(values) for name,values in fields.items()},
            'time':0.,'time_bits':'0000000000000000','generation':0,'generation_u64':'0',
            'accepted_steps':0,'max_steps':80,'source_steps':30,'next_source_active':True,
            'requested_dt':.02,'last_accepted_dt':None,'tracer_integral_at_source_cutoff':None,
            'managed_bytes':1119744,
            'metrics':{'max_u_v_w':[0.,0.,0.],'component_kinetic_energy':[0.,0.,0.],
                       'actual_divergence_max':0.,'courant':0.,'tracer_min':0.,'tracer_max':0.,
                       'tracer_integral':0.,'tracer_centroid':None,'tracer_axis_stddev':None,
                       'tracer_outside_source_fraction':None,'tracer_integral_change_since_cutoff':None,
                       'tracer_adjacent_slice_rms':[0.,0.,0.]}}


def receipt_fixture(included=True):
    receipt={'schema':'rheon-flow-browser-v1','included':included,'source_sha256':source_hashes(),
             'packet_hashes':{},'fluid_advances':0,'numerical_states':0,'screenshots':[]}
    if not included:
        return {**receipt,'unavailable_controls_disabled':True,'no_synthetic_state':True}
    receipt.update({'fixture':FIXTURE,'methods':list(METHODS),
        'packet_hashes':{name:'a'*64 for name in PACKET_FILES},
        'native_reference_states':172,'numerical_states':162,'fluid_advances':160,
        'accepted_states_per_method':80,'desktop_scroll_stops':3,'chapter_scroll_stops':2,'desktop_direct_links':5,
        'method_results':[{'method':method,'accepted_states':80,'final_time':1.4,'final_divergence_max':1e-6,
                           'final_component_energy':[.001]*3,'final_tracer_axis_rms':[.01]*3,
                           'source_off_integral_change':-.0001} for method in METHODS],
        'render_oracles':{'affine_indexing_probes':5,'beer_lambert_probes':3,'gpu_alpha_probes':6,
                         'actual_shader_alpha':True,'extreme_coordinate_probes':2,'gpu_paths':['hardware-linear','manual-trilinear']},
        'mobile':[{'width':width,'modes':['text','split','example'],'readable_text':True,
                   'no_horizontal_overflow':True,'mode_preserves_state_and_scroll':True,'compact_controls':True}
                  for width in (390,320)],
        'default_framing':[{'width':w,'max_abs_ndc':.92,'all_eight_corners_visible':True} for w in (1440,390,320)],
        'accepted_mobile':[{'width':width,'accepted_steps':80,'method':METHODS[-1],
            'full_fields_unchanged':True,'camera_works':True,'example_and_split':True,'no_new_requests':True,
            'desktop_context_viewport_emulation':True} for width in (390,320)],
        'compact':[{'width':width,'large_canvas':True,'controls_accessible':True,'no_implicit_advance':True}
                   for width in (1440,390)],
        'worker_roundtrip_ms':{'minimum':20.,'maximum':40.,'mean':30.,
                               'includes_transfer_and_ui_scheduling':True,'real_time_claimed':False}})
    for key in ('full_field_and_bit_comparison','independent_tensor_diagnostics','genuine_three_axis_variation',
                'fresh_host_native_comparison','actual_wasm_worker','every_accepted_worker_state','pause_and_reset',
                'persistent_single_worker','camera_scroll_and_modes_do_not_advance',
                'actual_canvas_changes_with_computation_and_camera'):
        receipt[key]=True
    return receipt


class IndependentTensorTests(unittest.TestCase):
    def test_camera_projection_exposes_clipped_domain_without_page_overflow(self):
        old=project_domain_corners([1.3,.95,1.6],[0,0,0],366,340)
        self.assertGreater(max(abs(v) for p in old for v in p),1)
        for width,height in ((410,400),(366,340),(296,272)):
            framed=project_domain_corners([1.5,1.1,1.85],[0,0,0],width,height)
            self.assertEqual(len(framed),8)
            self.assertLess(max(abs(v) for p in framed for v in p),1)

    def test_anisotropic_staggered_affine_divergence_and_energy(self):
        grid={'counts':[2,3,4],'spacing':[.25,.5,.75],'origin':[0,0,0]}
        state={'grid':grid}
        slopes=(1,-2,3)
        for d,name in enumerate(FIELDS[:3]):
            shape=field_counts(grid)[name]
            state[name]=[slopes[d]*(i,j,k)[d]*grid['spacing'][d]
                         for k in range(shape[2]) for j in range(shape[1]) for i in range(shape[0])]
        state['tracer']=[.25]*24
        observed=field_diagnostics(state,dt=.125,density=2)
        self.assertEqual(observed['actual_divergence_max'],2)
        self.assertEqual(observed['velocity_max'],[.5,3,9])
        self.assertEqual(observed['wall_max'],9)
        self.assertEqual(observed['courant'],.125*(2+6+12))
        expected_energy=math.prod(grid['spacing'])*sum(v*v for name in FIELDS[:3] for v in state[name])
        self.assertEqual(observed['kinetic_energy'],expected_energy)
        self.assertEqual(observed['tracer_integral'],.25*24*math.prod(grid['spacing']))
        self.assertEqual(observed['tracer_centroid'],[.25,.75,1.5])
        self.assertEqual(observed['tracer_axis_rms'],[0,0,0])

    def test_an_extruded_profile_is_not_genuine_three_axis_variation(self):
        grid={'counts':[3,2,4],'spacing':[1,1,1],'origin':[0,0,0]}
        state={'grid':grid,**{name:[0.]*math.prod(shape) for name,shape in field_counts(grid).items()}}
        state['tracer']=[i/4 for k in range(4) for j in range(2) for i in range(3)]
        observed=field_diagnostics(state)
        self.assertEqual(observed['tracer_axis_rms'],[.25,0,0])
        self.assertEqual(observed['tracer_integral'],6.)
        self.assertAlmostEqual(observed['tracer_centroid'][0],13/6)
        self.assertEqual(observed['tracer_centroid'][1:],[1,2])

    def test_rest_and_fixed_source_contract(self):
        check_state(rest_fixture())
        for key,value in [('density',2),('max_courant',2),('tracer_rate',20)]:
            with self.subTest(key=key):
                bad=rest_fixture();bad['parameters'][key]=value
                with self.assertRaisesRegex(RuntimeError,'fixed source'):
                    check_state(bad)
        bad=rest_fixture();bad['source']['lower'][2]=0
        with self.assertRaisesRegex(RuntimeError,'source marker'):
            check_state(bad)

    def test_changed_wall_interior_tracer_and_bits_fail_independently(self):
        cases=[('x',0,.125,'wall-normal'),('x',1,.125,'divergence'),
               ('tracer',0,-.25,'tracer bounds')]
        for name,offset,value,pattern in cases:
            with self.subTest(name=name,offset=offset):
                bad=rest_fixture();bad[name][offset]=value;bad['bits'][name][offset]=f32_bits(value)
                with self.assertRaisesRegex(RuntimeError,pattern): check_state(bad)
        bad=rest_fixture();bad['bits']['z'][123]^=1
        with self.assertRaisesRegex(RuntimeError,'bit mirror'): check_state(bad)
        bad=rest_fixture();bad['tracer'][0]=math.nan
        with self.assertRaisesRegex(RuntimeError,'finite tracer'): check_state(bad)

    def test_wrong_diagnostic_metrics_and_source_cutoff_are_rejected(self):
        for key,value in [('tracer_integral',1),('courant',.1),('tracer_adjacent_slice_rms',[0,0,.1]),
                          ('tracer_integral_change_since_cutoff',0)]:
            with self.subTest(key=key):
                bad=rest_fixture();bad['metrics'][key]=value
                with self.assertRaises(RuntimeError): check_state(bad)
        bad=rest_fixture();bad['next_source_active']=False
        with self.assertRaisesRegex(RuntimeError,'source switches'): check_state(bad)

    def test_report_must_match_fields_and_actual_adaptive_dt(self):
        state=rest_fixture()
        report={'dt':.02,'time':0,'generation':0,'actual_divergence_max':0,'courant':0,
                'kinetic_energy':0,'tracer_integral':0,'pressure':{'iterations':1,'true_residual_l2':0,
                'true_residual_max':0,'predicted_divergence_max':0},
                'forces':{'kinetic_energy_before':0,'kinetic_energy_after':.125,'applied_work':.125}}
        check_report(report,state)
        for key,value in [('dt',.04),('kinetic_energy',1),('courant',.5),('tracer_integral',1)]:
            with self.subTest(key=key):
                altered=deepcopy(report);altered[key]=value
                with self.assertRaises(RuntimeError): check_report(altered,state)
        altered=deepcopy(report);altered['forces']['applied_work']=.25
        with self.assertRaisesRegex(RuntimeError,'energy difference'): check_report(altered,state)
        altered=deepcopy(report);altered['pressure']['predicted_divergence_max']=1e-6
        with self.assertRaisesRegex(RuntimeError,'predicted divergence'): check_report(altered,state)

    def test_binary_decoder_keeps_exact_layout_and_rejects_truncation(self):
        state=rest_fixture()
        # Test endianness and x/y/z/tracer field offsets without asserting these
        # deliberately artificial values are valid fluid states.
        for d,name in enumerate(FIELDS):
            state[name][-1]=(d+1)/8
            state['bits'][name][-1]=f32_bits(state[name][-1])
        blob=b''.join(struct.pack('<f',v) for name in FIELDS for v in state[name])
        metadata={key:value for key,value in state.items() if key not in (*FIELDS,'bits')}
        frame={'state':metadata,'byte_offset':0,'byte_length':len(blob)}
        self.assertEqual(decode_frame(frame,blob),state)
        for bad_blob,bad_offset,bad_length in [(blob[:-1],0,len(blob)),(blob,-4,len(blob)),(blob,1,len(blob)),(blob,0,len(blob)-4)]:
            bad={**frame,'byte_offset':bad_offset,'byte_length':bad_length}
            with self.assertRaisesRegex(RuntimeError,'tensor range'): decode_frame(bad,bad_blob)


class ReceiptGateTests(unittest.TestCase):
    def test_source_only_cannot_claim_computation(self):
        original=receipt_fixture(False);validate_receipt(original,False)
        for key,value in [('fluid_advances',1),('numerical_states',1),('packet_hashes',{'x':'a'*64}),
                          ('unavailable_controls_disabled',False),('no_synthetic_state',False)]:
            with self.subTest(key=key):
                altered=deepcopy(original);altered[key]=value
                with self.assertRaises(ValueError): validate_receipt(altered,False)

    def test_required_native_render_and_mobile_coverage_cannot_be_omitted(self):
        original=receipt_fixture();validate_receipt(original,True)
        for key in ('packet_hashes','source_sha256','native_reference_states','numerical_states','fluid_advances',
                    'full_field_and_bit_comparison','genuine_three_axis_variation','actual_wasm_worker',
                    'every_accepted_worker_state','render_oracles','mobile','compact','method_results'):
            with self.subTest(key=key):
                altered=deepcopy(original);del altered[key]
                with self.assertRaises(ValueError): validate_receipt(altered,True)
        for section,key,value in [('render_oracles','gpu_alpha_probes',0),('worker_roundtrip_ms','real_time_claimed',True)]:
            altered=deepcopy(original);altered[section][key]=value
            with self.assertRaises(ValueError): validate_receipt(altered,True)
        altered=deepcopy(original);altered['method_results'][1]['final_tracer_axis_rms'][2]=0
        with self.assertRaisesRegex(ValueError,'three-axis'): validate_receipt(altered,True)
        altered=deepcopy(original);altered['mobile'][1]['compact_controls']=False
        with self.assertRaisesRegex(ValueError,'phone coverage'): validate_receipt(altered,True)


@unittest.skipUnless(os.environ.get('RHEON_FLOW_NATIVE'), 'requires a fresh native binary exporter directory')
class ActualNativeExportTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path=Path(os.environ['RHEON_FLOW_NATIVE'])
        cls.manifest=json.loads((path/'native-flow-reference.json').read_text())
        cls.blob=(path/'native-flow-fields.bin').read_bytes()

    def test_all_native_trajectories_and_transaction_probes(self):
        self.assertEqual(tuple(check_native_reference(self.manifest,self.blob)),METHODS)

    def test_changed_native_tensor_is_rejected_even_with_valid_json(self):
        blob=bytearray(self.blob)
        blob[:4]=struct.pack('<f',.25)
        with self.assertRaisesRegex(RuntimeError,'wall-normal'):
            check_native_reference(self.manifest,bytes(blob))

    def test_changed_binary_offsets_or_missing_trajectory_refuse(self):
        bad=deepcopy(self.manifest);bad['runs'][0]['frames'].pop()
        with self.assertRaisesRegex(RuntimeError,'all 81'): check_native_reference(bad,self.blob)
        bad=deepcopy(self.manifest);bad['runs'][0]['frames'][0]['byte_offset']=len(self.blob)
        with self.assertRaisesRegex(RuntimeError,'tensor range'): check_native_reference(bad,self.blob)


if __name__=='__main__':
    unittest.main()
