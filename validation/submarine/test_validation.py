"""Analytic/rational harness tests; synthetic records never claim native origin."""
import copy
from fractions import Fraction as Q
import hashlib
import itertools
import math
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import oracle as o
import run

FIXTURES = o.read_json(Path(__file__).with_name("fixtures.json"))
FIXTURE_SHA = o.digest(Path(__file__).with_name("fixtures.json"))


def synthetic_record(case):
    """Test-only analytic uniform/rest serialization, not a native capture."""
    f = FIXTURES["cases"][case]; op = o.Operator(f); g = op.grid
    if g.lower is not None:
        if any(Q(x) for x in f["velocity"]):
            raise ValueError("test factory supports body rest only")
        values = op.prescribed | {k: Q(0) for k in op.active}
    else:
        values = dict(op.prescribed)
        for a, i, key, mask in op.faces:
            if mask == 1:
                values[key] = Q.from_float(float(Q(f["velocity"][a])))
    p = {c: Q(f["pressure_gauge"]) for c in op.divergence}
    geometry = {"counts": list(g.counts), "origin": list(map(o.bits, g.origin)), "spacing": list(map(o.bits, g.spacing)),
                "body_lower": None if g.lower is None else list(map(o.bits, g.lower)), "body_upper": None if g.upper is None else list(map(o.bits, g.upper)),
                "cell_mask": [int(g.fluid(c)) for c in o.tuples(g.counts)],
                "face_masks": [[g.mask(a, i) for i in o.tuples(g.face_shape(a))] for a in range(3)],
                "face_dual_volumes": [[o.bits(op.face_dual[g.key(a, i)]) for i in o.tuples(g.face_shape(a))] for a in range(3)],
                "pressure_components": [op.labels.get(c, -1) for c in o.tuples(g.counts)],
                "component_gauges": [{"component": op.labels[c], "cell": o.flat(c, g.counts), "pressure": o.bits(v)} for c, v in op.gauges.items()]}
    r = {"schema": o.SCHEMA, "operator_profile": o.PROFILE, "fixture_id": case, "fixtures_sha256": FIXTURE_SHA, "accepted": True, "solve_id": 1,
         "equations": "steady-incompressible-newtonian-stokes", "frame": "fixed-right-handed-SI",
         "units": {"velocity": "m/s", "pressure": "Pa", "gradient": "1/s", "force": "N", "moment": "N m", "volume_forcing": "N/m^3"},
         "layout": {"ordering": "x-fast", "velocity_offsets": [[o.bits(v) for v in off] for off in o.OFFSETS], "pressure_offset": [o.bits(Q(1, 2))]*3},
         "controls": f, "discretization": FIXTURES["discretization"], "geometry": geometry, "forcing_semantics": f["forcing"],
         "solve_report": {"backend": "dense-scaled-partial-pivot", "actual_dofs": len(op.active)+len(op.pressure_unknowns), "min_scaled_pivot": o.bits(1), "retained_payload_bytes": 4096, "payload_limit_bytes": 8192},
         "geometry_sha256": hashlib.sha256(json.dumps(geometry, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
         "fields": {"volume_forcing": [[o.bits(0)]*math.prod(g.face_shape(a)) for a in range(3)], "velocity": [[o.bits(values.get(g.key(a, i), Q(0))) for i in o.tuples(g.face_shape(a))] for a in range(3)],
                    "pressure": [o.bits(p.get(c, Q(0))) for c in o.tuples(g.counts)]},
         "outer_traces": [{"axis": a, "side": side, "point": list(map(o.bits, point)), "velocity": list(map(o.bits, velocity))} for (a, side, point), velocity in op.outer_traces.items()],
         "body_traces": [{"point": list(map(o.bits, point)), "velocity": [o.bits(0)]*3} for point in op.body_traces],
         "reaction_sites": {key: {"component": a, "owner": op.boundary_owner[key], "point": list(map(o.bits, point))} for key, (a, point) in op.boundary_locations.items()},
         "wall_patches": [], "algebraic_reactions": {k: o.bits(0) for k in op.prescribed},
         "algebraic_loads": {name: [o.bits(0)]*3 for name in ("body_force", "outer_force", "body_moment", "outer_moment")},
         "physical_loads": {name: [o.bits(0)]*3 for name in ("pressure_force", "shear_force", "pressure_moment", "shear_moment")}, "dissipation": o.bits(0)}
    for patch in o.wall_patches(g):
        c, a, side, point, normal, area = patch
        r["wall_patches"].append({"cell": o.flat(c, g.counts), "axis": a, "side": side, "point": list(map(o.bits, point)), "normal": list(map(o.bits, normal)),
                                  "area": o.bits(area), "velocity": [o.bits(0)]*3, "pressure": o.bits(0), "gradient": [[o.bits(0)]*3 for _ in range(3)]})
    return r


def control(gradient=None, slope=None, offset=0, reference=None):
    # Six exact midpoint patches on the declared 1 x 1/4 x 1/4 cuboid.
    gradient = gradient or [[Q(0)]*3 for _ in range(3)]; slope = slope or [Q(0)]*3; reference = reference or [Q(0)]*3
    control_id = next(name for name, preset in FIXTURES["control_presets"].items() if preset["closed"] and
                      o.bits(Q(preset["pressure_offset"])) == o.bits(offset) and
                      list(map(o.bits, map(Q, preset["pressure_gradient"]))) == list(map(o.bits, slope)) and
                      [[o.bits(Q(x)) for x in row] for row in preset["analytic_gradient"]] == [list(map(o.bits, row)) for row in gradient])
    dims = [Q(1), Q(1, 4), Q(1, 4)]; patches = []; exact = []
    for axis, side in itertools.product(range(3), (-1, 1)):
        point = [Q(0)]*3; point[axis] = side*dims[axis]/2
        normal = [Q(0)]*3; normal[axis] = Q(side)
        area = dims[(axis+1)%3]*dims[(axis+2)%3]; pressure = Q(offset)+sum(slope[a]*point[a] for a in range(3))
        quantities = o.loads([(point, normal, area, pressure, gradient)], Q(1), reference)
        wall_velocity = tuple(sum(gradient[a][b]*point[b] for b in range(3)) for a in range(3))
        work = sum((quantities["pressure_force"][a]+quantities["shear_force"][a])*wall_velocity[a] for a in range(3))
        patches.append({"wall_velocity": list(map(o.bits, wall_velocity)), "mechanical_work": o.bits(work), "point": list(map(o.bits, point)), "normal": list(map(o.bits, normal)), "area": o.bits(area), "pressure": o.bits(pressure),
                        "gradient": [list(map(o.bits, row)) for row in gradient], **{k: list(map(o.bits, v)) for k, v in quantities.items()}})
        exact.append((point, normal, area, pressure, gradient))
    quantities = o.loads(exact, Q(1), reference)
    return {"kind": "constitutive-control", "control_id": control_id, "closed": True, "mu": o.bits(1), "analytic_gradient": [list(map(o.bits, row)) for row in gradient],
            "pressure_offset": o.bits(offset), "pressure_gradient": list(map(o.bits, slope)), "moment_reference": list(map(o.bits, reference)), "patches": patches,
            **{k: list(map(o.bits, v)) for k, v in quantities.items()}}


class Validation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.uniform = synthetic_record("exact-no-body")
        cls.rest = synthetic_record("tiny-rest")

    def fails(self, claim, callback):
        with self.assertRaises(o.Failure) as caught:
            callback()
        self.assertEqual(caught.exception.data["claim_id"], claim)
        return caught.exception.data

    def check_record(self, record):
        return o.validate_record(record, FIXTURES["cases"][record["fixture_id"]], FIXTURE_SHA)

    def test_actual_fixture_counts_and_bounds(self):
        op = o.Operator(FIXTURES["cases"]["tiny-straight"])
        self.assertEqual((len(op.active), len(op.pressure_unknowns)), (474, 195))
        self.assertEqual(len(o.wall_patches(op.grid)), 18)
        self.fails("exact-solve-cap", op.solve_exact)
        f = copy.deepcopy(FIXTURES["cases"]["tiny-straight"]); f["max_dofs"] = 668
        self.fails("bounds", lambda: o.Operator(f))

    def test_uniform_literal_exact_solution(self):
        op = o.Operator(FIXTURES["cases"]["exact-no-body"]); u, p = op.solve_exact()
        for key in op.active:
            component = int(key.split(":")[1]); expected = Q.from_float(float(Q(op.fixture["velocity"][component])))
            self.assertEqual(u[key], expected)
        self.assertTrue(all(v == 0 for v in p.values()))
        self.assertEqual(self.check_record(self.uniform)["rational_solve"], "compared")

    def test_body_rest_is_discrete_not_native_evidence(self):
        r = self.check_record(self.rest)
        self.assertEqual(r["mixed_dofs"], 669)
        self.assertEqual(r["max_momentum_defect_N"], 0)
        self.assertIn("not-established", r["provenance"])
        self.assertEqual(r["claims"]["N3"], "pending")

    def test_quadrature_clips_all_edges_and_dual_volumes(self):
        for name in ["tiny-rest", "exact-no-body", "tiny-axis-1", "tiny-axis-2"]:
            op = o.Operator(FIXTURES["cases"][name]); volume = sum(op.grid.volume for c in op.divergence)
            for axis in range(3):
                self.assertEqual(sum(op.face_dual[key] for a, _, key, _ in op.faces if a == axis), volume)
            shear = [s for s in op.strains if s.kind == "shear"]
            self.assertEqual(sum(s.weight for s in shear), 3*op.analytic.mu*volume)

    def test_rigid_rotation_extension_and_upper_reflection_sign(self):
        fixture = copy.deepcopy(FIXTURES["cases"]["exact-no-body"])
        op = o.Operator(fixture); g = op.grid
        def affine(point):
            x, y, _ = point
            return (-y, x, Q(0))
        values = {}
        for a, i, key, mask in op.faces:
            if mask: values[key] = affine(g.face_point(a, i))[a]
        for key, (a, point) in op.boundary_locations.items():
            values[key] = affine(point)[a]
        self.assertTrue(all(x == 0 for x in op.strains_at(values)))
        # Extension diag(1,-1,0) has dissipation 4 mu volume.
        for a, i, key, mask in op.faces:
            if mask: values[key] = (1 if a == 0 else -1 if a == 1 else 0)*g.face_point(a, i)[a]
        for key, (a, point) in op.boundary_locations.items():
            values[key] = (1 if a == 0 else -1 if a == 1 else 0)*point[a]
        d = sum(s.weight*e*e for s, e in zip(op.strains, op.strains_at(values)))
        self.assertEqual(d, 4*op.analytic.mu*sum(g.volume for c in op.divergence))

    def corner_rows(self, op, a=0, b=1, edge=(6, 3, 2)):
        prefix = "corner:"+":".join(map(str, (a, b)+edge))+":"
        return [row for row in op.strains if any(key.startswith(prefix) for key in row.coefficients)]

    def corner_variation(self, op):
        # Test-only divergence-free MAC curl with homogeneous prescribed normal data.
        g = op.grid; q = Q(1, 100); amplitude = q*g.spacing[0]**2
        psi = {(6, 4, 2): amplitude, (7, 3, 2): amplitude, (7, 4, 2): 2*amplitude,
               (5, 4, 2): amplitude, (7, 2, 2): amplitude}
        def stream(i, j, k):
            return psi.get((i, j, k), Q(0))
        values = dict(op.prescribed)
        for component, index, key, mask in op.faces:
            if not mask:
                continue
            i, j, k = index
            value = ((stream(i, j+1, k)-stream(i, j, k))/g.spacing[1] if component == 0 else
                     -(stream(i+1, j, k)-stream(i, j, k))/g.spacing[0] if component == 1 else Q(0))
            if mask == 2:
                self.assertEqual(value, 0)
            values[key] = value
        return values

    def test_corner_sector_coefficients_preserve_full_boundary_columns(self):
        op = o.Operator(FIXTURES["cases"]["tiny-rest"]); g = op.grid
        rows = self.corner_rows(op); self.assertEqual(len(rows), 3)
        A, B = g.key(0, (6, 2, 2)), g.key(0, (6, 3, 2))
        C, D = g.key(1, (5, 3, 2)), g.key(1, (6, 3, 2))
        Ga, Gb = "corner:0:1:6:3:2:0", "corner:0:1:6:3:2:1"
        expected = [{B: Q(8), Ga: Q(-8), C: Q(-8), Gb: Q(8)},
                    {A: Q(-8), Ga: Q(8), D: Q(8), Gb: Q(-8)},
                    {B: Q(8), Ga: Q(-8), D: Q(8), Gb: Q(-8)}]
        self.assertEqual([r.coefficients for r in rows], expected)
        self.assertTrue(all(r.weight == Q(1, 256) for r in rows))
        self.assertIn(A, op.prescribed); self.assertIn(C, op.prescribed)
        point = (Q(1, 2), Q(1, 8), Q(0))
        self.assertEqual(op.boundary_locations[Ga], (0, point))
        self.assertEqual(op.boundary_locations[Gb], (1, point))
        self.assertIn(point, op.body_traces)
        self.assertEqual((len(op.strains), len(op.prescribed), len(op.body_traces)), (1464, 780, 54))

    def test_corner_discriminator_retains_old_clipped_counterexample(self):
        op = o.Operator(FIXTURES["cases"]["tiny-rest"]); g = op.grid
        values = self.corner_variation(op)
        self.assertTrue(all(sum(c*values[k] for k, c in row.items()) == 0 for row in op.divergence.values()))
        rows = self.corner_rows(op)
        strains = [sum(c*values[k] for k, c in row.coefficients.items()) for row in rows]
        self.assertEqual(strains, [Q(1, 50), Q(-1, 50), Q(0)])
        sector_phi = sum(row.weight*value**2 for row, value in zip(rows, strains))/2
        self.assertEqual(sector_phi, Q(1, 640000))
        X = values[g.key(0, (6, 3, 2))]/g.spacing[1]
        Y = values[g.key(1, (6, 3, 2))]/g.spacing[0]
        clipped_phi = 3*op.analytic.mu*g.volume*(X+Y)**2/8
        self.assertEqual(clipped_phi, 0)
        self.assertGreater(sector_phi, clipped_phi)

    def affine_values(self, op, gradient, translation):
        def value(component, point):
            return translation[component]+sum(gradient[component][a]*point[a] for a in range(3))
        values = {key: value(a, op.grid.face_point(a, index)) for a, index, key, mask in op.faces if mask}
        values.update({key: value(a, point) for key, (a, point) in op.boundary_locations.items()})
        return values

    def test_every_corner_axis_and_orientation_has_affine_patch_consistency(self):
        gradient = [[Q(1), Q(2), Q(3)], [Q(-4), Q(5), Q(6)], [Q(-7), Q(8), Q(-9)]]
        for name in ["tiny-rest", "tiny-axis-1", "tiny-axis-2"]:
            op = o.Operator(FIXTURES["cases"][name]); values = self.affine_values(op, gradient, (Q(2), Q(3), Q(5)))
            corners = [row for row in op.strains if any(k.startswith("corner:") for k in row.coefficients)]
            self.assertEqual(len(corners), 72)
            for row in corners:
                key = next(k for k in row.coefficients if k.startswith("corner:")); _, a, b, *_ = key.split(":")
                a, b = int(a), int(b)
                expected = gradient[a][b]+gradient[b][a]
                self.assertEqual(sum(c*values[k] for k, c in row.coefficients.items()), expected)

    def test_rigid_virtual_boundary_translation_rotation_and_work(self):
        op = o.Operator(FIXTURES["cases"]["tiny-rest"])
        state = self.corner_variation(op)
        pressure = {c: op.grid.center(c)[0]+2*op.grid.center(c)[1] for c in op.divergence}
        residual = op.residuals(state, pressure)
        for gradient in [[[Q(0)]*3 for _ in range(3)],
                         [[Q(0), Q(-3), Q(2)], [Q(3), Q(0), Q(-1)], [Q(-2), Q(1), Q(0)]]]:
            virtual = self.affine_values(op, gradient, (Q(2), Q(3), Q(5)))
            self.assertTrue(all(s == 0 for s in op.strains_at(virtual)))
            self.assertEqual(sum(sum(terms)*virtual[key] for key, terms in residual.items()), 0)
        # Central exact differentiation includes the newly retained corner and face columns.
        def phi(values):
            return sum(row.weight*s*s for row, s in zip(op.strains, op.strains_at(values)))/2
        zero_pressure = {c: Q(0) for c in op.divergence}
        derivative = op.residuals(state, zero_pressure); delta = Q(1, 32)
        for key in ["corner:0:1:6:3:2:0", "corner:0:1:6:3:2:1", "face:0:6:2:2", "face:1:5:3:2"]:
            plus, minus = state.copy(), state.copy(); plus[key] += delta; minus[key] -= delta
            self.assertEqual((phi(plus)-phi(minus))/(2*delta), sum(derivative[key]))

    def test_full_dirichlet_affine_constitutive_energy(self):
        # All prescribed values move consistently for this constitutive diagnostic;
        # this is not a stationary-body native solution or a free-slip boundary test.
        gradient = [[Q(1), Q(2), Q(3)], [Q(4), Q(-1), Q(6)], [Q(7), Q(8), Q(0)]]
        for name in ["exact-no-body", "tiny-rest"]:
            op = o.Operator(FIXTURES["cases"][name])
            values = self.affine_values(op, gradient, (Q(2), Q(3), Q(5)))
            fluid_volume = op.grid.volume*len(op.divergence)
            expected = op.analytic.mu*fluid_volume*(
                2*sum(gradient[a][a]**2 for a in range(3))+
                sum((gradient[a][b]+gradient[b][a])**2 for a in range(3) for b in range(a+1, 3)))
            actual = sum(row.weight*s*s for row, s in zip(op.strains, op.strains_at(values)))
            self.assertEqual(actual, expected)

    def test_profile_epoch_and_corner_reaction_associations_are_explicit(self):
        self.assertEqual(self.rest["schema"], "rheon-submarine-i1-mac-v2")
        self.assertEqual(self.rest["operator_profile"], "rheon-i1-sector-half-cell-v1")
        for claim, mutate in [
            ("profile", lambda r: r.__setitem__("schema", "rheon-submarine-i1-mac-v1")),
            ("operator-profile", lambda r: r.__setitem__("operator_profile", "clipped-edge-v1")),
            ("fixture-identity", lambda r: r.__setitem__("fixtures_sha256", "6b6f0d55afaf3a92b6e88c116a0b858107e3e24e21a426f080d19345a78dab9b")),
            ("reaction-sites", lambda r: r["reaction_sites"]["corner:0:1:6:3:2:0"].__setitem__("point", [o.bits(Q(1, 2)), o.bits(0), o.bits(0)])),
            ("reaction-sites", lambda r: r["reaction_sites"]["corner:0:1:6:3:2:0"].__setitem__("component", False)),
            ("V04-reactions", lambda r: r["algebraic_reactions"].pop("corner:0:1:6:3:2:0")),
        ]:
            record = copy.deepcopy(self.rest); mutate(record)
            self.fails(claim, lambda: self.check_record(record))
        record = copy.deepcopy(self.rest)
        record["body_traces"] = [t for t in record["body_traces"] if t["point"] != [o.bits(Q(1, 2)), o.bits(Q(1, 8)), o.bits(0)]]
        self.fails("full-body-trace", lambda: self.check_record(record))

    def test_mms_mixed_derivatives_commute_and_wall_vanishes(self):
        op = o.Operator(FIXTURES["cases"]["tiny-mms"]); an = op.analytic
        dux = o.derivative(an.polys[0], 0); duy = o.derivative(an.polys[1], 1)
        self.assertEqual(dux, {e: -v for e, v in duy.items()})
        for patch in o.wall_patches(op.grid):
            self.assertEqual(an.velocity(patch[3]), (0, 0, 0))
        self.assertTrue(any(any(v for v in velocity) for velocity in op.outer_traces.values()))
        self.assertNotEqual(an.force((Q(1, 8), Q(1, 4), Q(3, 8))), (0, 0, 0))

    def test_wall_stencils_quadratic_exact_and_cartesian_sign(self):
        op = o.Operator(FIXTURES["cases"]["tiny-rest"]); g = op.grid
        for patch in o.wall_patches(g):
            _, axis, side, point, _, _ = patch
            values = {}
            for a, i, key, mask in op.faces:
                if mask:
                    distance = side*(g.face_point(a, i)[axis]-point[axis])
                    values[key] = 3*distance+2*distance**2
            pressure = {c: Q(7)+5*side*(g.center(c)[axis]-point[axis]) for c in op.divergence}
            p, grad = o.wall_trace(op, patch, values, pressure)
            self.assertEqual(p, 7)
            for a in range(3):
                self.assertEqual(grad[a][axis], side*3)

    def test_affine_pressure_hydrostatic_and_moment_shift(self):
        for slope in [[Q(1), Q(0), Q(0)], [Q(0), Q(1), Q(0)], [Q(0), Q(0), Q(-981, 100)]]:
            for offset in [0, 1, -1, 10**6, -10**6]:
                c = control(slope=slope, offset=offset)
                o.validate_control(c)
                force = tuple(o.number(x) for x in c["pressure_force"])
                for actual, expected in zip(force, (-Q(1, 16)*x for x in slope)):
                    self.assertAlmostEqual(float(actual), float(expected), places=12)
                shifted = control(slope=slope, offset=offset, reference=[Q(1, 8), Q(-1, 4), Q(3, 8)])
                expected = tuple(-x for x in o.cross([Q(1, 8), Q(-1, 4), Q(3, 8)], force))
                for actual, desired in zip(map(o.number, shifted["pressure_moment"]), expected):
                    self.assertAlmostEqual(float(actual), float(desired), places=12)
                o.validate_control(shifted)

    def test_patch_shear_discriminates_closed_load_cancellation(self):
        gradient = [[Q(0), Q(1), Q(0)], [Q(0)]*3, [Q(0)]*3]
        c = control(gradient=gradient)
        self.assertEqual(c["shear_force"], [o.bits(0)]*3)
        o.validate_control(c)
        bad = copy.deepcopy(c); patch = next(p for p in bad["patches"] if any(o.number(x) != 0 for x in p["shear_force"]))
        patch["shear_force"] = [o.bits(0)]*3
        self.fails("V06-patch-shear_force", lambda: o.validate_control(bad))

    def test_affine_adjacent_cell_factor_two_negative(self):
        c = control(slope=[Q(0), Q(1), Q(0)])
        # One-cell body breadth: wall y=-1/8, adjacent fluid center y=-1/4.
        c["patches"][2]["pressure"] = o.bits(Q(-1, 4))
        self.fails("V06-affine-pressure", lambda: o.validate_control(c))

    def test_each_mutation_hits_its_intended_claim(self):
        mutations = [
            ("units", lambda r: r["units"].__setitem__("pressure", "m^2/s^2")),
            ("association", lambda r: r["layout"].__setitem__("ordering", "z-fast")),
            ("fixture-identity", lambda r: r.__setitem__("fixtures_sha256", "0"*64)),
            ("accepted-solve", lambda r: r.__setitem__("accepted", False)),
            ("accepted-solve", lambda r: r.__setitem__("solve_id", 2)),
            ("steady-clock", lambda r: r.__setitem__("dt", o.bits(1))),
            ("geometry", lambda r: r["geometry"]["face_masks"][0].__setitem__(0, True)),
            ("geometry-identity", lambda r: r.__setitem__("geometry_sha256", "0"*64)),
            ("full-boundary-trace", lambda r: r["outer_traces"].pop()),
            ("V02-outer-trace", lambda r: r["outer_traces"][0]["velocity"].__setitem__(0, o.bits(1))),
            ("V04-reactions", lambda r: r["algebraic_reactions"].pop(next(iter(r["algebraic_reactions"])))),
            ("V04-dissipation", lambda r: r.__setitem__("dissipation", o.bits(1))),
            ("profile", lambda r: r["fields"]["pressure"].__setitem__(1, "7ff0000000000000")),
        ]
        for claim, mutate in mutations:
            with self.subTest(claim=claim):
                record = copy.deepcopy(self.uniform); mutate(record)
                self.fails(claim, lambda: self.check_record(record))

    def test_active_velocity_and_pressure_corruption(self):
        bad = copy.deepcopy(self.uniform); op = o.Operator(FIXTURES["cases"]["exact-no-body"])
        a, i, key, mask = next(x for x in op.faces if x[3] == 1)
        bad["fields"]["velocity"][a][o.flat(i, op.grid.face_shape(a))] = o.bits(1)
        self.fails("V01-momentum", lambda: self.check_record(bad))
        bad = copy.deepcopy(self.uniform); bad["fields"]["pressure"][1] = o.bits(1)
        self.fails("V01-momentum", lambda: self.check_record(bad))

    def test_wall_geometry_pressure_gradient_and_load_corruption(self):
        for claim, mutate in [
            ("wall-patch-geometry", lambda p: p.__setitem__("area", o.bits(2))),
            ("wall-pressure-stencil", lambda p: p.__setitem__("pressure", o.bits(1))),
            ("wall-gradient-stencil", lambda p: p["gradient"][0].__setitem__(0, o.bits(1))),
            ("V02-body-no-slip", lambda p: p["velocity"].__setitem__(0, o.bits(1))),
        ]:
            bad = copy.deepcopy(self.rest); mutate(bad["wall_patches"][0])
            self.fails(claim, lambda: self.check_record(bad))
        bad = copy.deepcopy(self.rest); bad["physical_loads"]["shear_force"][0] = o.bits(1)
        self.fails("N2-physical-shear_force", lambda: self.check_record(bad))

    def test_empty_control_and_changed_analytic_parameters_refused(self):
        c = control(); c["patches"] = []
        self.fails("V06-patch-geometry", lambda: o.validate_control(c))
        c = control(); c["analytic_gradient"][0][0] = o.bits(999)
        self.fails("control-fixture", lambda: o.validate_control(c))
        c = control(); c["patches"][0]["normal"] = [o.bits(1), o.bits(0), o.bits(0)]
        self.fails("V06-patch-geometry", lambda: o.validate_control(c))

    def test_net_pressure_roundoff_uses_patch_budget(self):
        c = control(slope=[Q(1), Q(0), Q(0)], offset=10**6)
        # y net force is mathematically zero; a valid floating accumulation
        # can leave a residual bounded by the opposing pressure patch terms.
        c["pressure_force"][1] = o.bits(1e-12)
        o.validate_control(c)
        c["pressure_force"][1] = o.bits(0.1)
        self.fails("V06-total-pressure_force", lambda: o.validate_control(c))

    def test_couette_and_poiseuille_patch_sign_work_and_units(self):
        for name in ["couette", "poiseuille"]:
            preset = FIXTURES["control_presets"][name]
            record = {"kind": "constitutive-control", "control_id": name, "closed": False, "mu": o.bits(1),
                      "moment_reference": [o.bits(0)]*3, "pressure_offset": o.bits(0),
                      "pressure_gradient": list(map(o.bits, map(Q, preset["pressure_gradient"]))),
                      "analytic_gradient": None if name == "poiseuille" else [[o.bits(Q(x)) for x in row] for row in preset["analytic_gradient"]], "patches": []}
            parsed = []
            for y, normal_y in [(0, 1), (1, -1)]:
                point = (Q(0), Q(y), Q(0)); normal = (Q(0), Q(normal_y), Q(0))
                gradient = [[Q(0), Q(1 if name == "couette" else 1-2*y), Q(0)], [Q(0)]*3, [Q(0)]*3]
                velocity = (Q(y if name == "couette" else 0), Q(0), Q(0))
                quantities = o.loads([(point, normal, Q(1), Q(0), gradient)], Q(1), (Q(0),)*3)
                work = sum(quantities["shear_force"][a]*velocity[a] for a in range(3))
                record["patches"].append({"point": list(map(o.bits, point)), "normal": list(map(o.bits, normal)), "area": o.bits(1), "pressure": o.bits(0),
                    "gradient": [list(map(o.bits, row)) for row in gradient], "wall_velocity": list(map(o.bits, velocity)), "mechanical_work": o.bits(work),
                    **{k: list(map(o.bits, v)) for k, v in quantities.items()}})
                parsed.append((point, normal, Q(1), Q(0), gradient))
            record.update({k: list(map(o.bits, v)) for k, v in o.loads(parsed, Q(1), (Q(0),)*3).items()})
            o.validate_control(record)
            if name == "couette":
                self.assertEqual(o.number(record["patches"][1]["mechanical_work"]), -1)
            else:
                self.assertEqual(o.number(record["shear_force"][0]), 2)
            bad = copy.deepcopy(record); bad["patches"][0]["shear_force"][0] = o.bits(0)
            self.fails("V06-patch-shear_force", lambda: o.validate_control(bad))

    def test_malformed_nested_objects_have_typed_refusals(self):
        for key, value, claim in [("solve_report", 0, "profile"), ("constitutive_controls", [0], "control-profile")]:
            bad = copy.deepcopy(self.uniform); bad[key] = value
            self.fails(claim, lambda: self.check_record(bad))
        self.fails("control-profile", lambda: o.validate_control(0))
        bad = copy.deepcopy(self.rest)
        patch = next(p for p in bad["wall_patches"] if p["axis"] == 0); patch["axis"] = False
        self.fails("wall-patch-geometry", lambda: self.check_record(bad))
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/"record.json"; bad = copy.deepcopy(self.uniform); bad["solve_report"] = 0; p.write_text(json.dumps(bad))
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("run.py")), "--suite", "stokes", "--source", "1"*40, "--input", str(p), "--output", str(Path(d)/"out")], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(json.loads(result.stderr)["claim_id"], "profile")
            self.assertNotIn("Traceback", result.stderr)

    def test_pivot_payload_and_point_source_mutations(self):
        for claim, mutate in [
            ("V01-pivot", lambda r: r["solve_report"].__setitem__("min_scaled_pivot", o.bits(1e-14))),
            ("payload", lambda r: r["solve_report"].__setitem__("retained_payload_bytes", 8193)),
            ("forcing-semantics", lambda r: r.__setitem__("forcing_semantics", "integrated-volume-forcing")),
            ("V07-analytic-forcing", lambda r: r["fields"]["volume_forcing"][0].__setitem__(0, o.bits(1))),
        ]:
            bad = copy.deepcopy(self.uniform); mutate(bad)
            self.fails(claim, lambda: self.check_record(bad))
        bad = copy.deepcopy(self.rest); bad["body_traces"].pop()
        self.fails("full-body-trace", lambda: self.check_record(bad))

    def test_scaled_pivot_refusal_and_zero_budget(self):
        self.assertEqual(o.solve_dense([[Q(2), Q(1)], [Q(1), Q(3)]], [Q(1), Q(2)]), [Q(1, 5), Q(3, 5)])
        self.fails("V01-pivot", lambda: o.solve_dense([[Q(1), Q(1)], [Q(1), Q(1)+Q(1, 10**14)]], [Q(1), Q(1)]))
        self.fails("V01-zero", lambda: o.close(Q(1, 10**100), Q(0), "V01-zero", Q(0)))

    def test_json_duplicates_bounds_and_cancellation_prefix(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/"x.json"; p.write_text('{"x":1,"x":2}')
            self.fails("profile", lambda: o.read_json(p))
            p.write_text('{"x":')
            self.fails("profile", lambda: o.read_json(p))
            p.write_bytes(b" "*(o.MAX_BYTES+1))
            self.fails("bounds", lambda: o.read_json(p))
            path = Path(d)/"atomic.json"
            run.write_json(path, {"terminal": "active"}); run.write_json(path, {"terminal": "cancelled"})
            self.assertEqual(o.read_json(path)["terminal"], "cancelled")
            self.assertFalse(Path(str(path)+".partial").exists())
            self.fails("fresh-output", lambda: run.output_directory(d, Path("/nonexistent-repo")))

    def test_missing_native_is_unavailable_not_skip_or_capture(self):
        with tempfile.TemporaryDirectory() as d:
            repo = Path(d)/"no-native-repo"
            subprocess.run(["git", "init", "--quiet", str(repo)], check=True)
            (repo/"README.md").write_text("Original no-native availability fixture.\n")
            subprocess.run(["git", "-C", str(repo), "add", "README.md"], check=True)
            subprocess.run(["git", "-C", str(repo), "-c", "user.name=I1AvailabilityFixture",
                            "-c", "user.email=i1-test@example.invalid", "-c", "commit.gpgsign=false", "commit", "--quiet",
                            "-m", "Create no-native availability fixture"], check=True)
            source = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
            self.assertEqual(subprocess.check_output(["git", "-C", str(repo), "ls-tree", "--name-only", source,
                                                     "examples/submarine_stokes.rs"], text=True).strip(), "")
            out = Path(d)/"capture"
            p = subprocess.run([sys.executable, str(Path(__file__).with_name("run.py")), "--suite", "stokes",
                                "--repo", str(repo), "--source", source, "--native-contract-sha256", FIXTURE_SHA,
                                "--output", str(out)], capture_output=True, text=True)
            self.assertEqual(p.returncode, 2)
            self.assertEqual(json.loads(p.stderr)["claim_id"], "native-capability")
            self.assertEqual(json.loads(p.stderr)["failure_kind"], "unavailable")
            self.assertFalse(out.exists())

    def test_nonzero_pressure_gauge_affine_rhs(self):
        fixture = copy.deepcopy(FIXTURES["cases"]["exact-no-body"]); fixture["pressure_gauge"] = "3"
        op = o.Operator(fixture); u, p = op.solve_exact()
        self.assertTrue(all(value == 3 for value in p.values()))
        for key in op.active:
            component = int(key.split(":")[1])
            self.assertEqual(u[key], Q.from_float(float(Q(fixture["velocity"][component]))))

    def test_build_environment_drops_flags_wrappers_credentials(self):
        from unittest.mock import patch
        with patch.dict(run.os.environ, {"RUSTFLAGS": "--cfg changed", "RUSTC_WRAPPER": "/fake/wrapper", "GH_TOKEN": "test-secret", "CARGO_PROFILE_RELEASE_OPT_LEVEL": "0"}):
            env = run.build_environment(Path("/tmp/test-only"))
        for key in ["RUSTFLAGS", "RUSTC_WRAPPER", "GH_TOKEN", "CARGO_PROFILE_RELEASE_OPT_LEVEL"]:
            self.assertNotIn(key, env)
        self.assertEqual(env["RUSTUP_TOOLCHAIN"], "1.92.0")

    def test_origin_pins_actual_source_archive_binary_fixture_and_case(self):
        repo = Path(__file__).resolve().parents[2]
        source = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
        tree = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"], text=True).strip()
        with tempfile.TemporaryDirectory() as d:
            root = Path(d); records = root/"case"; records.mkdir()
            input_path = records/"record.json"; input_path.write_text(json.dumps(self.uniform))
            exe = root/"test-only-not-a-native-executable"; exe.write_bytes(b"test-only")
            fixture_path = root/"fixtures.json"; fixture_path.write_bytes(run.FIXTURES.read_bytes())
            archive = root/"source.tar"
            archive.write_bytes(subprocess.check_output(["git", "-C", str(repo), "archive", "--format=tar", source]))
            stdout = root/"stdout"; stderr = root/"stderr"; stdout.write_bytes(b""); stderr.write_bytes(b"")
            manifest = {"schema": "rheon-submarine-i1-origin-v1", "terminal": "completed", "source_commit": source, "source_tree": tree, "dirty": False,
                "record_schema": o.SCHEMA, "operator_profile": o.PROFILE,
                "fixtures_sha256": FIXTURE_SHA, "source_archive_path": str(archive), "source_archive_sha256": o.digest(archive),
                "executable": {"path": str(exe), "sha256": o.digest(exe)},
                "validator": {"files": {p.name: o.digest(p) for p in [run.HERE/"run.py", run.HERE/"oracle.py", run.FIXTURES]}},
                "outputs": {"exact-no-body": {"path": str(input_path), "sha256": o.digest(input_path)}},
                "commands": [{"argv": [str(exe), "--case", "exact-no-body", "--fixtures", str(fixture_path), "--output", str(records)], "returncode": 0,
                              "stdout": {"path": str(stdout), "sha256": o.digest(stdout)}, "stderr": {"path": str(stderr), "sha256": o.digest(stderr)}}]}
            origin = root/"origin.json"
            def check(data):
                origin.write_text(json.dumps(data))
                return run.verify_origin(origin, o.digest(origin), source, input_path, FIXTURE_SHA, "exact-no-body", repo)
            result = check(manifest)
            self.assertEqual(result["status"], "externally-pinned-origin-integrity-checked")
            # This test manifest is not evidence that its toy executable was built.
            for claim, mutate in [
                ("origin-profile", lambda m: m.__setitem__("record_schema", "rheon-submarine-i1-mac-v1")),
                ("origin-profile", lambda m: m.__setitem__("operator_profile", "clipped-edge-v1")),
                ("origin-source", lambda m: m.__setitem__("source_commit", "1"*40)),
                ("origin-source", lambda m: m.__setitem__("source_tree", "1"*40)),
                ("origin-output", lambda m: m["outputs"]["exact-no-body"].__setitem__("sha256", "0"*64)),
                ("origin-executable", lambda m: m["executable"].__setitem__("sha256", "0"*64)),
                ("origin-fixture", lambda m: m.__setitem__("fixtures_sha256", "0"*64)),
                ("origin-source-archive", lambda m: m.__setitem__("source_archive_sha256", "0"*64)),
                ("origin-validator", lambda m: m["validator"]["files"].__setitem__("oracle.py", "0"*64)),
                ("origin-terminal", lambda m: m.__setitem__("terminal", "active")),
                ("origin-command", lambda m: m["commands"][0]["argv"].__setitem__(2, "wrong-case")),
                ("origin-profile", lambda m: m["commands"][0]["argv"].pop()),
                ("origin-profile", lambda m: m["commands"][0].__setitem__("argv", [str(exe), "--case", "exact-no-body", "--output", str(records), "--fixtures"])),
            ]:
                changed = copy.deepcopy(manifest); mutate(changed)
                self.fails(claim, lambda: check(changed))
            origin.write_text(json.dumps(manifest))
            self.fails("origin-integrity", lambda: run.verify_origin(origin, "0"*64, source, input_path, FIXTURE_SHA, "exact-no-body", repo))

    def test_cli_numerical_import_discloses_absent_origin(self):
        with tempfile.TemporaryDirectory() as d:
            input_path = Path(d)/"input.json"; input_path.write_text(json.dumps(self.uniform))
            out = Path(d)/"analysis"
            p = subprocess.run([sys.executable, str(Path(__file__).with_name("run.py")), "--suite", "stokes", "--source", "1"*40, "--input", str(input_path), "--output", str(out)], capture_output=True, text=True)
            self.assertEqual(p.returncode, 0, p.stderr)
            self.assertEqual(json.loads(p.stdout)["provenance"], "unavailable; numerical properties only")


if __name__ == "__main__":
    unittest.main()
