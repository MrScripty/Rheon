"""S1 oracle, layout, numerical-mutation and origin-boundary tests; run also -O.

Positive suite fields below are synthetic policy probes, never native evidence.
Only capture() executing a clean pinned native binary can supply native origin.
"""
from contextlib import contextmanager
import copy
import csv
import importlib.util
import json
import math
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import numpy as np

import verify_compressible as v


def educational_reference():
    path = v.REPO/"docs/research-book/compressible/reference.py"
    spec = importlib.util.spec_from_file_location("test_educational_llf", path)
    module = importlib.util.module_from_spec(spec)
    import sys
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def write_state(path, state):
    with path.open("w", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(v.CHANNELS)
        for row in state.transpose(2, 1, 0, 3).reshape(-1, 5):
            writer.writerow([format(float(x), ".17g") for x in row])


def synthetic_suite(root):
    """Construct known analytic gate probes with disclosed exterior compensation.

    The artificial Sod exterior only balances totals for policy-unit tests; it
    is neither a periodic Euler solution nor admissible native-origin evidence.
    """
    cases = []
    def add(kind, shape, initial, final, time, initialization):
        number = len(cases)
        frames = []
        for step, state in enumerate((initial, final)):
            filename = f"case-{number}-{step}.csv"
            write_state(root/filename, state)
            frames.append({"generation": step, "time_s": time if step else 0,
                           "requested_dt_s": time if step else 0,
                           "accepted_dt_s": time if step else 0, "file": filename})
        special = kind in ("uniform", "differential")
        cases.append({"id": f"synthetic-{number}", "kind": kind, "shape": list(shape),
                      "spacing_m": [0.2, 0.3, 0.4] if special else [1/n for n in shape],
                      "origin_m": [0.25, -0.5, 1.0] if special else [0.0]*3,
                      "boundary": "periodic", "layout": "x-fast", "initialization": initialization,
                      "frames": frames})
    shape = (3, 2, 1)
    density, velocity, pressure = 1.7, np.array([0.2, -0.4, 0.3]), 0.9
    uniform = np.empty(shape+(5,))
    uniform[...] = [density, *list(density*velocity), pressure/(v.GAMMA-1)+0.5*density*sum(velocity**2)]
    add("uniform", shape, uniform, uniform, 0.01, "uniform-v1")
    shape = (3, 4, 2)
    i, j, k = np.meshgrid(*[np.arange(n) for n in shape], indexing="ij")
    density = 1+0.01*(i+2*j+3*k)
    velocity = np.stack((0.2+0.01*j, -0.1+0.01*k, 0.15-0.01*i), axis=-1)
    pressure = 0.9+0.02*(2*i+j+k)
    reference = educational_reference()
    initial = reference.conservative(density, velocity, pressure)
    config = SimpleNamespace(shape=shape, spacing=np.array([0.2, 0.3, 0.4]), gamma=v.GAMMA, cfl=0.25)
    add("differential", shape, initial, reference.step(initial, 0.001, config), 0.001, "exported-initial-v1")
    for n in (8, 12, 20):
        shape = (n, n, n)
        add("acoustic", shape, v.acoustic_average(shape, 0),
            v.acoustic_average(shape, 0.08, attenuation=(1-0.5/n, 1-0.5/n)),
            0.08, "linear-conservative-cell-average-v1")
    for n in (24, 48, 64):
        shape = (n, 2, 2)
        initial = np.broadcast_to(v.sod_average(n, 0)[:, None, None, :], shape+(5,)).copy()
        final = np.broadcast_to(v.sod_average(n, 0.08)[:, None, None, :], shape+(5,)).copy()
        centers = (np.arange(n)+0.5)/n
        final[..., 0] += (0.1/n*np.sin(2*math.pi*centers))[:, None, None]
        outside = (centers <= 0.2) | (centers >= 0.8)
        delta = np.sum(final-initial, axis=(0, 1, 2))
        final[outside] -= delta/(int(np.sum(outside))*4)
        v.primitives(final)
        add("sod", shape, initial, final, 0.08, "sod-cell-average-v1")
    suite = {"schema_version": 1, "method": "llf-euler-periodic-v1", "gamma": 1.4,
             "gas_constant": 287.0, "cases": cases}
    (root/"suite.json").write_text(json.dumps(suite))
    return suite


class OracleTests(unittest.TestCase):
    def test_sod_initial_constant_cells_use_direct_analytic_averages(self):
        for n in (24, 48, 64):
            initial = v.sod_average(n, 0)
            expected = np.zeros((n, 5))
            expected[:n//2, 0], expected[n//2:, 0] = 1, 0.125
            expected[:n//2, 4] = 1/(v.GAMMA-1)
            expected[n//2:, 4] = 0.1/(v.GAMMA-1)
            np.testing.assert_array_equal(initial, expected)
        middle = v.sod_average(3, 0)[1]
        np.testing.assert_array_equal(middle, [0.5625, 0, 0, 0, 0.55/(v.GAMMA-1)])

    def test_sod_star_bracket_and_rankine_hugoniot(self):
        pressure, velocity, bracket = v.sod_star()
        self.assertAlmostEqual(pressure, 0.3031301780506468, places=14)
        self.assertAlmostEqual(velocity, 0.9274526200489499, places=14)
        self.assertLess(bracket, 2e-16)
        shock_x = v.sod_waves(0.08)[-1]
        speed = (shock_x-0.5)/0.08
        left = v.sod_point(np.array([shock_x-1e-10]), 0.08)[0]
        right = np.array([0.125, 0, 0, 0, 0.1/(v.GAMMA-1)])
        def flux(u):
            rho, mx, _, _, energy = u
            p = (v.GAMMA-1)*(energy-mx*mx/(2*rho))
            return np.array([mx, mx*mx/rho+p, 0, 0, (energy+p)*mx/rho])
        np.testing.assert_allclose(speed*(left-right), flux(left)-flux(right), atol=2e-14, rtol=2e-14)

    def test_sod_conservative_averages_integrate_open_tube_budget(self):
        # Isolated fan boundary fluxes: d(mass)/dt=0, d(momentum)/dt=1-.1,
        # d(E)/dt=0; this oracle identity is not the periodic S1-G target.
        for n in (24, 48, 64):
            average = v.sod_average(n, 0.08)
            totals = np.array([math.fsum(average[:, c])/n for c in range(5)])
            np.testing.assert_allclose(totals, [0.5625, 0.072, 0, 0, 1.375], atol=2e-15, rtol=2e-15)
            np.testing.assert_allclose(average, v.sod_average(n, 0.08, 16), atol=2e-14, rtol=2e-14)

    def test_sod_uses_averaged_conservative_state_before_pressure(self):
        n = 24
        averages = v.sod_average(n, 0.08)
        centers = (np.arange(n)+0.5)/n
        center_samples = v.sod_point(centers, 0.08)
        self.assertGreater(float(np.max(abs(averages-center_samples))), 0.02)
        # A cell crossing the contact has constant physical pressure but mixes
        # conservative density/momentum. This comparison tests the convention.
        require_index = int(v.sod_waves(0.08)[2]*n)
        p_from_u = v.primitives(averages)[2][require_index]
        self.assertAlmostEqual(p_from_u, v.sod_star()[0], places=12)

    def test_acoustic_average_sinc_and_isolated_phases(self):
        shape = (8, 12, 20)
        first = v.acoustic_average(shape, 0)
        last = v.acoustic_average(shape, 0.08)
        modes = v.acoustic_modes(first, last, 0.08)
        for result in modes:
            self.assertLess(result["relative_speed_error"], 1e-9)
            self.assertAlmostEqual(result["amplitude_ratio"], 1, places=9)
        # Independent tensor Gauss integration for one cell, all five linear U.
        nodes, weights = np.polynomial.legendre.leggauss(8)
        xyz = np.meshgrid(*[(nodes+1)/(2*n) for n in shape], indexing="ij")
        scalar = sum(weight*np.sin(2*math.pi*sum(k*x for k, x in zip(mode, xyz))+phase)
                     for mode, weight, phase in v.MODES)
        average = np.einsum("i,j,k,ijk->", weights, weights, weights, scalar)/8
        self.assertAlmostEqual((first[0, 0, 0, 0]-1)/v.AMPLITUDE, average, places=9)

    def test_measured_acoustic_linearization_defect_scales_quadratically(self):
        first = v.acoustic_nonlinear_residual(1e-6)
        doubled = v.acoustic_nonlinear_residual(2e-6)
        self.assertGreater(first, 0)
        self.assertLess(first, 1e-9)
        self.assertAlmostEqual(doubled/first, 4, delta=0.001)

    def test_primitive_and_time_domains_fail_under_optimized_python(self):
        for channel, value, diagnostic in ((0, 0, "nonpositive-density"), (4, -1, "internal-energy"),
                                            (2, float("nan"), "nonfinite-or-shape")):
            state = np.array([[1.0, 0, 0, 0, 2.5]])
            state[0, channel] = value
            with self.assertRaisesRegex(v.ValidationError, diagnostic):
                v.primitives(state)
        with self.assertRaisesRegex(v.ValidationError, "acoustic-time"):
            v.acoustic_average((8, 8, 8), float("nan"))


class ExportTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        cls.suite = synthetic_suite(cls.root)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    @contextmanager
    def changed_file(self, path, data):
        original = path.read_bytes()
        try:
            path.write_bytes(data)
            yield
        finally:
            path.write_bytes(original)

    @contextmanager
    def changed_suite(self, mutate):
        suite = copy.deepcopy(self.suite)
        mutate(suite)
        with self.changed_file(self.root/"suite.json", json.dumps(suite).encode()):
            yield

    def test_synthetic_suite_exercises_gates_without_claiming_native_origin(self):
        result = v.assess_suite(self.root)
        self.assertEqual(result["status"], "satisfied")
        self.assertLess(result["max_normalized_conservation_drift"], 2e-13)
        self.assertGreater(min(result["acoustic_orders"]), 0.6)
        self.assertEqual(result["accepted_steps_checked"], 8)

    def test_x_fast_layout_preserves_asymmetric_axes(self):
        state = np.arange(3*4*2*5, dtype=float).reshape(3, 4, 2, 5)*0.00001
        state[..., 0] += 1
        state[..., 4] += 2.5
        path = self.root/"layout-test.csv"
        try:
            write_state(path, state)
            np.testing.assert_array_equal(v.read_state(path, (3, 4, 2)), state)
        finally:
            path.unlink()

    def test_nonfinite_raw_field_reaches_admissibility_boundary(self):
        path = self.root/self.suite["cases"][0]["frames"][1]["file"]
        data = path.read_bytes().replace(b"1.7", b"nan", 1)
        with self.changed_file(path, data), self.assertRaisesRegex(v.ValidationError, "state/nonfinite-or-shape"):
            v.assess_suite(self.root)

    def test_declared_wrong_time_and_skipped_step_reject_at_clock_boundary(self):
        for field, value, diagnostic in (("time_s", 0.009, "time/accepted-dt-or-order"),
                                         ("generation", 2, "time/generation-or-missing-step")):
            def mutate(suite):
                suite["cases"][0]["frames"][1][field] = value
            with self.changed_suite(mutate), self.assertRaisesRegex(v.ValidationError, diagnostic):
                v.assess_suite(self.root)

    def test_frozen_and_wrong_phase_fields_reject_at_numerical_boundary(self):
        # Keep declared time, all five totals and data format valid. Only the
        # evolved acoustic field is wrong; origin checks are intentionally absent.
        for actual_time in (0.0, 0.04):
            saved = []
            try:
                for case in self.suite["cases"]:
                    if case["kind"] != "acoustic":
                        continue
                    path = self.root/case["frames"][1]["file"]
                    saved.append((path, path.read_bytes()))
                    n = case["shape"][0]
                    write_state(path, v.acoustic_average(tuple(case["shape"]), actual_time,
                                                        attenuation=(1-0.5/n, 1-0.5/n)))
                with self.assertRaisesRegex(v.ValidationError, "acoustic/density-refinement"):
                    v.assess_suite(self.root)
            finally:
                for path, data in saved:
                    path.write_bytes(data)

    def test_axis_permutation_of_final_fields_rejects_numerically(self):
        saved = []
        try:
            for case in self.suite["cases"]:
                if case["kind"] != "acoustic":
                    continue
                path = self.root/case["frames"][1]["file"]
                saved.append((path, path.read_bytes()))
                state = v.read_state(path, case["shape"]).swapaxes(0, 2).copy()
                state[..., [1, 3]] = state[..., [3, 1]]
                write_state(path, state)
            with self.assertRaisesRegex(v.ValidationError, "acoustic/unresolved-mode"):
                v.assess_suite(self.root)
        finally:
            for path, data in saved:
                path.write_bytes(data)

    def test_admitted_single_mode_speed_gate_discloses_second_mode_limitation(self):
        # Independent review's concrete counterexample. This intentionally
        # documents what the admitted sanity gate cannot decide; it is not
        # evidence that a 20%-slow second acoustic mode is physically correct.
        saved = []
        background = np.array([1, 0, 0, 0, 1/(v.GAMMA-1)])
        try:
            for case in self.suite["cases"]:
                if case["kind"] != "acoustic":
                    continue
                shape = tuple(case["shape"])
                n = shape[0]
                damping = {8: 0.5, 12: 0.7, 20: 0.9}[n]
                state = (v.acoustic_average(shape, 0.08, attenuation=(damping, 0))
                         + v.acoustic_average(shape, 0.064, attenuation=(0, damping))-background)
                path = self.root/case["frames"][1]["file"]
                saved.append((path, path.read_bytes()))
                write_state(path, state)
            result = v.assess_suite(self.root)
            self.assertEqual(result["status"], "satisfied")
            self.assertEqual(result["acoustic_phase_speed_gate_mode"], [1, 1, 1])
            self.assertAlmostEqual(result["acoustic"][-1]["modes"][1]["relative_speed_error"], 0.2, places=8)
        finally:
            for path, data in saved:
                path.write_bytes(data)

    def test_missing_pressure_energy_work_rejects_at_differential_boundary(self):
        case = self.suite["cases"][1]
        first, last = [self.root/f["file"] for f in case["frames"]]
        initial = v.read_state(first, case["shape"])
        final = v.read_state(last, case["shape"])
        _, velocity, pressure = v.primitives(initial)
        corrupted = final.copy()
        # Remove just pressure-work's conservative divergence from LLF energy.
        for d, h in enumerate(case["spacing_m"]):
            power = pressure*velocity[..., d]
            interface = 0.5*(power+np.roll(power, -1, axis=d))
            corrupted[..., 4] += 0.001/h*(interface-np.roll(interface, 1, axis=d))
        v.primitives(corrupted)
        np.testing.assert_allclose(v.integrated_totals(corrupted, case["spacing_m"]),
                                   v.integrated_totals(initial, case["spacing_m"]), atol=2e-15)
        original = last.read_bytes()
        try:
            write_state(last, corrupted)
            with self.assertRaisesRegex(v.ValidationError, "differential/conservative-field-mismatch"):
                v.assess_suite(self.root)
        finally:
            last.write_bytes(original)

    def test_conservation_rejects_finite_positive_energy_corruption(self):
        case = self.suite["cases"][0]
        path = self.root/case["frames"][1]["file"]
        state = v.read_state(path, case["shape"])
        state[0, 0, 0, 4] += 1e-6
        original = path.read_bytes()
        try:
            write_state(path, state)
            with self.assertRaisesRegex(v.ValidationError, "conservation/five-component-drift"):
                v.assess_suite(self.root)
        finally:
            path.write_bytes(original)

    def test_unsafe_path_and_duplicate_json_keys_reject(self):
        with self.changed_suite(lambda s: s["cases"][0]["frames"][0].update(file="../other.csv")), \
                self.assertRaisesRegex(v.ValidationError, "format/unsafe-artifact-path"):
            v.assess_suite(self.root)
        with self.changed_file(self.root/"suite.json", b'{"schema_version":1,"schema_version":1}'), \
                self.assertRaisesRegex(v.ValidationError, "format/duplicate-json-key"):
            v.assess_suite(self.root)


class OriginTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root/"fields").mkdir()
        (self.root/"fields/suite.json").write_text("{}")
        (self.root/"test-executable").write_text("synthetic test marker, not a native binary")
        self.origin = {"schema_version": 1, "status": "captured-native-run", "source_commit": "a"*40,
                       "source_export": "tracked-only-git-archive",
                       "verifier": v.verifier_identity(), "executable": "test-executable",
                       "executable_sha256": v.sha256(self.root/"test-executable"),
                       "artifacts": {"suite.json": v.sha256(self.root/"fields/suite.json")}}
        self.write_origin()

    def tearDown(self):
        self.temp.cleanup()

    def write_origin(self):
        (self.root/"origin.json").write_text(json.dumps(self.origin))
        return v.sha256(self.root/"origin.json")

    def test_mismatched_manifest_pin_rejects_before_numeric_evaluation(self):
        pin = self.write_origin()
        self.origin["source_commit"] = "b"*40
        self.write_origin()
        with patch.object(v, "assess_suite") as analytic, self.assertRaisesRegex(v.ValidationError, "origin/manifest-pin-mismatch"):
            v.verify_origin(self.root, pin, "a"*40)
        analytic.assert_not_called()

    def test_wrong_native_identity_rejects_at_origin_boundary(self):
        pin = self.write_origin()
        with self.assertRaisesRegex(v.ValidationError, "origin/native-identity-mismatch"):
            v.verify_origin(self.root, pin, "b"*40)

    def test_artifact_and_executable_hash_mutations_reject_at_origin_boundary(self):
        pin = self.write_origin()
        (self.root/"fields/suite.json").write_text('{"changed":true}')
        with self.assertRaisesRegex(v.ValidationError, "origin/artifact-hash-mismatch"):
            v.verify_origin(self.root, pin, "a"*40)
        (self.root/"test-executable").write_text("changed")
        with self.assertRaisesRegex(v.ValidationError, "origin/executable-mismatch"):
            v.verify_origin(self.root, pin, "a"*40)

    def test_capture_refuses_wrong_or_dirty_source_without_output(self):
        output = self.root/"fresh"
        with patch.object(v, "git", return_value="b"*40), \
                self.assertRaisesRegex(v.ValidationError, "origin/source-not-clean-pinned-commit"):
            v.capture(v.REPO, "a"*40, output)
        self.assertFalse(output.exists())

    def test_empty_git_marker_is_not_a_checkout_but_real_repository_is(self):
        (self.root/".git").mkdir()
        self.assertFalse(v.inside_checkout(self.root/"new-output"))
        repository = self.root/"actual-checkout"
        repository.mkdir()
        v.subprocess.run(["git", "init", "-q", str(repository)], check=True)
        self.assertTrue(v.inside_checkout(repository/"new-output"))

    def test_capture_rejects_unbound_build_configuration(self):
        for name in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"):
            with patch.dict(v.os.environ, {name: "--cfg unbound_probe"}):
                with self.assertRaisesRegex(v.ValidationError, "origin/unsupported-build-environment: "+name):
                    v.capture_build_environment(self.root)
        cargo_home = self.root/"cargo-home"
        cargo_home.mkdir()
        (cargo_home/"config.toml").write_text('[build]\nrustflags=["--cfg", "unbound_probe"]\n')
        with patch.dict(v.os.environ, {"CARGO_HOME": str(cargo_home)}):
            with self.assertRaisesRegex(v.ValidationError, "origin/unsupported-cargo-configuration"):
                v.capture_build_environment(self.root)

    def test_real_capture_protocol_excludes_ignored_build_script(self):
        # A real tiny native build/run tests capture protocol only. It writes no
        # CFD fields and is never passed to assess_suite or counted as S1 evidence.
        (self.root/".git").mkdir()
        source = self.root/"protocol-source"
        source.mkdir()
        v.subprocess.run(["git", "init", "-q", str(source)], check=True)
        (source/"examples").mkdir()
        (source/"Cargo.toml").write_text('[package]\nname="capture-protocol-probe"\nversion="0.1.0"\nedition="2024"\n')
        (source/"Cargo.lock").write_text('version=4\n[[package]]\nname="capture-protocol-probe"\nversion="0.1.0"\n')
        (source/".gitignore").write_text('/build.rs\n')
        (source/"examples/compressible_reference.rs").write_text('''#![allow(unexpected_cfgs)]
fn main() {
    let path = std::env::args().nth(2).expect("--output PATH");
    std::fs::create_dir(&path).unwrap();
    std::fs::write(format!("{path}/suite.json"), "{}").unwrap();
    std::fs::write(format!("{path}/ignored-input-probe.txt"),
        format!("{}", cfg!(unbound_probe))).unwrap();
}
''')
        v.subprocess.run(["git", "-C", str(source), "add", "."], check=True)
        v.subprocess.run(["git", "-C", str(source), "-c", "user.name=Capture protocol test",
                          "-c", "user.email=capture-test@example.invalid", "commit", "-qm", "test protocol source"], check=True)
        commit = v.git(source, "rev-parse", "HEAD")
        (source/"build.rs").write_text('fn main(){println!("cargo:rustc-cfg=unbound_probe");}\n')
        self.assertEqual(v.git(source, "status", "--porcelain"), "")
        output = self.root/"captured"
        pin = v.capture(source, commit, output)
        self.assertEqual(pin, v.sha256(output/"origin.json"))
        self.assertEqual((output/"fields/ignored-input-probe.txt").read_text(), "false")
        self.assertFalse((output/"source/build.rs").exists())
        self.assertTrue((source/"build.rs").is_file())
        origin = v.read_json(output/"origin.json")
        self.assertEqual(origin["source_export"], "tracked-only-git-archive")
        self.assertNotIn("build.rs", origin["source_sha256"])
        self.assertEqual(origin["source_commit"], commit)


if __name__ == "__main__":
    unittest.main()
