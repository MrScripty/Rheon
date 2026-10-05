"""Independent gate for the resolved slab laboratory, including its failures.

Only the stored two-column/one-Z-cell pulse and one-cell hydrostatic replay are
qualified. No moving-interface refinement or arbitrary topology claim.
"""
import csv
import json
import math
from pathlib import Path
import sys
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
METHODS = ("jacobi-pcg-v1", "sgs-pcg-v1")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def numeric_csv(path):
    with path.open() as stream:
        rows = [{key: float(value) for key, value in row.items()}
                for row in csv.DictReader(stream)]
    require(all(math.isfinite(value) for row in rows for value in row.values()),
            "nonfinite field: " + path.name)
    return rows


def near(value, expected, tolerance=1e-12):
    require(math.isfinite(value) and abs(value - expected) <= tolerance,
            f"value {value} differs from {expected}")


def ledger(row, before):
    values = [row[key] for key in ("volume_before", "volume_after", "inward",
                                  "outward", "source", "balance", "budget")]
    require(all(math.isfinite(value) for value in values), "nonfinite ledger")
    require(row["volume_before"] == before, "volume carry-forward")
    require(row["inward"] == row["outward"] == row["source"] == 0,
            "sealed source-free scenario ledger")
    balance = row["volume_after"] - row["volume_before"] + row["outward"] - row["inward"] - row["source"]
    budget = (64 * sys.float_info.epsilon) * (abs(row["volume_before"]) + abs(row["volume_after"]) + row["outward"] + row["inward"] + abs(row["source"]))
    require(row["balance"] == balance, "independent balance")
    require(row["budget"] == budget, "independent rounding budget")
    require(abs(balance) <= budget, "ledger exceeds budget")
    return row["volume_after"]


def frame(case, stem, nx, expected_fraction, expected_pressure, expected_faces):
    cells = numeric_csv(case / (stem + ".csv"))
    require(len(cells) == nx * 2, "cell count")
    for index, cell in enumerate(cells):
        require((cell["i"], cell["j"], cell["k"]) == (index % nx, index // nx, 0), "cell layout")
        near(cell["fraction"], expected_fraction[index])
        require(0 <= cell["fraction"] <= 1, "raw fraction bounds")
        near(cell["pressure"], expected_pressure[index])
    near(math.fsum(cell["fraction"] for cell in cells), nx)
    faces = numeric_csv(case / (stem + "-faces.csv"))
    expected_keys = set()
    for axis, dims in enumerate(((nx + 1, 2, 1), (nx, 3, 1), (nx, 2, 2))):
        for k in range(dims[2]):
            for j in range(dims[1]):
                for i in range(dims[0]):
                    expected_keys.add((axis, i, j, k))
    actual = {(row["axis"], row["i"], row["j"], row["k"]): row["velocity"] for row in faces}
    require(len(faces) == len(expected_keys) and set(actual) == expected_keys, "face layout")
    for key, velocity in actual.items():
        near(velocity, expected_faces.get(key, 0))
    divergence = []
    for j in range(2):
        for i in range(nx):
            divergence.append(actual[(0, i + 1, j, 0)] - actual[(0, i, j, 0)]
                              + actual[(1, i, j + 1, 0)] - actual[(1, i, j, 0)]
                              + actual[(2, i, j, 1)] - actual[(2, i, j, 0)])
    with Image.open(case / (stem + ".png")) as image:
        require(image.mode == "L" and image.size == (nx, 2), "native guidance layout")
        pixels = [image.getpixel((i, j)) for j in range(2) for i in range(nx)]
    expected_pixels = [math.floor(255 * -math.expm1(-8 * expected_fraction[i + nx * j]) + .5)
                       for j in (1, 0) for i in range(nx)]
    require(pixels == expected_pixels, "native pixel equation")
    return divergence


def verify(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "complete.json").read_text())
    require(manifest == {"model": "cell-aligned-slab-snapshot", "methods": 2,
                         "surface_reconstruction": False, "variable_density": False}, "completion scope")
    require({p.name for p in directory.iterdir() if p.is_dir()} == set(METHODS), "method set")
    results = []
    for method in METHODS:
        case = directory / method
        for stem, pressure in (("rest-initial", [0, 0]), ("rest-final", [.125, 0])):
            require(frame(case, stem, 1, [1, 0], pressure, {}) == [0, 0], "rest velocities")
        rest = numeric_csv(case / "rest-steps.csv")
        require(len(rest) == 16, "rest intervals")
        previous = 1
        for step, row in enumerate(rest, 1):
            require(row["step"] == step and row["time"] == .5 * step and row["dt"] == .5, "rest clock")
            near(row["pressure"], .125)
            near(row["wet_divergence"], 0)
            previous = ledger(row, previous)
            near(previous, 1, 0)
        frame(case, "pulse-initial", 2, [1, 1, 0, 0], [0] * 4, {})
        divergence = frame(case, "pulse-final", 2, [1, .9375, .0625, 0], [-.125, .125, 0, 0],
                           {(0, 1, 0, 0): -.125, (1, 0, 1, 0): .125, (1, 1, 1, 0): -.125})
        for value, expected in zip(divergence, [0, 0, -.125, .125]):
            near(value, expected)
        pulse = json.loads((case / "pulse.json").read_text())
        require(all(math.isfinite(value) for value in pulse.values() if isinstance(value, (int, float))), "nonfinite pulse metadata")
        near(ledger(pulse, 2), 2, 0)
        require(pulse["time"] == .5 and pulse["owned_array_bytes"] == 640, "pulse time/inventory")
        near(pulse["wet_divergence"], max(abs(value) for value in divergence[:2]))
        require(pulse["next_interval_rejected"] is True and pulse["accepted_bits_preserved"] is True
                and pulse["next_error"] == "FreeSurface(FractionMismatch { cell: 1 })", "next geometry rejection")
        with (case / "rounding-probes.csv").open() as stream:
            probes = list(csv.DictReader(stream))
        require([int(row["wet_layers"]) for row in probes] == [2, 4, 8, 16], "probe layers")
        for row in probes:
            rejected = method == "sgs-pcg-v1" and row["wet_layers"] == "16"
            require(row["accepted"] == ("false" if rejected else "true"), "probe acceptance evidence")
            require(row["preserved_on_rejection"] == "true", "probe rollback evidence")
            near(float(row["time"]), 0 if rejected else .125, 0)
            require(row["result"] == ("Volume(FractionBounds { cell: 0, fraction: 1.0000000000000004 })" if rejected else "accepted"), "retained probe result")
        results.append({"method": method, "rest_intervals": 16, "rest_pressure": .125,
                        "pulse_volume": 2, "pulse_mass_at_density_one": 2,
                        "pulse_fraction": [1, .9375, .0625, 0], "pulse_pressure": [-.125, .125, 0, 0],
                        "pulse_wet_divergence": divergence[:2], "pulse_air_divergence": divergence[2:],
                        "next_geometry_rejected_and_preserved": True,
                        "rounding_probe_rejected_layers": [16] if method == "sgs-pcg-v1" else []})
    return {"scope": "Resolved one-cell rest and two-column/one-Z-cell manufactured pulse; no evolved surface accuracy claim.",
            "finite_fields_pixels_ledger_carry_forward": "passed", "results": results}


if __name__ == "__main__":
    print(json.dumps(verify(Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "evidence/free-surface/demo"), indent=2))
