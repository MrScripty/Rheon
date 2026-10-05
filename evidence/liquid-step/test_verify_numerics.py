"""Execute tampered-packet rejection through the independent numerical gate."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
from PIL import Image
from verify_numerics import verify

packet = Path(__file__).resolve().parent
records = []
for kind in ["fraction", "clock", "first_pressure", "guidance_pixel"]:
    with tempfile.TemporaryDirectory(prefix="rheon-liquid-negative-") as temporary:
        demo = Path(temporary) / "demo"
        shutil.copytree(packet / "demo", demo)
        case = demo / "jacobi-pcg-v1-n16-c025"
        if kind == "clock":
            path = case / "run.json"
            data = json.loads(path.read_text())
            data["time"] = 1.0
            path.write_text(json.dumps(data))
        elif kind == "guidance_pixel":
            path = case / "final.png"
            with Image.open(path) as image:
                changed = image.copy()
            changed.putpixel((0, 0), 255)
            changed.save(path)
        else:
            path = case / ("cells.csv" if kind == "fraction" else "first-pressure.csv")
            with path.open() as stream:
                reader = csv.DictReader(stream)
                fields = reader.fieldnames
                data = list(reader)
            data[0]["fraction" if kind == "fraction" else "pressure"] = "0.5" if kind == "fraction" else "1"
            with path.open("w", newline="") as stream:
                writer = csv.DictWriter(stream, fieldnames=fields)
                writer.writeheader()
                writer.writerows(data)
        try:
            verify(demo)
        except ValueError as error:
            records.append({"fixture": kind, "rejected": True, "reason": str(error)})
        else:
            raise ValueError("Accepted tampered fixture: " + kind)
print(json.dumps(records, indent=2))
