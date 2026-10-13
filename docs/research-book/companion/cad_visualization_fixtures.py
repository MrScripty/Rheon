"""Independent analytical/synthetic lesson fixtures, not Rheon solver tests.

Run from the repository root:
    python3 docs/research-book/companion/cad_visualization_fixtures.py \
        --output .generated/cad-visualization/numerical-fixtures.json
Requires NumPy. Writes generated output only to the explicit destination.
Source: ../implementation/cad-visualization-contracts.md, Sections 8 and 15.
These mathematical witnesses do not exercise Rheon, its UI, or a CAD solver.
"""
from pathlib import Path
import argparse
import json
import math
import numpy as np


def close(actual, expected, tolerance=1e-12):
    if not np.allclose(actual, expected, rtol=tolerance, atol=tolerance):
        raise AssertionError((actual, expected))


def tensor_quantities(a):
    a = np.asarray(a, dtype=float)
    s = (a + a.T) / 2
    omega = (a - a.T) / 2
    curl = [a[2, 1] - a[1, 2], a[0, 2] - a[2, 0], a[1, 0] - a[0, 1]]
    q_rot = (np.sum(omega * omega) - np.sum(s * s)) / 2
    q_inv = (np.trace(a) ** 2 - np.trace(a @ a)) / 2
    eigenvalues = np.linalg.eigvalsh(s @ s + omega @ omega)
    return {
        "gradient_per_s": a.tolist(),
        "divergence_per_s": float(np.trace(a)),
        "vorticity_per_s": [float(v) for v in curl],
        "q_rotation_per_s2": float(q_rot),
        "q_invariant_per_s2": float(q_inv),
        "lambda2_per_s2": float(eigenvalues[1]),
    }


def generate():
    tensors = {
        "translation": tensor_quantities([[0, 0, 0], [0, 0, 0], [0, 0, 0]]),
        "rotation_a_2": tensor_quantities([[0, -2, 0], [2, 0, 0], [0, 0, 0]]),
        "shear_gamma_2": tensor_quantities([[0, 2, 0], [0, 0, 0], [0, 0, 0]]),
        "strain_a_2": tensor_quantities([[2, 0, 0], [0, -2, 0], [0, 0, 0]]),
        "expansion_a_2": tensor_quantities([[2, 0, 0], [0, 2, 0], [0, 0, 2]]),
    }
    close(tensors["rotation_a_2"]["vorticity_per_s"], [0, 0, 4])
    close([tensors["rotation_a_2"][k] for k in ["q_rotation_per_s2", "q_invariant_per_s2", "lambda2_per_s2"]], [4, 4, -4])
    close([tensors["shear_gamma_2"][k] for k in ["q_rotation_per_s2", "q_invariant_per_s2", "lambda2_per_s2"]], [0, 0, 0])
    close([tensors["strain_a_2"][k] for k in ["q_rotation_per_s2", "q_invariant_per_s2", "lambda2_per_s2"]], [-4, -4, 4])
    close([tensors["expansion_a_2"][k] for k in ["divergence_per_s", "q_rotation_per_s2", "q_invariant_per_s2", "lambda2_per_s2"]], [6, -6, 12, 4])

    H, W, L, mu, rho, G = .01, .02, .1, .001, 1000., 1.
    channel = {
        "input": {"height_m": H, "represented_width_m": W, "length_m": L, "mu_Pa_s": mu, "rho_kg_m3": rho, "minus_dp_dx_Pa_m": G},
        "assumption": "Planar fully developed laminar flow, no influential sidewalls; not a Rheon run.",
        "mean_speed_m_s": G * H**2 / (12 * mu),
        "max_speed_m_s": G * H**2 / (8 * mu),
        "volume_flux_m3_s": W * G * H**3 / (12 * mu),
        "pressure_drop_Pa": G * L,
        "wall_shear_magnitude_Pa": G * H / 2,
    }
    channel["reynolds_height"] = rho * channel["mean_speed_m_s"] * H / mu
    channel["reynolds_twice_height"] = 2 * channel["reynolds_height"]
    channel["profile"] = [{"y_m": float(y), "u_m_s": float(G*y*(H-y)/(2*mu))} for y in np.linspace(0, H, 9)]
    close(channel["max_speed_m_s"], 1.5 * channel["mean_speed_m_s"])

    h = np.array([.25, .125, .0625])
    f = 1 + 2*h*h
    f3, f2, f1 = f
    p = math.log(abs((f3-f2)/(f2-f1))) / math.log(2)
    extrapolated = f1 + (f1-f2)/(2**p-1)
    gci_percent = 100*1.25*abs((f1-f2)/f1)/(2**p-1)
    close(p, 2)
    close(extrapolated, 1)
    convergence = {"grid_spacing": h.tolist(), "synthetic_quantity": f.tolist(), "observed_order": p, "extrapolated": float(extrapolated), "fine_gci_percent": float(gci_percent), "safety_factor": 1.25}

    def poly6(r, support):
        return 315/(64*math.pi*support**9) * max(support**2-r*r, 0)**3

    support, spacing, mass = .2, .1, 1.
    positions = np.array([[i*spacing,j*spacing,k*spacing] for i in range(-2,3) for j in range(-2,3) for k in range(-2,3)])
    def density(x):
        return sum(mass*poly6(float(np.linalg.norm(p-x)), support) for p in positions)
    radius = np.linspace(0, support, 2001)
    integrand = np.array([4*math.pi*r*r*poly6(float(r), support) for r in radius])
    kernel_integral = support/2000/3 * (integrand[0]+integrand[-1]+4*sum(integrand[1:-1:2])+2*sum(integrand[2:-1:2]))
    close(kernel_integral, 1, 1e-10)
    kernel = {
        "data_origin": "Synthetic particle lattice and conventional poly6 reference kernel; not Rheon output.",
        "spacing_m": spacing, "support_radius_m": support, "particle_mass_kg": mass,
        "lattice_index_each_axis": [-2,-1,0,1,2], "particle_count": len(positions),
        "total_mass_kg": float(len(positions)*mass), "reference_density_kg_m3": 1000,
        "reference_parcel_volume_m3": .001,
        "kernel_integral_numeric": float(kernel_integral),
        "center_density_kg_m3": float(density(np.array([0,0,0]))),
        "outer_face_density_kg_m3": float(density(np.array([-.2,0,0]))),
        "warning": "Boundary density difference is kernel-support truncation in this fixture; normalization of a display attribute does not correct the physical method.",
    }

    trajectories = {"U_m_s": 1., "a_m_s2": 1., "observation_time_s": 2., "pathline_release_time_s": 0., "samples": []}
    for x in np.linspace(0,2,9):
        trajectories["samples"].append({"x_m": float(x), "streamline_y_m": float(2*x), "pathline_y_m": float(x*x/2), "streakline_y_m": float(2*x-x*x/2)})
    close(trajectories["samples"][-1]["pathline_y_m"], 2)
    close(trajectories["samples"][-1]["streakline_y_m"], 2)

    # Uniform pressure on six unit-area faces of a centered cube.
    normals = np.array([[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]], dtype=float)
    centers = normals / 2
    tractions = -100*normals
    close(np.sum(tractions, axis=0), [0,0,0])
    close(np.sum(np.cross(centers, tractions), axis=0), [0,0,0])
    loads = {"closed_unit_cube_pressure_Pa": 100, "net_force_N": [0,0,0], "net_moment_about_center_N_m": [0,0,0], "open_patch_normal": [1,0,0], "open_patch_area_m2": 2, "open_patch_pressure_Pa": 100, "open_patch_force_N": [-200,0,0]}

    output = {
        "schema_version": 1,
        "data_origin": "Analytical/synthetic research fixtures. No Rheon solver was run.",
        "verification": "Assertions passed using independent arithmetic/tensor calculations; tolerances here validate these calculations, not production solver accuracy.",
        "tensor_cases": tensors, "planar_channel": channel, "convergence": convergence,
        "poly6_lattice": kernel, "time_dependent_curves": trajectories, "pressure_loads": loads,
    }
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="Generated JSON destination, outside tracked source")
    args = parser.parse_args()
    output = generate()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    print(f"All analytical fixture checks passed. Wrote {args.output}.")


if __name__ == "__main__":
    main()
