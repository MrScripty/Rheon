"""Independent CPU fixture: correction to the same linear Pa pressure target."""
import hashlib
import json
from pathlib import Path
import sys

manifest_bytes = Path(sys.argv[-1]).read_bytes()
metadata = json.loads(manifest_bytes)
ready = {"kind": "adapter_ready", "id": metadata["id"], "version": metadata["version"],
         "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
         "device": "cpu", "optimizer_updates": 0}
print(json.dumps(ready), flush=True)
for serialized in sys.stdin:
    item = json.loads(serialized)
    coordinates = item["inputs"]["locations"]["values"]
    previous = item["inputs"]["prior"]["values"]
    assert item["inputs"]["prior"]["units"] == "Pa"
    correction = [(location - coordinates[0]) - pressure
                  for location, pressure in zip(coordinates, previous)]
    response = {"kind": "candidate", "id": item["id"], "role": "pressure_correction",
                "pressure": correction, "device": "cpu", "optimizer_updates": 0}
    print(json.dumps(response), flush=True)
