"""Exact two-column pressure solve and falsification of its straight cap path.

This is bounded research; no advancing simulation or new native owner. All
Fraction requirements also execute under python -O. Historical reference bytes
are imported unchanged; the new mesh has two columns and arbitrary cap motion.
"""
from functools import lru_cache
from fractions import Fraction as Q
from pathlib import Path
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('pressure_ps_base', ROOT / 'evidence/fitted-height-periodic-repair/reference.py')
base = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = base
spec.loader.exec_module(base)
require = base.require
INITIAL_CAP = [(Q(0), Q(1)), (Q(1, 2), Q(5, 4)), (Q(1), Q(1))]
DT = Q(1, 20)


def mesh(time=Q(0), cap_velocity=((Q(0), Q(0)),) * 3):
    require(0 <= time <= DT and len(cap_velocity) == 3, 'bounded research path')
    require(cap_velocity[0] == cap_velocity[2], 'periodic cap velocity')
    points = [(base.Dual(Q(i, 2)), base.Dual(Q(0))) for i in range(3)]
    points += [tuple(base.Dual(x + time * u, u) for x, u in zip(p, v)) for p, v in zip(INITIAL_CAP, cap_velocity)]
    macro = [(0, 1, 4), (0, 4, 3), (1, 2, 5), (1, 5, 4)]
    centers = []
    for tri in macro:
        centers.append(len(points))
        points.append(base.scale(base.add(base.add(points[tri[0]], points[tri[1]]), points[tri[2]]), Q(1, 3)))
    adjacent = {}
    for t, tri in enumerate(macro):
        for k in range(3):
            adjacent.setdefault(tuple(sorted((tri[k], tri[(k + 1) % 3]))), []).append(t)
    left, right = (0, 3), (2, 5)
    adjacent[left].append(adjacent[right][0])
    adjacent[right].append(adjacent[left][0])
    edge_nodes, constraints = {}, {}
    for edge, owners in sorted(adjacent.items()):
        a, b = (points[i] for i in edge)
        if len(owners) == 2:
            p, q = (points[centers[i]] for i in owners)
            if edge == left:
                q = (q[0] - 1, q[1])
            elif edge == right:
                q = (q[0] + 1, q[1])
            ab, pq = base.sub(b, a), base.sub(q, p)
            fraction = base.cross(base.sub(p, a), pq) / base.cross(ab, pq)
            require(0 < fraction.v < 1, 'physical PS intersection inside edge')
            pos = base.add(a, base.scale(ab, fraction))
        else:
            pos = base.scale(base.add(a, b), Q(1, 2))
        edge_nodes[edge] = len(points)
        points.append(pos)
        if len(owners) == 1 and (all(i < 3 for i in edge) or all(3 <= i < 6 for i in edge)):
            constraints[edge_nodes[edge]] = edge
    micro = []
    for t, tri in enumerate(macro):
        for k in range(3):
            a, b = tri[k], tri[(k + 1) % 3]
            e, c = edge_nodes[tuple(sorted((a, b)))], centers[t]
            micro.extend([(a, e, c), (e, b, c)])
    representative = list(range(len(points)))
    representative[2], representative[5] = 0, 3
    representative[edge_nodes[right]] = edge_nodes[left]
    compact = {i: j for j, i in enumerate(sorted(set(representative)))}
    ids = [compact[i] for i in representative]
    constraints = {ids[i]: tuple(ids[j] for j in ends) for i, ends in constraints.items()}
    bottom = {ids[i] for i in range(3)} | {i for i, ends in constraints.items() if all(j in {ids[k] for k in range(3)} for j in ends)}
    return points, micro, ids, constraints, bottom


def transpose(a):
    return [list(row) for row in zip(*a)]


def product(a, b):
    bt = transpose(b)
    return [[base.dot(row, col) for col in bt] for row in a]


@lru_cache(maxsize=32)
def assemble(time=Q(0), cap_velocity=((Q(0), Q(0)),) * 3):
    points, micro, ids, constraints, bottom = mesh(time, cap_velocity)
    mass, rate, k, div, areas = base.operators(points, micro, mu=Q(1, 20))
    n = max(ids) + 1
    rx = base.scalar_embedding(n, constraints)
    ry = base.scalar_embedding(n, constraints, bottom)
    cols = len(rx[0]) + len(ry[0])
    r = base.zeros(3 * len(points), cols)
    for raw, node in enumerate(ids):
        r[3 * raw] = rx[node] + [Q(0)] * len(ry[0])
        r[3 * raw + 1] = [Q(0)] * len(rx[0]) + ry[node]
    d = product(div, r)
    columns = base.independent_columns(d)
    require(len(columns) == 16 and cols == 22 and len(points) == 19 and len(micro) == 24, 'two-column dimensions and exact pressure rank')
    pressure = [[row[j] for j in columns] for row in d]
    b = product(transpose(pressure), [[-area * x for x in row] for area, row in zip(areas, d)])
    mr = product(transpose(r), [[mass[i // 3] * x for x in row] for i, row in enumerate(r)])
    kr = product(transpose(r), product(k, r))
    return dict(points=points, micro=micro, ids=ids, constraints=constraints, bottom=bottom,
                R=r, D=d, Q=pressure, B=b, M=mr, K=kr, mass=base.merge(mass, ids),
                rate=base.merge(rate, ids), areas=areas, pressure_columns=columns)


@lru_cache(maxsize=1)
def force_solve():
    a = assemble()
    old = [Q(0)] * 22
    for raw, p in enumerate(a['points']):
        row = a['R'][3 * raw]
        for j, x in enumerate(row):
            if x == 1:
                old[j] = p[1].v
    require(base.mv(a['D'], old) == [Q(0)] * 24, 'initial shear is strongly divergence free')
    bt = transpose(a['B'])
    system = [[a['M'][i][j] + DT * a['K'][i][j] for j in range(22)] + [DT * x for x in bt[i]] for i in range(22)]
    system += [row + [Q(0)] * 16 for row in a['B']]
    rhs = base.mv(a['M'], old) + [Q(0)] * 16
    solution = base.solve(system, rhs)
    z, p = solution[:22], solution[22:]
    require(base.mv(system, solution) == rhs, 'exact solved momentum and pressure residual')
    require(base.mv(a['D'], z) == [Q(0)] * 24, 'exact full strong divergence after pressure solve')
    require(any(p), 'pressure actually solved and nonzero')
    raw = base.mv(a['R'], z)
    cap = tuple((raw[3 * i], raw[3 * i + 1]) for i in range(3, 6))
    delta = [x - y for x, y in zip(z, old)]
    energy_old = base.dot(old, base.mv(a['M'], old)) / 2
    energy_new = base.dot(z, base.mv(a['M'], z)) / 2
    increment = base.dot(delta, base.mv(a['M'], delta)) / 2
    strain = base.dot(z, base.mv(a['K'], z))
    pressure_work = base.dot(z, base.mv(bt, p))
    require(energy_new - energy_old + increment + DT * strain == 0 and pressure_work == 0, 'exact full xy strain and pressure work')
    return dict(old=old, z=z, p=p, cap_velocity=cap, energy_old=energy_old, energy_new=energy_new,
                increment=increment, strain=strain, pressure_work=pressure_work)


@lru_cache(maxsize=16)
def path_sample(time):
    solved = force_solve()
    a = assemble(Q(time), solved['cap_velocity'])
    raw = base.mv(a['R'], solved['z'])
    divergence = base.mv(a['D'], solved['z'])
    flux = {}
    div_mass = [Q(0)] * 16
    div_rate = []
    for t, tri in enumerate(a['micro']):
        points = [a['points'][i] for i in tri]
        twice = base.cross(base.sub(points[1], points[0]), base.sub(points[2], points[0]))
        dd = Q(0)
        for i in range(3):
            j, k = (i + 1) % 3, (i + 2) % 3
            gx = (points[j][1] - points[k][1]) / twice
            gy = (points[k][0] - points[j][0]) / twice
            dd += gx.d * raw[3 * tri[i]] + gy.d * raw[3 * tri[i] + 1]
            div_mass[a['ids'][tri[i]]] += a['areas'][t] * divergence[t]  # rho=3 / 3
        div_rate.append(dd)
        center = base.scale(base.add(base.add(points[0], points[1]), points[2]), Q(1, 3))
        for i in range(3):
            j, k = (i + 1) % 3, (i + 2) % 3
            segment = base.sub(center, base.scale(base.add(points[i], points[j]), Q(1, 2)))
            relative = [sum(w * (raw[3 * tri[l] + d] - points[l][d].d) for l, w in zip((i, j, k), (Q(5, 12), Q(5, 12), Q(1, 6)))) for d in range(2)]
            f = 3 * (relative[0] * segment[1].v - relative[1] * segment[0].v)
            u, v = a['ids'][tri[i]], a['ids'][tri[j]]
            if u != v:
                pair = min(u, v), max(u, v)
                flux[pair] = flux.get(pair, Q(0)) + (f if u < v else -f)
    balance = [x + y for x, y in zip(a['rate'], base.flux_divergence(flux, 16))]
    require(balance == div_mass, 'exact physical Reynolds identity INCLUDING divergence defect')
    require(sum(a['mass']) == Q(27, 8) and sum(balance) == 0, 'global volume hides local path defect')
    for i in range(3, 6):
        require(tuple(raw[3 * i + d] for d in range(2)) == tuple(a['points'][i][d].d for d in range(2)), 'actual material cap velocity')
    return dict(assembly=a, raw_velocity=raw, divergence=divergence, divergence_rate=div_rate,
                flux=flux, continuity_defect=balance, divergence_mass=div_mass)


def summary():
    solved = force_solve()
    initial, end = path_sample(Q(0)), path_sample(DT)
    rows = []
    for t in (Q(0), DT / 4, DT / 2, 3 * DT / 4, DT):
        p = path_sample(t)
        rows.append(dict(time=str(t), actual_mass=str(sum(p['assembly']['mass'])),
                         divergence_max=float(max(map(abs, p['divergence']))),
                         continuity_defect_max=float(max(map(abs, p['continuity_defect']))),
                         min_microarea=float(min(p['assembly']['areas'])),
                         min_mass=float(min(p['assembly']['mass']))))
    nodal_defect_derivative = [Q(0)] * 16
    for t, tri in enumerate(initial['assembly']['micro']):
        for raw in tri:
            nodal_defect_derivative[initial['assembly']['ids'][raw]] += initial['assembly']['areas'][t] * initial['divergence_rate'][t]
    require(any(nodal_defect_derivative), 'exact integrated local GCL leading defect')
    witness = max(range(24), key=lambda i: abs(initial['divergence_rate'][i]))
    require(initial['divergence_rate'][witness] != 0 and any(end['divergence']), 'precise nonzero material-path obstruction')
    return dict(scope='solved nonuniform xy pressure/strain force test; rejected straight material path, no advancing API',
                columns=2, raw_nodes=19, periodic_nodes=16, microtriangles=24, xy_unknowns=22, pressure_modes=16,
                dt=str(DT), initial_velocity='(y,0,0)', external_load=0, solved_pressure_nonzero=True,
                reduced_velocity=[str(x) for x in solved['z']], pressure_coefficients=[str(x) for x in solved['p']],
                reconstructed_pressure_max=float(max(map(abs, base.mv(assemble()['Q'], solved['p'])))),
                momentum_residual_exact='0', initial_and_force_divergence_exact='0', pressure_work_exact='0',
                energy_old=str(solved['energy_old']), energy_new=str(solved['energy_new']),
                increment_loss=str(solved['increment']), strain_power=str(solved['strain']), work_residual_exact='0',
                cap_velocity=[[str(x) for x in v] for v in solved['cap_velocity']],
                obstruction_microtriangle=witness, obstruction_nodes=initial['assembly']['micro'][witness],
                exact_divergence_derivative_at_zero=str(initial['divergence_rate'][witness]),
                derivative_at_zero_float=float(initial['divergence_rate'][witness]),
                exact_endpoint_divergence_on_witness=str(end['divergence'][witness]),
                exact_endpoint_continuity_defect=[str(x) for x in end['continuity_defect']],
                exact_nodal_continuity_derivative_at_zero=[str(x) for x in nodal_defect_derivative], samples=rows,
                production_step_enabled=False, new_Lean_claims=0)


if __name__ == '__main__':
    print(json.dumps(summary(), indent=2))
