"""Render actual qualified observations and independently derived coefficients.

No simulator run or acceptance decision is made here. Inputs are the byte-equal
normal/optimized equation audit results, and their hashes accompany the figures.
"""
import csv
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

P = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=False)
INPUTS = [P / "consistency-qualification" / ("normal-" + x + ".json")
          for x in ("study", "taylor")]
study, taylor = [json.loads(p.read_text()) for p in INPUTS]
if study["status"] != "PASS" or len(study["rows"]) != 2:
    raise ValueError("qualified actual two-field study required")
for p in INPUTS:
    other = p.with_name(p.name.replace("normal-", "optimized-"))
    if p.read_bytes() != other.read_bytes():
        raise ValueError("actual mode equality required")

def exact_norm(values):
    return float(np.linalg.norm([float(Fraction(x)) for x in values]))

plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 9})
fig, axes = plt.subplots(2, 3, figsize=(14, 8), constrained_layout=True)
colors = {"initial": "#2864a0", "pressure_state": "#b55030"}
csv_rows = []
for field, repeated, exact in zip(study["rows"], study["repeated_pressure"], taylor["rows"]):
    kind = field["kind"]
    if kind != exact["kind"] or kind != repeated["kind"]:
        raise ValueError("matching field identities required")
    rows = field["rows"]
    h = np.array([r["interval"] for r in rows])
    color = colors[kind]
    def values(key):
        return np.array([r[key] for r in rows])
    axes[0, 0].loglog(h, values("actual_path_momentum_rate_norm"), "o-", color=color, label=kind)
    axes[0, 1].semilogx(h, values("integrated_actual_path_momentum_norm") / h**2,
                       "o-", color=color, label=kind + " observed")
    axes[0, 1].axhline(exact_norm(exact["actual_path_momentum_defect_coefficient"]),
                       linestyle="--", color=color, label=kind + " exact limit")
    axes[0, 2].loglog(h, values("pressure_endpoint_coefficient_error"), "o-", color=color,
                     label=kind + " endpoint")
    axes[0, 2].loglog(h, values("pressure_actual_average_coefficient_error"), "s--", color=color,
                     label=kind + " average")
    axes[1, 0].semilogx(h, values("first_pressure_difference_minus_true_derivative"),
                       "o-", color=color, label=kind + " observed")
    axes[1, 0].axhline(rows[0]["expected_nonvanishing_first_pressure_derivative_error"],
                       linestyle="--", color=color, label=kind + " exact limit")
    axes[1, 1].loglog([r["interval"] for r in repeated["rows"]],
                     [r["native_pressure_ALE_pullback_RMS_error"] for r in repeated["rows"]],
                     "o-", color=color, label=kind)
    for r, rp in zip(rows, repeated["rows"]):
        csv_rows.append({"kind": kind, "h": r["interval"],
                        "actual_momentum_rate_L2": r["actual_path_momentum_rate_norm"],
                        "actual_momentum_defect_over_h2_L2": r["integrated_actual_path_momentum_norm"] / r["interval"]**2,
                        "exact_momentum_defect_over_h2_limit_L2": exact_norm(exact["actual_path_momentum_defect_coefficient"]),
                        "pressure_endpoint_coefficient_L2_error": r["pressure_endpoint_coefficient_error"],
                        "pressure_average_coefficient_L2_error": r["pressure_actual_average_coefficient_error"],
                        "invalid_pressure_derivative_L2_error": r["first_pressure_difference_minus_true_derivative"],
                        "exact_invalid_pressure_derivative_L2_limit": r["expected_nonvanishing_first_pressure_derivative_error"],
                        "native_pressure_reference_area_RMS_error_at_T_0_1": rp["native_pressure_ALE_pullback_RMS_error"]})

pressure = taylor["rows"][1]
wrong = [max(abs(float(Fraction(v))) for v in pressure["frozen_divergence_counterexample"]["nonvanishing_strong_rate_defect"]),
         max(abs(float(Fraction(v))) for v in pressure["dropped_mass_derivative_counterexample"]["nonvanishing_momentum_rate_defect"])]
axes[1, 2].bar(["wrong D a = 0", "wrong omission of Mdot z"], wrong, color=["#704d89", "#88764c"])
axes[1, 2].set_yscale("log")
for i, value in enumerate(wrong):
    axes[1, 2].text(i, value * 1.2, f"{value:.6g}", ha="center")
titles = ["Actual momentum rate defect vanishes", "Scaled truncation coefficient remains nonzero",
          "Algebraic pressure value errors vanish", "Invalid evolved-pressure derivative comparison fails",
          "Repeated native pressure vs independent DAE", "Exact wrong-equation counterexamples"]
labels = ["||E_path|| / h", "||E_path|| / h²", "Coefficient L2 error", "||(Pi_h - p0)/h - pdot0||",
          "ALE pullback reference-area RMS error", "Nonvanishing maximum rate component"]
for i, ax in enumerate(axes.flat):
    ax.set_title(titles[i])
    ax.set_ylabel(labels[i])
    ax.grid(True, which="both", alpha=.2)
    if i < 5:
        ax.set_xlabel("Step interval h (decreases leftwards)")
        ax.legend(fontsize=7)
axes[1, 2].set_ylim(min(wrong) / 2, max(wrong) * 3)
fig.suptitle("Conditional temporal consistency with the declared semidiscrete ALE donor system\n"
             "Actual frozen native/research data; exact rational coefficients; no continuum spatial or pressure-rate claim", fontsize=13)
fig.savefig(OUT / "equation-consistency.png", dpi=170, metadata={"Software": "Rheon equation audit"})
fig.savefig(OUT / "equation-consistency.pdf", metadata={"Title": "Rheon equation consistency audit", "CreationDate": None, "ModDate": None})
plt.close(fig)
with (OUT / "equation-consistency.csv").open("w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=list(csv_rows[0]))
    writer.writeheader()
    writer.writerows(csv_rows)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
(OUT / "receipt.json").write_text(json.dumps({"status": "PASS", "actual_observed_rows": len(csv_rows),
    "input_sha256": {str(p.relative_to(P.parent.parent)): sha(p) for p in INPUTS},
    "renderer_sha256": sha(Path(__file__)), "output_sha256": {p.name: sha(p) for p in sorted(OUT.iterdir())},
    "scope": "Actual qualified numerical observations with independent exact limiting coefficients; wrong-equation bars are exact counterexamples, not observed native failures."}, indent=2) + "\n")
print("PASS rendered ten actual observations and independent limiting coefficients")
