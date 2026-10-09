"""Retrieve immutable comparison evidence into ignored storage, never new baselines.

Prefer existing Git objects; a shallow checkout downloads only three hash-locked
small files from the exact public commit. Scientific experiments are not used to
produce expected results, and a missing/corrupt baseline fails closed.
"""
from pathlib import Path
import hashlib
import json
import subprocess
from urllib.request import urlopen

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def historical_baselines():
    config = json.loads((HERE / 'baseline-sources.json').read_text())
    directory = ROOT / '.generated/research-baselines' / config['commit']
    for name, identity in config['files'].items():
        target = directory / name
        if target.exists():
            raw = target.read_bytes()
        else:
            path = config['path'] + '/' + name
            result = subprocess.run(['git', 'show', config['commit'] + ':' + path],
                                    cwd=ROOT, capture_output=True, timeout=30)
            if result.returncode == 0:
                raw = result.stdout
            else:
                url = ('https://raw.githubusercontent.com/' + config['repository']
                       + '/' + config['commit'] + '/' + path)
                with urlopen(url, timeout=30) as response:
                    raw = response.read(identity['bytes'] + 1)
            if len(raw) != identity['bytes'] or hashlib.sha256(raw).hexdigest() != identity['sha256']:
                raise ValueError('historical baseline identity mismatch: ' + name)
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open('xb') as stream:
                stream.write(raw)
        if len(raw) != identity['bytes'] or hashlib.sha256(raw).hexdigest() != identity['sha256']:
            raise ValueError('historical baseline identity mismatch: ' + name)
    return directory


if __name__ == '__main__':
    print(historical_baselines())
