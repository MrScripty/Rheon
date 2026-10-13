"""Export only the pinned real lab checkpoint; never execute pickle or train.

Usage: python3 tools/export_ai_pressure.py PATH_TO_LAB_MODEL_PT OUTPUT
The output is an ignored build input, not a committed numerical fixture.
"""
import hashlib
import io
from pathlib import Path
import sys
import zipfile

CHECKPOINT_SHA256 = '25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9'
LAB_COMMIT = 'e8a90bfef6b2955c502008ccc62443e29dd9f2ec'


def export(checkpoint):
    raw = Path(checkpoint).read_bytes()
    if hashlib.sha256(raw).hexdigest() != CHECKPOINT_SHA256:
        raise ValueError('checkpoint SHA256 does not match frozen trajectory-v2')
    # The exact pinned pickle was inspected: contiguous f64 gains in data/0,
    # contiguous [32,32] basis in data/1. No general torch/pickle deserialization.
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        if archive.read('model/byteorder') != b'little':
            raise ValueError('checkpoint endianness')
        gains = archive.read('model/data/0')
        basis = archive.read('model/data/1')
    if (len(gains), len(basis)) != (32767 * 8, 1024 * 8):
        raise ValueError('checkpoint tensor sizes')
    return b'RHEONAI1' + basis + gains


if __name__ == '__main__':
    payload = export(sys.argv[1])
    with Path(sys.argv[2]).open('xb') as output:
        output.write(payload)
    print('export_sha256=' + hashlib.sha256(payload).hexdigest())
