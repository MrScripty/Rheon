"""Generate bounded computed fixtures into a fresh external directory."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import platform

import numpy as np

from oracles import acoustic_density, acoustic_phase_speed, sod_exact, sod_star
from reference import Config, acoustic_initial, diagnostics, integrate, primitive, sod_initial


SOURCE_FILES = ("reference.py", "oracles.py", "generate.py", "test_reference.py")


def source_hashes():
    root = Path(__file__).resolve().parent
    return {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in SOURCE_FILES}


def qualify():
    """Measured refinement evidence with independent continuum references."""
    amplitude, end_time = 1e-6, 0.08
    acoustic_errors, speeds, attenuation, acoustic_probes = [], [], [], []
    for n in (8, 12, 20):
        config = Config((n, n, n))
        initial = acoustic_initial(config, amplitude)
        final = integrate(initial, config, [0, end_time])[-1][1]
        expected = acoustic_density(config.shape, end_time, amplitude)
        acoustic_errors.append(float(np.mean(np.abs(final[..., 0] - expected)) / amplitude))
        speed, ratio = acoustic_phase_speed(initial[..., 0], final[..., 0], end_time)
        speeds.append(speed)
        attenuation.append(ratio)
        acoustic_probes.append({"shape": list(config.shape), "initial_density": initial[..., 0].ravel().tolist(),
                                "final_density": final[..., 0].ravel().tolist()})
    orders = [float(np.log(a / b) / np.log(nb / na)) for a, b, na, nb in
              zip(acoustic_errors, acoustic_errors[1:], (8, 12), (12, 20))]
    sod_errors, pressure_errors, sod_probes = [], [], []
    for n in (24, 48, 64):
        config = Config((n, 2, 2))
        final = integrate(sod_initial(config), config, [0, end_time])[-1][1]
        density, _, pressure = primitive(final, config.gamma)
        x = (np.arange(n) + 0.5) / n
        rho_exact, _, p_exact = sod_exact(x, end_time)
        interior = (x > 0.2) & (x < 0.8)
        sod_errors.append(float(np.mean(np.abs(density[interior, 0, 0] - rho_exact[interior]))))
        pressure_errors.append(float(np.mean(np.abs(pressure[interior, 0, 0] - p_exact[interior]))))
        sod_probes.append({"shape": list(config.shape), "final_density_profile": density[:, 0, 0].tolist(),
                           "final_pressure_profile": pressure[:, 0, 0].tolist()})
    sound = float(np.sqrt(1.4))
    speed_error = abs(speeds[-1] / sound - 1)
    acoustic_pass = all(b < a for a, b in zip(acoustic_errors, acoustic_errors[1:])) and min(orders) > 0.6 and speed_error < 0.08
    sod_pass = all(b < a for a, b in zip(sod_errors, sod_errors[1:])) and pressure_errors[-1] < 0.04
    if not acoustic_pass or not sod_pass:
        raise RuntimeError(f"continuum checks failed: acoustic={acoustic_errors}, speed={speeds}; Sod={sod_errors}")
    pstar, ustar = sod_star()
    return {"status": "satisfied", "scope": "bounded Python reference only",
            "acoustic": {"grid_sizes": [8, 12, 20], "amplitude": amplitude, "time": end_time,
                         "normalized_density_l1_errors": acoustic_errors, "observed_orders": orders,
                         "phase_speeds": speeds, "expected_sound_speed": sound,
                         "finest_relative_phase_speed_error": speed_error, "amplitude_ratios": attenuation,
                         "probes": acoustic_probes,
                         "acceptance": "errors decrease; observed orders > 0.6; finest sound-speed error < 8%"},
            "sod": {"grid_shapes": [[24, 2, 2], [48, 2, 2], [64, 2, 2]], "time": end_time,
                    "comparison_window": [0.2, 0.8], "density_l1_errors": sod_errors,
                    "pressure_l1_errors": pressure_errors, "exact_star_pressure": pstar,
                    "exact_star_velocity": float(ustar),
                    "probes": sod_probes,
                    "acceptance": "density errors decrease; finest pressure L1 error < 0.04"}}


def compact(array):
    # Full binary64 decimal round-trip values preserve conservative identities.
    return np.asarray(array).tolist()


def build_case(case_id, initializer, title, end_time):
    config = Config()
    initial = initializer(config)
    initial_totals = np.sum(initial, axis=(0, 1, 2)) / np.prod(config.shape)
    frames, density_values, pressure_values, speed_values = [], [], [], []
    for time, state, steps in integrate(initial, config, np.linspace(0, end_time, 7)):
        density, velocity, pressure = primitive(state, config.gamma)
        diag = diagnostics(state, config, initial_totals, steps)
        if case_id == "planar_sod_periodic":
            x = (np.arange(config.shape[0]) + 0.5) / config.shape[0]
            expected, _, _ = sod_exact(x, time)
            window = (x > 0.2) & (x < 0.8)
            error = np.mean(np.abs(density[window, 0, 0] - expected[window]))
        else:
            error = np.mean(np.abs(density - acoustic_density(config.shape, time, 0.02, config.gamma)))
        diag["density_l1_reference_error"] = float(error)
        frames.append({"time": time, "conserved": compact(state.reshape(-1, 5)),
                       "density": compact(density.ravel()), "velocity": compact(velocity.reshape(-1, 3)),
                       "pressure": compact(pressure.ravel()), "diagnostics": diag})
        density_values.extend((float(density.min()), float(density.max())))
        pressure_values.extend((float(pressure.min()), float(pressure.max())))
        speed = np.linalg.norm(velocity, axis=-1)
        speed_values.extend((float(speed.min()), float(speed.max())))
    return {"id": case_id, "title": title,
            "grid": {"shape": list(config.shape), "extent": [[0, 1], [0, 1], [0, 1]],
                     "centering": "cell", "flatten_order": "C (z fastest)"},
            "parameters": {"gamma": config.gamma, "cfl": config.cfl, "boundary": "periodic",
                           "seed": 0, "randomness": "none", "dimensionless_end_time": end_time,
                           "initial_condition": "Sod left (rho,u,p)=(1,0,1), right=(0.125,0,0.1), jump x=0.5; second periodic jump x=0"
                           if case_id == "planar_sod_periodic" else "linear acoustic modes k=(1,1,1),(1,-1,2); amplitudes 0.02,0.01; phases 0,pi/3; rho0=p0=1",
                           "reference_error_scope": "isolated central Sod fan, 0.2<x<0.8, before periodic interaction"
                           if case_id == "planar_sod_periodic" else "linear acoustics; finite amplitude also introduces nonlinear discrepancy"},
            "field_ranges": {"density": [min(density_values), max(density_values)],
                             "pressure": [min(pressure_values), max(pressure_values)],
                             "speed": [min(speed_values), max(speed_values)]}, "frames": frames}


def validate_results(results):
    """Producer boundary check; consumers should also check their own contract."""
    if results["schema_version"] != 1 or results["provenance"]["source_sha256"] != source_hashes():
        raise ValueError("invalid schema or stale source provenance")
    for case in results["cases"]:
        shape = tuple(case["grid"]["shape"])
        config = Config(shape, case["parameters"]["gamma"], case["parameters"]["cfl"])
        initial_totals = np.asarray(case["frames"][0]["diagnostics"]["conserved_totals"])
        previous_time = -1.0
        for frame in case["frames"]:
            state = np.asarray(frame["conserved"]).reshape(shape + (5,))
            density, velocity, pressure = primitive(state, config.gamma)
            if frame["time"] <= previous_time:
                raise ValueError("nonincreasing frame times")
            previous_time = frame["time"]
            for name, expected in (("density", density.ravel()), ("pressure", pressure.ravel()),
                                   ("velocity", velocity.reshape(-1, 3))):
                if not np.array_equal(np.asarray(frame[name]), expected):
                    raise ValueError(f"primitive mismatch: {name}")
            actual = diagnostics(state, config, initial_totals, frame["diagnostics"]["steps"])
            for key, value in actual.items():
                if not np.allclose(value, frame["diagnostics"][key], rtol=1e-13, atol=1e-14):
                    raise ValueError(f"diagnostic mismatch: {key}")
            if actual["relative_conservation_drift"] > 2e-13:
                raise ValueError("periodic conservation failed")
    validate_record_origin(results)
    validate_qualification(results["qualification"])
    max_drift = max(frame["diagnostics"]["relative_conservation_drift"]
                    for case in results["cases"] for frame in case["frames"])
    if not np.isclose(max_drift, results["qualification"]["max_relative_conservation_drift"], rtol=1e-13, atol=1e-16):
        raise ValueError("qualification mismatch: max_relative_conservation_drift")


def validate_record_origin(results):
    """Replay bounded records with this solver; analytic accuracy is separate.

    This checks that a packet records the declared computation, rather than
    accepting self-consistent but unevolved fields. It is neither an independent
    accuracy oracle nor a proof of floating-point solver correctness.
    """
    initializers = {"planar_sod_periodic": sod_initial, "oblique_acoustic": acoustic_initial}
    for case in results["cases"]:
        if case["id"] not in initializers:
            raise ValueError("unsupported recording case")
        config = Config(tuple(case["grid"]["shape"]), case["parameters"]["gamma"], case["parameters"]["cfl"])
        end_time = case["parameters"]["dimensionless_end_time"]
        if not np.isfinite(end_time) or not 0 < end_time <= 1:
            raise ValueError("invalid recording end time")
        times = np.linspace(0, end_time, 7)
        if len(case["frames"]) != len(times) or not np.allclose(
                [frame["time"] for frame in case["frames"]], times, rtol=0, atol=1e-15):
            raise ValueError("recording time grid mismatch")
        expected = integrate(initializers[case["id"]](config), config, times)
        for index, (frame, (_, state, steps)) in enumerate(zip(case["frames"], expected)):
            if frame["diagnostics"]["steps"] != steps:
                raise ValueError(f"recording step count mismatch: {case['id']} frame {index}")
            if not np.allclose(np.asarray(frame["conserved"]), state.reshape(-1, 5), rtol=1e-13, atol=1e-14):
                raise ValueError(f"recording state mismatch: {case['id']} frame {index}")

    # Refinement probes must record this computation too; consistent error
    # summaries from unrelated probe fields do not establish their origin.
    expected_report = qualify()
    report = results["qualification"]
    if set(report) != set(expected_report) | {"max_relative_conservation_drift"}:
        raise ValueError("qualification replay shape mismatch")

    def match(actual, expected, path):
        if isinstance(expected, dict):
            if not isinstance(actual, dict) or set(actual) != set(expected):
                raise ValueError("qualification replay shape mismatch: " + path)
            for key, value in expected.items():
                match(actual[key], value, path + "." + key)
        elif isinstance(expected, list):
            if not isinstance(actual, list) or len(actual) != len(expected):
                raise ValueError("qualification replay shape mismatch: " + path)
            for index, value in enumerate(expected):
                match(actual[index], value, path + f"[{index}]")
        elif isinstance(expected, float):
            if type(actual) not in (int, float) or not math.isclose(actual, expected, rel_tol=1e-13, abs_tol=1e-14):
                raise ValueError("qualification replay mismatch: " + path)
        elif type(actual) is not type(expected) or actual != expected:
            raise ValueError("qualification replay mismatch: " + path)

    for key, value in expected_report.items():
        match(report[key], value, key)


def validate_qualification(report):
    """Recompute continuum errors from recorded probes, without time stepping."""
    if report["status"] != "satisfied":
        raise ValueError("qualification status is not satisfied")
    acoustic, sod = report["acoustic"], report["sod"]
    if acoustic["grid_sizes"] != [8, 12, 20] or acoustic["amplitude"] != 1e-6 or acoustic["time"] != 0.08:
        raise ValueError("unsupported acoustic qualification parameters")
    if sod["grid_shapes"] != [[24, 2, 2], [48, 2, 2], [64, 2, 2]] or sod["time"] != 0.08 or sod["comparison_window"] != [0.2, 0.8]:
        raise ValueError("unsupported Sod qualification parameters")
    errors, speeds, ratios = [], [], []
    if len(acoustic["probes"]) != 3 or len(sod["probes"]) != 3:
        raise ValueError("missing qualification probes")
    for n, probe in zip(acoustic["grid_sizes"], acoustic["probes"]):
        shape = (n, n, n)
        if probe["shape"] != list(shape):
            raise ValueError("acoustic probe shape mismatch")
        initial = np.asarray(probe["initial_density"]).reshape(shape)
        final = np.asarray(probe["final_density"]).reshape(shape)
        if not np.all(np.isfinite(final)) or np.any(final <= 0):
            raise ValueError("invalid acoustic probe density")
        if not np.allclose(initial, acoustic_density(shape, 0, acoustic["amplitude"]), rtol=0, atol=5e-16):
            raise ValueError("acoustic probe initial condition mismatch")
        exact = acoustic_density(shape, acoustic["time"], acoustic["amplitude"])
        errors.append(float(np.mean(np.abs(final - exact)) / acoustic["amplitude"]))
        speed, ratio = acoustic_phase_speed(initial, final, acoustic["time"])
        speeds.append(speed)
        ratios.append(ratio)
    orders = [float(np.log(a / b) / np.log(nb / na)) for a, b, na, nb in
              zip(errors, errors[1:], (8, 12), (12, 20))]
    sound = float(np.sqrt(1.4))
    speed_error = abs(speeds[-1] / sound - 1)
    measured = {"normalized_density_l1_errors": errors, "observed_orders": orders, "phase_speeds": speeds,
                "expected_sound_speed": sound, "finest_relative_phase_speed_error": speed_error, "amplitude_ratios": ratios}
    for key, actual in measured.items():
        if not np.allclose(actual, acoustic[key], rtol=1e-13, atol=1e-14):
            raise ValueError(f"qualification mismatch: acoustic.{key}")
    if not all(b < a for a, b in zip(errors, errors[1:])) or min(orders) <= 0.6 or speed_error >= 0.08:
        raise ValueError("acoustic continuum acceptance failed")
    errors, pressure_errors = [], []
    for shape, probe in zip(sod["grid_shapes"], sod["probes"]):
        if shape != probe["shape"]:
            raise ValueError("Sod probe shape mismatch")
        density = np.asarray(probe["final_density_profile"])
        pressure = np.asarray(probe["final_pressure_profile"])
        n = shape[0]
        if density.shape != (n,) or pressure.shape != (n,) or not np.all(np.isfinite(density)) or not np.all(np.isfinite(pressure)) or np.any(density <= 0) or np.any(pressure <= 0):
            raise ValueError("invalid Sod probe profiles")
        x = (np.arange(n) + 0.5) / n
        rho_exact, _, p_exact = sod_exact(x, sod["time"])
        interior = (x > 0.2) & (x < 0.8)
        errors.append(float(np.mean(np.abs(density[interior] - rho_exact[interior]))))
        pressure_errors.append(float(np.mean(np.abs(pressure[interior] - p_exact[interior]))))
    pstar, ustar = sod_star()
    for key, actual in {"density_l1_errors": errors, "pressure_l1_errors": pressure_errors,
                        "exact_star_pressure": pstar, "exact_star_velocity": ustar}.items():
        if not np.allclose(actual, sod[key], rtol=1e-13, atol=1e-14):
            raise ValueError(f"qualification mismatch: sod.{key}")
    if not all(b < a for a, b in zip(errors, errors[1:])) or pressure_errors[-1] >= 0.04:
        raise ValueError("Sod continuum acceptance failed")


def generate(output):
    output = Path(output).resolve()
    if output.exists():
        raise ValueError("output must be a fresh nonexistent directory")
    # Managed environments may contain empty read-only .git sentinel directories.
    # A real checkout has a worktree pointer file or a repository HEAD.
    if any((parent / ".git").is_file() or (parent / ".git" / "HEAD").is_file()
           for parent in (output, *output.parents)):
        raise ValueError("generated output must be outside every Git checkout")
    qualification = qualify()
    cases = [build_case("planar_sod_periodic", sod_initial, "Planar Sod shock tube in a periodic 3D cube", 0.08),
             build_case("oblique_acoustic", acoustic_initial, "Two crossing 3D acoustic waves", 0.08)]
    qualification["max_relative_conservation_drift"] = max(frame["diagnostics"]["relative_conservation_drift"] for case in cases for frame in case["frames"])
    results = {"schema_version": 1, "provenance": {"source_sha256": source_hashes(),
               "python": platform.python_version(), "numpy": np.__version__,
               "method": "unsplit first-order conservative finite volume; Rusanov/LLF; explicit Euler",
               "units": "dimensionless", "state_order": ["rho", "rho_u", "rho_v", "rho_w", "total_energy"]},
               "limitations": ["educational tiny Python reference; not the production Rust solver",
                               "inviscid ideal gas on uniform periodic Cartesian cells",
                               "no turbulence, geometry, viscosity, or physical validation claim",
                               "first-order diffusion damps small-grid waves and smears shocks"],
               "cases": cases, "qualification": qualification}
    validate_results(results)
    output.mkdir(parents=True)
    (output / "results.json").write_text(json.dumps(results, separators=(",", ":"), allow_nan=False) + "\n")
    (output / "qualification.json").write_text(json.dumps(qualification, indent=2, allow_nan=False) + "\n")
    return qualification


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="fresh external output directory")
    args = parser.parse_args()
    report = generate(args.output)
    summary = {**report, "acoustic": {k: v for k, v in report["acoustic"].items() if k != "probes"},
               "sod": {k: v for k, v in report["sod"].items() if k != "probes"}}
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
