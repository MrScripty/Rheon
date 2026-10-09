"""Discover existing producer receipts and atomically publish recorded snapshots.
Never imports a numerical provider, runs an example, or requalifies physics.
"""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tempfile
from build import external

SCHEMA = 'rheon-producer-output-catalog-v1'
LIMITS = {'roots':8, 'directories':64, 'files':512, 'entries':128,
          'record_bytes':8*1024*1024, 'receipt_bytes':2*1024*1024,
          'record_total':32*1024*1024, 'receipt_total':32*1024*1024,
          'catalog_bytes':256*1024}
HEX = re.compile(r'^[0-9a-f]{64}$')
HEAD = re.compile(r'^[0-9a-f]{40}$')
NAME = re.compile(r'^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$')
FIXTURE = re.compile(r'^[a-z][a-z0-9-]{0,119}$')

def require(ok, message):
    if not ok: raise ValueError(message)

def sha(raw): return hashlib.sha256(raw).hexdigest()

def unique(pairs):
    result={}
    for key,value in pairs:
        require(key not in result, 'Duplicate JSON field'); result[key]=value
    return result

def decode(raw):
    return json.loads(raw.decode('utf-8'),object_pairs_hook=unique,
        parse_constant=lambda value: (_ for _ in ()).throw(ValueError('Nonfinite JSON')))

def read(path, limit):
    # Never follow a producer-controlled link; read only bounded regular files.
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd); require(stat.S_ISREG(before.st_mode), 'Not a regular output file')
        require(0<before.st_size<=limit, 'Output file exceeds its byte limit or is empty')
        with os.fdopen(fd,'rb',closefd=False) as stream: raw=stream.read(limit+1)
        after=os.fstat(fd)
        require(len(raw)==before.st_size and (before.st_size,before.st_mtime_ns)==(after.st_size,after.st_mtime_ns), 'Output changed while reading')
        return raw
    finally: os.close(fd)

def discover(roots):
    require(len(roots)<=LIMITS['roots'], 'At most 8 producer roots')
    aliases=set();directories=[];count=0
    for alias,root in roots:
        require(NAME.fullmatch(alias) and alias not in aliases, 'Invalid or duplicate producer name');aliases.add(alias)
        root=Path(root); require(not root.is_symlink() and root.is_dir(), 'Producer root must be a regular directory')
        root=root.resolve(); pending=[(root,alias)];root_names=[]
        # The existing producers put packets at the root or one child level.
        with os.scandir(root) as scan:
            for item in scan:
                count+=1;require(count<=LIMITS['files'], 'Discovery exceeds 512 directory entries')
                root_names.append(item.name)
                if item.is_dir(follow_symlinks=False) and not item.name.startswith('.'):
                    pending.append((Path(item.path),alias+'/'+item.name))
        for directory,label in sorted(pending,key=lambda pair:pair[1]):
            require(len(label)<=240, 'Producer-relative label too long')
            names=root_names
            if directory!=root:
                names=[]
                with os.scandir(directory) as scan:
                    for item in scan:
                        count+=1;require(count<=LIMITS['files'], 'Discovery exceeds 512 directory entries')
                        names.append(item.name)
            directories.append((directory,label,root,frozenset(names)))
            require(len(directories)<=LIMITS['directories'], 'Discovery exceeds 64 directories')
    return directories

def build_catalog(roots):
    directories=discover(roots); entries=[];blobs={};record_total=receipt_total=0
    receipts={}
    def add(label,state,message,**fields):
        require(len(entries)<LIMITS['entries'], 'Catalog exceeds 128 entries')
        entries.append(dict(id=sha(label.encode()),label=label,state=state,message=message,**fields))
    def store(raw):
        digest=sha(raw);blobs[digest]=raw;return {'sha256':digest,'bytes':len(raw)}
    # Read root receipts before children so a failed aggregate cannot look complete.
    for directory,label,root,names in directories:
        path=directory/'qualification.json'
        if 'qualification.json' not in names:continue
        if not path.exists() and not path.is_symlink(): continue
        try:
            raw=read(path,min(LIMITS['receipt_bytes'],LIMITS['receipt_total']-receipt_total));receipt_total+=len(raw)
            receipt=decode(raw);require(isinstance(receipt,dict), 'Receipt must be an object')
            receipts[directory]=(receipt,raw,store(raw))
        except (OSError,ValueError,UnicodeError) as error:
            receipts[directory]=(None,None,None);add(label,'failed','Unreadable producer receipt: '+str(error)[:200])
    for directory,label,root,names in directories:
        item=receipts.get(directory)
        if item is None:
            candidate=any(name in ('records.json','results.json') or name.endswith('.stdout.json') for name in names)
            if candidate:add(label,'incomplete','Recorded files exist without a producer completion receipt')
            continue
        receipt,raw,ref=item
        if receipt is None:continue
        provenance={'receipt':ref}
        head=receipt.get('source_head')
        if isinstance(head,str) and HEAD.fullmatch(head):provenance['source_head']=head
        aggregate=receipts.get(root)
        if directory!=root and aggregate and aggregate[0] is None:
            add(label,'failed','Parent producer receipt is unavailable');continue
        if directory!=root and aggregate and aggregate[0] and 'binaries' in aggregate[0]:
            parent,parent_raw,parent_ref=aggregate
            provenance['pipeline_receipt']=parent_ref
            if parent.get('qualified') is not True or parent.get('source_clean') is not True:
                add(label,'failed' if parent.get('failure') else 'incomplete','Producer pipeline did not complete',provenance=provenance);continue
            if not isinstance(parent.get('evidence_sha256'),dict) or parent['evidence_sha256'].get(str(directory.relative_to(root)/'qualification.json'))!=sha(raw) or parent.get('source_head')!=head:
                add(label,'failed','Child receipt differs from completed producer pipeline',provenance=provenance);continue
        if receipt.get('qualified') is False or receipt.get('failure'):
            add(label,'failed' if receipt.get('failure') else 'incomplete',str(receipt.get('failure') or 'Producer did not record completion')[:240],provenance=provenance);continue
        if 'binaries' in receipt and 'source_clean' in receipt:
            state='completed' if receipt.get('qualified') is True and receipt.get('source_clean') is True and isinstance(head,str) and HEAD.fullmatch(head) else 'failed'
            add(label,state,'Producer pipeline receipt; recordings are listed from child oracle packets',provenance=provenance);continue
        if receipt.get('schema')=='rheon-obstacle-flow-qualification-v1':
            if receipt.get('clean') is not True:
                add(label,'failed','Producer receipt does not record clean source',provenance=provenance);continue
            roster=[('records.json',receipt.get('records_sha256'),'rheon-obstacle-flow-records-v1')]
        elif isinstance(receipt.get('fixtures'),list) and receipt.get('qualified') is True:
            if not 0<len(receipt['fixtures'])<=32:
                add(label,'failed','Invalid producer fixture roster',provenance=provenance);continue
            roster=[];names=set()
            for fixture in receipt['fixtures']:
                name=fixture.get('name') if isinstance(fixture,dict) else None
                if not isinstance(name,str) or not FIXTURE.fullmatch(name) or name in names:
                    roster=None;break
                names.add(name);name+='.stdout.json';digest=fixture.get('stdout_sha256')
                if not isinstance(receipt.get('evidence_sha256'),dict) or receipt['evidence_sha256'].get(name)!=digest:roster=None;break
                roster.append((name,digest,'rheon-rigid-motion-json-v1'))
            if roster is None:
                add(label,'failed','Invalid or unbound producer fixture roster',provenance=provenance);continue
        else:
            add(label,'unsupported','No catalog adapter for this producer receipt',provenance=provenance);continue
        if not isinstance(head,str) or not HEAD.fullmatch(head):
            add(label,'failed','Missing immutable producer source identity',provenance=provenance);continue
        for name,digest,format_name in roster:
            display=label+'/'+name
            if not isinstance(digest,str) or not HEX.fullmatch(digest):
                add(display,'failed','Invalid recorded output digest',provenance=provenance);continue
            path=directory/name
            if not path.exists() and not path.is_symlink():
                add(display,'incomplete','Producer-declared output is missing',provenance=provenance);continue
            try:
                content=read(path,min(LIMITS['record_bytes'],LIMITS['record_total']-record_total));record_total+=len(content)
                require(sha(content)==digest, 'Output bytes differ from producer receipt')
                value=decode(content);require(isinstance(value,dict), 'Recording must be an object')
                require((format_name=='rheon-obstacle-flow-records-v1' and value.get('schema')==format_name and isinstance(value.get('shear_cases'),list)) or
                    (format_name=='rheon-rigid-motion-json-v1' and isinstance(value.get('initial'),dict) and isinstance(value.get('steps'),list)), 'Producer recording format mismatch')
                add(display,'completed','Recorded output; visualization does not requalify physics',format=format_name,record=store(content),provenance=provenance)
            except (OSError,ValueError,UnicodeError) as error:
                add(display,'failed',str(error)[:240],provenance=provenance)
    # Includes aggregate and unsupported receipts, even when their branch above
    # did not read a recording. Publication refuses a mixed completion snapshot.
    for directory,(receipt,raw,ref) in receipts.items():
        if raw is not None:require(read(directory/'qualification.json',LIMITS['receipt_bytes'])==raw, 'Producer receipt changed during discovery')
    entries.sort(key=lambda entry:entry['label'])
    catalog={'schema':SCHEMA,'entries':entries,'limits':LIMITS,'scope':'Recorded output discovery only; no running simulation or numerical qualification'}
    encoded=(json.dumps(catalog,indent=2,allow_nan=False)+'\n').encode()
    require(len(encoded)<=LIMITS['catalog_bytes'], 'Catalog exceeds 256 KiB')
    return encoded,blobs

def publish(roots,output):
    output=Path(output);require(not output.is_symlink(), 'Catalog directory cannot be a symlink')
    output=external(output)
    for _,root in roots:
        resolved=Path(root).resolve();require(not output.is_relative_to(resolved) and not resolved.is_relative_to(output), 'Catalog must be separate from producer inputs')
    output.mkdir(parents=True,exist_ok=True)
    lock=os.open(output/'.catalog.lock',os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW|os.O_NONBLOCK,0o600)
    try:
        require(stat.S_ISREG(os.fstat(lock).st_mode), 'Catalog lock must be a regular file')
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        encoded,blobs=build_catalog(roots)
        destination=output/'blobs';require(not destination.is_symlink(), 'Blob directory cannot be a symlink');destination.mkdir(exist_ok=True)
        for digest,raw in blobs.items():
            path=destination/(digest+'.json')
            if path.exists() or path.is_symlink():require(read(path,max(LIMITS['receipt_bytes'],LIMITS['record_bytes']))==raw,'Stored immutable blob differs')
            else:
                # A complete blob becomes visible in one rename before the catalog.
                with tempfile.NamedTemporaryFile(dir=destination,prefix='.blob-',delete=False) as stream:
                    temp=Path(stream.name);stream.write(raw);stream.flush();os.fsync(stream.fileno())
                try:os.replace(temp,path)
                finally:temp.unlink(missing_ok=True)
        target=output/'catalog.json';require(not target.is_symlink(),'Catalog file cannot be a symlink')
        directory_fd=os.open(destination,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:os.fsync(directory_fd)
        finally:os.close(directory_fd)
        with tempfile.NamedTemporaryFile(dir=output,prefix='.catalog-',delete=False) as stream:
            temp=Path(stream.name);stream.write(encoded);stream.flush();os.fsync(stream.fileno())
        try:os.replace(temp,target)
        finally:temp.unlink(missing_ok=True)
        directory_fd=os.open(output,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:os.fsync(directory_fd)
        finally:os.close(directory_fd)
        return {'catalog_sha256':sha(encoded),'entries':len(decode(encoded)['entries']),'blobs':len(blobs)}
    finally:os.close(lock)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--producer-root',action='append',default=[],metavar='NAME=PATH')
    parser.add_argument('--output-dir',type=Path,required=True)
    args=parser.parse_args();roots=[]
    for spec in args.producer_root:
        require('=' in spec,'Producer root must be NAME=PATH');name,path=spec.split('=',1);roots.append((name,Path(path)))
    print(json.dumps(publish(roots,args.output_dir)))
