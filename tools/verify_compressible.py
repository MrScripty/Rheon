#!/usr/bin/env python3
"""Independent S1 cell-average continuum checks and fresh native-origin capture.

Continuum oracles never import a numerical flux. The educational LLF is imported
only inside the separately labelled S1-D differential check. NumPy pin: 2.3.5.
Format and proof limits: docs/plans/cpu-cfd-validation/reports/s1-export-proposal.md.
"""
import argparse
import csv
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tarfile
from types import SimpleNamespace
import uuid

import numpy as np

REPO = Path(__file__).resolve().parents[1]
GAMMA = 1.4
AMPLITUDE = 1e-6
END_TIME = 0.08
MODES = (((1, 1, 1), 1.0, 0.0), ((1, -1, 2), 0.5, math.pi/3))
EPS = np.finfo(float).eps
CHANNELS = ["rho", "mx", "my", "mz", "E"]
EXPECTED_CASES = {("uniform", (3, 2, 1)), ("differential", (3, 4, 2))}
EXPECTED_CASES |= {("acoustic", (n, n, n)) for n in (8, 12, 20)}
EXPECTED_CASES |= {("sod", (n, 2, 2)) for n in (24, 48, 64)}


class ValidationError(ValueError):
    """A named S1 input, numerical, or origin gate was not established."""


def require(condition, diagnostic):
    if not condition:
        raise ValidationError(diagnostic)


def finite_number(value):
    return type(value) in (int, float) and math.isfinite(value)


def primitives(state):
    u = np.asarray(state, dtype=float)
    require(u.ndim >= 2 and u.shape[-1] == 5 and np.all(np.isfinite(u)), "state/nonfinite-or-shape")
    require(np.all(u[..., 0] > 0), "state/nonpositive-density")
    with np.errstate(over="ignore", invalid="ignore", divide="ignore"):
        velocity = u[..., 1:4]/u[..., 0, None]
        internal = u[..., 4]-0.5*np.sum(u[..., 1:4]*velocity, axis=-1)
        pressure = (GAMMA-1)*internal
        temperature = pressure/(287.0*u[..., 0])
    require(np.all(np.isfinite(pressure)) and np.all(pressure > 0)
            and np.all(np.isfinite(temperature)) and np.all(temperature > 0), "state/nonpositive-internal-energy-or-temperature")
    return u[..., 0], velocity, pressure


def acoustic_average(shape, time, amplitude=AMPLITUDE, attenuation=(1.0, 1.0)):
    """Exact volume average of the stated linear conservative acoustic field."""
    require(len(shape) == 3 and all(type(n) is int and n >= 2 for n in shape), "oracle/acoustic-shape")
    require(finite_number(time) and 0 <= time <= END_TIME, "oracle/acoustic-time")
    coordinates = np.meshgrid(*[(np.arange(n)+0.5)/n for n in shape], indexing="ij")
    result = np.zeros(tuple(shape)+(5,), dtype=float)
    result[..., 0] = 1.0
    result[..., 4] = 1/(GAMMA-1)
    sound = math.sqrt(GAMMA)
    for (mode, weight, phase), damping in zip(MODES, attenuation):
        length = math.sqrt(sum(k*k for k in mode))
        argument = 2*math.pi*sum(k*x for k, x in zip(mode, coordinates))
        argument -= 2*math.pi*length*sound*time
        volume_factor = math.prod(float(np.sinc(k/n)) for k, n in zip(mode, shape))
        wave = amplitude*weight*damping*volume_factor*np.sin(argument+phase)
        result[..., 0] += wave
        for d in range(3):
            result[..., d+1] += sound*mode[d]/length*wave
        result[..., 4] += GAMMA/(GAMMA-1)*wave
    return result


def acoustic_modes(initial, final, time):
    shape = initial.shape[:3]
    coordinates = np.meshgrid(*[(np.arange(n)+0.5)/n for n in shape], indexing="ij")
    results = []
    for mode, _, _ in MODES:
        basis = np.exp(-2j*math.pi*sum(k*x for k, x in zip(mode, coordinates)))
        start = np.mean((initial[..., 0]-1)*basis)
        end = np.mean((final[..., 0]-1)*basis)
        require(abs(start) > 1e-9 and abs(end) > 1e-15, "acoustic/unresolved-mode")
        speed = -float(np.angle(end/start))/(2*math.pi*math.sqrt(sum(k*k for k in mode))*time)
        results.append({"mode": list(mode), "phase_speed": speed,
                        "relative_speed_error": abs(speed/math.sqrt(GAMMA)-1),
                        "amplitude_ratio": float(abs(end/start))})
    return results


def acoustic_nonlinear_residual(amplitude, n=12):
    """Local nonlinear Euler PDE defect of the linear reference, not an error bound.

    Differentiate its waves and the ideal-gas physical flux analytically. This
    measures linearization's O(epsilon²) residual independently of a time solver.
    """
    coordinates = np.meshgrid(*[(np.arange(n)+0.5)/n]*3, indexing="ij")
    state = np.zeros((n, n, n, 5))
    state[..., 0], state[..., 4] = 1, 1/(GAMMA-1)
    temporal = np.zeros_like(state)
    gradients = [np.zeros_like(state) for _ in range(3)]
    sound = math.sqrt(GAMMA)
    for mode, weight, phase in MODES:
        length = math.sqrt(sum(k*k for k in mode))
        angle = 2*math.pi*sum(k*x for k, x in zip(mode, coordinates))+phase
        vector = np.array([1, *[sound*k/length for k in mode], GAMMA/(GAMMA-1)])
        state += (amplitude*weight*np.sin(angle))[..., None]*vector
        cosine = (amplitude*weight*np.cos(angle))[..., None]*vector
        temporal -= 2*math.pi*length*sound*cosine
        for d in range(3):
            gradients[d] += 2*math.pi*mode[d]*cosine
    rho, velocity, pressure = primitives(state)
    residual = temporal.copy()
    for d, derivative in enumerate(gradients):
        drho, dm, de = derivative[..., 0], derivative[..., 1:4], derivative[..., 4]
        dv = (dm-velocity*drho[..., None])/rho[..., None]
        dp = (GAMMA-1)*(de-np.sum(velocity*dm, axis=-1)+0.5*np.sum(velocity**2, axis=-1)*drho)
        residual[..., 0] += dm[..., d]
        residual[..., 1:4] += dm*velocity[..., d, None]+state[..., 1:4]*dv[..., d, None]
        residual[..., d+1] += dp
        residual[..., 4] += (de+dp)*velocity[..., d]+(state[..., 4]+pressure)*dv[..., d]
    return float(np.max(abs(residual)))


def sod_star():
    """Monotone scalar root with an observed binary64 bracket, no FV solver."""
    def curve(p, rho, pressure):
        sound = math.sqrt(GAMMA*pressure/rho)
        if p > pressure:
            return (p-pressure)*math.sqrt(2/((GAMMA+1)*rho)/(p+(GAMMA-1)/(GAMMA+1)*pressure))
        return 2*sound/(GAMMA-1)*((p/pressure)**((GAMMA-1)/(2*GAMMA))-1)
    low, high = 0.0, 1.0
    for _ in range(100):
        middle = (low+high)/2
        if middle in (low, high):
            break
        if curve(middle, 1.0, 1.0)+curve(middle, 0.125, 0.1) > 0:
            high = middle
        else:
            low = middle
    p = (low+high)/2
    velocity = (curve(p, 0.125, 0.1)-curve(p, 1.0, 1.0))/2
    return p, velocity, high-low


def sod_waves(time):
    p, velocity, _ = sod_star()
    c_left = math.sqrt(GAMMA)
    c_star = c_left*p**((GAMMA-1)/(2*GAMMA))
    shock = math.sqrt(GAMMA*0.1/0.125)*math.sqrt((GAMMA+1)/(2*GAMMA)*(p/0.1)+(GAMMA-1)/(2*GAMMA))
    return [0.5+time*s for s in (-c_left, velocity-c_star, velocity, shock)]


def sod_point(x, time):
    """Isolated central exact Sod conservative state, before periodic interaction."""
    x = np.asarray(x, dtype=float)
    if time == 0:
        rho = np.where(x < 0.5, 1.0, 0.125)
        pressure = np.where(x < 0.5, 1.0, 0.1)
        velocity = np.zeros_like(x)
    else:
        p, velocity_star, _ = sod_star()
        c_left = math.sqrt(GAMMA)
        c_star = c_left*p**((GAMMA-1)/(2*GAMMA))
        beta = (GAMMA-1)/(GAMMA+1)
        rho_star_right = 0.125*(p/0.1+beta)/(beta*p/0.1+1)
        xi = (x-0.5)/time
        head, tail, contact, shock = [(v-0.5)/time for v in sod_waves(time)]
        rho, pressure, velocity = np.full_like(x, 0.125), np.full_like(x, 0.1), np.zeros_like(x)
        left = xi <= head
        rho[left], pressure[left] = 1.0, 1.0
        fan = (xi > head) & (xi <= tail)
        speed = 2/(GAMMA+1)*(c_left-(GAMMA-1)*xi[fan]/2)
        rho[fan] = (speed/c_left)**(2/(GAMMA-1))
        pressure[fan] = (speed/c_left)**(2*GAMMA/(GAMMA-1))
        velocity[fan] = 2/(GAMMA+1)*(c_left+xi[fan])
        left_star = (xi > tail) & (xi <= contact)
        rho[left_star], pressure[left_star], velocity[left_star] = p**(1/GAMMA), p, velocity_star
        right_star = (xi > contact) & (xi <= shock)
        rho[right_star], pressure[right_star], velocity[right_star] = rho_star_right, p, velocity_star
    result = np.zeros(x.shape+(5,), dtype=float)
    result[..., 0], result[..., 1] = rho, rho*velocity
    result[..., 4] = pressure/(GAMMA-1)+0.5*rho*velocity*velocity
    return result


def sod_average(n, time, quadrature_order=8):
    require(type(n) is int and n > 0 and finite_number(time) and 0 <= time <= END_TIME, "oracle/sod-domain")
    nodes, weights = np.polynomial.legendre.leggauss(quadrature_order)
    splits = sod_waves(time) if time else [0.5]
    result = np.empty((n, 5))
    for i in range(n):
        low, high = i/n, (i+1)/n
        cuts = [low]+[v for v in splits if low < v < high]+[high]
        integral = np.zeros(5)
        for a, b in zip(cuts, cuts[1:]):
            points = (a+b)/2+(b-a)/2*nodes
            integral += (b-a)/2*np.sum(weights[:, None]*sod_point(points, time), axis=0)
        result[i] = integral*n
    return result


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "format/duplicate-json-key")
        result[key] = value
    return result


def read_json(path):
    require(path.stat().st_size <= 8 << 20, "format/manifest-size-limit")
    def invalid_constant(value):
        raise ValidationError("format/nonfinite-json-number: "+value)
    def finite_float(value):
        number = float(value)
        require(math.isfinite(number), "format/nonfinite-json-number")
        return number
    try:
        return json.loads(path.read_text(), object_pairs_hook=_unique_object,
                          parse_constant=invalid_constant, parse_float=finite_float)
    except json.JSONDecodeError as error:
        raise ValidationError("format/invalid-json") from error


def safe_file(root, name):
    require(isinstance(name, str) and name and not Path(name).is_absolute()
            and ".." not in Path(name).parts, "format/unsafe-artifact-path")
    path = root/name
    require(root in path.resolve().parents and path.is_file()
            and not any(p.is_symlink() for p in [path, *path.parents] if p != root), "format/unsafe-or-missing-artifact")
    return path


def read_state(path, shape):
    cells = math.prod(shape)
    require(path.stat().st_size <= cells*160+200, "format/field-size-limit")
    with path.open(newline="") as stream:
        reader = csv.reader(stream)
        require(next(reader, None) == CHANNELS, "format/field-columns")
        values = []
        for row in reader:
            require(len(row) == 5 and len(values) < cells, "format/field-row-count-or-width")
            try:
                values.append([float(x) for x in row])
            except ValueError as error:
                raise ValidationError("format/field-number") from error
    require(len(values) == cells, "format/field-row-count-or-width")
    # Export is x-fast, unlike educational C-order (z-fast) arrays.
    state = np.asarray(values).reshape(shape[2], shape[1], shape[0], 5).transpose(2, 1, 0, 3)
    primitives(state)
    return state


def integrated_totals(state, spacing):
    volume = math.prod(spacing)
    return np.array([math.fsum(float(x) for x in state[..., c].ravel())*volume for c in range(5)])


def differential_step(initial, final, dt, shape, spacing):
    """S1-D translation only; this function has no continuum acceptance authority."""
    path = REPO/"docs/research-book/compressible/reference.py"
    spec = importlib.util.spec_from_file_location("rheon_educational_llf", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    config = SimpleNamespace(shape=tuple(shape), spacing=np.array(spacing), gamma=GAMMA, cfl=0.25)
    density, velocity, pressure = primitives(initial)
    require(np.all((density >= 0.5) & (density <= 2))
            and np.all((pressure >= 0.5) & (pressure <= 2)) and np.all(abs(velocity) <= 1),
            "differential/outside-roundoff-envelope")
    require(all(np.max(abs(np.diff(initial, axis=d))) > 1e-4 for d in range(3)), "differential/missing-axis-variation")
    require(dt <= module.stable_dt(initial, config)*(1+32*EPS), "differential/dt-outside-reference-bound")
    expected = module.step(initial, dt, config)
    # Each cell executes a bounded sequence of flux/EOS/reduction arithmetic;
    # Six incident LLF face contributions, each with <64 scalar primitive/flux
    # operations, plus update/reassociation: use gamma_512, the standard rounded
    # operation accumulation envelope. The bounded states above keep EOS division
    # away from singularities. This is a roundoff model, not a solver-error proof.
    gamma_512 = 512*EPS/(1-512*EPS)
    tolerance = gamma_512*np.maximum(np.maximum(abs(expected), abs(initial)), 1)
    require(np.all(abs(final-expected) <= tolerance), "differential/conservative-field-mismatch")
    return float(np.max(abs(final-expected)))


def assess_suite(root):
    root = Path(root).resolve()
    require(np.__version__ == "2.3.5", "environment/numpy-version-must-be-2.3.5")
    suite = read_json(root/"suite.json")
    require(type(suite.get("schema_version")) is int and suite["schema_version"] == 1 and suite.get("method") == "llf-euler-periodic-v1"
            and suite.get("gamma") == GAMMA and suite.get("gas_constant") == 287.0, "format/suite-method-eos")
    cases = suite.get("cases")
    require(isinstance(cases, list) and len(cases) == 8, "format/expected-eight-cases")
    seen, names, artifacts = set(), set(), {"suite.json"}
    acoustic, sod, uniform, differential = [], [], [], []
    worst_drift = 0.0
    retained_steps = 0
    for case in cases:
        require(isinstance(case, dict), "format/case-object")
        kind, shape = case.get("kind"), case.get("shape")
        require(isinstance(shape, list) and len(shape) == 3
                and all(type(n) is int and 1 <= n <= 64 for n in shape), "format/case-shape")
        key = (kind, tuple(shape))
        require(key in EXPECTED_CASES, "format/fixture-population")
        require(key not in seen, "format/duplicate-case")
        seen.add(key)
        name = case.get("id")
        require(isinstance(name, str) and name and name.isascii() and name not in names, "format/case-id")
        names.add(name)
        spacing, origin = case.get("spacing_m"), case.get("origin_m")
        require(isinstance(spacing, list) and len(spacing) == 3 and all(finite_number(h) and h > 0 for h in spacing)
                and isinstance(origin, list) and len(origin) == 3 and all(finite_number(x) for x in origin), "format/geometry")
        require(case.get("layout") == "x-fast" and case.get("boundary") == "periodic", "format/layout-boundary")
        if kind in ("acoustic", "sod"):
            require(spacing == [1/n for n in shape] and origin == [0.0]*3, "format/unit-cube-geometry")
            require(case.get("initialization") == ("linear-conservative-cell-average-v1" if kind == "acoustic" else "sod-cell-average-v1"), "format/cell-average-convention")
            target = END_TIME
        else:
            require(spacing == [0.2, 0.3, 0.4] and origin == [0.25, -0.5, 1.0], "format/asymmetric-geometry")
            require(case.get("initialization") == ("uniform-v1" if kind == "uniform" else "exported-initial-v1"), "format/initialization")
            target = 0.01 if kind == "uniform" else 0.001
        frames = case.get("frames")
        require(isinstance(frames, list) and 2 <= len(frames) <= 10001, "format/accepted-frame-count")
        first = previous = initial_totals = None
        previous_time = 0.0
        for generation, frame in enumerate(frames):
            require(isinstance(frame, dict) and type(frame.get("generation")) is int
                    and frame["generation"] == generation, "time/generation-or-missing-step")
            t, requested, dt = (frame.get(k) for k in ("time_s", "requested_dt_s", "accepted_dt_s"))
            require(all(finite_number(x) for x in (t, requested, dt)), "time/nonfinite-or-missing-control")
            if generation == 0:
                require((t, requested, dt) == (0, 0, 0), "time/initial-record")
            else:
                require(dt > 0 and requested >= dt and t > previous_time and t <= target+8*EPS
                        and abs((t-previous_time)-dt) <= 8*EPS*max(1, abs(t)), "time/accepted-dt-or-order")
            filename = frame.get("file")
            path = safe_file(root, filename)
            require(filename not in artifacts, "format/reused-field-artifact")
            artifacts.add(filename)
            state = read_state(path, shape)
            totals = integrated_totals(state, spacing)
            if generation == 0:
                first, initial_totals = state, totals
            drift = float(np.max(abs(totals-initial_totals)/np.maximum(abs(initial_totals), 1)))
            require(drift <= 2e-13, "conservation/five-component-drift")
            worst_drift = max(worst_drift, drift)
            retained_steps += generation > 0
            if kind == "uniform":
                rho, velocity, p = 1.7, np.array([0.2, -0.4, 0.3]), 0.9
                expected = np.array([rho, *list(rho*velocity), p/(GAMMA-1)+0.5*rho*sum(velocity**2)])
                require(np.all(abs(state-expected) <= 64*EPS*np.maximum(abs(expected), 1)), "uniform/conservative-state")
            if kind == "differential" and generation:
                differential.append(differential_step(previous, state, dt, shape, spacing))
            previous, previous_time = state, t
        require(abs(previous_time-target) <= 8*EPS, "time/required-final-time")
        if kind == "acoustic":
            require(np.all(abs(first-acoustic_average(shape, 0)) <= 16*EPS), "acoustic/initial-cell-average")
            error = float(np.mean(abs(previous[..., 0]-acoustic_average(shape, END_TIME)[..., 0]))/AMPLITUDE)
            acoustic.append({"n": shape[0], "normalized_density_l1": error,
                             "modes": acoustic_modes(first, previous, END_TIME)})
        elif kind == "sod":
            initial = np.broadcast_to(sod_average(shape[0], 0)[:, None, None, :], tuple(shape)+(5,))
            require(np.all(abs(first-initial) <= 16*EPS), "sod/initial-cell-average")
            exact = sod_average(shape[0], END_TIME)
            refined = sod_average(shape[0], END_TIME, quadrature_order=16)
            uncertainty = float(np.max(abs(exact-refined)))
            require(uncertainty < 1e-12, "sod/quadrature-uncertainty")
            centers = (np.arange(shape[0])+0.5)/shape[0]
            window = (centers > 0.2) & (centers < 0.8)
            require(np.all(np.max(abs(previous-previous[:, :1, :1, :]), axis=(0, 1, 2)) < 1e-12), "sod/nonplanar-state")
            pressure = primitives(previous)[2][:, 0, 0]
            pressure_exact = primitives(exact)[2]
            sod.append({"n": shape[0], "density_l1": float(np.mean(abs(previous[window, 0, 0, 0]-exact[window, 0]))),
                        "pressure_l1": float(np.mean(abs(pressure[window]-pressure_exact[window]))),
                        "quadrature_max_difference": uncertainty})
        elif kind == "uniform":
            uniform.append(name)
    require(seen == EXPECTED_CASES, "format/fixture-population")
    require(len(differential) == 1, "differential/expected-one-step")
    acoustic.sort(key=lambda x: x["n"])
    sod.sort(key=lambda x: x["n"])
    errors = [r["normalized_density_l1"] for r in acoustic]
    require(all(0 < b < a for a, b in zip(errors, errors[1:])), "acoustic/density-refinement")
    orders = [math.log(a/b)/math.log(nb/na) for a, b, na, nb in zip(errors, errors[1:], (8, 12), (12, 20))]
    require(min(orders) > 0.6, "acoustic/order-below-0.6")
    require(acoustic[-1]["modes"][0]["relative_speed_error"] < 0.08, "acoustic/phase-speed-error")
    require(all(b["density_l1"] < a["density_l1"] for a, b in zip(sod, sod[1:])), "sod/density-refinement")
    require(sod[-1]["pressure_l1"] < 0.04, "sod/pressure-l1-error")
    return {"status": "satisfied", "scope": "S1-B/D/E/F/G exported native fields only; S1-A/C remain public Rust API checks",
            "acoustic": acoustic, "acoustic_orders": orders, "sod": sod,
            "acoustic_phase_speed_gate_mode": [1, 1, 1],
            "acoustic_other_mode_speed_scope": "reported only; the admitted aggregate-density gates do not separately bound its phase speed",
            "sod_root_bracket_width": sod_star()[2], "max_normalized_conservation_drift": worst_drift,
            "accepted_steps_checked": retained_steps, "differential_max_error": max(differential),
            "artifacts": sorted(artifacts),
            "acoustic_nonlinear_local_residual_max": acoustic_nonlinear_residual(AMPLITUDE),
            "acoustic_nonlinear_residual_amplitude_doubling_ratio": acoustic_nonlinear_residual(2*AMPLITUDE)/acoustic_nonlinear_residual(AMPLITUDE),
            "linearization_scope": "local nonlinear PDE defect only, not a global solution-error bound; no finite-amplitude exact-Euler claim"}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def verifier_identity():
    files = ["tools/verify_compressible.py", "tools/test_compressible_verifier.py",
             "docs/research-book/compressible/reference.py"]
    return {"commit": git(REPO, "rev-parse", "HEAD"), "sha256": {p: sha256(REPO/p) for p in files},
            "numpy": np.__version__}


def inside_checkout(path):
    """A sandbox's empty .git marker is not an actual Git checkout."""
    path = Path(path).resolve()
    for parent in (path, *path.parents):
        if not (parent/".git").exists():
            continue
        result = subprocess.run(["git", "-C", str(parent), "rev-parse", "--show-toplevel"],
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                text=True, check=False)
        if result.returncode == 0 and Path(result.stdout.strip()).resolve() == parent:
            return True
    return False


def capture_build_environment(source_export):
    """S1 admits the pinned default Cargo build, with no config/wrapper overrides.

    Reject unsupported build authority rather than attributing its effects to
    the source commit. This checks paths only; it never reads credentials.
    """
    explicit = {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "RUSTC", "RUSTDOC",
                "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET",
                "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"}
    overrides = [name for name, value in os.environ.items() if value and
                 (name in explicit or (name.startswith("CARGO_TARGET_")
                                       and name.endswith(("_RUSTFLAGS", "_LINKER"))))]
    require(not overrides, "origin/unsupported-build-environment: "+",".join(sorted(overrides)))
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home()/".cargo"))).resolve()
    directories = [p/".cargo" for p in (source_export, *source_export.parents)]+[cargo_home]
    require(not any((p/name).exists() for p in directories for name in ("config", "config.toml")),
            "origin/unsupported-cargo-configuration")
    # These conventional profile settings cannot inject code, but affect the
    # build and therefore remain explicit run inputs rather than ambient facts.
    return {k: value for k, value in os.environ.items() if k.startswith("CARGO_PROFILE_")}


def verify_origin(run, expected_origin_hash, expected_native_commit):
    run = Path(run).resolve()
    origin_path = run/"origin.json"
    require(sha256(origin_path) == expected_origin_hash, "origin/manifest-pin-mismatch")
    origin = read_json(origin_path)
    require(origin.get("schema_version") == 1 and origin.get("status") == "captured-native-run"
            and origin.get("source_export") == "tracked-only-git-archive"
            and origin.get("source_commit") == expected_native_commit, "origin/native-identity-mismatch")
    require(origin.get("verifier") == verifier_identity(), "origin/verifier-identity-mismatch")
    require(origin.get("executable_sha256") == sha256(safe_file(run, origin.get("executable"))), "origin/executable-mismatch")
    files = origin.get("artifacts")
    require(isinstance(files, dict) and "suite.json" in files, "origin/artifact-inventory")
    for name, digest in files.items():
        require(sha256(safe_file(run/"fields", name)) == digest, "origin/artifact-hash-mismatch: "+name)
    result = assess_suite(run/"fields")
    require(set(result["artifacts"]) <= set(files), "origin/unbound-artifact")
    result["origin_manifest_sha256"] = expected_origin_hash
    result["native_commit"] = expected_native_commit
    return result


def capture(source, commit, output):
    source, output = Path(source).resolve(), Path(output).resolve()
    require(len(commit) == 40 and all(c in "0123456789abcdef" for c in commit), "origin/full-native-commit-required")
    require(git(source, "rev-parse", "HEAD") == commit and not git(source, "status", "--porcelain"), "origin/source-not-clean-pinned-commit")
    require(not inside_checkout(output), "origin/output-inside-checkout")
    output.mkdir(parents=True, exist_ok=False)
    # Compile an immutable tracked-only export, never the caller's checkout.
    # Ignored/untracked build.rs, include! inputs and local Cargo configuration
    # must not acquire the pinned commit's source identity through a clean status.
    archive = output/"source.tar"
    with archive.open("wb") as stream:
        subprocess.run(["git", "-C", str(source), "archive", "--format=tar", commit],
                       stdout=stream, check=True, timeout=60)
    source_export = output/"source"
    with tarfile.open(archive) as snapshot:
        snapshot.extractall(source_export, filter="data")
    build_environment = capture_build_environment(source_export)
    build = ["cargo", "+1.92.0", "build", "--locked", "--release", "--no-default-features",
             "--example", "compressible_reference", "--target-dir", str(output/"build"),
             "--message-format=json"]
    tracked = git(source, "ls-tree", "-r", "--name-only", commit).splitlines()
    input_hashes = {p: sha256(source_export/p) for p in tracked}
    verifier = verifier_identity()
    with (output/"build.jsonl").open("w") as stdout, (output/"build.log").open("w") as stderr:
        subprocess.run(build, cwd=source_export, stdout=stdout, stderr=stderr, timeout=600, check=True)
    messages = [json.loads(line) for line in (output/"build.jsonl").read_text().splitlines()]
    candidates = [m["executable"] for m in messages if m.get("reason") == "compiler-artifact"
                  and m.get("manifest_path") == str(source_export/"Cargo.toml")
                  and m.get("target", {}).get("name") == "compressible_reference"
                  and m["target"].get("kind") == ["example"] and m.get("executable")]
    require(len(candidates) == 1, "origin/exact-cargo-example-executable-required")
    executable = Path(candidates[0]).resolve()
    require((output/"build") in executable.parents and executable.is_file(), "origin/executable-outside-fresh-build")
    executable_hash = sha256(executable)
    invocation = [str(executable), "--output", str(output/"fields")]
    with (output/"native.log").open("w") as log:
        subprocess.run(invocation, cwd=source_export, stdout=log, stderr=subprocess.STDOUT, timeout=300, check=True)
    require(git(source, "rev-parse", "HEAD") == commit and not git(source, "status", "--porcelain")
            and input_hashes == {p: sha256(source_export/p) for p in tracked}, "origin/source-changed-during-capture")
    require(executable_hash == sha256(executable) and verifier == verifier_identity(), "origin/build-or-verifier-changed")
    artifacts = {}
    for path in sorted((output/"fields").rglob("*")):
        require(not path.is_symlink(), "origin/symlink-artifact")
        if path.is_file():
            name = str(path.relative_to(output/"fields"))
            safe_file(output/"fields", name)
            artifacts[name] = sha256(path)
    origin = {"schema_version": 1, "status": "captured-native-run", "run_id": str(uuid.uuid4()),
              "source_commit": commit, "source_tree": git(source, "rev-parse", "HEAD^{tree}"),
              "source_sha256": input_hashes, "verifier": verifier, "executable": str(executable.relative_to(output)),
              "source_export": "tracked-only-git-archive", "source_archive_sha256": sha256(archive),
              "build_environment": build_environment,
              "executable_sha256": executable_hash, "build_command": build, "run_command": invocation,
              "rustc": subprocess.check_output(["rustc", "+1.92.0", "--version", "--verbose"], text=True),
              "artifacts": artifacts, "origin_limit": "external manifest pin required; not a cryptographic signature or physical-correctness proof"}
    (output/"origin.json").write_text(json.dumps(origin, indent=2, allow_nan=False)+"\n")
    return sha256(output/"origin.json")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="operation", required=True)
    start = commands.add_parser("capture", help="build and execute a fresh pinned native example, then verify")
    start.add_argument("--source-repo", type=Path, required=True)
    start.add_argument("--native-commit", required=True)
    start.add_argument("--output", type=Path, required=True)
    check = commands.add_parser("verify", help="verify retained fields against an externally pinned origin manifest")
    check.add_argument("--run", type=Path, required=True)
    check.add_argument("--origin-sha256", required=True)
    check.add_argument("--native-commit", required=True)
    args = parser.parse_args()
    output_existed = args.operation == "capture" and args.output.exists()
    try:
        if args.operation == "capture":
            pin = capture(args.source_repo, args.native_commit, args.output)
            result = verify_origin(args.output, pin, args.native_commit)
            (args.output/"assessment.json").write_text(json.dumps(result, indent=2, allow_nan=False)+"\n")
        else:
            result = verify_origin(args.run, args.origin_sha256, args.native_commit)
        print(json.dumps(result, indent=2, allow_nan=False))
    except (OSError, subprocess.SubprocessError, ValueError, KeyError, TypeError) as error:
        if args.operation == "capture" and not output_existed and args.output.is_dir():
            failed = {"status": "failed", "diagnostic": str(error), "native_commit": args.native_commit,
                      "origin_manifest_sha256": sha256(args.output/"origin.json")
                      if (args.output/"origin.json").is_file() else None}
            with (args.output/"assessment-failed.json").open("x") as stream:
                stream.write(json.dumps(failed, indent=2, allow_nan=False)+"\n")
        parser.exit(1, f"S1 validation failed: {error}\n")


if __name__ == "__main__":
    main()
