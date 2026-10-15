"""CPU contract fixture: p(x)=1 Pa/m*(x-x0), not a trained checkpoint."""
import hashlib
import json
from pathlib import Path
import sys

path = Path(sys.argv[-1])
raw = path.read_bytes()
manifest = json.loads(raw)
print(json.dumps(dict(kind="adapter_ready", id=manifest["id"],
    version=manifest["version"], manifest_sha256=hashlib.sha256(raw).hexdigest(),
    device="cpu", optimizer_updates=0)), flush=True)
for line in sys.stdin:
    request = json.loads(line)
    x = request["inputs"]["coordinate"]["values"]
    assert request["inputs"]["coordinate"]["units"] == "m"
    result = [value - x[0] for value in x]
    print(json.dumps(dict(kind="candidate", id=request["id"],
        role="full_pressure", pressure=result, device="cpu", optimizer_updates=0)), flush=True)
