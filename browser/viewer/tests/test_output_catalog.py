from pathlib import Path
import fcntl,hashlib,json,os,sys,tempfile,unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import output_catalog as catalog

HEAD='1'*40
class Catalog(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.producer=self.root/'producer';self.producer.mkdir();self.output=self.root/'published'
    def flow(self,directory=None):
        directory=directory or self.producer;directory.mkdir(exist_ok=True)
        raw=b'{"schema":"rheon-obstacle-flow-records-v1","shear_cases":[]}'
        (directory/'records.json').write_bytes(raw)
        receipt={'schema':'rheon-obstacle-flow-qualification-v1','clean':True,'source_head':HEAD,'records_sha256':hashlib.sha256(raw).hexdigest()}
        (directory/'qualification.json').write_text(json.dumps(receipt));return receipt
    def entries(self):return json.loads((self.output/'catalog.json').read_bytes())['entries']
    def publish(self):return catalog.publish([('saved',self.producer)],self.output)
    def test_completed_snapshot_exact_bytes_and_receipt_are_retained(self):
        self.flow();self.publish();entry=self.entries()[0];self.assertEqual(entry['state'],'completed')
        for key,file in [('record','records.json'),('receipt','qualification.json')]:
            ref=entry[key] if key=='record' else entry['provenance'][key]
            self.assertEqual((self.output/'blobs'/(ref['sha256']+'.json')).read_bytes(),(self.producer/file).read_bytes())
        original=(self.output/'catalog.json').read_bytes();self.publish();self.assertEqual((self.output/'catalog.json').read_bytes(),original)
    def test_partial_failed_and_changed_records_never_become_completed(self):
        (self.producer/'records.json').write_text('{}');self.publish();self.assertEqual(self.entries()[0]['state'],'incomplete')
        self.flow();self.publish();old=self.entries()[0]['record'];(self.producer/'records.json').write_text('{"changed":true}')
        self.publish();self.assertEqual(self.entries()[0]['state'],'failed');self.assertTrue((self.output/'blobs'/(old['sha256']+'.json')).exists())
        (self.producer/'qualification.json').write_text(json.dumps({'qualified':False,'failure':'original producer failed','source_head':HEAD}))
        self.publish();self.assertEqual(self.entries()[0]['message'],'original producer failed')
    def test_parent_pipeline_failure_blocks_finished_child(self):
        child=self.producer/'oracle-debug';self.flow(child)
        (self.producer/'qualification.json').write_text(json.dumps({'qualified':False,'source_clean':True,'binaries':{},'failure':'pipeline failed','source_head':HEAD}))
        self.publish();self.assertTrue(all(e['state']=='failed' for e in self.entries()))
    def test_all_parent_failure_markers_block_completed_children(self):
        child=self.producer/'packet';self.flow(child)
        for parent in [{'qualified':False,'failure':'failed without binaries'},
                       {'qualified':True,'source_clean':True,'binaries':{},'failure':'contradictory producer failure'}]:
            (self.producer/'qualification.json').write_text(json.dumps(parent))
            self.publish();self.assertTrue(all(e['state']=='failed' and 'record' not in e for e in self.entries()))
    def test_directory_swap_after_discovery_never_reads_outside_root(self):
        child=self.producer/'packet';self.flow(child);outside=self.root/'outside';self.flow(outside)
        real=catalog.discover
        def swap(roots):
            result=real(roots);child.rename(self.producer/'old-packet');child.symlink_to(outside,target_is_directory=True);return result
        with patch('output_catalog.discover',side_effect=swap):self.publish()
        self.assertTrue(all(e['state']=='failed' and 'record' not in e for e in self.entries()))
        self.assertEqual(list((self.output/'blobs').iterdir()),[])
    def test_receipt_rewrite_during_discovery_preserves_previous_catalog(self):
        self.flow();self.publish();old=(self.output/'catalog.json').read_bytes();real=catalog.read
        def rewrite(path,limit):
            raw=real(path,limit)
            if Path(path).name=='records.json':
                changed=json.loads((self.producer/'qualification.json').read_text());changed['failure']='producer changed during read'
                (self.producer/'qualification.json').write_text(json.dumps(changed))
            return raw
        with patch('output_catalog.read',side_effect=rewrite):
            with self.assertRaisesRegex(ValueError,'changed during discovery'):self.publish()
        self.assertEqual((self.output/'catalog.json').read_bytes(),old)
    def test_rigid_receipt_fixture_identity_and_duplicate_roster_refuse(self):
        raw=b'{"initial":{},"steps":[]}';digest=hashlib.sha256(raw).hexdigest();(self.producer/'saved.stdout.json').write_bytes(raw)
        receipt={'source_head':HEAD,'qualified':True,'fixtures':[{'name':'saved','stdout_sha256':digest}],'evidence_sha256':{'saved.stdout.json':digest}}
        (self.producer/'qualification.json').write_text(json.dumps(receipt));self.publish();self.assertEqual(self.entries()[0]['state'],'completed')
        receipt['fixtures']*=2;(self.producer/'qualification.json').write_text(json.dumps(receipt));self.publish();self.assertEqual(self.entries()[0]['state'],'failed')
    def test_quota_failure_and_atomic_rename_failure_preserve_previous_catalog(self):
        self.flow();self.publish();old=(self.output/'catalog.json').read_bytes()
        with patch.dict(catalog.LIMITS,{'files':1}):
            with self.assertRaisesRegex(ValueError,'512'):self.publish()
        self.assertEqual((self.output/'catalog.json').read_bytes(),old)
        real=catalog.os.replace
        def fail_catalog(source,target):
            if Path(target).name=='catalog.json':raise OSError('injected atomic publication failure')
            return real(source,target)
        with patch('output_catalog.os.replace',side_effect=fail_catalog):
            with self.assertRaisesRegex(OSError,'injected'):self.publish()
        self.assertEqual((self.output/'catalog.json').read_bytes(),old)
        self.assertFalse(list(self.output.glob('.catalog-*')))
    def test_symlinks_special_files_and_competing_writer_refuse(self):
        self.flow();self.publish();old=(self.output/'catalog.json').read_bytes()
        (self.producer/'records.json').unlink();(self.producer/'records.json').symlink_to(self.root/'outside')
        self.publish();self.assertEqual(self.entries()[0]['state'],'failed')
        (self.producer/'records.json').unlink();os.mkfifo(self.producer/'records.json');self.publish();self.assertEqual(self.entries()[0]['state'],'failed')
        with (self.output/'.catalog.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            with self.assertRaises(BlockingIOError):self.publish()
        (self.output/'catalog.json').unlink();(self.output/'catalog.json').symlink_to(self.root/'outside')
        with self.assertRaisesRegex(ValueError,'symlink'):self.publish()
        self.assertFalse((self.root/'outside').exists())
    def test_input_output_overlap_and_invalid_roots_refuse(self):
        with self.assertRaisesRegex(ValueError,'separate'):catalog.publish([('saved',self.producer)],self.producer/'published')
        with self.assertRaisesRegex(ValueError,'duplicate'):catalog.publish([('saved',self.producer),('saved',self.producer)],self.output)
        with self.assertRaisesRegex(ValueError,'8'):catalog.publish([(str(i),self.producer) for i in range(9)],self.output)

if __name__=='__main__':unittest.main()
