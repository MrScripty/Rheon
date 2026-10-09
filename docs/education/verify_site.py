from pathlib import Path
from html.parser import HTMLParser
from urllib.parse import urlsplit,unquote
import argparse,json,hashlib,re,zipfile,posixpath,subprocess,ipaddress
from pypdf import PdfReader
from pdf_freshness import verify_pdf
parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path);args=parser.parse_args()
ROOT=args.output_dir if args.output_dir is not None else Path(__file__).resolve().parent/'_site'
verify_pdf(Path(__file__).resolve().parents[2], ROOT/'downloads/Rheon-expanded-book.pdf', artifact_dir=args.output_dir)
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
    markdown_links=[]
    def links(node):
        if isinstance(node,dict):
            if node.get('t') in ('Link','Image'):markdown_links.append(node['c'][-1][0])
            for value in node.values():links(value)
        elif isinstance(node,list):
            for value in node:links(value)
    members=set(bundle.namelist());checked=0
    for name in sorted(n for n in members if n.endswith('.md')):
        markdown_links.clear()
        parsed=subprocess.run(['pandoc','-f','markdown+tex_math_single_backslash','-t','json'],input=bundle.read(name),capture_output=True,check=True)
        links(json.loads(parsed.stdout))
        for target in markdown_links:
            url=urlsplit(target)
            if url.scheme or url.netloc or not url.path:continue
            resolved=posixpath.normpath(posixpath.join(posixpath.dirname(name),unquote(url.path)))
            if resolved not in members:raise ValueError('Broken Markdown ZIP link: '+name+': '+target)
            checked+=1
    print('PASS Markdown ZIP resolves',checked,'local links/images across every packaged Markdown file')
pdf=PdfReader(ROOT/'downloads/Rheon-expanded-book.pdf');pdf_links=0
for page in pdf.pages:
    for annotation in page.get('/Annots',[]):
        action=annotation.get_object().get('/A',{})
        if action.get('/S')!='/URI':continue
        url=urlsplit(str(action.get('/URI','')));host=url.hostname
        try:loopback=ipaddress.ip_address(host).is_loopback if host else False
        except ValueError:loopback=host=='localhost'
        if loopback:raise ValueError('Temporary preview URL embedded in reading PDF: '+url.geturl())
        pdf_links+=1
print('PASS reading PDF has',pdf_links,'portable URI links and no localhost destinations')
print('PASS',receipt['chapters'],'sections,',receipt['rendered_math_expressions'],'math expressions,',count,'local asset/link targets')

assert receipt["historical_proof_status"]=="checked"
assert receipt["current_proof_status"]=="reviewed-current-source-inventory-matched"
proof=json.loads((ROOT/"proof-qualification.json").read_text())
repo=Path(__file__).resolve().parents[2]
pins=json.loads((repo/"proofs/source-inventory.json").read_text())
sequence=json.loads((ROOT/"native-sequence.json").read_text())
assert hashlib.sha256((repo/"proofs/source-inventory.json").read_bytes()).hexdigest()==sequence["proof_inventory_sha256"]
for name,digest in pins.items():
    assert hashlib.sha256((repo/"proofs"/name).read_bytes()).hexdigest()==digest,name
for name,digest in receipt['linked_source_files'].items():
    assert hashlib.sha256((ROOT/'source-files'/name).read_bytes()).hexdigest()==digest,name
print("PASS current proof bytes match the reviewed source inventory; historical kernel receipt remains separate")
