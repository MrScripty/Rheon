"""Independently check exported fractions against geometry and a binomial oracle."""
import csv
import json
import math
from pathlib import Path


def verify(root):
    results = []
    for summary in csv.DictReader((root / "example.csv").open()):
        n = int(summary["cells"])
        steps = int(summary["steps"])
        courant = float(summary["courant"])
        h = 1.0 / n
        dt = courant * h / 0.25
        rows = list(csv.DictReader(
            (root / "spatial-reference" / (summary["scenario"] + ".csv")).open()
        ))
        assert len(rows) == n
        fractions = []
        exact = []
        centers = []
        for i, row in enumerate(rows):
            fraction = float(row["fraction"])
            center = (i + 0.5) * h
            overlap = max(0.0, min((i + 1) * h, 0.625) - max(i * h, 0.375)) / h
            donor = math.fsum(
                math.comb(steps, k) * courant**k * (1.0 - courant)**(steps - k)
                for k in range(steps + 1) if n // 4 <= i - k < n // 2
            )
            assert int(row["cell"]) == i and float(row["center"]) == center
            assert 0.0 <= fraction <= 1.0
            assert float(row["exact_fraction"]) == overlap
            assert abs(fraction - donor) <= 8.0 * math.ulp(1.0)
            fractions.append(fraction)
            exact.append(overlap)
            centers.append(center)
        volume = h * math.fsum(fractions)
        error = h * math.fsum(abs(f - e) for f, e in zip(fractions, exact))
        centroid = h * math.fsum(f * x for f, x in zip(fractions, centers)) / volume
        assert volume == 0.25 == float(summary["liquid_volume"])
        assert volume * 1000.0 == float(summary["liquid_mass"])
        assert abs(error - float(summary["cell_average_l1_volume_error"])) <= 1e-15
        assert abs(centroid - 0.5) <= 2e-16
        assert abs(abs(centroid - 0.5) - float(summary["centroid_error"])) <= 2e-16
        assert steps * dt == 0.5 == float(summary["time"])
        assert float(summary["max_balance_error"]) == 0.0
        assert sum(0.0 < f < 1.0 for f in fractions) == int(summary["mixed_cells"])
        assert 56 * n + 8 == int(summary["array_bytes"])
        results.append({
            "scenario": summary["scenario"], "cells": n, "spacing_x": h,
            "dt": dt, "steps": steps, "courant": courant, "final_time": steps * dt,
            "liquid_volume": volume, "liquid_mass": volume * 1000.0,
            "cell_average_l1_volume_error": error,
            "recomputed_centroid_error": abs(centroid - 0.5),
            "reported_centroid_error": float(summary["centroid_error"]),
            "max_balance_error": 0.0, "mixed_cells": int(summary["mixed_cells"]),
            "array_bytes": int(summary["array_bytes"]),
            "independent_binomial_oracle": "passed every exported cell",
        })
    assert len(results) == 4
    assert results[0]["cell_average_l1_volume_error"] > results[1]["cell_average_l1_volume_error"] > results[2]["cell_average_l1_volume_error"]
    assert results[3]["cell_average_l1_volume_error"] > results[2]["cell_average_l1_volume_error"]
    return {
        "slab": results,
        "one_step_3d_cube_l1_volume_error": 0.65625,
        "one_step_3d_cube_result_source": "tests/liquid_volume_contract.rs / unsplit_three_dimensional_transfer_is_bounded_but_diffuses_geometric_cube (actual passed fixture)",
        "scope": "Volume and spatial accuracy; no isolated timing or physical calibration claim.",
        "dt_refinement_limit": "At fixed h, smaller dt increases first-order donor-cell diffusion.",
    }


if __name__ == "__main__":
    print(json.dumps(verify(Path(__file__).resolve().parent), indent=2))
