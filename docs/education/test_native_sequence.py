"""Recorded-data admission and source-only publication without replacement runs."""
from pathlib import Path
import hashlib,json,re,tempfile,unittest
from native_sequence import metadata,publish,validate_labs

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
    def test_stale_lean_bytes_cannot_reuse_accepted_inventory(self):
        repo=self.root/'repo';proof=repo/'proofs';proof.mkdir(parents=True);src=proof/'A.lean';src.write_text('checked source')
        pins=proof/'source-inventory.json';pins.write_text(json.dumps({'A.lean':hashlib.sha256(src.read_bytes()).hexdigest()}))
        here=repo/'docs/education';here.mkdir(parents=True);(here/'native-sequence.json').write_text(json.dumps({'proof_inventory_sha256':hashlib.sha256(pins.read_bytes()).hexdigest()}))
        metadata(repo);src.write_text('unreviewed source')
        with self.assertRaisesRegex(ValueError,'Lean source'):metadata(repo)
    def test_changed_inventory_bytes_require_requalification(self):
        repo=self.root/'repo';proof=repo/'proofs';proof.mkdir(parents=True)
        src=proof/'A.lean';src.write_text('checked source')
        inventory={'A.lean':hashlib.sha256(src.read_bytes()).hexdigest()}
        pins=proof/'source-inventory.json';pins.write_text(json.dumps(inventory))
        here=repo/'docs/education';here.mkdir(parents=True)
        (here/'native-sequence.json').write_text(json.dumps({
            'proof_inventory_sha256':hashlib.sha256(pins.read_bytes()).hexdigest()}))
        metadata(repo)
        # Identical entries with different inventory bytes still need review.
        pins.write_text(json.dumps(inventory,indent=2)+'\n')
        with self.assertRaisesRegex(ValueError,'Current Lean inventory differs'):
            metadata(repo)
    def test_repository_metadata_admits_current_reviewed_proofs(self):
        repo=Path(__file__).resolve().parents[2]
        data=metadata(repo)
        current=data['current_reconstruction_proof']
        for key in ('proof_inventory_sha256','public_theorems',
                    'audited_declarations','explicit_expected_declarations','lean_ci'):
            self.assertEqual(data[key],current[key])
        pins=json.loads((repo/'proofs/source-inventory.json').read_text())
        self.assertIn('Rheon/AlignedStrain.lean',pins)
        theorem_count=sum(len(re.findall(r'^theorem ',
            (repo/'proofs'/name).read_text(),re.M))
            for name in pins if name.startswith('Rheon/'))
        self.assertEqual(data['public_theorems'],theorem_count)
        audit=(repo/'proofs/AxiomAudit.lean').read_text()
        expected=re.search(r'let expected : Array Name := #\[(.*?)\]',audit,re.S).group(1)
        self.assertEqual(data['explicit_expected_declarations'],len(re.findall(r'`Rheon\.',expected)))
        self.assertEqual(data['local_kernel_receipt_sha256'],
                         current['local_qualification']['receipt_sha256'])
        self.assertTrue(current['local_qualification']['lean_checked'])
        self.assertEqual(current['local_qualification']['status'],'passed')


if __name__=='__main__':unittest.main()
