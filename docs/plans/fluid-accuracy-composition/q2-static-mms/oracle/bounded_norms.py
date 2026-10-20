"""Prospective exact-moment L2 / enclosing Bernstein Euclidean L1.

No quadrature samples or candidate imports. Nothing executes on import.
"""
from fractions import Fraction as F
from heapq import heappush, heappop
import exact_polynomial as ep
from validated import require, dot, norm_interval, sqrt_interval


def vector_value(polynomials, point):
    return tuple(ep.evaluate(p, point) for p in polynomials)


def add_vector(a, b):
    require(len(a) == len(b), 'vector polynomial dimensions')
    return tuple(ep.add(x, y) for x, y in zip(a, b))


def triangle_area(triangle):
    a, b, c = triangle
    area = ep.cross((b[0]-a[0], b[1]-a[1]), (c[0]-a[0], c[1]-a[1]))/2
    require(area > 0, 'positive exact norm triangle')
    return area


def midpoint(a, b):
    return tuple((x+y)/2 for x, y in zip(a, b))


def triangle_bernstein(polynomials, triangle):
    # Degree2 Bernstein coefficients: vertices plus three mixed coefficients.
    require(all(i+j <= 2 for p in polynomials for i, j in p), 'quadratic vector reconstruction')
    vertices = [vector_value(polynomials, p) for p in triangle]
    mixed = []
    for i, j in ((0, 1), (1, 2), (2, 0)):
        v = vector_value(polynomials, midpoint(triangle[i], triangle[j]))
        mixed.append(tuple(2*x-(a+b)/2 for x, a, b in zip(v, vertices[i], vertices[j])))
    return vertices+mixed


def triangle_l1_bounds(polynomials, triangle):
    area = triangle_area(triangle)
    coefficients = triangle_bernstein(polynomials, triangle)
    integral = tuple(area*sum((b[d] for b in coefficients), F(0))/6 for d in range(len(polynomials)))
    lower = norm_interval(integral)[0]
    upper = area*sum((norm_interval(b)[1] for b in coefficients), F(0))/6
    require(0 <= lower <= upper, 'enclosing triangle norm')
    return lower, upper


def segment_bernstein(polynomials, a, b):
    require(all(i+j <= 2 for p in polynomials for i, j in p), 'quadratic segment reconstruction')
    va, vb, vm = vector_value(polynomials, a), vector_value(polynomials, b), vector_value(polynomials, midpoint(a, b))
    mixed = tuple(2*x-(y+z)/2 for x, y, z in zip(vm, va, vb))
    return va, mixed, vb


def segment_l1_bounds(polynomials, a, b, weight):
    # weight is a rational measure independent of source-edge norm. For traction
    # q=delta_sigma*dperp, source parameter dt is exactly this measure.
    require(weight > 0, 'positive parameter measure')
    coefficients = segment_bernstein(polynomials, a, b)
    integral = tuple(weight*sum((c[d] for c in coefficients), F(0))/3 for d in range(len(polynomials)))
    lower = norm_interval(integral)[0]
    upper = weight*sum((norm_interval(c)[1] for c in coefficients), F(0))/3
    require(0 <= lower <= upper, 'enclosing segment norm')
    return lower, upper


def triangle_children(triangle):
    pairs = ((0, 1, 2), (1, 2, 0), (2, 0, 1))
    lengths = [sum(((triangle[j][d]-triangle[i][d])**2 for d in range(2)), F(0)) for i, j, k in pairs]
    selected = max(range(3), key=lambda i: (lengths[i], -i))
    i, j, k = pairs[selected]; a, b, c = triangle[i], triangle[j], triangle[k]
    m = midpoint(a, b)
    return (a, m, c), (m, b, c)


class Subdivision:
    """A case-wide resource ledger shared by every norm and denominator."""
    def __init__(self, budget):
        self.budget = budget
        self.created = 0
        self.maximum_depth = 0

    def allocate(self, depth):
        self.budget.check()
        require(depth <= 24, 'bounded L1 depth24 refusal')
        self.created += 1
        self.maximum_depth = max(self.maximum_depth, depth)
        require(self.created <= 2000000, 'bounded L1 two-million-piece resource refusal')


def l1_enclosure(items, tolerance, ledger):
    """items: ('triangle',polys,triangle) or ('segment',polys,(a,b,weight))."""
    require(F(0) < tolerance, 'positive frozen integral width')
    heap = []; sequence = 0; lower = F(0); upper = F(0)
    def push(kind, p, geometry, depth):
        nonlocal sequence, lower, upper
        ledger.allocate(depth)
        bounds = triangle_l1_bounds(p, geometry) if kind == 'triangle' else segment_l1_bounds(p, *geometry)
        lo, hi = bounds; lower += lo; upper += hi
        heappush(heap, (-(hi-lo), sequence, kind, p, geometry, depth, lo, hi)); sequence += 1
    for kind, p, geometry in items:
        require(kind in ('triangle', 'segment') and len(p) in (1, 2), 'declared L1 piece schema')
        push(kind, p, geometry, 0)
    require(heap, 'nonempty physical norm domain')
    while upper-lower > tolerance:
        ledger.budget.check()
        width, seq, kind, p, geometry, depth, lo, hi = heappop(heap)
        require(width < 0, 'positive unresolved L1 width')
        lower -= lo; upper -= hi
        if kind == 'triangle':
            for child in triangle_children(geometry):
                push(kind, p, child, depth+1)
        else:
            a, b, weight = geometry; m = midpoint(a, b)
            push(kind, p, (a, m, weight/2), depth+1)
            push(kind, p, (m, b, weight/2), depth+1)
    return lower, upper


def fan(poly):
    out = []
    for i in range(1, len(poly)-1):
        tri = poly[0], poly[i], poly[i+1]
        determinant = ep.cross((tri[1][0]-tri[0][0], tri[1][1]-tri[0][1]), (tri[2][0]-tri[0][0], tri[2][1]-tri[0][1]))
        require(determinant >= 0, 'CCW norm fan')
        if determinant:
            out.append(tri)
    return out


def square_polynomial(vector):
    return ep.add(*(ep.multiply(p, p) for p in vector))


def volume_l2_squared(items):
    return sum((ep.triangle_integral(square_polynomial(p), tri) for p, tri in items), F(0))


def supremum_bounds(items):
    """Enclosing upper bound plus sampled lower, not an exact attained max."""
    lower = F(0); upper = F(0)
    for kind, p, geometry in items:
        if kind == 'triangle':
            coefficients = triangle_bernstein(p, geometry)
            samples = [vector_value(p, v) for v in geometry]
        else:
            a, b, weight = geometry
            coefficients = segment_bernstein(p, a, b)
            samples = [vector_value(p, v) for v in (a, midpoint(a, b), b)]
        lower = max(lower, max((norm_interval(v)[0] for v in samples), default=F(0)))
        upper = max(upper, max((norm_interval(v)[1] for v in coefficients), default=F(0)))
    require(lower <= upper, 'supremum enclosure')
    return lower, upper
