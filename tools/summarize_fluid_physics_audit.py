"""Evaluate the predeclared 2026-10-19 audit criteria; preserve failures.

Usage: python3 tools/summarize_fluid_physics_audit.py EXTERNAL_PROBE_DIRECTORY
This is an audit report, not a new production acceptance gate. A successfully
completed report may contain failed physical screens. Missing cases are errors.
"""
import csv
import json
import math
from pathlib import Path
import sys


def summarize(directory):
    directory = Path(directory)
    with (directory / 'measurements.csv').open() as stream:
        rows = list(csv.DictReader(stream))
    data = {}
    for row in rows:
        key = row['case'], int(row['n']), row['variant'], row['metric']
        if key in data:
            raise ValueError('duplicate measurement: ' + str(key))
        value = float(row['value'])
        if not math.isfinite(value):
            raise ValueError('nonfinite measurement: ' + str(key))
        data[key] = value
    checks = []

    def check(name, category, value, bound, passed):
        checks.append(dict(name=name, category=category, value=value,
                           criterion=bound, passed=bool(passed)))

    def get(case, n, variant, metric):
        return data[case, n, variant, metric]

    methods = ['jacobi-pcg-v1', 'sgs-pcg-v1']
    for method in methods:
        for n in [8, 16, 32]:
            for metric, limit in [('pressure_max_error_pa', 0.01),
                                  ('speed_max_m_s', 1e-6),
                                  ('divergence_max_s_inv', 1e-5)]:
                x = get('hydro', n, method, metric)
                check(f'hydro/{method}/{n}/{metric}', 'numerical', x,
                      f'<{limit}', x < limit)
        errors = [get('pressure_mms', n, method, 'rms_error_pa')
                  for n in [8, 16, 32, 64]]
        for n, a, b in zip([16, 32, 64], errors, errors[1:]):
            check(f'pressure_mms/{method}/{n}/ratio', 'numerical', b/a,
                  '0.22..0.28', 0.22 <= b/a <= 0.28)
    errors = [get('channel', n, 'steady', 'relative_rms') for n in [8, 16, 32]]
    check('channel/8/relative_rms', 'accuracy_target', errors[0], '<0.02', errors[0] < 0.02)
    for n, a, b in zip([16, 32], errors, errors[1:]):
        check(f'channel/{n}/ratio', 'numerical', b/a, '<0.4', b/a < 0.4)
    for case, metric, ns, limit, variant in [
        ('channel_time', 'rms_semidiscrete', [8, 16, 32], 0.65, 'sine'),
        ('viscous', 'rms_continuum', [8, 16, 32], 0.5, 'vortex')]:
        errors = [get(case, n, variant, metric) for n in ns]
        for n, a, b in zip(ns[1:], errors, errors[1:]):
            check(f'{case}/{n}/ratio', 'numerical', b/a, f'<{limit}', b/a < limit)
        for n in ns:
            x = get(case, n, variant, 'max_energy_increase_j')
            check(f'{case}/{n}/energy', 'numerical', x, '<=0', x <= 0)
    for case in ['rest', 'submerged_box', 'surface_box', 'barrier', 'release']:
        for n in [8, 12, 16]:
            for metric, limit in [('max_relative_mass_drift', 1e-12),
                                  ('penetrating_centers_all_steps', 0),
                                  ('barrier_crossings_all_steps', 0)]:
                x = get(case, n, 'particle', metric)
                check(f'{case}/{n}/{metric}', 'geometric_mass', x,
                      f'<={limit}', x <= limit)
            if case == 'rest':
                for metric, limit in [('max_speed_m_s', 0.01*math.sqrt(9.80665)),
                                      ('density_relative_rms', 0.01),
                                      ('hydrostatic_pressure_gauge_relative_rms', 0.05)]:
                    x = get(case, n, 'particle', metric)
                    check(f'{case}/{n}/{metric}', 'physical_screen', x,
                          f'<{limit}', x < limit)
    for n in [0, 1, 2]:
        x = get('sweep', n, 'rotated', 'penetrations')
        check(f'sweep/{n}/penetrations', 'geometric_mass', x, '==0', x == 0)
        x = get('sweep', n, 'rotated', 'max_normal_speed_m_s')
        check(f'sweep/{n}/normal_speed', 'geometric_mass', x, '<1e-10', x < 1e-10)
    x = get('invalid', 0, 'open_solid', 'refused')
    check('invalid/open_solid/refused', 'geometric_mass', x, '==1', x == 1)
    # An independent infinite-lattice smooth-shear moment, not a native flow run.
    h = 2.0
    offsets = [(i, j, k) for i in range(-2, 3) for j in range(-2, 3)
               for k in range(-2, 3) if i*i+j*j+k*k < h*h]
    rho_ratio = sum(315/(64*math.pi*h**9)*(h*h-i*i-j*j-k*k)**3
                    for i, j, k in offsets)
    viscosity_ratio = sum(45/(math.pi*h**6)*(h-math.sqrt(i*i+j*j+k*k))*j*j/2
                          for i, j, k in offsets if i*i+j*j+k*k > 0)/rho_ratio**2
    lattice = dict(support_over_spacing=h, density_over_reference=rho_ratio,
                   smooth_shear_nu_over_reference=viscosity_ratio,
                   scope='Uniform infinite cubic particle lattice, no wall, no pressure force, '
                         'leading smooth-shear moment of implemented viscosity; no native transient claim.')
    contacts = []
    for case in ['submerged_box', 'surface_box', 'barrier', 'release']:
        for n in [8, 12, 16]:
            with (directory / f'{case}-{n}-final.csv').open() as stream:
                particles = list(csv.DictReader(stream))
            count = 0
            final_inside = 0
            for row in particles:
                point = [float(row[k]) for k in ['x', 'y', 'z']]
                if case == 'barrier':
                    near = (.05 <= point[0] <= .95 and .05 <= point[2] <= .95
                            and abs(point[1]-.4-(point[0]-.05)*.2/.9) < 1e-7)
                else:
                    center = [.675, .175, .5] if case == 'release' else [.5]*3
                    half = ([.075, .175, .15] if case == 'release' else
                            [.15, .25, .15] if case == 'surface_box' else [.15]*3)
                    angle = 0 if case == 'release' else .37
                    delta = [point[i]-center[i] for i in range(3)]
                    x = math.cos(angle)*delta[0]+math.sin(angle)*delta[1]
                    y = -math.sin(angle)*delta[0]+math.cos(angle)*delta[1]
                    delta = [x, y, delta[2]]
                    final_inside += int(all(abs(delta[i]) < half[i]-1e-9 for i in range(3)))
                    near = (all(abs(delta[i]) <= half[i]+1e-7 for i in range(3))
                            and min(abs(abs(delta[i])-half[i]) for i in range(3)) < 1e-7)
                count += int(near)
            contacts.append(dict(case=case, n=n, final_contact_band_centers=count, final_analytic_inside_centers=final_inside, band_m=1e-7))
    return dict(measurements=len(data), checks=checks, geometry_observations=contacts,
                failed=[c['name'] for c in checks if not c['passed']], lattice=lattice,
                scope='CPU numerical verification and physical suitability screens; no experimental validation.')


if __name__ == '__main__':
    print(json.dumps(summarize(sys.argv[1]), indent=2, allow_nan=False))
