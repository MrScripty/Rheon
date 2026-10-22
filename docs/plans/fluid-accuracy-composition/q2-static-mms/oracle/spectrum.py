"""Prospective exact verified H1/L2 spectrum; generators are untrusted.

This is a finite-instance diagnostic. No beta cutoff or gauge is introduced.
"""
from fractions import Fraction as F
from validated import require, generate, inverse_defect, transpose, multiply, norm_inf, sqrt_interval, dot


def certified_spectrum(op, budget):
    g, b = op['H1'], op['B']; nv, nc = len(g), len(b)
    require(g == transpose(g) and len(g) == len(b[0]), 'consistent symmetric H1 dimensions')
    r, unused = generate(g, [F(0)]*nv, budget)
    delta = inverse_defect(g, r, budget)
    inverse_error_inf = delta*norm_inf(r)/(1-delta)
    rt = transpose(r)
    skew = [[r[i][j]-rt[i][j] for j in range(nv)] for i in range(nv)]
    rs = [[(r[i][j]+rt[i][j])/2 for j in range(nv)] for i in range(nv)]
    # G^-1-Rs is symmetric, hence its spectral norm is bounded by its row norm.
    inverse_error_2 = inverse_error_inf+norm_inf(skew)/2
    t0 = multiply(multiply(b, rs, budget), transpose(b), budget)
    require(t0 == transpose(t0), 'exact symmetric spectral center')
    eps_t = norm_inf(b)*norm_inf(transpose(b))*inverse_error_2
    q = [op['volumes'][c] for c in op['cells']]
    require(len(q) == nc and all(v > 0 for v in q), 'actual positive pressure L2 volumes')
    d_intervals = []
    for v in q:
        lo, hi = sqrt_interval(v)
        require(lo > 0, 'resolved positive pressure-mass square root')
        d_intervals.append((1/hi, 1/lo))
    d0 = [(lo+hi)/2 for lo, hi in d_intervals]
    eps_d = max((hi-lo)/2 for lo, hi in d_intervals)
    d_bound = max(hi for lo, hi in d_intervals); d0_bound = max(d0)
    eps_s = d_bound*d_bound*eps_t+eps_d*(d_bound+d0_bound)*norm_inf(t0)
    s0 = [[d0[i]*t0[i][j]*d0[j] for j in range(nc)] for i in range(nc)]
    # The external supervisor fixes BLAS/OpenMP1. SciPy supplies only a center.
    import numpy as np
    from scipy.linalg import eigh
    budget.check()
    floating = np.array([[float(v) for v in row] for row in s0], dtype=float)
    require(bool(np.isfinite(floating).all()), 'finite ordinary spectral generator input')
    eigenvalues, eigenvectors = eigh(floating, driver='evr', check_finite=True)
    require(eigenvalues.shape == (nc,) and eigenvectors.shape == (nc, nc), 'complete spectral generator dimensions')
    require(bool(np.isfinite(eigenvalues).all()) and bool(np.isfinite(eigenvectors).all()), 'finite spectral generator')
    values = [F.from_float(float(v)) for v in eigenvalues]
    require(all(values[i] <= values[i+1] for i in range(nc-1)), 'sorted generated spectrum')
    v = [[F.from_float(float(eigenvectors[i, j])) for j in range(nc)] for i in range(nc)]
    vt = transpose(v)
    vtv = multiply(vt, v, budget)
    orth_defect = [[vtv[i][j]-F(i == j) for j in range(nc)] for i in range(nc)]
    omega = norm_inf(orth_defect)
    require(omega < 1, 'verified spectral basis orthogonality defect below one')
    vl = [[v[i][j]*values[j] for j in range(nc)] for i in range(nc)]
    reconstructed = multiply(vl, vt, budget)
    residual = [[s0[i][j]-reconstructed[i][j] for j in range(nc)] for i in range(nc)]
    require(residual == transpose(residual), 'symmetric exact spectral residual')
    distance = (1+omega)*omega/(1-omega)
    epsilon = norm_inf(residual)+distance*(2+omega)*max(map(abs, values))+eps_s
    intervals = [(x-epsilon, x+epsilon) for x in values]
    beta = []
    for lo, hi in intervals:
        require(hi >= 0, 'consistent nonnegative Gram spectral upper endpoint')
        # Zero here is a mathematical lower bound for an SPD Gram operator, not
        # a positivity threshold. A nonpositive computed lower stays unresolved.
        beta.append((F(0) if lo <= 0 else sqrt_interval(lo)[0], sqrt_interval(hi)[1]))
    physical_mode = [d0[i]*v[i][0] for i in range(nc)]
    alternating = [F((-1)**((c % op['n'])+(c // op['n']))) for c in op['cells']]
    alt_norm2 = sum((volume*x*x for volume, x in zip(q, alternating)), F(0))
    alt_center = dot(alternating, [dot(row, alternating) for row in t0])/alt_norm2
    # T's spectral error bounds a^T(T-T0)a in original pressure coordinates.
    alt_radius = eps_t*sum((x*x for x in alternating), F(0))/alt_norm2
    constant = [F(1)]*nc
    const_norm2 = sum(q, F(0))
    const_center = dot(constant, [dot(row, constant) for row in t0])/const_norm2
    const_radius = eps_t*nc/const_norm2
    mode_norm2 = sum((volume*x*x for volume, x in zip(q, physical_mode)), F(0))
    require(mode_norm2 > 0, 'nonzero generated pressure pattern')
    alt_overlap2 = sum((volume*a*x for volume, a, x in zip(q, alternating, physical_mode)), F(0))**2/(alt_norm2*mode_norm2)
    const_overlap2 = sum((volume*x for volume, x in zip(q, physical_mode)), F(0))**2/(const_norm2*mode_norm2)
    def normalized_pattern(pattern,norm2):
        lo,hi = sqrt_interval(norm2)
        require(lo > 0, 'resolved positive pattern Qp norm')
        return [(x/hi,x/lo) if x >= 0 else (x/lo,x/hi) for x in pattern]
    return {'definition': 'Qp^-1/2 B H1^-1 B^T Qp^-1/2',
            'inverse_defect': delta, 'inverse_error_inf': inverse_error_inf,
            'orthogonality_defect': omega, 'spectral_center_residual_inf': norm_inf(residual),
            'spectral_radius': epsilon, 'eigenvalue_intervals': intervals, 'beta_intervals': beta,
            'positive_beta_lower_resolved': intervals[0][0] > 0,
            'smallest_mode_pressure_coefficients': physical_mode,
            'mode_claim': 'generator direction; no individual eigenvector enclosure or clustered-mode uniqueness claim',
            'alternating_cell_pressure': alternating,
            'alternating_Qp_normalized_intervals':normalized_pattern(alternating,alt_norm2),
            'constant_Qp_normalized_intervals':normalized_pattern(constant,const_norm2),
            'generated_mode_Qp_norm_squared':mode_norm2,
            'alternating_rayleigh_interval': (alt_center-alt_radius, alt_center+alt_radius),
            'constant_rayleigh_interval': (const_center-const_radius, const_center+const_radius),
            'generated_mode_squared_alternating_overlap': alt_overlap2,
            'generated_mode_squared_constant_overlap': const_overlap2,
            'inverse_generator': r, 'spectral_generator_values': values,
            'spectral_generator_vectors': v,
            'rank_exact': op['rank'], 'pressure_rows': nc, 'constant_mode_retained': True}
