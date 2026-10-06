"""Independently rebuild the exact operators and reject misleading native data."""
import argparse
import copy
from fractions import Fraction as Q
import json
import math
import reference as r


def near(actual, exact, tolerance=1e-11):
    r.require(math.isfinite(actual) and abs(actual - float(exact)) <= tolerance, 'native/exact arithmetic comparison')


def vector(actual, exact, tolerance=1e-11):
    r.require(len(actual) == len(exact), 'complete vector length')
    for x, y in zip(actual, exact):
        near(x, y, tolerance)


def verify(data):
    r.require(data['advancing_state_published'] is False, 'no advancing state publication')
    near(data['dt'], r.DT, 0)
    r.require((data['frame_bytes'], data['fixed_assembly_bytes'], data['fixed_solver_bytes']) == (23584, 26304, 24016), 'bounded native diagnostic memory')
    a, solved = r.assemble(), r.force_solve()
    r.require(data['pressure_columns'] == a['pressure_columns'], 'actual pressure image basis')
    vector(data['reduced_velocity'], solved['z'])
    vector(data['pressure_coefficients'], solved['p'])
    for row, exact in zip(data['cap_velocity'], solved['cap_velocity']):
        vector(row, exact)
    r.require(len(data['cap_velocity']) == 3, 'complete material cap velocity')
    z = [Q(x) for x in data['reduced_velocity']]
    p = [Q(x) for x in data['pressure_coefficients']]
    residual = [m + r.DT * (k + force) - old for m, k, force, old in zip(r.base.mv(a['M'], z), r.base.mv(a['K'], z), r.base.mv(r.transpose(a['B']), p), r.base.mv(a['M'], solved['old']))]
    near(data['momentum_residual'], max(map(abs, residual)))
    r.require(max(map(abs, residual)) < Q(1, 10**11), 'independent full xy momentum residual')
    r.require(max(map(abs, r.base.mv(a['D'], z))) < Q(1, 10**11), 'independent force divergence')
    for key in ['energy_old', 'energy_new', 'pressure_work']:
        near(data[key], solved[key])
    near(data['strain_power'], solved['strain'])
    near(data['increment_loss'], solved['increment'])
    near(data['work_error'], 0)
    r.require(len(data['samples']) == 5, 'all rejected candidate samples')
    for row, time in zip(data['samples'], (Q(0), r.DT / 4, r.DT / 2, 3 * r.DT / 4, r.DT)):
        exact = r.path_sample(time)
        s = exact['assembly']
        near(row['path_parameter'], time)
        r.require(row['triangles'] == [list(tri) for tri in s['micro']], 'actual microtriangle topology')
        r.require(len(row['nodes']) == len(s['points']), 'complete rebuilt PS geometry')
        for raw, (node, point) in enumerate(zip(row['nodes'], s['points'])):
            vector(node, [point[0].v, point[1].v, s['ids'][raw]])
        vector(row['mass'], s['mass'])
        r.require(len(row['velocity']) == 16, 'complete xy and third component velocity')
        raw_velocity = [x for raw, node in enumerate(s['ids']) for x in row['velocity'][node]]
        vector(raw_velocity, exact['raw_velocity'])
        vector(row['divergence'], exact['divergence'])
        vector(row['pressure_values'], r.base.mv(a['Q'], solved['p']) if time == 0 else [Q(0)] * 24)
        vector(row['mass_rates'], s['rate'])
        vector(row['continuity_defect'], exact['continuity_defect'])
        r.require(len(row['flux']) == len(exact['flux']), 'complete oriented physical face inventory')
        r.require(len({(i, j) for i, j, f in row['flux']}) == len(row['flux']), 'unique physical face pairs')
        for i, j, value in row['flux']:
            near(value, exact['flux'][(i, j)])
        near(row['divergence_max'], max(map(abs, exact['divergence'])))
        near(row['continuity_defect_max'], max(map(abs, exact['continuity_defect'])))
        near(row['total_mass'], sum(s['mass']))
        near(row['geometric_identity_error'], 0)
    return dict(momentum_residual=data['momentum_residual'], force_work_error=data['work_error'],
                final_divergence=data['samples'][-1]['divergence_max'],
                final_local_continuity_defect=data['samples'][-1]['continuity_defect_max'],
                exact_path_obstruction_preserved=True, accepted_advancing_steps=0)


def negative_tests(data):
    for kind in ['zero_pressure', 'omit_vertical_momentum', 'zero_path_divergence', 'zero_local_continuity',
                 'force_pressure_reused', 'nonmaterial_geometry', 'flux_circulation', 'false_publication']:
        bad = copy.deepcopy(data)
        if kind == 'zero_pressure':
            bad['pressure_coefficients'] = [0.] * 16
        elif kind == 'omit_vertical_momentum':
            bad['reduced_velocity'][12:] = [0.] * 10
        elif kind == 'zero_path_divergence':
            bad['samples'][-1]['divergence'] = [0.] * 24
            bad['samples'][-1]['divergence_max'] = 0.
        elif kind == 'zero_local_continuity':
            bad['samples'][-1]['continuity_defect'] = [0.] * 16
            bad['samples'][-1]['continuity_defect_max'] = 0.
        elif kind == 'force_pressure_reused':
            bad['samples'][-1]['pressure_values'] = bad['samples'][0]['pressure_values'][:]
        elif kind == 'nonmaterial_geometry':
            bad['samples'][-1]['nodes'][3][1] += 0.001
        elif kind == 'flux_circulation':
            # A real triangle cycle leaves every face marginal unchanged.
            faces = bad['samples'][-1]['flux']
            lookup = {(int(i), int(j)): k for k, (i, j, f) in enumerate(faces)}
            tri = bad['samples'][-1]['triangles'][0]
            ids = [int(bad['samples'][-1]['nodes'][raw][2]) for raw in tri]
            before = r.base.flux_divergence({(int(i), int(j)): Q(f) for i, j, f in faces}, 16)
            # Exact dyadic increments on rounded native fluxes, tolerantly
            # checked here because addition rounds before the validator sees it.
            for i, j in zip(ids, ids[1:] + ids[:1]):
                faces[lookup[min(i, j), max(i, j)]][2] += 1 / 1024 if i < j else -1 / 1024
            after = r.base.flux_divergence({(int(i), int(j)): Q(f) for i, j, f in faces}, 16)
            r.require(max(abs(x-y) for x,y in zip(before,after)) < Q(1,10**15), 'circulation retains zero balance')
        else:
            bad['advancing_state_published'] = True
        try:
            verify(bad)
        except (ValueError, KeyError):
            print('REJECTED', kind)
        else:
            raise ValueError('native corruption accepted: ' + kind)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('payload')
    parser.add_argument('--negative-self-test', action='store_true')
    args = parser.parse_args()
    data = json.load(open(args.payload))
    result = verify(data)
    if args.negative_self_test:
        negative_tests(data)
    print('PASS', json.dumps(result, sort_keys=True))
