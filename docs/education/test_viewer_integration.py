from pathlib import Path
import hashlib,json,tempfile,unittest
from unittest.mock import patch
from viewer_integration import publish,inventory,OWNED

PIN='3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4';HEAD='1'*40
class ViewerIntegration(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.repo=self.root/'repo';self.source=self.root/'prebuilt';self.output=self.root/'edition'
        (self.repo/'browser/viewer').mkdir(parents=True);self.source.mkdir();self.output.mkdir()
        (self.repo/'browser/viewer/build.py').write_text("PIN = '"+PIN+"'\n")
        hashes={}
        for name in (*OWNED,'component.json','kenoma/pkg/human_wasm_bg.wasm','kenoma/vendor/sqljs/sql-wasm.wasm','kenoma/vendor/sqljs/LICENSE'):
            raw=(json.dumps({'commit':PIN,'protocol':1,'rig_version':1}) if name=='component.json' else 'test-runtime-'+name).encode()
            path=self.source/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
            if name in OWNED:(self.repo/'browser/viewer'/name).write_bytes(raw)
            hashes[name]=hashlib.sha256(raw).hexdigest()
        (self.source/'package-receipt.json').write_text(json.dumps({'schema':'rheon-viewer-package-v1','rheon_head':HEAD,'rheon_dirty':False,'kenoma_commit':PIN,'files_sha256':hashes}))
        (self.source/'outputs').mkdir();(self.source/'outputs/catalog.json').write_text(json.dumps({'schema':'rheon-producer-output-catalog-v1','entries':[]}))
        self.head=patch('viewer_integration.subprocess.check_output',return_value=HEAD+'\n');self.head.start();self.addCleanup(self.head.stop)
    def test_optional_absence_and_exact_current_package_copy(self):
        self.assertIsNone(publish(self.repo,self.output))
        result=publish(self.repo,self.output,self.source);self.assertTrue(result['included'])
        for name,sha in result['files_sha256'].items():self.assertEqual(hashlib.sha256((self.output/'viewer'/name).read_bytes()).hexdigest(),sha)
    def test_source_mismatch_stale_head_and_symlink_refuse_before_copy(self):
        original=(self.source/'shell.js').read_bytes();(self.source/'shell.js').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError,'runtime bytes'):publish(self.repo,self.output,self.source)
        self.assertFalse((self.output/'viewer').exists());(self.source/'shell.js').write_bytes(original)
        with patch('viewer_integration.subprocess.check_output',return_value='2'*40):
            with self.assertRaisesRegex(ValueError,'clean source head'):inventory(self.repo,self.source)
        (self.source/'shell.js').unlink();(self.source/'shell.js').symlink_to(self.repo/'browser/viewer/shell.js')
        with self.assertRaisesRegex(ValueError,'symlink'):inventory(self.repo,self.source)
    def test_changed_producer_snapshot_refuses_without_mutating_edition(self):
        raw=b'{"recorded":true}';digest=hashlib.sha256(raw).hexdigest();(self.source/'outputs/blobs').mkdir();(self.source/'outputs/blobs'/(digest+'.json')).write_bytes(b'changed')
        (self.source/'outputs/catalog.json').write_text(json.dumps({'schema':'rheon-producer-output-catalog-v1','entries':[{'state':'completed','record':{'sha256':digest,'bytes':len(raw)}}]}))
        with self.assertRaisesRegex(ValueError,'snapshot blob'):publish(self.repo,self.output,self.source)
        self.assertFalse((self.output/'viewer').exists())

if __name__=='__main__':unittest.main()
