"""Admit a prebuilt viewer into the existing edition without compiling or solving."""
from pathlib import Path, PurePosixPath
import hashlib,json,os,re,stat,subprocess

OWNED=('index.html','viewer.css','shell.css','shell.js','view.js','adapters.js','catalog.js','published.js','app.js')
SHA=re.compile(r'^[0-9a-f]{64}$')

def require(ok,message):
    if not ok:raise ValueError(message)

def read(root,name,limit):
    path=PurePosixPath(name)
    require(not path.is_absolute() and all(p not in ('.','..') for p in path.parts) and '\\' not in name,'Invalid viewer path')
    target=Path(root)
    for part in path.parts:
        target=target/part;require(not target.is_symlink(),'Viewer symlinks are refused')
    fd=os.open(target,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd);require(stat.S_ISREG(before.st_mode) and 0<before.st_size<=limit,'Viewer file size/type refused')
        with os.fdopen(fd,'rb',closefd=False) as stream:raw=stream.read(limit+1)
        after=os.fstat(fd);require(len(raw)==before.st_size and (before.st_size,before.st_mtime_ns)==(after.st_size,after.st_mtime_ns),'Viewer changed while reading')
        return raw
    finally:os.close(fd)

def digest(raw):return hashlib.sha256(raw).hexdigest()

def inventory(repo,source):
    source=Path(source);require(source.is_dir() and not source.is_symlink(),'Prebuilt viewer must be a regular directory')
    package_raw=read(source,'package-receipt.json',256*1024);package=json.loads(package_raw)
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
    require(package.get('schema')=='rheon-viewer-package-v1' and package.get('rheon_head')==head and package.get('rheon_dirty') is False,'Prebuilt viewer must bind this exact clean source head')
    pin=re.search(r"^PIN = '([0-9a-f]{40})'$",(Path(repo)/'browser/viewer/build.py').read_text(),re.M).group(1)
    require(package.get('kenoma_commit')==pin,'Prebuilt Kenoma pin differs')
    hashes=package.get('files_sha256');require(isinstance(hashes,dict) and 1<=len(hashes)<=128,'Invalid runtime inventory')
    for name in (*OWNED,'component.json','kenoma/pkg/human_wasm_bg.wasm','kenoma/vendor/sqljs/sql-wasm.wasm','kenoma/vendor/sqljs/LICENSE'):
        require(name in hashes,'Required viewer runtime is absent: '+name)
    component=json.loads(read(source,'component.json',4096));require(component.get('commit')==pin and component.get('protocol')==1 and component.get('rig_version')==1,'Viewer component identity differs')
    items={'package-receipt.json':package_raw};total=len(package_raw)
    for name,sha in hashes.items():
        require(isinstance(name,str) and SHA.fullmatch(sha) and not name.startswith('outputs/'),'Invalid runtime hash')
        raw=read(source,name,8*1024*1024);require(digest(raw)==sha,'Prebuilt runtime bytes differ: '+name)
        if name in OWNED:require(raw==(Path(repo)/'browser/viewer'/name).read_bytes(),'Viewer runtime differs from current source: '+name)
        total+=len(raw);require(total<=32*1024*1024,'Viewer runtime exceeds 32 MiB');items[name]=raw
    catalog_raw=read(source,'outputs/catalog.json',256*1024);catalog=json.loads(catalog_raw)
    require(catalog.get('schema')=='rheon-producer-output-catalog-v1' and isinstance(catalog.get('entries'),list) and len(catalog['entries'])<=128,'Invalid producer catalog')
    refs={}
    for entry in catalog['entries']:
        require(entry.get('state') in ('completed','failed','incomplete','unsupported'),'Invalid producer state')
        if 'record' in entry:
            require(entry['state']=='completed','Unavailable output has a record');ref=entry['record'];refs[ref['sha256']]=(ref,8*1024*1024)
        for key in ('receipt','pipeline_receipt'):
            if key in entry.get('provenance',{}):
                ref=entry['provenance'][key];refs[ref['sha256']]=(ref,2*1024*1024)
    blob_total=0
    for sha,(ref,limit) in refs.items():
        require(SHA.fullmatch(sha) and type(ref['bytes']) is int and 0<ref['bytes']<=limit,'Invalid producer blob reference')
        name='outputs/blobs/'+sha+'.json';raw=read(source,name,limit)
        require(len(raw)==ref['bytes'] and digest(raw)==sha,'Producer snapshot blob differs')
        blob_total+=len(raw);require(blob_total<=64*1024*1024,'Producer blobs exceed 64 MiB');items[name]=raw
    items['outputs/catalog.json']=catalog_raw
    return package,items

def publish(repo,output,source=None):
    output=Path(output)
    if source is None:
        (output/'viewer-integration.json').write_text(json.dumps({'included':False})+'\n');return None
    package,items=inventory(repo,source)
    target=output/'viewer';require(not target.exists() and not target.is_symlink(),'Viewer output must be fresh')
    target.mkdir()
    for name,raw in items.items():
        path=target/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
    receipt={'included':True,'rheon_head':package['rheon_head'],'kenoma_commit':package['kenoma_commit'],'files_sha256':{name:digest(raw) for name,raw in items.items()},'producer_catalog_sha256':digest(items['outputs/catalog.json']),'solver_runs':0}
    (output/'viewer-integration.json').write_text(json.dumps(receipt,indent=2)+'\n');return receipt
