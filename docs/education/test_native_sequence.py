"""Recorded-data admission and source-only publication without replacement runs."""
from pathlib import Path
import hashlib,json,re,tempfile,unittest
from native_sequence import metadata,publish,validate_labs
from unittest.mock import patch

class NativeSequence(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name);self.input=self.root/'input';self.output=self.root/'output';self.output.mkdir()
        lab=self.input/'wall-friction';lab.mkdir(parents=True)
        (lab/'index.html').write_text('<meta charset=utf-8><h1>Recorded case</h1>')
        (lab/'case.csv').write_text('time,velocity\n1,0.25\n')
        self.data={'labs':[{'id':'wall-friction','model':'finite slip','reference':'H+2ell','counts':{'cases':6,'saved_times':4},'files':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in lab.iterdir()}}]}
    def test_changed_csv_and_unrecorded_file_are_refused(self):
        validate_labs(self.data,self.input)
        p=self.input/'wall-friction/case.csv';original=p.read_bytes();p.write_bytes(original+b'changed')
        with self.assertRaisesRegex(ValueError,'Recorded lab bytes'):validate_labs(self.data,self.input)
        p.write_bytes(original);(p.parent/'unexpected.csv').write_text('unreviewed')
        with self.assertRaisesRegex(ValueError,'Recorded lab bytes'):validate_labs(self.data,self.input)
    def test_source_only_build_states_absence_without_fabricating_a_lab(self):
        body=publish(self.data,self.output)
        self.assertIn('not included',body);self.assertFalse((self.output/'native-labs').exists())
        self.assertEqual(json.loads((self.output/'native-presentation.json').read_text())['numerical_calls'],0)
    def test_nested_unreviewed_content_is_refused(self):
        extra=self.input/'wall-friction/unreviewed';extra.mkdir();(extra/'data.csv').write_text('unreviewed')
        with self.assertRaisesRegex(ValueError,'only declared regular files'):validate_labs(self.data,self.input)
    def test_derived_playback_notice_preserves_original_records(self):
        before={p.name:p.read_bytes() for p in (self.input/'wall-friction').iterdir()}
        publish(self.data,self.output,self.input)
        self.assertIn('Snapshot playback',(self.output/'native-labs/wall-friction/index.html').read_text())
        self.assertEqual((self.output/'native-labs/wall-friction/case.csv').read_bytes(),before['case.csv'])
        self.assertEqual({p.name:p.read_bytes() for p in (self.input/'wall-friction').iterdir()},before)
    def test_repository_metadata_admits_current_reviewed_proofs(self):
        repo=Path(__file__).resolve().parents[2]
        data=metadata(repo)
        current=data.get('current_reviewed_proof',data['current_reconstruction_proof'])
        for key in ('public_theorems',
                    'audited_declarations','explicit_expected_declarations','lean_ci'):
            self.assertEqual(data[key],current[key])
        sources=sorted((repo/'proofs/Rheon').rglob('*.lean'))
        self.assertIn(repo/'proofs/Rheon/AlignedStrain.lean',sources)
        theorem_count=sum(len(re.findall(r'^theorem ',path.read_text(),re.M))
                          for path in sources)
        self.assertEqual(data['public_theorems'],theorem_count)
        audit=(repo/'proofs/AxiomAudit.lean').read_text()
        expected=re.search(r'let expected : Array Name := #\[(.*?)\]',audit,re.S).group(1)
        self.assertEqual(data['explicit_expected_declarations'],len(re.findall(r'`Rheon\.',expected)))
        # Recorded metadata cannot itself turn a pending kernel check into a pass.
        self.assertIn(current['status'], ('reviewed-pending-kernel-qualification', 'passed'))
        if current['status'] == 'passed':
            self.assertIsInstance(current['audited_declarations'], int)
            self.assertGreaterEqual(current['audited_declarations'], current['explicit_expected_declarations'])
            self.assertTrue(current.get('lean_ci') or current.get('local_qualification'))
        else:
            self.assertIsNone(current['audited_declarations'])
        historical=data['current_reconstruction_proof']
        self.assertEqual(historical['public_theorems'],106)
        self.assertEqual(historical['audited_declarations'],161)
        self.assertTrue(historical['local_qualification']['lean_checked'])
        self.assertEqual(historical['local_qualification']['status'],'passed')

    def test_expanded_metadata_cannot_fall_back_or_claim_an_unqualified_pass(self):
        repo=Path(__file__).resolve().parents[2]
        path=repo/'docs/education/native-sequence.json'
        original_read=Path.read_text
        original=json.loads(path.read_text())
        mutations=[lambda d:d.pop('current_reviewed_proof'),
                   lambda d:d.update(schema='rheon-native-sequence-source-v1'),
                   lambda d:d.pop('schema'),
                   lambda d:d['current_reviewed_proof'].update(status='passed',audited_declarations=None),
                   lambda d:d['current_reviewed_proof'].update(public_theorems=106),
                   lambda d:d['current_reviewed_proof'].update(status='unrecognized')]
        for mutate in mutations:
            data=json.loads(json.dumps(original));mutate(data)
            def read(p,*args,**kwargs):
                return json.dumps(data) if p==path else original_read(p,*args,**kwargs)
            with patch.object(Path,'read_text',read),self.assertRaises(ValueError):
                metadata(repo)


if __name__=='__main__':unittest.main()
