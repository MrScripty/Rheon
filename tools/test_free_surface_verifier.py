from generated_fixtures import fixture
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from PIL import Image
from verify_free_surface import ROOT, verify


class FreeSurfaceVerifierTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = fixture("free_surface")
    def test_actual_slab_evidence(self):
        self.assertEqual(len(verify(self.fixture)["results"]), 2)

    def test_adversarial_fields_ledgers_render_and_rejection(self):
        for change in ("nan_pressure", "infinite_velocity", "fraction", "air_pressure", "before", "outward", "balance", "budget", "carry", "pixel", "preserved", "probe_failure"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                demo = Path(temporary) / "demo"
                shutil.copytree(self.fixture, demo)
                case = demo / "jacobi-pcg-v1"
                if change == "pixel":
                    path = case / "pulse-final.png"
                    with Image.open(path) as original:
                        image = original.copy()
                    image.putpixel((0, 0), image.getpixel((0, 0)) ^ 1)
                    image.save(path)
                elif change in ("preserved", "before", "outward", "balance", "budget"):
                    path = case / "pulse.json"
                    content = json.loads(path.read_text())
                    content[{"preserved": "accepted_bits_preserved", "before": "volume_before", "outward": "outward", "balance": "balance", "budget": "budget"}[change]] = False if change == "preserved" else 999
                    path.write_text(json.dumps(content))
                else:
                    filename, field, index, value = {
                        "nan_pressure": ("pulse-final.csv", "pressure", 0, "nan"),
                        "infinite_velocity": ("pulse-final-faces.csv", "velocity", 0, "inf"),
                        "fraction": ("pulse-final.csv", "fraction", 2, ".5"),
                        "air_pressure": ("pulse-final.csv", "pressure", 2, ".125"),
                        "carry": ("rest-steps.csv", "volume_before", 1, "999"),
                        "probe_failure": ("rounding-probes.csv", "preserved_on_rejection", 3, "false"),
                    }[change]
                    path = case / filename
                    with path.open() as stream:
                        reader = csv.DictReader(stream)
                        fields, rows = reader.fieldnames, list(reader)
                    rows[index][field] = value
                    with path.open("w", newline="") as stream:
                        writer = csv.DictWriter(stream, fieldnames=fields)
                        writer.writeheader()
                        writer.writerows(rows)
                with self.assertRaises(ValueError):
                    verify(demo)


if __name__ == "__main__":
    unittest.main()
