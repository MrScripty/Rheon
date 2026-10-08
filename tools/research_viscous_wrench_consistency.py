"""Fixed unit-fixture forensic research. No native operator or expected values change.

Freshly run the frozen native bridge, compare its dyadic actions exactly, then
separate wall-gradient, cubature and lift ablations. Ablations are NOT coupled
operators or corrections. All output is external and newly created.
"""
import argparse
from fractions import Fraction as Q
import hashlib
from itertools import product
import json
from pathlib import Path
import struct
import subprocess
import check_viscous_boundary_wrench as qualified_checker

BASE = '7d3c8df2d9149e5cd643a2af1f29f2ccf94bfb7e'
ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def exact(bits):
    return Q.from_float(struct.unpack('>d', bytes.fromhex(bits))[0])


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def add(a, b):
    return [x+y for x, y in zip(a, b)]


def quadratic_derivative(d1, d2, trace, u1, u2):
    if not 0 < d1 < d2:
        raise ValueError('two distinct outward samples required')
    return (-(d1+d2)/(d1*d2)*trace
            + d2/(d1*(d2-d1))*u1 - d1/(d2*(d2-d1))*u2)


def derivative_consistency_controls():
    records = []
    for d1, d2 in [(Q(1, 4), Q(3, 4)), (Q(3, 10), Q(7, 10)), (Q(5, 8), Q(11, 8))]:
        for a, b, c in [(Q(2), Q(-3), Q(5)), (Q(0), Q(1), Q(0)), (Q(0), Q(0), Q(1))]:
            profile = lambda r: a+b*r+c*r*r
            recovered = quadratic_derivative(d1, d2, a, profile(d1), profile(d2))
            if recovered != b:
                raise ValueError('quadratic reproduction failed')
            records.append({'distances': [d1, d2], 'coefficients': [a, b, c], 'derivative': recovered})
        omega = Q(7, 3)
        derivative = quadratic_derivative(d1, d2, Q(2), 2+omega*d1, 2+omega*d2)
        if derivative-omega != 0:
            raise ValueError('both rigid shear derivatives must cancel')
    for distances in [(Q(0), Q(1)), (Q(1), Q(1)), (Q(2), Q(1))]:
        try:
            quadratic_derivative(*distances, Q(0), Q(1), Q(2))
        except ValueError:
            continue
        raise ValueError('unsupported derivative support accepted')
    return records


def jet(position, alpha=Q(225), beta=Q(1)):
    """Differentiate the unchanged polynomial directly in exact arithmetic."""
    vals = []
    for t in position:
        p, d = (t-1)*(t-2), 2*t-3
        vals.append((p*p, 2*p*d, 2*d*d+4*p))
    (X, X1, X2), (Y, Y1, Y2), (Z, Z1, _) = vals
    H = 1+beta*(position[0]-Q(3, 2))
    A, A1, A2 = H*X, beta*X+H*X1, 2*beta*X1+H*X2
    u = [alpha*A*Y1*Z, -alpha*A1*Y*Z, Q(0)]
    normal = 2*alpha*A1*Y1*Z
    xy = alpha*(A*Y2-A2*Y)*Z
    xz, yz = alpha*A*Y1*Z1, -alpha*A1*Y*Z1
    stress = [[normal, xy, xz], [xy, -normal, yz], [xz, yz, Q(0)]]
    return u, stress


def surface_cubature(nodes):
    total = [Q(0)]*6
    for normal in range(3):
        tangent = [d for d in range(3) if d != normal]
        for wall, sign in [(Q(1), -1), (Q(2), 1)]:
            for (a, wa), (b, wb) in product(nodes, repeat=2):
                point = [Q(0)]*3
                point[normal] = wall
                point[tangent[0]], point[tangent[1]] = a, b
                u, stress = jet(point)
                if u != [0, 0, 0] or stress[normal][normal] != 0:
                    raise ValueError('exact stationary no-slip/zero-normal-stress control failed')
                f = [wa*wb*sign*stress[i][normal] for i in range(3)]
                total = add(total, f+cross([x-Q(3, 2) for x in point], f))
    return total


def flat_observable(mode):
    """Use actual flat-row support; never use these outputs to change old rows."""
    total = [Q(0)]*6
    for normal in range(3):
        for wall, sign in [(Q(1), -1), (Q(2), 1)]:
            for component in [d for d in range(3) if d != normal]:
                layer = 3-normal-component
                for z in [Q(5, 4), Q(7, 4)]:
                    point = [Q(0)]*3
                    point[component], point[layer], point[normal] = Q(3, 2), z, wall
                    if mode == 'exact-wall-gradient':
                        traction = sign*jet(point)[1][component][normal]
                    else:
                        sample = point.copy()
                        sample[normal] += sign*Q(1, 4)
                        U1 = jet(sample)[0][component]
                        traction = 4*U1
                        if mode == 'quadratic-wall-jet':
                            sample[normal] = wall+sign*Q(3, 4)
                            U2 = jet(sample)[0][component]
                            traction = quadratic_derivative(Q(1, 4), Q(3, 4), Q(0), U1, U2)
                    f = [Q(0)]*3
                    f[component] = Q(1, 4)*traction
                    if mode == 'original-flat-lift':
                        point[normal] += sign*Q(1, 4)
                    total = add(total, f+cross([x-Q(3, 2) for x in point], f))
    return total


def native_exact(raw):
    field = next(f for f in raw['fields'] if f['name'] == 'polynomial-tilted-curl')
    u, mu = list(map(exact, field['values'])), exact(raw['meta']['mu'])
    groups = {k: [Q(0)]*6 for k in ['normal', 'flat', 'corner', 'interior']}
    force, outer, loss = [Q(0)]*len(u), [Q(0)]*6, Q(0)
    for row in raw['rows']:
        w = exact(row['weight'])
        if w < 0:
            raise ValueError('negative energy weight')
        s = sum((exact(t['coefficient'])*u[t['active']] for t in row['terms']), Q(0))
        loss += mu*w*s*s
        for term in row['terms']:
            force[term['active']] -= mu*w*s*exact(term['coefficient'])
        for k in range(6):
            groups[row['boundary']][k] -= mu*w*s*exact(row['solid'][k])
            outer[k] -= mu*w*s*exact(row['outer'][k])
        for k in range(6):
            rigid = Q(0)
            for term in row['terms']:
                face = raw['active'][term['active']]
                axis, r = face['axis'], [exact(x)-exact(c) for x, c in zip(face['position'], raw['reference'])]
                basis = [[1, 0, 0, 0, r[2], -r[1]], [0, 1, 0, -r[2], 0, r[0]], [0, 0, 1, r[1], -r[0], 0]][axis]
                rigid += exact(term['coefficient'])*basis[k]
            if rigid+exact(row['solid'][k])+exact(row['outer'][k]) != 0:
                raise ValueError('unit exact row rigid closure failed')
    total = [sum(g[k] for g in groups.values()) for k in range(6)]
    if total != list(map(exact, field['solid_wrench'])) or force != list(map(exact, field['force'])):
        raise ValueError('fresh native dyadic action differs from exact finite model')
    work = sum((v*f for v, f in zip(u, force)), Q(0))
    if work != -loss:
        raise ValueError('exact finite-model stationary energy/work failed')
    fluid = [Q(0)]*6
    for face, f in zip(raw['active'], force):
        vector = [Q(0)]*3
        vector[face['axis']] = f
        fluid = add(fluid, vector+cross([exact(x)-exact(c) for x, c in zip(face['position'], raw['reference'])], vector))
    if add(add(fluid, total), outer) != [0]*6:
        raise ValueError('exact fluid/solid/outer resultant closure failed')
    shift = [Q(2, 3), Q(-1, 4), Q(5, 7)]
    shifted = total[:3]+[a-b for a, b in zip(total[3:], cross(shift, total[:3]))]
    return {'groups': groups, 'total': total, 'fluid': fluid, 'outer': outer,
            'dissipation': loss, 'force_work': work,
            'native_minus_exact_dissipation': exact(field['dissipation'])-loss,
            'native_minus_exact_force_work': exact(field['force_work'])-work,
            'native_arithmetic_scope': 'accepted only by unchanged source-bound contribution tolerance; no IEEE enclosure',
            'shifted_reference_wrench_exact': shifted,
            'reference_shift_scope': 'exact finite map transformation; no shifted native bounded specimen claimed'}


def serialize(value):
    if isinstance(value, Q):
        return str(value)
    if isinstance(value, dict):
        return {k: serialize(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return list(map(serialize, value))
    return value


def run(executable, output):
    if output.exists():
        raise ValueError('new external output required')
    if subprocess.run(['git', '-C', str(output.parent), 'rev-parse', '--is-inside-work-tree'], capture_output=True).returncode == 0:
        raise ValueError('output must remain outside Git')
    before = sha(executable)
    accepted_binaries = {'9f1f77115fac7c8907ee592cd534e030fc07d907b66ab0ce0d2064e661b6d504',
                         'cfd7d8590914f63253f3450fe1ce4240b5b3f4e9ab8e60b6fd2df58a6b39155e'}
    if before not in accepted_binaries:
        raise ValueError('only the preserved source-qualified frozen bridge is admitted')
    subprocess.run(['git', 'diff', '--exit-code', BASE, '--', 'src', 'proofs', 'examples',
                    'tests', 'experiments', 'Cargo.toml', 'Cargo.lock', '.gitignore'],
                   cwd=ROOT, check=True, capture_output=True)
    checker = ROOT/'tools/check_viscous_boundary_wrench.py'
    if sha(checker) != 'd25b2316e2ffe27ee9943a475a3878770f58a054d3186c05ff7bd09d3b30310c':
        raise ValueError('unchanged qualified arithmetic checker required')
    output.mkdir(parents=True)
    cases = {}
    for name in ['unit-center', 'polynomial-bounded']:
        path = output/(name+'.json')
        subprocess.run([str(executable), name, str(path)], check=True, capture_output=True, timeout=30)
        raw = json.loads(path.read_text())
        qualified_checker.verify(path, name)
        cases[name] = {'raw_sha256': sha(path), 'finite_model': native_exact(raw)}
    if before != sha(executable):
        raise ValueError('frozen executable changed')
    coarse = json.loads((output/'unit-center.json').read_text())
    control = next(f for f in coarse['fields'] if f['name'] == 'polynomial-tilted-curl')
    if any(exact(x) for x in control['values']):
        raise ValueError('sampling-nullspace control no longer zero')
    boole = [(1+Q(i, 4), Q(w, 90)) for i, w in enumerate([7, 32, 12, 32, 7])]
    continuum = surface_cubature(boole)
    if continuum != [0, Q(-1, 2), 0, 0, 0, -1]:
        raise ValueError('unchanged analytic expected load failed')
    ablations = {name: flat_observable(name) for name in ['exact-wall-gradient', 'secant-wall-moment', 'original-flat-lift', 'quadratic-wall-jet']}
    flat = cases['polynomial-bounded']['finite_model']['groups']['flat']
    if ablations['original-flat-lift'] != flat:
        raise ValueError('separately derived flat geometry action differs')
    result = {'kind': 'exact fixed-fixture root-cause research; no production correction',
              'preserved_native_head': BASE, 'research_source_sha256': sha(Path(__file__)),
              'unchanged_native_checker_sha256': sha(checker),
              'native_executable': str(executable), 'native_executable_sha256': before,
              'cases': cases, 'unchanged_continuum_wrench': continuum,
              'wall_observable_ablations': ablations,
              'rejected_quadratic_stencil_consistency_controls': derivative_consistency_controls(),
              'quadratic_candidate_rejected': ablations['quadratic-wall-jet'][1] > 0,
              'coarse_samples_all_zero': True,
              'coarse_sampling_information_blocker': 'zero and this nonzero-load smooth solenoidal field share all48 samples and stationary traces',
              'ablations_change_no_expected_values': True,
              'ablations_preserve_joint_work_or_closure': False,
              'pressure_coupling': False, 'stepping': False, 'physical_load_qualification': False}
    (output/'research.json').write_text(json.dumps(serialize(result), indent=2)+'\n')
    print(json.dumps({'output': str(output/'research.json'), 'sha256': sha(output/'research.json'),
                      'native_torque': str(cases['polynomial-bounded']['finite_model']['total'][5]),
                      'physical_torque': str(continuum[5]), 'quadratic_candidate_rejected': True}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--executable', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run(args.executable.resolve(), args.output.resolve())
