from pathlib import Path
from html.parser import HTMLParser
from urllib.parse import urlsplit,unquote
import json,hashlib,re,zipfile
from pdf_freshness import verify_pdf
ROOT=Path(__file__).resolve().parent/'_site'
verify_pdf(Path(__file__).resolve().parents[2], ROOT/'downloads/Rheon-expanded-book.pdf')
class Links(HTMLParser):
    def __init__(self):super().__init__();self.targets=[]
    def handle_starttag(self,tag,attrs):
        for key,value in attrs:
            if key in ['href','src'] and value:self.targets.append(value)
fail=[];count=0
for path in ROOT.rglob('*.html'):
    parser=Links();parser.feed(path.read_text())
    for target in parser.targets:
        u=urlsplit(target)
        if u.scheme or u.netloc or not u.path:continue
        candidate=(path.parent/unquote(u.path)).resolve();count+=1
        if not candidate.is_file():fail.append((str(path),target))
receipt=json.loads((ROOT/'build-receipt.json').read_text())
assert receipt['chapters']==31 and receipt['rendered_math_expressions']>=400
assert receipt['reference_data_sha256']==hashlib.sha256((ROOT/'reference-data.json').read_bytes()).hexdigest()
assert not fail,fail
assert (ROOT/'downloads/Rheon-expanded-book.pdf').stat().st_size>100000
with zipfile.ZipFile(ROOT/'downloads/Rheon-expanded-markdown.zip') as bundle:
    markdown=bundle.read('Rheon-expanded-book.md')
    assert markdown==(ROOT/'downloads/Rheon-expanded-book.md').read_bytes()
    figures=re.findall(r'!\[[^\]]*\]\(([^)]+)\)',markdown.decode())
    for target in figures:
        assert bundle.read(target)==(ROOT/target).read_bytes(),target
    print('PASS Markdown ZIP resolves',len(figures),'actual relative figure paths')
print('PASS',receipt['chapters'],'sections,',receipt['rendered_math_expressions'],'math expressions,',count,'local asset/link targets')

assert receipt["proof_status"]=="checked"
proof=json.loads((ROOT/"proof-qualification.json").read_text())
repo=Path(__file__).resolve().parents[2]
for name,digest in proof["source_inventory"].items():
    assert hashlib.sha256((repo/"proofs"/name).read_bytes()).hexdigest()==digest,name
print("PASS exact proof inventory matches local qualification")
