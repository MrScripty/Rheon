"""Independent geometric/binomial, stored-field divergence and PNG checks.

Run with no argument for committed demo, or with a new demo directory. Explicit
requirements remain active under python -O; no repository code is imported.
"""
import csv
import json
import math
from pathlib import Path
import sys
from PIL import Image


def require(condition, message):
    if not condition:
        raise ValueError(message)


def rows(path):
    with path.open() as stream:
        return list(csv.DictReader(stream))


def verify(directory):
    require(json.loads((directory / "complete.json").read_text()) == {"scenarios": 8, "transport_only": True}, "completion manifest")
    summaries = []
    for scenario in sorted(p for p in directory.iterdir() if p.is_dir()):
        run = json.loads((scenario / "run.json").read_text())
        n, ny, nz = run["counts"]
        h = [1.0 / n, 1.0 / ny, 1.0 / nz]
        cell_volume = math.prod(h)
        steps = rows(scenario / "steps.csv")
        require(len(steps) == run["steps"] and run["time"] == 0.5, "step clock")
        require(run["carrier_generation"] == run["volume_version"] == len(steps), "versions")
        require(run["free_surface_pressure"] is False and run["surface_reconstruction"] is False, "scope")
        c = run["requested_courant"]
        for index, step in enumerate(steps, 1):
            dt = float(step["dt"])
            require(dt == c * h[0] / 0.25, "actual interval")
            require(float(step["time"]) == index * dt, "per-step time")
            require(int(step["generation"]) == int(step["volume_version"]) == index, "per-step identities")
            require(float(step["carrier_divergence"]) <= 1e-5 and float(step["volume_divergence"]) <= 1e-5, "direct divergence gates")
            require(float(step["outward_courant"]) <= 1.0, "local Courant")
            require(abs(float(step["balance"])) <= float(step["rounding_budget"]), "volume budget")
            require(abs(float(step["volume_after"]) - 0.125) <= 1e-12, "represented volume")
            require(abs(float(step["mass_after"]) - 100.0) <= 1e-9, "represented mass")
            require(float(step["source"]) == 0.0, "no liquid sources")
            require(int(step["total_bytes"]) == int(step["owned_bytes"]) + int(step["boundary_bytes"]), "capacity sum")
        require(int(steps[0]["pressure_iterations"]) > 0, "actual nonzero carrier solve")
        cells = rows(scenario / "cells.csv")
        require(len(cells) == n * ny * nz, "cell count")
        fractions = {}
        oracle = [0.5 * math.fsum(math.comb(len(steps), k) * c**k * (1.0-c)**(len(steps)-k)
                  for k in range(len(steps)+1) if n//4 <= i-k < n//2) for i in range(n)]
        error = 0.0
        oracle_error = 0.0
        centroid_sum = []
        first_pressure = rows(scenario / "first-pressure.csv")
        require(len(first_pressure) == len(cells), "first pressure shape")
        pressure_error = 0.0
        for cell, row in enumerate(cells):
            i, j, k = [int(row[d]) for d in ["i", "j", "k"]]
            require(cell == i + n * (j + ny*k), "independent cell layout")
            f = float(row["fraction"])
            require(math.isfinite(f) and 0.0 <= f <= 1.0, "raw fraction bounds")
            require(math.isfinite(float(row["pressure"])), "finite accepted pressure")
            fractions[i,j,k] = f
            oracle_error = max(oracle_error, abs(f-oracle[i]))
            overlap = max(0.0, min((i+1)*h[0],0.625)-max(i*h[0],0.375)) / h[0]
            error += abs(f-0.5*overlap)*cell_volume
            require(float(row["x"]) == (i+0.5)*h[0], "cell positions")
            centroid_sum.append(f*float(row["x"])*cell_volume)
            first = first_pressure[cell]
            require(int(first["cell"]) == cell, "pressure layout")
            exact_pressure = -1000.0*0.25*(i*h[0])/float(steps[0]["dt"])
            pressure_error = max(pressure_error, abs(float(first["pressure"])-exact_pressure))
        require(oracle_error <= 2e-10, "every-cell independent binomial oracle")
        require(pressure_error <= 1e-5, "manufactured first gauge pressure")
        volume = math.fsum(fractions.values())*cell_volume
        require(abs(volume-0.125) <= 1e-12, "independent represented volume")
        centroid = math.fsum(centroid_sum)/volume
        require(abs(centroid-0.5) <= 1e-10, "translated centroid")
        faces = rows(scenario / "faces.csv")
        face_fields = [{} for _ in range(3)]
        velocity_error = 0.0
        for row in faces:
            axis = int(row["axis"])
            index = tuple(int(row[d]) for d in ["i","j","k"])
            require(index not in face_fields[axis], "duplicate face")
            value = float(row["velocity"])
            require(math.isfinite(value), "finite stored face")
            face_fields[axis][index] = value
            velocity_error = max(velocity_error, abs(value-(0.25 if axis==0 else 0.0)))
        face_count = (n+1)*ny*nz + n*(ny+1)*nz + n*ny*(nz+1)
        require(len(faces) == face_count, "face shapes")
        divergence = 0.0
        for index in fractions:
            terms = []
            for d in range(3):
                high = list(index); high[d] += 1
                terms.append((face_fields[d][tuple(high)]-face_fields[d][index])/h[d])
            divergence = max(divergence,abs(math.fsum(terms)))
        require(divergence <= 1e-5, "independently recomputed all-cell divergence")
        require(abs(divergence-float(steps[-1]["carrier_divergence"])) <= 1e-12, "carrier divergence report")
        require(abs(divergence-float(steps[-1]["volume_divergence"])) <= 1e-12, "liquid divergence report")
        require(velocity_error <= 1e-8, "projected constant carrier")
        require(run["owned_array_bytes"] == 16*face_count+80*n*ny*nz, "20-buffer facade capacity")
        require(run["boundary_array_bytes"] == 4*face_count+56*n*ny*nz, "existing boundary workspace capacity")
        for filename, field in [("initial.png", [0.5 if n//4 <= i < n//2 else 0.0 for i in range(n)]),
                                ("first.png", [0.5*((1-c)*float(n//4 <= i < n//2)+c*float(n//4 <= i-1 < n//2)) for i in range(n)]),
                                ("final.png", None)]:
            with Image.open(scenario / filename) as image:
                require(image.mode == "L" and image.size == (n,ny), "guidance dimensions/mode")
                pixels = list(image.tobytes())
            expected = []
            for j in reversed(range(ny)):
                for i in range(n):
                    integral = field[i] if field is not None else math.fsum(fractions[i,j,k]*h[2] for k in range(nz))
                    expected.append(math.floor(255*(-math.expm1(-8*integral))+0.5))
            require(pixels == expected, "independent occupancy projection: " + filename)
        summaries.append({"scenario":scenario.name,"method":run["method"],"cells":n*ny*nz,"n":n,
                          "courant":c,"steps":len(steps),"time":0.5,"liquid_volume":volume,"liquid_mass":volume*800,
                          "l1_volume_shape_error":error,"centroid_error":abs(centroid-0.5),
                          "max_binomial_fraction_error":oracle_error,"first_pressure_error_pa":pressure_error,
                          "actual_divergence_max":divergence,"uniform_velocity_error":velocity_error,
                          "owned_array_bytes":run["owned_array_bytes"],"boundary_array_bytes":run["boundary_array_bytes"]})
    require(len(summaries)==8,"scenario count")
    for method in ["jacobi-pcg-v1","sgs-pcg-v1"]:
        errors = [next(r["l1_volume_shape_error"] for r in summaries if r["method"]==method and r["n"]==n and r["courant"]==0.25) for n in [16,32,64]]
        require(errors[0]>errors[1]>errors[2],"spatial refinement")
        fine = next(r["l1_volume_shape_error"] for r in summaries if r["method"]==method and r["n"]==64 and r["courant"]==0.125)
        require(fine>errors[-1],"fixed-h smaller-dt diffusion limitation")
    return {"scenarios":summaries,"scope":"Transport-only fixed all-fluid carrier; no free-surface pressure, physical surface reconstruction, material calibration or performance qualification."}


if __name__ == "__main__":
    directory = Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent / "demo"
    print(json.dumps(verify(directory),indent=2))
