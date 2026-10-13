"""Balanced whole-step initial-residual experiment; preserve every sample."""
import json
import random
import subprocess
import sys
from pathlib import Path

baseline, candidate, output = sys.argv[1:]
random.seed(602)
rows = []
for workers in [1, 4]:
    for observed in [False, True]:
        orders = [["baseline", "candidate"], ["candidate", "baseline"]] * 6
        random.shuffle(orders)
        for trial, order in enumerate(orders):
            pair = {}
            for label in order:
                binary = baseline if label == "baseline" else candidate
                command = [binary, "32", str(workers), "128", "adaptive", "jacobi-pcg-v1", "rest"]
                if observed:
                    command.append("observed")
                result = subprocess.run(command, capture_output=True, text=True, check=True)
                fields = {}
                for line in result.stdout.splitlines()[:2]:
                    fields.update(item.split("=", 1) for item in line.split(" samples=")[0].split())
                pair[label] = fields
                rows.append(dict(version=label, workers=workers, observed=observed,
                                 trial=trial, order=order, command=command,
                                 fields=fields, stdout=result.stdout))
                Path(output).write_text(json.dumps(rows, indent=2) + "\n")
            for key in ["steps", "accepted_time", "divergence", "managed_bytes", "capture_bytes",
                        "fingerprint", "iterations", "residual", "report_fingerprint"]:
                if pair["baseline"][key] != pair["candidate"][key]:
                    raise RuntimeError((workers, observed, trial, key))
            print(workers, observed, trial, pair['baseline']['frame_seconds'], pair['candidate']['frame_seconds'], flush=True)
