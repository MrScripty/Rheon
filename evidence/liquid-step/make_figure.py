"""Render native guidance images and independently measured shape errors."""
import json
from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from PIL import Image

root = Path(__file__).resolve().parent
case = root / "demo/jacobi-pcg-v1-n64-c025"
data = json.loads((root / "numerical-summary.json").read_text())["scenarios"]
fig = plt.figure(figsize=(10, 6), layout="constrained")
grid = fig.add_gridspec(3, 3, height_ratios=[1, 1, 1.4])
for col, (name, title) in enumerate([
        ("initial.png", "Accepted initial fraction"),
        ("first.png", "First coupled publication"),
        ("final.png", "Accepted t = 0.5 s")]):
    axis = fig.add_subplot(grid[0, col])
    with Image.open(case / name) as im:
        axis.imshow(im, cmap="gray", vmin=0, vmax=255, interpolation="nearest", aspect="auto")
    axis.set_title(title, fontsize=10)
    axis.set_xticks([]); axis.set_yticks([])
line = fig.add_subplot(grid[1, :])
import csv
with (case / "cells.csv").open() as stream:
    cells = [r for r in csv.DictReader(stream) if r["j"] == "0" and r["k"] == "0"]
line.plot([float(r["x"]) for r in cells], [float(r["fraction"]) for r in cells], label="Stored fraction (Jacobi, n = 64)")
line.step([0, 0.375, 0.625, 1], [0, 0.5, 0, 0], where="post", label="Exact translated overlap", linestyle="--")
line.set_xlim(0,1);line.set_ylim(-0.02,0.55)
line.set_xlabel("x (m)");line.set_ylabel("Liquid fraction");line.legend(fontsize=8)
axis = fig.add_subplot(grid[2, :2])
for method, marker in [("jacobi-pcg-v1","o"),("sgs-pcg-v1","x")]:
    points = sorted((r for r in data if r["method"] == method and r["courant"] == 0.25),key=lambda r:r["n"])
    axis.plot([r["n"] for r in points],[r["l1_volume_shape_error"] for r in points],marker=marker,label=method)
axis.set_xlabel("X cells (fixed 8 × 4 transverse cells)")
axis.set_ylabel("L1 volume shape error (m³)")
axis.legend(fontsize=8);axis.grid(alpha=0.2)
note = fig.add_subplot(grid[2,2]);note.axis("off")
note.text(0,0.95,"Both carrier methods:\n0.125 m³ represented volume\n100 kg represented mass\nMatched end clocks: 0.5 s\n\nAt n = 64:\nC = 0.25: error 0.030195\nC = 0.125: error 0.032637\nSmaller dt adds diffusion.\n\nFraction-integral guidance;\nno reconstructed surface.",va="top",fontsize=9)
fig.suptitle("Rheon: transport-only coupled carrier / liquid-volume evidence",fontsize=14)
fig.savefig(root / "guidance-comparison.png",dpi=160,metadata={"Software":"Rheon evidence / Matplotlib"})
plt.close(fig)
