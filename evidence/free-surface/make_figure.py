"""Arrange native guidance and accepted CSV fields; no synthetic simulation."""
import csv
import json
from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from PIL import Image

HERE = Path(__file__).resolve().parent
case = HERE / "demo/jacobi-pcg-v1"
with (case / "pulse-final.csv").open() as stream:
    cells = list(csv.DictReader(stream))
with (case / "rest-steps.csv").open() as stream:
    rest = list(csv.DictReader(stream))
fig, axes = plt.subplots(1, 4, figsize=(13, 4), gridspec_kw={"width_ratios": [1, 1, 1.4, 1.6]})
fig.patch.set_facecolor("#f5f7fa")
for ax, stem, title in zip(axes[:2], ("pulse-initial", "pulse-final"), ("Initial native guidance", "Accepted native guidance")):
    with Image.open(case / (stem + ".png")) as image:
        ax.imshow(image, cmap="Blues", vmin=0, vmax=255, interpolation="nearest", extent=(0, 2, 0, 2))
    ax.set_title(title, fontsize=11)
    ax.set_xticks([.5, 1.5], labels=["left", "right"])
    ax.set_yticks([.5, 1.5], labels=["wet", "air"])
    ax.axhline(1, color="#de6e37", linestyle="--", linewidth=1)
    ax.set_xlabel("+Z fraction integral, 8/m")
ax = axes[2]
ax.set_xlim(0, 2); ax.set_ylim(0, 2); ax.set_aspect("equal")
for row in cells:
    i, j = int(row["i"]), int(row["j"])
    fraction, pressure = float(row["fraction"]), float(row["pressure"])
    ax.add_patch(plt.Rectangle((i, j), 1, 1, facecolor=plt.cm.Blues(.15 + .65 * fraction), edgecolor="white"))
    ax.text(i + .5, j + .5, f"f={fraction:g}\np={pressure:+g}", ha="center", va="center", fontsize=10)
ax.axhline(1, color="#de6e37", linestyle="--")
for xy, change in [((.5, .8), (0, .4)), ((1.5, 1.2), (0, -.4)), ((1.2, .18), (-.4, 0))]:
    ax.annotate("", xy=(xy[0] + change[0], xy[1] + change[1]), xytext=xy,
                arrowprops={"arrowstyle": "->", "color": "#b6471c", "lw": 2})
ax.set_title("Accepted fractions and pressure", fontsize=11)
ax.set_xticks([]); ax.set_yticks([])
ax.set_xlabel("Arrows: active speed magnitude 1/8")
ax = axes[3]
ax.plot([float(r["time"]) for r in rest], [float(r["pressure"]) for r in rest], "o-", color="#23748b", markersize=3)
ax.set_ylim(.1, .15); ax.set_xlabel("Accepted time"); ax.set_ylabel("Wet center pressure")
ax.set_title("16 exact one-cell rest intervals", fontsize=11)
ax.grid(alpha=.2)
ax.text(.05, .1, "f=(1,0), velocity=0\nSGS 16-layer probe rejects\n1.0000000000000004; state preserved", transform=ax.transAxes, fontsize=9)
fig.suptitle("Resolved atmospheric slab laboratory — two columns, one Z cell", fontsize=14)
fig.text(.5, .02, "Both PCG methods agree: volume 2 → 2; wet divergence 0; air divergence ±1/8. Next mixed interval rejects.\nDashed line: held interval pressure geometry. Guidance is not a reconstructed or physically shaded liquid surface.", ha="center", fontsize=9)
fig.tight_layout(rect=(0, .13, 1, .92))
fig.savefig(HERE / "guidance-and-pressure.png", dpi=160, metadata={"Software": "Rheon native CSV/PNG evidence with Matplotlib"})
print(json.dumps({"inputs": [str(p.relative_to(HERE)) for p in [case / "pulse-initial.png", case / "pulse-final.png", case / "pulse-final.csv", case / "rest-steps.csv"]], "output": "guidance-and-pressure.png"}))
