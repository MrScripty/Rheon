"""Packet and browser-input rejection gates, including optimized Python."""
from copy import deepcopy
from fractions import Fraction as F
from pathlib import Path
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from aligned_strain_packet import (ROOT, SCHEMA, basis_and_corners, browser_expectation,
                                  encoded, external_new_directory, packet_data,
                                  sha, source_core, validate_packet, publish, verify_published)
from aligned_strain_oracle import Dump, Geometry, Refusal, reference


def algebra_fixture():
    """Synthetic export unit fixture only; native evidence comes from packet env."""
    g = Geometry((3,3,3),(F(1),)*3,(F(0),)*3,(1,1,1),(2,2,2))
    faces,rows,k,b = reference(g)
    return packet_data(Dump(g,faces,rows,k,[F(0)]*len(faces),[F(0)]*len(faces),(F(0),F(0),F(0),b)),{})


class ControlBasisTest(unittest.TestCase):
    def test_specimen_preserves_rows_mass_and_reflected_corner_matrix(self):
        data = algebra_fixture()
        self.assertEqual((len(data['active']),len(data['rows'])),(48,210))
        self.assertEqual(sum(not r['terms'] for r in data['rows']),6)
        self.assertTrue(all(f['mass']==1 for f in data['active']))
        self.assertEqual(len(data['cornerEdges']),12)
        for edge in data['cornerEdges']:
            sign = edge['solidSigns'][0]*edge['solidSigns'][1]
            self.assertEqual(edge['unitPatchMatrix'],[[2,sign],[sign,2]])
            self.assertEqual(len(edge['rowIds']),3)
            result=browser_expectation(data,corner_selection=edge['id'])
            local_loss=sum(F(data['rows'][i]['weight'])*result['strains'][i]**2 for i in edge['rowIds'])
            self.assertEqual(local_loss,4+2*sign)

    def test_control_algebra_and_local_rotation_scope(self):
        data = algebra_fixture()
        for preset,expected in [('corner',(8,10)),('normal',(48,10)),('shear',(48,56)),('rotation',(4,2))]:
            result = browser_expectation(data,preset)
            self.assertEqual((result['normalD'],result['shearD']),expected)
            self.assertEqual(result['D']+result['work'],0)
            self.assertEqual(result['B'],17)
        local = browser_expectation(data,'rotation')
        for row in data['presets']['rotation']['focusRowIds']:
            self.assertEqual(local['strains'][row],0)
        self.assertGreater(local['D'],0)

    def test_small_large_and_zero_viscosity_controls(self):
        data = algebra_fixture()
        for value in (1e-6,-2,2):
            result = browser_expectation(data,'corner',value,-value,2)
            self.assertEqual(result['D']+result['work'],0)
        result = browser_expectation(data,'shear',2,-2,0)
        self.assertEqual((result['D'],result['normalD'],result['shearD'],result['work']), (0,0,0,0))
        self.assertTrue(all(x==0 for x in result['forces']))
        self.assertTrue(any(x for x in result['action']))

    def test_control_bounds_and_changed_basis_refuse(self):
        data = algebra_fixture()
        for kwargs in ({'mu':-1},{'mu':3},{'amplitude':3},{'secondary':float('nan')},{'preset':'unqualified'}):
            with self.subTest(kwargs=kwargs),self.assertRaises(Refusal):
                browser_expectation(data,**kwargs)
        data['presets']['corner']['basis'][0][0]=999
        with self.assertRaisesRegex(Refusal,'independent reference'):
            browser_expectation(data)

    def test_output_reuse_and_git_destination_refuse(self):
        with tempfile.TemporaryDirectory() as temp:
            existing = Path(temp)/'existing';existing.mkdir()
            with self.assertRaisesRegex(Refusal,'fresh'):
                external_new_directory(ROOT,existing)
        with self.assertRaisesRegex(Refusal,'outside'):
            external_new_directory(ROOT,ROOT/'packet-test')

    def test_source_only_publication_states_absence(self):
        with tempfile.TemporaryDirectory() as temp:
            self.assertIsNone(publish(ROOT,Path(temp)))
            self.assertIsNone(verify_published(ROOT,Path(temp)))
            self.assertFalse((Path(temp)/'aligned-strain-packet').exists())

    def test_native_core_pinned_to_immutable_primary(self):
        self.assertIn('src/aligned_strain.rs',source_core(ROOT))

    def test_dense3d_extension_binds_both_files_and_rejects_changed_sources(self):
        sources = source_core(ROOT)
        for name in ('src/lib.rs', 'src/dense3d_sequence.rs'):
            self.assertEqual(sources[name], sha(ROOT/name))
        for changed in ('src/lib.rs', 'src/dense3d_sequence.rs', 'src/aligned_strain.rs'):
            def tampered(path):
                return '0'*64 if Path(path) == ROOT/changed else sha(path)
            with self.subTest(changed=changed), patch('aligned_strain_packet.sha', side_effect=tampered):
                with self.assertRaisesRegex(Refusal, 'changed qualified native source'):
                    source_core(ROOT)


@unittest.skipUnless(os.environ.get('RHEON_ALIGNED_PACKET'),'qualification supplies a real native packet')
class NativePacketRejectionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.packet = Path(self.temp.name)/'packet'
        shutil.copytree(Path(os.environ['RHEON_ALIGNED_PACKET']),self.packet)

    def write_data(self,data):
        (self.packet/'data.json').write_text(encoded(data))
        receipt=json.loads((self.packet/'qualification.json').read_text())
        receipt['data_sha256']=sha(self.packet/'data.json')
        (self.packet/'qualification.json').write_text(encoded(receipt))

    def test_real_packet_and_browser_inputs_pass(self):
        receipt=validate_packet(ROOT,self.packet)
        self.assertEqual((receipt['oracle_summary']['active_faces'],receipt['oracle_summary']['rows'],
                          receipt['oracle_summary']['zero_rows'],receipt['oracle_summary']['B_exact']), (48,210,6,'17'))
        data=json.loads((self.packet/'data.json').read_text())
        self.assertEqual(browser_expectation(data)['D'],18)

    def test_published_packet_and_renderer_bindings(self):
        output=Path(self.temp.name)/'published';output.mkdir()
        data=publish(ROOT,output,self.packet)
        for name in ('aligned_strain.js','aligned_strain.css'):
            shutil.copyfile(ROOT/'docs/education'/name,output/name)
        self.assertEqual(verify_published(ROOT,output),data)
        with (output/'aligned_strain.js').open('a') as renderer:renderer.write('\nchanged')
        with self.assertRaisesRegex(Refusal,'renderer binding'):
            verify_published(ROOT,output)

    def test_rehashed_crafted_payloads_refuse(self):
        original=json.loads((self.packet/'data.json').read_text())
        mutations=(
            lambda d:d['rows'][0]['terms'][0].__setitem__('coefficient',-d['rows'][0]['terms'][0]['coefficient']),
            lambda d:d['rows'].pop(next(i for i,r in enumerate(d['rows']) if not r['terms'])),
            lambda d:d['active'][0].__setitem__('mass',2),
            lambda d:d['cornerEdges'][0]['unitPatchMatrix'][0].__setitem__(1,0),
            lambda d:d['presets']['corner']['basis'][0].__setitem__(0,1),
            lambda d:d['meta'].__setitem__('outerBoundary','moving wall'),
            lambda d:d['source'].__setitem__('commit','forged'),
            lambda d:d.__setitem__('unreviewed',True),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                data=deepcopy(original);mutation(data);self.write_data(data)
                with self.assertRaises(Refusal):validate_packet(ROOT,self.packet)

    def test_rehashed_corrupt_native_tsv_refuses(self):
        lines=(self.packet/'native.tsv').read_text().splitlines()
        index=next(i for i,line in enumerate(lines) if line.startswith('ROW\t') and line.split('\t')[10]!='0')
        row=lines[index].split('\t');row[12]='4008000000000000';lines[index]='\t'.join(row)
        (self.packet/'native.tsv').write_text('\n'.join(lines)+'\n')
        receipt=json.loads((self.packet/'qualification.json').read_text())
        receipt['records_sha256']=sha(self.packet/'native.tsv')
        (self.packet/'qualification.json').write_text(encoded(receipt))
        with self.assertRaises(Refusal):validate_packet(ROOT,self.packet)

    def test_unreviewed_files_and_receipt_scope_refuse(self):
        (self.packet/'unexpected').write_text('extra')
        with self.assertRaisesRegex(Refusal,'regular file roster'):validate_packet(ROOT,self.packet)
        (self.packet/'unexpected').unlink()
        receipt=json.loads((self.packet/'qualification.json').read_text())
        receipt['stepping_authorized']=True
        (self.packet/'qualification.json').write_text(encoded(receipt))
        with self.assertRaises(Refusal):validate_packet(ROOT,self.packet)

    def test_changed_source_binary_and_receipt_digest_refuse(self):
        original=json.loads((self.packet/'qualification.json').read_text())
        for key,value in [('source_tree','0'*40),('native_source_head','0'*40),
                          ('executable_sha256','0'*64),('primary_qualification_sha256','0'*64),
                          ('clean',False),('unreviewed','extra')]:
            with self.subTest(key=key):
                changed=deepcopy(original);changed[key]=value
                (self.packet/'qualification.json').write_text(encoded(changed))
                with self.assertRaises((Refusal,subprocess.CalledProcessError)):
                    validate_packet(ROOT,self.packet)


if __name__ == '__main__':
    unittest.main()
