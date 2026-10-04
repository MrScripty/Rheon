"""Fixed finite pressure-gradient fixture, not a surface-tension discretization."""
import numpy as np


def pressure_fixture():
    # Four pressures and six oriented faces. Each row uses A/ell times (p_right-p_left).
    # Faces: 0->1, 1->2, 2->3, 3->0, 0->2, 1->3; A/ell: 2, 3/2, 1/4, 2, 4, 1/2.
    gradient = np.array([[-2, 2, 0, 0], [0, -1.5, 1.5, 0], [0, 0, -.25, .25],
                         [2, 0, 0, -2], [-4, 0, 4, 0], [0, -.5, 0, .5]])
    pressure = np.array([2., 5., -1., 7.])
    # Independent hand arithmetic: 2*3, (3/2)*(-6), (1/4)*8, 2*(-5), 4*(-3), (1/2)*2.
    expected_force = np.array([6., -9., 2., -10., -12., 1.])
    return gradient, pressure, expected_force


def check_pressure_force(candidate, tolerance=1e-10):
    expected = pressure_fixture()[2]
    candidate = np.asarray(candidate)
    if candidate.shape != expected.shape or not np.all(np.isfinite(candidate)):
        raise ValueError('T7 force must have six finite components')
    residual = float(np.linalg.norm(candidate-expected))
    if residual > tolerance:
        raise ValueError(f'T7 independent pressure-force mismatch: {residual}')
    return residual
