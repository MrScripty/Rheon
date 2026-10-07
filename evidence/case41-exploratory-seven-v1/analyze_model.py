"""External captured-model diagnostics only; no Newton correction or native call."""
from pathlib import Path
from fractions import Fraction as Q
import importlib.util
import json
import sys
import mpmath as mp

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def run(dps):
    mp.mp.dps = dps
    checked = json.loads((P / 'analysis-v2-normal.json').read_text())
    require = module('captured_model_reader', P / 'analyze_capture_v2.py').require
    require(checked['status'] == 'PASS_COMPLETE_EXPLORATORY_SEVEN_EQUATION_CAPTURE', 'validated complete capture')
    algebra = module('captured_exact_algebra', ROOT / 'evidence/forcing-case41-scale-diagnosis-v1/analyze.py')
    metric = module('captured_mass_metric', ROOT / 'evidence/case41-scaled-fd-preparation-v1/analyze_metric_v2.py')
    prefix = 'test research_public_call::case41_paired_candidate_roster ... '
    records = [json.loads(line.removeprefix(prefix)) for line in (P / 'native.log').read_text().splitlines()
               if line.startswith('{') or line.startswith(prefix)]
    baseline = next(row for row in records if row['event'] == 'fd_equation' and row['column'] == -1)
    J = [[Q(x) for x in row] for row in checked['native_linear_model']]
    A, B = [row[:6] for row in J], [row[6:] for row in J]
    nominal_mass = Q(27, 8)
    M = [[sum(Q(baseline['end_mass'][i]) * Q(baseline['r'][i][d][j]) * Q(baseline['r'][i][d][k])
              for i in range(16) for d in range(2)) for k in range(22)] for j in range(22)]
    metric.exact_positive_definite(M)
    W = [[x / nominal_mass for x in row] for row in algebra.inverse(M)]
    BtW = algebra.mul(algebra.transpose(B), W)
    pressure_gram = algebra.mul(BtW, B)
    metric.exact_positive_definite(pressure_gram)
    projection = algebra.mul(algebra.mul(B, algebra.inverse(pressure_gram)), BtW)
    projected_A = algebra.mul(projection, A)
    complement_A = [[x - y for x, y in zip(a, b)] for a, b in zip(A, projected_A)]
    require(algebra.mul(BtW, complement_A) == [[Q(0)] * 6 for _ in range(16)], 'exact pressure-complement orthogonality')
    complement_gram = algebra.mul(algebra.mul(algebra.transpose(complement_A), W), complement_A)
    pivots = metric.exact_positive_definite(complement_gram)
    require(algebra.mul(projection, projection) == projection, 'exact pressure projector idempotence')

    def mpq(value):
        return mp.mpf(value.numerator) / value.denominator

    def matrix(values):
        return mp.matrix([[mpq(value) for value in row] for row in values])

    scale = [Q(1)] * 6 + [nominal_mass] * 16
    scaled_J = [[x * scale[j] / nominal_mass for j, x in enumerate(row)] for row in J]
    whiten = mp.cholesky(matrix(W)).T
    dual_J = whiten * matrix([[x * scale[j] for j, x in enumerate(row)] for row in J])

    def singular_summary(values):
        singular = list(mp.svd(values, compute_uv=False))
        return dict(singular_values=[float(x) for x in singular],
                    condition_2=float(singular[0] / singular[-1]))

    gram_mp = matrix(complement_gram)
    eigenvalues, vectors = mp.eigsy(gram_mp)
    directions = []
    for j in range(6):
        direction = [vectors[i, j] for i in range(6)]
        pivot = max(range(6), key=lambda i: abs(direction[i]))
        if direction[pivot] < 0:
            direction = [-x for x in direction]
        directions.append(dict(sigma=float(mp.sqrt(eigenvalues[j])),
                               acceleration_direction=[float(x) for x in direction]))
    native_r = list(map(Q, baseline['rate']))
    pressure_r = algebra.mv(projection, native_r)
    complement_r = [x - y for x, y in zip(native_r, pressure_r)]
    square = lambda v: algebra.dot(v, algebra.mv(W, v))
    require(square(native_r) == square(pressure_r) + square(complement_r), 'exact weighted residual Pythagoras')
    exact_A = [[Q(x) for x in row] for row in checked['exact_stored_field_FD_differences']]
    entry_gap = max(abs(x - y) for row, exact_row in zip(A, exact_A) for x, y in zip(row, exact_row))
    return dict(status='PASS_CAPTURED_MODEL_DIAGNOSTICS_ONLY', precision_digits=dps,
                raw=singular_summary(matrix(J)), uniform_physical_scaled=singular_summary(matrix(scaled_J)),
                dual_mass_scaled=singular_summary(dual_J),
                complementary_acceleration=singular_summary(whiten * matrix(complement_A)),
                complementary_acceleration_directions= directions,
                exact_complementary_Gram_LDL_pivots=[str(x) for x in pivots],
                exact_pressure_projector_idempotent=True, exact_weighted_complement=True,
                dual_mass_residual_norm=float(mp.sqrt(mpq(square(native_r)))),
                dual_mass_pressure_range_norm=float(mp.sqrt(mpq(square(pressure_r)))),
                dual_mass_pressure_complement_norm=float(mp.sqrt(mpq(square(complement_r)))),
                native_FD_vs_exact_stored_field_FD_max_entry_gap=float(entry_gap),
                scales=dict(length=1, velocity=1, time=1, mass='27/8', acceleration=1,
                            force='27/8', pressure_coordinate='27/8'),
                native_Newton_gate_unchanged=1e-13, native_physical_rate_gate_unchanged=1e-11,
                new_native_equations=0, new_Newton_corrections=0, new_owner_advances=0,
                historical_memory_qualified=False, FD_truncation_bound=None,
                smooth_derivative_certified=False, arithmetic_floor_claimed=False,
                physical_state_error_bound=None, production_adoption=False,
                scope='Conditioning of the captured prescribed FD/B model in frozen metrics; no correction, trajectory remedy, smooth derivative or threshold justification.')


if __name__ == '__main__':
    print(json.dumps(run(int(sys.argv[1]) if len(sys.argv) > 1 else 80), indent=2, sort_keys=True))
