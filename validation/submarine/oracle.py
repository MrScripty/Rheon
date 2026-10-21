"""Independent bounded MAC Stokes oracle; no native solver imports or captures."""
from __future__ import annotations

from dataclasses import dataclass
from fractions import Fraction as Q
import hashlib
import itertools
import json
import math
from pathlib import Path
import re
import struct

EPS = 2.0 ** -52
SCHEMA = "rheon-submarine-i1-mac-v2"
PROFILE = "rheon-i1-sector-half-cell-v1"
MAX_BYTES = 16 * 1024 * 1024
MAX_DOFS = 4096
EXACT_MAX_DOFS = 128
PIVOT_MIN = Q(1, 10**12)
OFFSETS = ((0, Q(1, 2), Q(1, 2)), (Q(1, 2), 0, Q(1, 2)), (Q(1, 2), Q(1, 2), 0))


class Failure(Exception):
    def __init__(self, kind, claim, detail, measured=None, allowed=None):
        self.data = {"failure_kind": kind, "claim_id": claim, "detail": detail}
        if measured is not None:
            self.data["measured"] = measured
        if allowed is not None:
            self.data["allowed"] = allowed
        super().__init__(detail)


def require(condition, claim, detail, kind="invalid"):
    if not condition:
        raise Failure(kind, claim, detail)


def bits(value):
    return struct.pack(">d", float(value)).hex()


def number(value, claim="profile"):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{16}", value), claim,
            "f64 must be lowercase 16-digit numeric IEEE-754 to_bits hex")
    x = struct.unpack(">d", bytes.fromhex(value))[0]
    require(math.isfinite(x), claim, "nonfinite f64")
    return Q.from_float(x)


def vector(values, length=3, claim="profile"):
    require(isinstance(values, list) and len(values) == length, claim, "wrong vector length")
    return tuple(number(x, claim) for x in values)


def integer(x, minimum, maximum, claim="profile"):
    require(type(x) is int and minimum <= x <= maximum, claim, "integer outside supported bounds")
    return x


def _object(pairs):
    obj = {}
    for key, value in pairs:
        require(key not in obj, "profile", "duplicate JSON key: " + key)
        obj[key] = value
    return obj


def read_json(path):
    p = Path(path)
    require(p.is_file(), "availability", "missing JSON: " + str(p), "unavailable")
    require(p.stat().st_size <= MAX_BYTES, "bounds", "JSON exceeds 16 MiB cap")
    try:
        obj = json.loads(p.read_text(), object_pairs_hook=_object,
                          parse_constant=lambda s: (_ for _ in ()).throw(Failure("invalid", "profile", "JSON nonfinite: " + s)))
        require(isinstance(obj, dict), "profile", "JSON root must be an object")
        return obj
    except (ValueError, UnicodeError, RecursionError) as exc:
        raise Failure("invalid", "profile", "malformed JSON") from exc


def digest(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def tuples(shape):
    for k in range(shape[2]):
        for j in range(shape[1]):
            for i in range(shape[0]):
                yield (i, j, k)


def flat(index, shape):
    i, j, k = index
    return i + shape[0] * (j + shape[1] * k)


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def close(actual, expected, claim, scale=None, factor=4096):
    defect = abs(actual - expected)
    # No unit-sized floor: exact zero terms require exact zero defect.
    magnitude = abs(actual) + abs(expected) if scale is None else scale
    allowance = Q.from_float(factor * EPS) * magnitude
    if defect > allowance:
        raise Failure("invalid", claim, "roundoff budget exceeded", float(defect), float(allowance))


@dataclass(frozen=True)
class Grid:
    counts: tuple
    origin: tuple
    spacing: tuple
    lower: tuple | None
    upper: tuple | None

    @classmethod
    def fixture(cls, fixture):
        counts = tuple(integer(x, 2, 128, "geometry") for x in fixture["counts"])
        require(len(counts) == 3 and math.prod(counts) <= 32768, "bounds", "unsupported cell count")
        lower = None if fixture["body_lower"] is None else tuple(Q(x) for x in fixture["body_lower"])
        upper = None if fixture["body_upper"] is None else tuple(Q(x) for x in fixture["body_upper"])
        grid = cls(counts, tuple(Q(x) for x in fixture["origin"]), tuple(Q(x) for x in fixture["spacing"]), lower, upper)
        require(all(x > 0 for x in grid.spacing), "geometry", "nonpositive spacing")
        require((lower is None) == (upper is None), "geometry", "partial body bounds")
        if lower is not None:
            for a in range(3):
                lo = (lower[a]-grid.origin[a])/grid.spacing[a]
                hi = (upper[a]-grid.origin[a])/grid.spacing[a]
                require(lo.denominator == hi.denominator == 1 and 2 <= lo < hi <= counts[a]-2,
                        "geometry", "body must align and retain two fluid layers")
        return grid

    @property
    def volume(self):
        return math.prod(self.spacing)

    def center(self, index):
        return tuple(self.origin[a] + (index[a]+Q(1, 2))*self.spacing[a] for a in range(3))

    def fluid(self, index):
        if any(index[a] < 0 or index[a] >= self.counts[a] for a in range(3)):
            return False
        if self.lower is None:
            return True
        p = self.center(index)
        return not all(self.lower[a] < p[a] < self.upper[a] for a in range(3))

    def face_shape(self, a):
        return tuple(self.counts[b] + (a == b) for b in range(3))

    def face_point(self, a, index):
        return tuple(self.origin[b] + (index[b]+OFFSETS[a][b])*self.spacing[b] for b in range(3))

    def face_cells(self, a, index):
        low = list(index); low[a] -= 1
        return tuple(low), index

    def mask(self, a, index):
        if any(index[b] < 0 or index[b] >= self.face_shape(a)[b] for b in range(3)):
            return 0
        count = sum(self.fluid(c) for c in self.face_cells(a, index))
        return (0, 2, 1)[count]  # 0 masked, 1 active, 2 prescribed

    def key(self, a, index):
        return "face:" + ":".join(map(str, (a,)+index))

    def point_key(self, a, point):
        index = tuple((point[b]-self.origin[b])/self.spacing[b]-OFFSETS[a][b] for b in range(3))
        require(all(i.denominator == 1 for i in index), "wall-stencil", "point does not coincide with MAC sample")
        return self.key(a, tuple(int(i) for i in index))

    def components(self):
        cells = [c for c in tuples(self.counts) if self.fluid(c)]
        labels = {}; groups = []
        for seed in cells:
            if seed in labels:
                continue
            component = len(groups); stack = [seed]; group = []; labels[seed] = component
            while stack:
                c = stack.pop(); group.append(c)
                for a in range(3):
                    for sign in (-1, 1):
                        d = list(c); d[a] += sign; d = tuple(d)
                        if self.fluid(d) and d not in labels:
                            labels[d] = component; stack.append(d)
            groups.append(sorted(group, key=lambda c: flat(c, self.counts)))
        return labels, groups


# Analytic polynomial operations are independent of the native implementation.
def polynomial_mul(a, b):
    result = {}
    for p, x in a.items():
        for q, y in b.items():
            e = tuple(p[i]+q[i] for i in range(3))
            result[e] = result.get(e, Q(0)) + x*y
    return {e: x for e, x in result.items() if x}


def derivative(poly, axis, order=1):
    for _ in range(order):
        result = {}
        for exponent, value in poly.items():
            if exponent[axis]:
                e = list(exponent); e[axis] -= 1
                result[tuple(e)] = value*exponent[axis]
        poly = result
    return poly


def evaluate(poly, point):
    return sum((v*math.prod(point[a]**e[a] for a in range(3)) for e, v in poly.items()), Q(0))


class Analytic:
    def __init__(self, fixture, grid):
        self.fixture = fixture; self.grid = grid
        self.mu = Q(fixture["mu"]); self.rho = Q(fixture["rho"])
        require(self.mu > 0 and self.rho > 0 and Q(fixture["U_ref"]) > 0, "controls", "nonpositive physical scale")
        self.polys = None
        if fixture["boundary"] == "polynomial-mms-dirichlet":
            require(grid.lower is not None, "controls", "MMS requires cuboid")
            self.center = tuple((grid.lower[a]+grid.upper[a])/2 for a in range(3))
            widths = tuple((grid.upper[a]-grid.lower[a])/2 for a in range(3))
            psi = {(0, 0, 0): Q(fixture["mms_amplitude"])}
            for a, width in enumerate(widths):
                poly = {}
                for power, coefficient in [(4, Q(1)), (2, -2*width**2), (0, width**4)]:
                    exponent = [0]*3; exponent[a] = power; poly[tuple(exponent)] = coefficient
                psi = polynomial_mul(psi, poly)
            self.polys = (derivative(psi, 1), {e: -v for e, v in derivative(psi, 0).items()}, {})
            self.pressure_poly = {(0, 0, 0): Q(fixture["pressure_gauge"])}
            for a in range(3):
                for order, field in [(1, "pressure_linear"), (2, "pressure_quadratic")]:
                    e = [0]*3; e[a] = order; self.pressure_poly[tuple(e)] = Q(fixture[field][a])
        else:
            require(fixture["boundary"] == "uniform-dirichlet" and fixture["forcing"] == "zero",
                    "controls", "unsupported boundary/source policy", "unsupported")

    def local(self, point):
        return tuple(point[a]-self.center[a] for a in range(3))

    def velocity(self, point):
        if self.polys is None:
            return tuple(Q(x) for x in self.fixture["velocity"])
        return tuple(evaluate(p, self.local(point)) for p in self.polys)

    def pressure(self, point):
        return Q(self.fixture["pressure_gauge"]) if self.polys is None else evaluate(self.pressure_poly, self.local(point))

    def force_terms(self, point, axis):
        if self.polys is None:
            return [Q(0)]
        p = self.local(point)
        result = [v*math.prod(p[a]**e[a] for a in range(3)) for e, v in derivative(self.pressure_poly, axis).items()]
        for direction in range(3):
            result.extend(-self.mu*v*math.prod(p[a]**e[a] for a in range(3)) for e, v in derivative(self.polys[axis], direction, 2).items())
        return result

    def force(self, point):
        if self.polys is None:
            return (Q(0),)*3
        p = self.local(point)
        return tuple(evaluate(derivative(self.pressure_poly, a), p) - self.mu*sum(
            (evaluate(derivative(self.polys[a], b, 2), p) for b in range(3)), Q(0)) for a in range(3))


@dataclass
class Strain:
    coefficients: dict
    weight: Q
    kind: str


class Operator:
    """Geometry-derived functional and incidence, including all affine boundaries."""
    def __init__(self, fixture):
        self.fixture = fixture; self.grid = g = Grid.fixture(fixture); self.analytic = an = Analytic(fixture, g)
        cap = min(MAX_DOFS, integer(fixture["max_dofs"], 1, MAX_DOFS, "bounds"))
        # A padded cuboid's surrounding fluid is connected; count before rows/matrix allocation.
        fluid_count = 0
        for c in tuples(g.counts):
            fluid_count += int(g.fluid(c))
            require(fluid_count-1 <= cap, "bounds", "pressure DOFs alone exceed mixed cap")
        active_count = 0
        for a in range(3):
            for index in tuples(g.face_shape(a)):
                active_count += int(g.mask(a, index) == 1)
                require(active_count+fluid_count-1 <= cap, "bounds", "mixed DOF cap exceeded before operator allocation")
        self.active = []; self.prescribed = {}; self.boundary_owner = {}; self.face_forcing = {}
        self.face_dual = {}; self.faces = []; self.strains = []; self.divergence = {}
        self.outer_traces = {}; self.body_traces = {}; self.boundary_locations = {}
        for a in range(3):
            for index in tuples(g.face_shape(a)):
                key = g.key(a, index); mask = g.mask(a, index)
                self.faces.append((a, index, key, mask))
                self.face_dual[key] = sum(g.fluid(c) for c in g.face_cells(a, index))*g.volume/2
                self.face_forcing[key] = self.face_dual[key]*an.force(g.face_point(a, index))[a] if mask else Q(0)
                if mask == 1:
                    self.active.append(key)
                elif mask == 2:
                    exterior = index[a] in (0, g.counts[a]); point = g.face_point(a, index)
                    self.prescribed[key] = Q.from_float(float(an.velocity(point)[a])) if exterior else Q(0)
                    self.boundary_owner[key] = "outer" if exterior else "body"
                    self.boundary_locations[key] = (a, point)
                    if exterior:
                        self.trace(a, -1 if index[a] == 0 else 1, point)
                    else:
                        self.body_traces[point] = (Q(0),)*3
        self.labels, self.groups = g.components()
        for c in tuples(g.counts):
            if not g.fluid(c):
                continue
            row = {}
            for a in range(3):
                up = list(c); up[a] += 1
                row[g.key(a, tuple(up))] = g.volume/g.spacing[a]
                row[g.key(a, c)] = -g.volume/g.spacing[a]
                # Each normal strain lives at one full fluid cell.
                self.strains.append(Strain({g.key(a, tuple(up)): 1/g.spacing[a], g.key(a, c): -1/g.spacing[a]},
                                           2*an.mu*g.volume, "normal"))
            self.divergence[c] = row
        for a, b in itertools.combinations(range(3), 2):
            shape = tuple(g.counts[t]+(t in (a, b)) for t in range(3))
            for edge in tuples(shape):
                incident = []
                for da, db in itertools.product((-1, 0), repeat=2):
                    c = list(edge); c[a] += da; c[b] += db
                    if g.fluid(tuple(c)):
                        incident.append(tuple(c))
                if not incident:
                    continue
                point = tuple(g.origin[t]+(edge[t]+(0 if t in (a, b) else Q(1, 2)))*g.spacing[t] for t in range(3))
                if len(incident) == 3:
                    # Full corner Dirichlet columns remain even when their current value is zero.
                    # Do not collapse distinct sector gradients into one centered edge row.
                    self.body_traces[point] = (Q(0),)*3
                    for sector_cell in incident:
                        row = {}
                        for component, direction in [(a, b), (b, a)]:
                            sign = 1 if sector_cell[direction] == edge[direction] else -1
                            sample = list(edge); sample[direction] = sector_cell[direction]
                            require(g.mask(component, tuple(sample)) != 0, "strain-corner", "missing corner face sample", "unsupported")
                            face = g.key(component, tuple(sample))
                            trace = "corner:" + ":".join(map(str, (a, b)+edge+(component,)))
                            self.prescribed[trace] = Q(0)
                            self.boundary_owner[trace] = "body"
                            self.boundary_locations[trace] = (component, point)
                            coefficient = 2*sign/g.spacing[direction]
                            row[face] = coefficient; row[trace] = -coefficient
                        self.strains.append(Strain(row, an.mu*g.volume/4, "shear"))
                    continue
                row = {}
                for component, direction in [(a, b), (b, a)]:
                    low = list(edge); low[direction] -= 1; low = tuple(low); high = edge
                    ml, mh = g.mask(component, low), g.mask(component, high)
                    require(ml or mh, "strain-corner", "unsupported edge with two absent derivative samples", "unsupported")
                    h = g.spacing[direction]
                    if ml and mh:
                        terms = {g.key(component, high): 1/h, g.key(component, low): -1/h}
                    else:
                        exterior = edge[direction] in (0, g.counts[direction])
                        key = "ghost:" + ":".join(map(str, (a, b)+edge+(component,)))
                        self.prescribed[key] = Q.from_float(float(an.velocity(point)[component])) if exterior else Q(0)
                        self.boundary_owner[key] = "outer" if exterior else "body"
                        self.boundary_locations[key] = (component, point)
                        if exterior:
                            self.trace(direction, -1 if edge[direction] == 0 else 1, point)
                        else:
                            self.body_traces[point] = (Q(0),)*3
                        terms = ({g.key(component, high): 2/h, key: -2/h} if mh else
                                 {key: 2/h, g.key(component, low): -2/h})
                    for key, value in terms.items():
                        row[key] = row.get(key, Q(0)) + value
                self.strains.append(Strain(row, an.mu*g.volume*Q(len(incident), 4), "shear"))
        self.gauges = {group[0]: an.pressure(g.center(group[0])) for group in self.groups}
        self.pressure_unknowns = [c for c in self.divergence if c not in self.gauges]
        n = len(self.active)+len(self.pressure_unknowns)
        require(n <= min(MAX_DOFS, integer(fixture["max_dofs"], 1, MAX_DOFS, "bounds")), "bounds", "active mixed DOF cap exceeded")
        # Check every component's prescribed flux before gauge elimination.
        for group in self.groups:
            flux_terms = [coef*self.prescribed.get(key, Q(0)) for c in group for key, coef in self.divergence[c].items()]
            close(sum(flux_terms, Q(0)), Q(0), "V03-component-compatibility", sum(map(abs, flux_terms), Q(0)))

    def trace(self, a, side, point):
        self.outer_traces[(a, side, point)] = self.analytic.velocity(point)

    def strains_at(self, values):
        return [sum((coef*values[key] for key, coef in s.coefficients.items()), Q(0)) for s in self.strains]

    def residuals(self, values, pressure):
        terms = {key: [-self.face_forcing.get(key, Q(0))] for key in values}
        for s, value in zip(self.strains, self.strains_at(values)):
            for key, coef in s.coefficients.items():
                terms[key].extend(s.weight*coef*other_coef*values[other] for other, other_coef in s.coefficients.items())
        for c, row in self.divergence.items():
            for key, coef in row.items():
                terms[key].append(-coef*pressure[c])
        return terms

    def solve_exact(self):
        keys = self.active + self.pressure_unknowns
        require(len(keys) <= EXACT_MAX_DOFS, "exact-solve-cap", "rational solve exceeds 128 mixed DOFs", "unsupported")
        positions = {key: i for i, key in enumerate(keys)}; n = len(keys)
        matrix = [[Q(0) for _ in range(n)] for _ in range(n)]; rhs = [Q(0)]*n
        for s in self.strains:
            constant = sum((coef*self.prescribed.get(key, Q(0)) for key, coef in s.coefficients.items()), Q(0))
            for key, coef in s.coefficients.items():
                if key not in positions:
                    continue
                i = positions[key]; rhs[i] -= s.weight*coef*constant
                for other, c in s.coefficients.items():
                    if other in positions:
                        matrix[i][positions[other]] += s.weight*coef*c
        for key in self.active:
            rhs[positions[key]] += self.face_forcing[key]
        for c, row in self.divergence.items():
            constant = sum((coef*self.prescribed.get(key, Q(0)) for key, coef in row.items()), Q(0))
            for key, coef in row.items():
                if key not in positions:
                    continue
                i = positions[key]
                if c in self.gauges:
                    rhs[i] += coef*self.gauges[c]
                else:
                    j = positions[c]; matrix[i][j] -= coef; matrix[j][i] += coef
            if c not in self.gauges:
                rhs[positions[c]] = -constant
        scales = [Q(self.fixture["U_ref"]) if isinstance(key, str) else self.analytic.mu*Q(self.fixture["U_ref"])/Q(self.fixture["L_ref"]) for key in keys]
        dimensionless = [[matrix[i][j]*scales[j] for j in range(n)] for i in range(n)]
        scaled_solution = solve_dense(dimensionless, rhs)
        solution = [scaled_solution[i]*scales[i] for i in range(n)]
        values = self.prescribed | {key: solution[positions[key]] for key in self.active}
        pressure = self.gauges | {c: solution[positions[c]] for c in self.pressure_unknowns}
        return values, pressure


def solve_dense(matrix, rhs):
    n = len(rhs)
    require(n <= EXACT_MAX_DOFS and len(matrix) == n and all(len(row) == n for row in matrix), "exact-solve-cap", "invalid dense dimensions")
    rows = []
    for row, value in zip(matrix, rhs):
        scale = max(map(abs, row), default=Q(0))
        require(scale > 0, "V01-pivot", "zero matrix row", "unsupported")
        rows.append([Q(x)/scale for x in row]+[Q(value)/scale])
    for column in range(n):
        pivot = max(range(column, n), key=lambda i: abs(rows[i][column]))
        require(abs(rows[pivot][column]) >= PIVOT_MIN, "V01-pivot", "scaled pivot below frozen 1e-12 refusal", "unsupported")
        rows[column], rows[pivot] = rows[pivot], rows[column]
        for i in range(column+1, n):
            ratio = rows[i][column]/rows[column][column]
            if ratio:
                for j in range(column, n+1):
                    rows[i][j] -= ratio*rows[column][j]
    result = [Q(0)]*n
    for i in reversed(range(n)):
        result[i] = (rows[i][-1]-sum((rows[i][j]*result[j] for j in range(i+1, n)), Q(0)))/rows[i][i]
    return result


def wall_patches(grid):
    if grid.lower is None:
        return []
    patches = []
    for c in tuples(grid.counts):
        if not grid.fluid(c):
            continue
        for a in range(3):
            for side in (-1, 1):
                solid = list(c); solid[a] -= side; solid = tuple(solid)
                if all(0 <= solid[t] < grid.counts[t] for t in range(3)) and not grid.fluid(solid):
                    point = list(grid.center(c)); point[a] -= side*grid.spacing[a]/2
                    normal = tuple(Q(side if t == a else 0) for t in range(3))
                    patches.append((c, a, side, tuple(point), normal, grid.volume/grid.spacing[a]))
    return patches


def wall_trace(op, patch, values, pressure, with_scales=False):
    c, axis, side, point, _, _ = patch; g = op.grid; h = g.spacing[axis]
    far = list(c); far[axis] += side; far = tuple(far)
    require(g.fluid(far), "wall-stencil", "missing second pressure layer", "unsupported")
    p = (3*pressure[c]-pressure[far])/2
    gradient = [[Q(0)]*3 for _ in range(3)]
    gradient_scale = [[Q(0)]*3 for _ in range(3)]
    for component in range(3):
        samples = []; sample_scales = []
        for depth in (1, 2):
            location = list(point)
            location[axis] += side*h*(depth if component == axis else Q(2*depth-1, 2))
            if component == axis:
                key = g.point_key(component, location)
                require(key in values, "wall-stencil", "missing normal MAC sample", "unsupported")
                samples.append(values[key]); sample_scales.append(abs(values[key]))
            else:
                v = []
                for offset in (-1, 1):
                    q = location.copy(); q[component] += offset*g.spacing[component]/2
                    key = g.point_key(component, q)
                    require(key in values, "wall-stencil", "missing tangential MAC sample", "unsupported")
                    v.append(values[key])
                samples.append(sum(v)/2); sample_scales.append(sum(map(abs, v))/2)
        # Stationary body g=0; tangential derivatives along its flat face vanish.
        gradient[component][axis] = side*((4*samples[0]-samples[1])/(2*h) if component == axis else
                                          (9*samples[0]-samples[1])/(3*h))
        gradient_scale[component][axis] = ((4*sample_scales[0]+sample_scales[1])/(2*h) if component == axis else
                                            (9*sample_scales[0]+sample_scales[1])/(3*h))
    if with_scales:
        return p, gradient, (3*abs(pressure[c])+abs(pressure[far]))/2, gradient_scale
    return p, gradient


def loads(patches, mu, reference):
    totals = {name: [Q(0)]*3 for name in ("pressure_force", "shear_force", "pressure_moment", "shear_moment")}
    for point, normal, area, pressure, gradient in patches:
        fp = tuple(-pressure*n*area for n in normal)
        fs = tuple(mu*sum(((gradient[a][b]+gradient[b][a])*normal[b] for b in range(3)), Q(0))*area for a in range(3))
        arm = tuple(point[a]-reference[a] for a in range(3))
        for key, vals in [("pressure_force", fp), ("shear_force", fs), ("pressure_moment", cross(arm, fp)), ("shear_moment", cross(arm, fs))]:
            for a in range(3):
                totals[key][a] += vals[a]
    return totals


def load_budgets(patches, mu, reference, trace_scales=None):
    totals = {name: [Q(0)]*3 for name in ("pressure_force", "shear_force", "pressure_moment", "shear_moment")}
    for index, (point, normal, area, pressure, gradient) in enumerate(patches):
        pressure_scale, gradient_scale = (abs(pressure), [[abs(x) for x in row] for row in gradient]) if trace_scales is None else trace_scales[index]
        fp = tuple(pressure_scale*abs(n)*area for n in normal)
        fs = tuple(mu*sum((gradient_scale[a][b]+gradient_scale[b][a])*abs(normal[b]) for b in range(3))*area for a in range(3))
        arm = tuple(abs(point[a]-reference[a]) for a in range(3))
        for name, force in [("pressure", fp), ("shear", fs)]:
            for a in range(3):
                b, c = (a+1)%3, (a+2)%3
                totals[name+"_force"][a] += force[a]
                totals[name+"_moment"][a] += arm[b]*force[c]+arm[c]*force[b]
    return totals


def validate_control(record):
    """Frozen constitutive-only controls; no solver or provenance acceptance."""
    require(isinstance(record, dict), "control-profile", "control must be a JSON object")
    fixtures = read_json(Path(__file__).with_name("fixtures.json"))
    control_id = record.get("control_id")
    require(isinstance(control_id, str) and control_id in fixtures["control_presets"], "control-fixture", "unknown frozen control", "unsupported")
    preset = fixtures["control_presets"][control_id]
    require(record.get("kind") == "constitutive-control", "control-profile", "not a constitutive control")
    for name in ("pressure_offset", "pressure_gradient", "analytic_gradient"):
        expected = preset[name]
        def encode(value):
            return [encode(v) for v in value] if isinstance(value, list) else None if value is None else bits(Q(value))
        require(record.get(name) == encode(expected), "control-fixture", "analytic controls differ from frozen fixture")
    require(record.get("closed") is preset["closed"] and record.get("mu") == bits(1), "control-fixture", "wrong control viscosity/closure")
    reference = vector(record["moment_reference"])
    require(list(reference) in [list(map(Q, r)) for r in fixtures["control_references"]], "control-fixture", "unadmitted moment reference")
    p0 = number(record["pressure_offset"]); slope = vector(record["pressure_gradient"]); mu = Q(1)
    expected_patches = []
    if preset["closed"]:
        low = list(map(Q, fixtures["control_body_lower"])); high = list(map(Q, fixtures["control_body_upper"]))
        for axis, side in itertools.product(range(3), (-1, 1)):
            point = [(low[a]+high[a])/2 for a in range(3)]; point[axis] = low[axis] if side == -1 else high[axis]
            normal = [Q(0)]*3; normal[axis] = side
            area = math.prod(high[a]-low[a] for a in range(3) if a != axis)
            expected_patches.append((point, normal, area))
    else:
        expected_patches = [([Q(0), Q(y), Q(0)], [Q(0), Q(sign), Q(0)], Q(1)) for y, sign in [(0, 1), (1, -1)]]
    require(isinstance(record.get("patches"), list) and len(record["patches"]) == len(expected_patches), "V06-patch-geometry", "empty/incomplete frozen control surface")
    parsed = []
    for patch, (point, normal, area) in zip(record["patches"], expected_patches):
        require(vector(patch["point"]) == tuple(point) and vector(patch["normal"]) == tuple(normal) and number(patch["area"]) == area,
                "V06-patch-geometry", "patch differs from frozen point/normal/area/order")
        expected_p = p0+sum((slope[a]*point[a] for a in range(3)), Q(0))
        close(number(patch["pressure"]), expected_p, "V06-affine-pressure", abs(p0)+sum(abs(slope[a]*point[a]) for a in range(3)))
        gradient = [[Q(v) for v in row] for row in preset["analytic_gradient"]] if preset["analytic_gradient"] is not None else [[Q(0), 1-2*point[1], Q(0)], [Q(0)]*3, [Q(0)]*3]
        actual_gradient = [vector(row) for row in patch["gradient"]]
        require(len(actual_gradient) == 3, "control-profile", "invalid gradient shape")
        for a, b in itertools.product(range(3), repeat=2):
            close(actual_gradient[a][b], gradient[a][b], "V06-gradient")
        expect = loads([(point, normal, area, expected_p, gradient)], mu, reference)
        patch_budgets = load_budgets([(point, normal, area, expected_p, gradient)], mu, reference)
        for name, vals in expect.items():
            for component, (actual, desired) in enumerate(zip(vector(patch[name]), vals)):
                close(actual, desired, "V06-patch-"+name, patch_budgets[name][component])
        velocity = (point[1], Q(0), Q(0)) if control_id == "couette" else (point[1]*(1-point[1]), Q(0), Q(0)) if control_id == "poiseuille" else tuple(sum(gradient[a][b]*point[b] for b in range(3)) for a in range(3))
        require(vector(patch["wall_velocity"]) == velocity, "V06-control-wall-velocity", "wrong analytic control wall velocity")
        work_terms = [(expect["pressure_force"][a]+expect["shear_force"][a])*velocity[a] for a in range(3)]
        close(number(patch["mechanical_work"]), sum(work_terms, Q(0)), "V06-patch-work", sum(map(abs, work_terms), Q(0)))
        parsed.append((point, normal, area, expected_p, gradient))
    expected = loads(parsed, mu, reference); budgets = load_budgets(parsed, mu, reference)
    for name, vals in expected.items():
        for component, (actual, desired) in enumerate(zip(vector(record[name]), vals)):
            close(actual, desired, "V06-total-"+name, budgets[name][component])
    return {"status": "validated-constitutive-control", "control_id": control_id, "native_solve": False}


def _validate_record(record, fixture, fixtures_sha256):
    require(isinstance(record, dict) and record.get("schema") == SCHEMA, "profile", "unsupported native record schema", "unsupported")
    require(record.get("operator_profile") == PROFILE, "operator-profile", "unadmitted corner operator profile", "unsupported")
    require(record.get("fixtures_sha256") == fixtures_sha256, "fixture-identity", "fixture bytes mismatch")
    require(record.get("accepted") is True and record.get("solve_id") == 1 and type(record["solve_id"]) is int,
            "accepted-solve", "fresh exporter must contain accepted solve 1")
    require(not any(k in record for k in ("dt", "time", "timestep")), "steady-clock", "steady record must not invent a clock")
    require(record.get("equations") == "steady-incompressible-newtonian-stokes" and record.get("frame") == "fixed-right-handed-SI",
            "physics", "wrong equations/frame")
    require(record.get("units") == {"velocity": "m/s", "pressure": "Pa", "gradient": "1/s", "force": "N", "moment": "N m", "volume_forcing": "N/m^3"},
            "units", "incorrect or missing SI field units")
    require(record.get("layout") == {"ordering": "x-fast", "velocity_offsets": [[bits(v) for v in off] for off in OFFSETS], "pressure_offset": [bits(Q(1, 2))]*3},
            "association", "wrong MAC ordering/offsets")
    approved_profile = read_json(Path(__file__).with_name("fixtures.json"))["discretization"]
    require(record.get("discretization") == approved_profile, "discretization", "unadmitted quadrature/wall/pivot/forcing semantics", "unsupported")
    require(record.get("controls") == fixture, "controls", "control identity changed")
    op = Operator(fixture); g = op.grid
    expected_geometry = {"counts": list(g.counts), "origin": list(map(bits, g.origin)), "spacing": list(map(bits, g.spacing)),
        "body_lower": None if g.lower is None else list(map(bits, g.lower)), "body_upper": None if g.upper is None else list(map(bits, g.upper)),
        "cell_mask": [int(g.fluid(c)) for c in tuples(g.counts)],
        "face_masks": [[g.mask(a, i) for i in tuples(g.face_shape(a))] for a in range(3)],
        "face_dual_volumes": [[bits(op.face_dual[g.key(a, i)]) for i in tuples(g.face_shape(a))] for a in range(3)],
        "pressure_components": [op.labels.get(c, -1) for c in tuples(g.counts)],
        "component_gauges": [{"component": op.labels[c], "cell": flat(c, g.counts), "pressure": bits(p)} for c, p in op.gauges.items()]}
    require(json.dumps(record.get("geometry"), sort_keys=True, separators=(",", ":")) == json.dumps(expected_geometry, sort_keys=True, separators=(",", ":")), "geometry", "geometry/masks/dual-volumes/components/gauges differ from fixture")
    require(record.get("geometry_sha256") == hashlib.sha256(json.dumps(expected_geometry, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
            "geometry-identity", "geometry digest mismatch")
    require(record.get("forcing_semantics") == fixture["forcing"], "forcing-semantics", "point-value and integrated-flux forcing cannot be interchanged", "unsupported")
    report = record["solve_report"]
    require(isinstance(report, dict), "profile", "solve_report must be a JSON object")
    require(report.get("backend") == "dense-scaled-partial-pivot" and report.get("actual_dofs") == len(op.active)+len(op.pressure_unknowns), "solve-report", "unsupported backend/DOF report", "unsupported")
    require(number(report["min_scaled_pivot"]) >= PIVOT_MIN, "V01-pivot", "native reported scaled pivot below frozen refusal", "unsupported")
    payload = integer(report["retained_payload_bytes"], 1, 2**31, "payload")
    payload_limit = integer(report["payload_limit_bytes"], 1, 2**31, "payload")
    require(payload <= payload_limit, "payload", "native retained payload exceeds caller limit")
    fields = record["fields"]; values = dict(op.prescribed); pressure = {}
    require(isinstance(fields["velocity"], list) and len(fields["velocity"]) == 3, "association", "three velocity arrays required")
    require(isinstance(fields.get("volume_forcing"), list) and len(fields["volume_forcing"]) == 3, "association", "three native point-forcing arrays required")
    for a in range(3):
        velocity = vector(fields["velocity"][a], math.prod(g.face_shape(a)))
        forcing = vector(fields["volume_forcing"][a], math.prod(g.face_shape(a)))
        for index in tuples(g.face_shape(a)):
            key = g.key(a, index); val = velocity[flat(index, g.face_shape(a))]; mask = g.mask(a, index)
            source = forcing[flat(index, g.face_shape(a))]
            if mask:
                physical_terms = op.analytic.force_terms(g.face_point(a, index), a)
                close(source, sum(physical_terms, Q(0)), "V07-analytic-forcing", sum(map(abs, physical_terms), Q(0)))
                op.face_forcing[key] = source*op.face_dual[key]
            else:
                require(source == 0, "masked-field", "masked forcing must be zero")
            if mask == 0:
                require(fields["velocity"][a][flat(index, g.face_shape(a))] == bits(0), "masked-field", "masked face must be canonical +0")
            elif mask == 2:
                close(val, op.prescribed[key], "V02-prescribed-face")
                values[key] = val
            else:
                values[key] = val
    p = vector(fields["pressure"], math.prod(g.counts))
    for c in tuples(g.counts):
        if g.fluid(c):
            pressure[c] = p[flat(c, g.counts)]
        else:
            require(fields["pressure"][flat(c, g.counts)] == bits(0), "masked-field", "masked pressure must be canonical +0")
    for c, desired in op.gauges.items():
        close(pressure[c], desired, "pressure-gauge")
    traces = record["outer_traces"]
    require(isinstance(traces, list) and len(traces) == len(op.outer_traces), "full-boundary-trace", "missing/duplicate outer trace")
    seen = set(); observed_outer = {}
    for trace in traces:
        key = (integer(trace["axis"], 0, 2), integer(trace["side"], -1, 1), vector(trace["point"]))
        require(key in op.outer_traces and key not in seen, "full-boundary-trace", "wrong/duplicate trace association")
        seen.add(key)
        captured = vector(trace["velocity"])
        if key[2] in observed_outer:
            require(observed_outer[key[2]] == captured, "full-boundary-trace", "inconsistent full-vector traces at an outer intersection")
        observed_outer[key[2]] = captured
        for actual, desired in zip(captured, op.outer_traces[key]):
            close(actual, desired, "V02-outer-trace")
    body_traces = record["body_traces"]
    require(isinstance(body_traces, list) and len(body_traces) == len(op.body_traces), "full-body-trace", "missing/duplicate body boundary trace")
    body_seen = set()
    for trace in body_traces:
        point = vector(trace["point"])
        require(point in op.body_traces and point not in body_seen, "full-body-trace", "wrong/duplicate body trace point")
        body_seen.add(point)
        require(vector(trace["velocity"]) == (Q(0),)*3, "V02-body-no-slip", "body full-vector trace is nonzero")
    for key, (component, point) in op.boundary_locations.items():
        if key.startswith("ghost:") and op.boundary_owner[key] == "outer":
            values[key] = observed_outer[point][component]
    residual = op.residuals(values, pressure)
    max_residual = 0.0
    for key in op.active:
        terms = residual[key]; defect = sum(terms, Q(0))
        close(defect, Q(0), "V01-momentum", sum(map(abs, terms), Q(0)))
        max_residual = max(max_residual, abs(float(defect)))
    max_div = 0.0
    for c, row in op.divergence.items():
        terms = [coef*values[key] for key, coef in row.items()]; defect = sum(terms, Q(0))
        close(defect, Q(0), "V01-divergence", sum(map(abs, terms), Q(0)))
        max_div = max(max_div, abs(float(defect/g.volume)))
    normalized_div = max_div*float(Q(fixture["L_ref"])/Q(fixture["U_ref"]))
    require(normalized_div <= 1e-8, "V02-divergence", "normalized physical divergence exceeds 1e-8")
    expected_sites = {key: {"component": component, "owner": op.boundary_owner[key], "point": list(map(bits, point))}
                      for key, (component, point) in op.boundary_locations.items()}
    require(json.dumps(record.get("reaction_sites"), sort_keys=True, separators=(",", ":")) == json.dumps(expected_sites, sort_keys=True, separators=(",", ":")),
            "reaction-sites", "wrong/missing prescribed face, ghost or corner reaction site")
    reactions = record["algebraic_reactions"]
    require(isinstance(reactions, dict) and set(reactions) == set(op.prescribed), "V04-reactions", "incomplete algebraic boundary reaction inventory")
    for key in op.prescribed:
        terms = residual[key]
        close(number(reactions[key]), sum(terms, Q(0)), "V04-reactions", sum(map(abs, terms), Q(0)))
    algebraic = {name: [Q(0)]*3 for name in ("body_force", "outer_force", "body_moment", "outer_moment")}
    force_terms = [[] for _ in range(3)]; moment_terms = [[] for _ in range(3)]
    algebraic_budgets = {name: [Q(0)]*3 for name in algebraic}
    global_force_budget = [Q(0)]*3; global_moment_budget = [Q(0)]*3
    reference = tuple(Q(x) for x in fixture["moment_reference"])
    for key in op.prescribed:
        component, point = op.boundary_locations[key]; owner = op.boundary_owner[key]
        force = [Q(0)]*3; force[component] = number(reactions[key])
        arm = tuple(point[a]-reference[a] for a in range(3)); moment = cross(arm, force)
        sign = -1 if owner == "body" else 1
        for a in range(3):
            algebraic[owner+"_force"][a] += sign*force[a]
            algebraic[owner+"_moment"][a] += sign*moment[a]
            algebraic_budgets[owner+"_force"][a] += abs(force[a])
            b, c = (a+1)%3, (a+2)%3
            algebraic_budgets[owner+"_moment"][a] += abs(arm[b]*force[c])+abs(arm[c]*force[b])
            force_terms[a].append(force[a]); moment_terms[a].append(moment[a])
    for a, index, key, mask in op.faces:
        if not mask:
            continue
        force = [Q(0)]*3; force[a] = op.face_forcing[key]
        point = g.face_point(a, index); moment = cross(tuple(point[b]-reference[b] for b in range(3)), force)
        for b in range(3):
            force_terms[b].append(force[b]); moment_terms[b].append(moment[b])
    locations = {key: (a, g.face_point(a, index)) for a, index, key, mask in op.faces if mask}
    locations.update(op.boundary_locations)
    for key, (component, point) in locations.items():
        magnitude = sum(map(abs, residual[key]))
        global_force_budget[component] += magnitude
        arm = tuple(point[a]-reference[a] for a in range(3))
        for a in range(3):
            b, c = (a+1)%3, (a+2)%3
            if component == c: global_moment_budget[a] += abs(arm[b])*magnitude
            if component == b: global_moment_budget[a] += abs(arm[c])*magnitude
    for a in range(3):
        close(sum(force_terms[a], Q(0)), Q(0), "V04-momentum-closure", global_force_budget[a]+sum(map(abs, force_terms[a]), Q(0)))
        close(sum(moment_terms[a], Q(0)), Q(0), "V04-moment-closure", global_moment_budget[a]+sum(map(abs, moment_terms[a]), Q(0)))
    for name, expected in algebraic.items():
        for component, (actual, desired) in enumerate(zip(vector(record["algebraic_loads"][name]), expected)):
            close(actual, desired, "V04-"+name, algebraic_budgets[name][component])
    strain_values = op.strains_at(values)
    dissipation = sum((s.weight*e*e for s, e in zip(op.strains, strain_values)), Q(0))
    work_terms = [number(reactions[key])*values[key] for key in op.prescribed]
    forcing_work = sum((op.face_forcing.get(key, Q(0))*values[key] for key in values), Q(0))
    dissipation_scale = sum(s.weight*sum(abs(coef*values[key]) for key, coef in s.coefficients.items())**2 for s in op.strains)
    close(number(record["dissipation"]), dissipation, "V04-dissipation", dissipation_scale)
    close(dissipation, sum(work_terms, Q(0))+forcing_work, "V04-work",
          dissipation_scale+sum(sum(map(abs, residual[key]))*abs(values[key]) for key in op.prescribed)+abs(forcing_work))
    expected_patches = wall_patches(g); native_patches = record["wall_patches"]
    require(isinstance(native_patches, list) and len(native_patches) == len(expected_patches), "wall-patches", "incomplete wall patch inventory")
    patches = []; trace_scales = []
    for native, patch in zip(native_patches, expected_patches):
        c, a, side, point, normal, area = patch
        integer(native["cell"], 0, math.prod(g.counts)-1, "wall-patch-geometry")
        integer(native["axis"], 0, 2, "wall-patch-geometry")
        integer(native["side"], -1, 1, "wall-patch-geometry")
        require(native["cell"] == flat(c, g.counts) and native["axis"] == a and native["side"] == side and
                vector(native["point"]) == point and vector(native["normal"]) == normal and number(native["area"]) == area,
                "wall-patch-geometry", "incorrect wall patch point/normal/area/order")
        require(all(v == 0 for v in vector(native["velocity"])), "V02-body-no-slip", "stationary body trace is nonzero")
        wall_p, grad, p_scale, grad_scale = wall_trace(op, patch, values, pressure, with_scales=True)
        close(number(native["pressure"]), wall_p, "wall-pressure-stencil", p_scale)
        exported_grad = [vector(row) for row in native["gradient"]]
        require(len(exported_grad) == 3, "wall-gradient-stencil", "wrong gradient shape")
        for i, j in itertools.product(range(3), repeat=2):
            close(exported_grad[i][j], grad[i][j], "wall-gradient-stencil", grad_scale[i][j])
        patches.append((point, normal, area, wall_p, grad)); trace_scales.append((p_scale, grad_scale))
    total = loads(patches, op.analytic.mu, tuple(Q(x) for x in fixture["moment_reference"]))
    budgets = load_budgets(patches, op.analytic.mu, reference, trace_scales)
    for name, expected in total.items():
        for component, (actual, desired) in enumerate(zip(vector(record["physical_loads"][name]), expected)):
            close(actual, desired, "N2-physical-"+name, budgets[name][component])
    result = {"status": "validated-discrete-record", "schema": SCHEMA, "operator_profile": PROFILE, "max_momentum_defect_N": max_residual,
              "max_divergence_1_s": max_div, "mixed_dofs": len(op.active)+len(op.pressure_unknowns),
              "physical_loads": {name: list(map(float, v)) for name, v in total.items()},
              "algebraic_loads": {name: list(map(float, v)) for name, v in algebraic.items()},
              "claims": {"N1": "pending-suite-controls", "N2": "pending-suite-controls", "N3": "pending", "N7": "pending"},
              "provenance": "not-established-by-numerical-validation", "dissipation_W": float(dissipation)}
    if "constitutive_controls" in record:
        controls = record["constitutive_controls"]
        require(isinstance(controls, list), "control-profile", "control collection must be an array")
        require(all(isinstance(c, dict) for c in controls), "control-profile", "every control must be a JSON object")
        ids = [c.get("control_id") for c in controls]
        approved = read_json(Path(__file__).with_name("fixtures.json"))["control_presets"]
        require(len(ids) == len(set(ids)) and set(ids) == set(approved), "control-fixture", "incomplete/duplicate frozen control inventory")
        result["constitutive_controls"] = [validate_control(c) for c in controls]
    else:
        result["constitutive_controls"] = "unavailable; complete N1/N2 controls still pending"
    if len(op.active)+len(op.pressure_unknowns) <= EXACT_MAX_DOFS:
        exact_u, exact_p = op.solve_exact()
        for key in op.active:
            close(values[key], exact_u[key], "exact-state", Q(fixture["U_ref"]))
        for c in pressure:
            close(pressure[c], exact_p[c], "exact-pressure", op.analytic.mu*Q(fixture["U_ref"])/Q(fixture["L_ref"]))
        result["rational_solve"] = "compared"
    else:
        result["rational_solve"] = "unsupported-above-128-dofs; all-independent-rows-checked"
    if op.analytic.polys is not None:
        # Pointwise continuum values are references, not exact discrete solutions.
        velocity_error = sum((op.face_dual[key]*(values[key]-op.analytic.velocity(g.face_point(a, i))[a])**2
                              for a, i, key, mask in op.faces if mask == 1), Q(0))
        pressure_error = sum((g.volume*(pressure[c]-op.analytic.pressure(g.center(c)))**2 for c in pressure), Q(0))
        result["coarse_mms_velocity_l2"] = math.sqrt(float(velocity_error))
        result["coarse_mms_pressure_l2"] = math.sqrt(float(pressure_error))
        result["V07"] = "pending-fine-refinement; point-value-times-dual-volume forcing"
    return result


def validate_record(record, fixture, fixtures_sha256):
    try:
        return _validate_record(record, fixture, fixtures_sha256)
    except (KeyError, TypeError, ValueError, OverflowError) as exc:
        raise Failure("invalid", "profile", "missing/malformed required record member: " + str(exc)) from exc
