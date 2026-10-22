"""Prospective exact certificates; no selected work on import.

High precision only generates untrusted centers. Fractions verify every bound.
No assertions, tolerance-rank decisions, estimated-bound fallback or pivot floor.
"""
from fractions import Fraction as F
from math import isqrt
from time import monotonic


def require(condition, message):
    if not condition:
        raise RuntimeError('CV3 input/certificate guard: ' + message)


class Budget:
    def __init__(self, seconds=1800):
        require(0 < seconds <= 1800, 'fixed case time cap')
        self.deadline = monotonic() + seconds

    def check(self):
        require(monotonic() <= self.deadline, 'independent case time resource refusal')


def dot(a, b):
    require(len(a) == len(b), 'complete dot vectors')
    return sum((x*y for x, y in zip(a, b)), F(0))


def action(a, x):
    return [dot(row, x) for row in a]


def transpose(a):
    require(a and a[0] and all(len(row) == len(a[0]) for row in a), 'rectangular nonempty matrix')
    return list(map(list, zip(*a)))


def norm_inf(a):
    require(a and a[0] and all(len(row) == len(a[0]) for row in a), 'rectangular matrix norm')
    return max(sum(map(abs, row), F(0)) for row in a)


def sparse_columns(a):
    at = transpose(a)
    return [[(i, v) for i, v in enumerate(col) if v] for col in at]


def multiply(a, b, budget):
    require(a and b and len(a[0]) == len(b), 'matrix product dimensions')
    cols = sparse_columns(b)
    out = []
    for row in a:
        budget.check()
        out.append([sum((row[k]*v for k, v in col), F(0)) for col in cols])
    return out


def sqrt_interval(q, bits=256):
    q = F(q)
    require(q >= 0 and bits == 256, 'nonnegative exact square root and frozen bits')
    scale = 1 << bits
    scaled = q*scale*scale
    m = isqrt(scaled.numerator // scaled.denominator)
    lo = F(m, scale)
    return lo, lo if lo*lo == q else F(m+1, scale)


def norm_interval(values):
    if len(values) == 1:
        value = abs(F(values[0]))
        return value, value
    return sqrt_interval(sum((F(v)*v for v in values), F(0)))


def mpf_fraction(value):
    sign, mantissa, exponent, bitcount = value._mpf_
    require(bitcount >= 0 and mantissa >= 0, 'finite real mpf tuple')
    out = F((-1 if sign else 1)*mantissa)
    return out*(2**exponent if exponent >= 0 else F(1, 2**(-exponent)))


def generate(matrix, rhs, budget):
    """Fixed120dps pivoted LU, inverse and center. Not itself an enclosure."""
    from mpmath import mp
    n = len(matrix)
    require(0 < n <= 800 and all(len(r) == n for r in matrix), 'bounded square target')
    require(len(rhs) == n, 'complete target RHS')
    ctx = mp.clone(); ctx.dps = 120
    def convert(q):
        q = F(q)
        value = ctx.mpf(q.numerator)/ctx.mpf(q.denominator)
        require(ctx.isfinite(value), 'finite rational generator input')
        return value
    budget.check()
    a = ctx.matrix([[convert(v) for v in row] for row in matrix])
    lu, pivots = ctx.LU_decomp(a, overwrite=False, use_cache=False)
    require(all(ctx.isfinite(lu[i, j]) for i in range(n) for j in range(n)), 'finite LU generator')
    require(all(lu[i, i] != 0 for i in range(n)), 'nonzero generator pivot')
    def solve(b):
        budget.check()
        z = ctx.U_solve(lu, ctx.L_solve(lu, b, pivots))
        require(all(ctx.isfinite(v) for v in z), 'finite generated center')
        return [mpf_fraction(v) for v in z]
    center = solve(ctx.matrix([convert(v) for v in rhs]))
    inverse = [[F(0)]*n for _ in range(n)]
    for j in range(n):
        unit = ctx.matrix(n, 1); unit[j] = 1
        col = solve(unit)
        for i in range(n):
            inverse[i][j] = col[i]
    return inverse, center


def inverse_defect(matrix, inverse, budget):
    """Exact streamed ||I-R*C||inf; no dense defect allocation."""
    n = len(matrix)
    require(n > 0 and len(inverse) == n and all(len(r) == n for r in matrix+inverse), 'complete inverse certificate')
    cols = sparse_columns(matrix)
    delta = F(0)
    for i, row in enumerate(inverse):
        budget.check()
        total = F(0)
        for j, col in enumerate(cols):
            value = sum((row[k]*v for k, v in col), F(0))
            total += abs(F(i == j)-value)
        delta = max(delta, total)
    require(delta < 1, 'verified inverse defect strictly below one')
    return delta


def certify(matrix, rhs, inverse, center, budget):
    delta = inverse_defect(matrix, inverse, budget)
    residual = [b-v for b, v in zip(rhs, action(matrix, center))]
    require(len(rhs) == len(center) == len(matrix), 'complete certified center')
    preconditioned = action(inverse, residual)
    eta = max(map(abs, preconditioned), default=F(0))/(1-delta)
    # This is an exact Neumann-series enclosure, not precision agreement.
    return {'delta': delta, 'eta': eta, 'center': center,
            'residual_inf': max(map(abs, residual), default=F(0)),
            'inverse_norm_inf': norm_inf(inverse), 'precision_digits': 120,
            'inverse_generator': inverse}


def solve_certified(matrix, rhs, budget):
    inverse, center = generate(matrix, rhs, budget)
    return certify(matrix, rhs, inverse, center, budget)


def ratio_interval(a, b):
    require(0 <= a[0] <= a[1] and 0 < b[0] <= b[1], 'positive norm denominator enclosure')
    return a[0]/b[1], a[1]/b[0]


def add_radius(interval, radius):
    require(radius >= 0 and 0 <= interval[0] <= interval[1], 'nonnegative norm radius')
    return max(F(0), interval[0]-radius), interval[1]+radius


def uncertainty_budget(interval, radius):
    require(0 <= radius and 0 < interval[0] <= interval[1], 'positive error for one-percent uncertainty')
    return radius <= interval[0]/100


def finest_gate(interval):
    return 0 <= interval[0] <= interval[1] <= F(1, 20)


def order_gate(coarse, fine):
    # r >= 2^(9/10) iff r^10 >= 2^9; no rounded log or threshold constant.
    require(0 <= fine[0] <= fine[1] and 0 <= coarse[0] <= coarse[1], 'ordered error intervals')
    return fine[1] > 0 and coarse[0]**10 >= (1 << 9)*fine[1]**10


def logarithm2_interval(q):
    """Exact rational atanh-series enclosure, fixed64 terms; display only.

    Order decisions themselves use the exact integer-power comparison above.
    """
    q = F(q); require(q > 0, 'positive exact order logarithm')
    k = q.numerator.bit_length()-q.denominator.bit_length()
    scale = F(1 << k) if k >= 0 else F(1,1 << (-k))
    if q < scale:
        k -= 1; scale /= 2
    require(scale <= q < 2*scale, 'exact binary logarithm reduction')
    def atanh_bounds(t):
        require(0 <= t <= F(1,2), 'bounded log-series parameter')
        total = F(0); power = t
        for j in range(64):
            total += 2*power/(2*j+1); power *= t*t
        tail = 2*power/(129*(1-t*t))
        return total,total+tail
    def ln_reduced(y):
        t = (y-1)/(y+1); require(0 <= t <= F(1,3), 'reduced exact logarithm argument')
        # Display-only rigorous argument enclosure prevents powers of a huge
        # exact error-ratio denominator from dominating source resource bounds.
        step = 1 << 256; scaled = t*step
        floor = scaled.numerator//scaled.denominator
        ceil = -((-scaled).numerator//(-scaled).denominator)
        return atanh_bounds(F(floor,step))[0],atanh_bounds(F(ceil,step))[1]
    a,b = ln_reduced(q/scale),ln_reduced(F(2))
    require(b[0] > 0, 'positive exact log-two enclosure')
    lo,hi = k+a[0]/b[1],k+a[1]/b[0]
    # Display enclosure only. Keep exact order/threshold comparisons separately;
    # outward dyadic endpoints avoid enormous decimal rational report payloads.
    scale = 1 << 128
    lower = (lo*scale).numerator//(lo*scale).denominator
    upper = -((-hi*scale).numerator//(-hi*scale).denominator)
    return F(lower,scale),F(upper,scale)


def observed_order_interval(coarse,fine):
    if coarse[0] <= 0 or fine[1] <= 0:
        return {'lower':None,'upper':None,'status':'unresolved nonpositive norm endpoint'}
    lower = logarithm2_interval(coarse[0]/fine[1])[0]
    upper = None if fine[0] <= 0 else logarithm2_interval(coarse[1]/fine[0])[1]
    return {'lower':lower,'upper':upper,'status':'outward exact logarithm enclosure'}


def json_exact(value):
    if isinstance(value, F):
        require(value.numerator.bit_length() <= 3000000 and value.denominator.bit_length() <= 3000000,
                'bounded exact report integer bit resource limit')
        return {'numerator': str(value.numerator), 'denominator': str(value.denominator)}
    if isinstance(value, dict):
        return {str(k): json_exact(v) for k, v in value.items()}
    if isinstance(value, (tuple, list)):
        return [json_exact(v) for v in value]
    return value
