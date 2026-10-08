"""Render one independently checked native facet-load example; no physics run."""
import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path


def render(evidence, output):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d.art3d import Poly3DCollection

    evidence, output = Path(evidence).resolve(), Path(output).resolve()
    root = Path(__file__).resolve().parents[1]
    if output.exists() or output.is_relative_to(root):
        raise ValueError("fresh output directory outside repository required")
    receipt = json.loads((evidence / "qualification.json").read_text())
    if receipt.get("qualified") is not True:
        raise ValueError("qualified native comparison required")
    name = "varying-traction-centroid-counterexample"
    raw = (evidence / (name + ".stdout.json")).read_bytes()
    record = next(r for r in receipt["fixtures"] if r["name"] == name)
    if hashlib.sha256(raw).hexdigest() != record["stdout_sha256"]:
        raise ValueError("native output hash mismatch")
    fixture_raw = (evidence / (name + ".fixture.json")).read_bytes()
    if hashlib.sha256(fixture_raw).hexdigest() != receipt["evidence_sha256"][name + ".fixture.json"]:
        raise ValueError("fixture hash mismatch")
    fixture, native = json.loads(fixture_raw), json.loads(raw)
    points = [[float(Fraction(x)) for x in p] for p in fixture["vertices"]]
    forces = native["triangle_loads"][0]["nodal_force"]
    fig = plt.figure(figsize=(10, 6), facecolor="#f5f7fb")
    ax = fig.add_subplot(111, projection="3d", facecolor="#f5f7fb")
    ax.add_collection3d(Poly3DCollection([points], facecolors="#6b9ce2", alpha=.35,
                                      edgecolors="#304f7c", linewidths=2))
    for p, f in zip(points, forces):
        ax.quiver(*p, *f, color="#b34834", arrow_length_ratio=.22, linewidth=2.5)
        ax.text(p[0], p[1], p[2] + f[2] + .08, f"{f[2]:g} N", color="#833522")
    ax.set(xlim=(-.2, 2.3), ylim=(-.2, 1.3), zlim=(0, .9),
           xlabel="x (m)", ylabel="y (m)", zlabel="z (m)")
    ax.view_init(elev=24, azim=-63)
    fig.suptitle("Varying traction: actual consistent mesh forces", fontsize=16, x=.52, y=.96)
    fig.text(.04, .11, "Corner traction (N/m²):  0, (0, 0, 3), 0\n"
             f"Native force (N): {native['force']}\n"
             f"Native torque about origin (N m): {native['torque']}", fontsize=11)
    fig.text(.04, .025, "Arrows show nodal force vectors on the retained triangle. Static load example; no body or fluid advancement.", fontsize=9)
    fig.subplots_adjust(bottom=.24, top=.89, left=.05, right=.98)
    output.mkdir(parents=True)
    fig.savefig(output / "mesh-traction-example.jpg", dpi=170, pil_kwargs={"quality":85})
    plt.close(fig)
    manifest = {"source_head": receipt["source_head"], "source_dirty": receipt["source_dirty"],
                "native_output_sha256": hashlib.sha256(raw).hexdigest(), "physics_steps": 0,
                "jpeg_quality":85,
                "files_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                 for p in output.iterdir()}}
    (output / "render-receipt.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    print(json.dumps(render(args.evidence, args.output), indent=2))
