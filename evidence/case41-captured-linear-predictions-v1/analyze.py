"""Predictions from captured fields only. Never import a driver or call native code."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib
import importlib.util
import json
import math
import struct
import sys
import mpmath as mp

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
OLD = ROOT / 'evidence/case41-exploratory-seven-v1'


def require(ok, msg):
    if not ok:
        raise ValueError(msg)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def run(dps):
    mp.mp.dps = dps
    policy = json.loads((P / 'protocol.json').read_text())
    for path, digest in policy['input_sha256'].items():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, 'input hash: ' + path)
    inventory = json.loads((OLD / 'result-inventory.json').read_text())
    for path, digest in inventory['sha256'].items():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, 'old result hash: ' + path)
    checked = json.loads((OLD / 'analysis-v2-normal.json').read_text())
    prior_model = json.loads((OLD / 'model-120-normal.json').read_text())
    require(checked['status'] == 'PASS_COMPLETE_EXPLORATORY_SEVEN_EQUATION_CAPTURE', 'complete original capture')
    a = load('pred_exact_algebra', ROOT / 'evidence/forcing-case41-scale-diagnosis-v1/analyze.py')
    oracle = a.a
    prefix = 'test research_public_call::case41_paired_candidate_roster ... '
    records = [json.loads(line.removeprefix(prefix)) for line in (OLD / 'native.log').read_text().splitlines()
               if line.startswith('{') or line.startswith(prefix)]
    equations = [row for row in records if row['event'] == 'fd_equation']
    columns = [row for row in records if row['event'] == 'fd_column']
    require([e['column'] for e in equations] == [-1, 0, 1, 2, 3, 4, 5], 'exact captured roster')
    e = equations[0]
    A = [[Q(columns[j]['native_column'][i]) for j in range(6)] for i in range(22)]
    B = [[Q(e['end_b'][j][i]) for j in range(16)] for i in range(22)]
    J = [x + y for x, y in zip(A, B)]
    require(J == [[Q(x) for x in row] for row in checked['native_linear_model']], 'raw model binding')
    C = [Q(1)] * 6 + [Q(27, 8)] * 16
    F = Q(27, 8)
    Jhat = [[x * C[j] / F for j, x in enumerate(row)] for row in J]
    inv = a.inverse(J)
    invhat = [[x * F / C[i] for x in row] for i, row in enumerate(inv)]
    require(a.mul(Jhat, invhat) == [[Q(i == j) for j in range(22)] for i in range(22)], 'scaled exact inverse')
    norm = oracle.normq
    frobsq = lambda mat: sum(x*x for row in mat for x in row)
    root = lambda q: float(mp.sqrt(mp.mpf(q.numerator) / q.denominator))
    vec = lambda values: dict(norm=norm(values), components=[float(x) for x in values])
    predict = lambda residual: a.mv(inv, [-x for x in residual])
    r = list(map(Q, e['rate']))
    direct = list(map(Q, e['direct_rate']))
    c = dict(json.loads((OLD / 'fixture.json').read_text())['original_case'], acceleration=[.0625, -.125, .03125])
    rex, _ = oracle.exact(c, e)
    rdex, _ = oracle.exact(c, e, direct=True)
    evaluation_gap = [x-y for x, y in zip(r, rex)]
    s = predict(r)
    require([x+y for x, y in zip(r, a.mv(J, s))] == [Q(0)]*22, 'exact model prediction zero')
    shat = [x / scale for x, scale in zip(s, C)]
    # Round only the mathematical projection, then forecast the coordinate lattice.
    # No vector is supplied to a native equation or controller.
    sr = [Q(float(x)) for x in s]
    lattice = [Q(float(Q(x) + dx)) - Q(x) for x, dx in zip(e['unknown'], sr)]
    round_residual = [x+y for x, y in zip(r, a.mv(J, sr))]
    lattice_residual = [x+y for x, y in zip(r, a.mv(J, lattice))]
    denominator = [sum(abs(x)*abs(y) for x, y in zip(row, sr)) + abs(rhs) for row, rhs in zip(J, r)]
    berr = max(abs(x)/y if y else Q(0) for x, y in zip(round_residual, denominator))
    require(all(y or not x for x, y in zip(round_residual, denominator)), 'explicit zero BERR policy')
    bound_component = [sum(abs(x)*abs(y) for x, y in zip(row, [x-y for x, y in zip(lattice, s)])) for row in J]
    require(all(abs(x) <= y for x, y in zip(lattice_residual, bound_component)), 'lattice residual bound')
    # Isolate actual captured constraint defects with known endpoint values fixed.
    # This changes inertia algebra only: all other stored fields stay fixed.
    z = list(map(Q, e['end_z']))
    chart = list(oracle.chart(tuple(map(tuple, e['end_d'])), tuple(z[j] for j in oracle.KNOWN)))
    dz = [x-y for x, y in zip(z, chart)]
    D = [[Q(x) for x in row] for row in e['end_d']]
    M = [[sum(Q(e['end_mass'][n])*Q(e['r'][n][d][i])*Q(e['r'][n][d][j])
              for n in range(16) for d in range(2)) for j in range(22)] for i in range(22)]
    constraint_force = [x / Q(c['h']) for x in a.mv(M, dz)]
    adjusted = [x-y for x, y in zip(rex, constraint_force)]
    adjusted_replay, _ = oracle.exact(c, e, z=chart)
    require(adjusted == adjusted_replay, 'exact frozen-inertia constraint decomposition')
    constraints = a.mv(D, z)
    chart_constraints = a.mv(D, chart)
    require(all(chart_constraints[i] == 0 for i in oracle.ROWS), 'all selected rows exact')
    exact_A = [[Q(x) for x in row] for row in checked['exact_stored_field_FD_differences']]
    pure_rate_A = [[(Q(equations[j+1]['rate'][i])-r[i])/Q(equations[j+1]['delta']) for j in range(6)] for i in range(22)]
    scaled_gap = [[(x-y) / F for x, y in zip(row, exactrow)] + [Q(0)]*16 for row, exactrow in zip(A, exact_A)]
    scaled_round_gap = [[(x-y) / F for x, y in zip(row, exactrow)] + [Q(0)]*16 for row, exactrow in zip(A, pure_rate_A)]
    invfrobsq = frobsq(invhat)
    theta_sq = invfrobsq * frobsq(scaled_gap)
    require(theta_sq < 1, 'Neumann bound for TWO CAPTURED matrices only')
    theta = root(theta_sq)
    Jfields = [x+y for x, y in zip(exact_A, B)]
    sfields = a.mv(a.inverse(Jfields), [-x for x in r])
    # Predicted residual under the alternative captured-field matrix, not F(x+s).
    alt_residual = [x+y for x, y in zip(r, a.mv(Jfields, s))]
    residual_vectors = dict(native=r, exact_stored=rex, native_minus_exact_stored=evaluation_gap,
                            native_direct=direct, exact_stored_direct=rdex,
                            stable_minus_direct=[x-y for x, y in zip(rex, rdex)],
                            selected_constraint_inertia=constraint_force,
                            exact_stored_minus_selected_constraint_inertia=adjusted)
    predicted_vectors = {k: predict(v) for k, v in residual_vectors.items()}
    require(predicted_vectors['native'] == [x+y for x, y in zip(predicted_vectors['exact_stored'], predicted_vectors['native_minus_exact_stored'])], 'linear prediction evaluation decomposition')
    require(predicted_vectors['exact_stored'] == [x+y for x, y in zip(predicted_vectors['selected_constraint_inertia'], predicted_vectors['exact_stored_minus_selected_constraint_inertia'])], 'linear prediction constraint decomposition')
    # W projections quantify residual contributions; reuses declared mass metric.
    W = [[x/F for x in row] for row in a.inverse(M)]
    BtW = a.mul(a.transpose(B), W)
    projection = a.mul(a.mul(B, a.inverse(a.mul(BtW, B))), BtW)
    diagnostics = {}
    for label, v in residual_vectors.items():
        pv = a.mv(projection, v)
        cv = [x-y for x, y in zip(v, pv)]
        square = lambda w: a.dot(w, a.mv(W, w))
        require(square(v) == square(pv)+square(cv), 'exact weighted decomposition: '+label)
        diagnostics[label] = dict(residual=vec(v), predicted_displacement=vec(predicted_vectors[label]),
                                 scaled_displacement_norm=norm([x/scale for x, scale in zip(predicted_vectors[label], C)]),
                                 dual_mass_pressure_norm=root(square(pv)), dual_mass_complement_norm=root(square(cv)))
    return dict(status='PASS_CAPTURED_LINEAR_PREDICTIONS_ONLY', precision_digits=dps,
                reused_diagnostics=dict(source=policy['prior_model_source'], result=policy['prior_result'],
                    exact_ranks=dict(A=6, B=16, J=22), raw_condition=prior_model['raw']['condition_2'],
                    scaled_condition=prior_model['uniform_physical_scaled']['condition_2'],
                    dual_mass_condition=prior_model['dual_mass_scaled']['condition_2'],
                    complementary_condition=prior_model['complementary_acceleration']['condition_2']),
                matrix=dict(rows=22, columns=22, A_nominal_denominators=[x['delta'] for x in equations[1:]],
                    A_actual_to_nominal=[float((Q(x['unknown'][j])-Q(e['unknown'][j]))/Q(x['delta'])) for j, x in enumerate(equations[1:])],
                    binary64_bits=[[struct.pack('>d', float(x)).hex() for x in row] for row in J],
                    coordinate_scale=[str(x) for x in C], residual_scale=str(F),
                    scaled_formula='Jhat=J*diag(C)/F; rhat=r/F; shat=diag(C)^-1*s',
                    exact_scaled_inverse_Frobenius_squared=str(invfrobsq), scaled_inverse_Frobenius_bound=root(invfrobsq)),
                residual_diagnostics=diagnostics,
                projection=dict(sign='J*s=-r', exact_displacement=[str(x) for x in s],
                    scaled_displacement_norm=norm(shat), acceleration_norm=norm(s[:6]), pressure_coordinate_norm=norm(s[6:]),
                    exact_model_residual_zero=True, rounded_displacement_components=[float(x) for x in sr],
                    rounded_displacement_model_residual=vec(round_residual), rounded_displacement_BERR=float(berr),
                    BERR_scope='binary64 rounding of exact mathematical projection; NOT native linear solver BERR',
                    coordinate_lattice_increments=[float(x) for x in lattice],
                    correction_to_coordinate_ulp=[float(abs(x)/Q(math.ulp(value))) for x, value in zip(s, e['unknown'])],
                    coordinate_lattice_model_residual=vec(lattice_residual),
                    coordinate_lattice_residual_component_bound=vec(bound_component),
                    coordinate_lattice_displacement_error_norm=norm([x-y for x, y in zip(lattice, s)]),
                    ideal_geometry_displacement=[float(Q(c['h'])**2*x/2) for x in s[:3]],
                    ideal_known_velocity_displacement=[float(Q(c['h'])*x) for x in s[:6]],
                    ideal_geometry_scope='frozen geometry formula only; excludes geometry rounding and moving-chart sensitivities'),
                constraint_comparison=dict(captured_native_maximum=e['maximum_constraints'],
                    exact_stored_full_Dz_max=float(max(map(abs, constraints))),
                    exact_stored_selected_Dz_max=float(max(abs(constraints[i]) for i in oracle.ROWS)),
                    exact_selected_chart_full_Dz_max=float(max(map(abs, chart_constraints))),
                    z_minus_exact_selected_chart=vec(dz), known_values_unchanged=True,
                    scope='Frozen D selected-row reconciliation; inertia-only contribution with all other fields fixed. Full unselected rows are not forced to zero; not a new constraint solve in the numerical method.'),
                rounding_limits=dict(native_FD_vs_exact_native_rates_scaled_Frobenius=root(frobsq(scaled_round_gap)),
                    native_FD_vs_exact_stored_fields_scaled_Frobenius=root(frobsq(scaled_gap)),
                    captured_matrix_Neumann_theta=theta,
                    alternative_captured_matrix_inverse_bound=root(invfrobsq)/(1-theta),
                    actual_alternative_captured_matrix_displacement_gap=norm([x/C[i]-s[i]/C[i] for i, x in enumerate(sfields)]),
                    conservative_alternative_displacement_gap_bound=theta/(1-theta)*norm(shat),
                    alternative_captured_matrix_residual_at_projection=vec(alt_residual),
                    native_vs_exact_stored_force_gap=norm(evaluation_gap),
                    Newton_gate_excess=norm(r)-1e-13,
                    scope='All bounds refer to captured binary data and exact algebra. No FD truncation, force assembly, real-map evaluation or nonlinear remainder bound.'),
                new_native_equations=0, applied_corrections=0, owner_advances=0,
                native_linear_calls=0, historical_memory_qualified=False,
                historical_subtotal=66368, memory_cap=67584,
                FD_truncation_bound=None, smooth_derivative_bound=None, true_residual_evaluation_bound=None,
                nonlinear_prediction_verified=False, physical_state_error_bound=None,
                rank_precision_scope='Exact rank of captured rational matrix; existing SVD displays at 80/120 digits, not interval certified singular values.',
                scope='External linear predictions only. No constructed native candidate, equation, owner, retry, accepted step or gate change.')


if __name__ == '__main__':
    print(json.dumps(run(int(sys.argv[1]) if len(sys.argv)>1 else 80), indent=2, sort_keys=True))
