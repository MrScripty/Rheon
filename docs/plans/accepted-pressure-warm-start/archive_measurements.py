#!/usr/bin/env python3
"""Lossless text archive for reproducible measurements, without hundreds of files.
Usage: archive_measurements.py pack directory archive.jsonl
       archive_measurements.py unpack archive.jsonl empty_directory
"""
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys

operation, source, destination = sys.argv[1:]
if operation == 'pack':
    root = Path(source)
    if Path(destination).resolve().is_relative_to(root.resolve()):
        raise SystemExit('archive destination must be outside source directory')
    if any(not path.is_file() for path in root.iterdir()):
        raise SystemExit('archive input contains unexpected nested/non-file content')
    with Path(destination).open('x') as output:
        for path in sorted(root.iterdir()):
            if path.is_file() and path.name != 'summary.json':
                raw = path.read_bytes()
                record = {'name': path.name, 'sha256': hashlib.sha256(raw).hexdigest(),
                          'content': raw.decode('utf-8')}
                output.write(json.dumps(record, separators=(',', ':')) + '\n')
elif operation == 'unpack':
    root = Path(destination)
    root.mkdir(parents=True, exist_ok=True)
    if any(root.iterdir()):
        raise SystemExit('refusing to overwrite nonempty directory')
    for line in Path(source).read_text().splitlines():
        record = json.loads(line)
        name = PurePosixPath(record['name'])
        if name.is_absolute() or len(name.parts) != 1 or name.name in ('.', '..'):
            raise SystemExit('invalid archive member')
        raw = record['content'].encode('utf-8')
        if hashlib.sha256(raw).hexdigest() != record['sha256']:
            raise SystemExit('archive content hash mismatch')
        with (root / name.name).open('xb') as output:
            output.write(raw)
else:
    raise SystemExit('expected pack or unpack')
