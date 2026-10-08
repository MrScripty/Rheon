"""Fresh exact-algebra checks and adversarial native-dump qualification."""
from copy import deepcopy
from fractions import Fraction as F
from pathlib import Path
import os
import struct
import subprocess
import sys
import tempfile
import unittest

from aligned_strain_oracle import (Geometry, Refusal, active_faces, apply, bound,
                                    canonical_terms, matrix, mutation_suite, reference, strain_rows, verify)


def geometry(counts=(3, 3, 3), spacing=(1, 1, 1), origin=(0, 0, 0), lower=(1, 1, 1), upper=(2, 2, 2)):
    return Geometry(counts, tuple(F.from_float(float(x)) for x in spacing),
                    tuple(F.from_float(float(x)) for x in origin), lower, upper)


def bits(x):
    return struct.pack('>d', float(x)).hex()


def synthetic_dump(g):
    """Parser/mutation unit fixture only; never advertised as native evidence."""
    faces, rows, stiffness, b = reference(g)
    field = [F((i * 7) % 19 - 9, 8) for i in range(len(faces))]
    action, d = apply(rows, field)
    lines = [['META', *g.counts, *map(bits, g.spacing), *map(bits, g.origin),
              *g.lower, *g.upper, bits(g.density), bits(g.viscosity)]]
    for i, f in enumerate(faces):
        lines.append(['ACTIVE', i, f.axis, f.flat, *f.p, *map(bits, f.position),
                      bits(f.area), bits(f.distance), bits(f.mass)])
    for i, r in enumerate(rows):
        terms = [v for face, a in r.terms for v in (face, bits(a))]
        lines.append(['ROW', i, *r.family, *r.p, r.quadrant, bits(r.weight), r.closure, len(r.terms), *terms])
    lines += [['MATRIX', i, j, bits(v)] for (i, j), v in sorted(stiffness.items())]
    lines += [['FIELD', i, bits(v)] for i, v in enumerate(field)]
    lines += [['ACTION', i, bits(v)] for i, v in enumerate(action)]
    lines.append(['LEDGER', bits(g.viscosity * d), bits(-g.viscosity * d), bits(0), bits(b)])
    return [[str(x) for x in line] for line in lines]


def write_records(path, records):
    path.write_text('\n'.join('\t'.join(row) for row in records) + '\n')


class RationalGeometryTest(unittest.TestCase):
    def test_fresh_unit_geometry_shape_and_bound(self):
        faces, rows, stiffness, b = reference(geometry())
        self.assertEqual((len(faces), len(rows), sum(not r.terms for r in rows)), (48, 210, 6))
        self.assertEqual(b, 17)
        self.assertEqual(len(stiffness), 408)
        self.assertTrue(all(face.mass == 1 for face in faces))
        self.assertEqual(sum(r.weight for r in rows if r.family[0] == r.family[1]), 156)

    def test_corner_reflections_and_anisotropy(self):
        g = geometry(spacing=(2, 3, 5))
        faces, rows, _, _ = reference(g)
        for ea, eb in ((1, 1), (1, 2), (2, 1), (2, 2)):
            with self.subTest(edge=(ea, eb)):
                selected = [r for r in rows if r.family == (0, 1) and r.p == (ea, eb, 1)]
                ids = sorted({i for r in selected for i, _ in r.terms})
                self.assertEqual(len(selected), 3)
                self.assertEqual(len(ids), 2)
                m = matrix(selected, len(faces))
                u = next(i for i in ids if faces[i].axis == 0)
                v = next(i for i in ids if faces[i].axis == 1)
                self.assertEqual(m[u, u], F(20, 3))
                self.assertEqual(m[v, v], 15)
                self.assertEqual(m[u, v], 5 * (1 if ea == eb else -1))
        unit_faces, unit_rows, _, _ = reference(geometry())
        corner = [r for r in unit_rows if r.family == (0, 1) and r.p == (1, 1, 1)]
        ids = sorted({i for r in corner for i, _ in r.terms})
        m = matrix(corner, len(unit_faces))
        self.assertEqual([[m[i, j] for j in ids] for i in ids], [[2, 1], [1, 2]])

    def test_flat_actual_volume_over_distance_squared(self):
        g = geometry(spacing=(2, 3, 5))
        faces, rows, _, _ = reference(g)
        # A wider obstacle exposes a genuine flat edge.
        g = geometry(counts=(4, 3, 3), spacing=(2, 3, 5), upper=(3, 2, 2))
        faces, rows, _, _ = reference(g)
        selected = [r for r in rows if r.family == (0, 1) and r.p == (2, 1, 1)]
        self.assertEqual(len(selected), 2)
        self.assertTrue(all(len(r.terms) == 1 for r in selected))
        delta = F(3, 2)
        volumes = sum(r.weight for r in selected)
        a_eff = volumes / delta
        self.assertEqual(volumes / delta**2, a_eff / delta)
        self.assertEqual(next(iter(matrix(selected, len(faces)).values())), a_eff / delta)

    def test_nonmidpoint_represented_distances_and_true_sector_volumes(self):
        g = geometry(spacing=(.3, .7, 1.1), origin=(100000000, -100000000, .1))
        faces, rows, _, _ = reference(g)
        self.assertNotEqual(g.center(0, 0) - g.plane(0, 1), -g.width(0, 0) / 2)
        selected = [r for r in rows if r.family == (0, 1) and r.p == (1, 1, 1)]
        for row in selected:
            sa = 1 if row.quadrant & 1 else -1
            sb = 1 if row.quadrant & 2 else -1
            cell = (1 + (sa - 1) // 2, 1 + (sb - 1) // 2, 1)
            expected = abs(g.center(0, cell[0]) - g.plane(0, 1)) * abs(
                g.center(1, cell[1]) - g.plane(1, 1)) * g.width(2, 1)
            self.assertEqual(row.weight, expected)
        self.assertTrue(any(row.weight != g.volume((0, 0, 1)) / 4 for row in selected))
        self.assertTrue(all(face.mass == g.density * face.area * face.distance for face in faces))

    def test_cross_component_affine_strain_and_compatible_local_rotation(self):
        g = geometry(counts=(5, 5, 5), lower=(3, 3, 3), upper=(4, 4, 4), spacing=(2, 3, 5))
        faces, rows, _, _ = reference(g)
        gradients = ((F(2), F(3), F(-1)), (F(4), F(-2), F(5)), (F(6), F(-3), F(1)))
        field = [sum(gradients[f.axis][d] * f.position[d] for d in range(3)) for f in faces]
        patch = [r for r in rows if r.p == (1, 1, 1)]
        for row in patch:
            a, b = row.family
            expected = gradients[a][a] if a == b else gradients[a][b] + gradients[b][a]
            self.assertEqual(sum(v * field[i] for i, v in row.terms), expected)
        # This local full-fluid rotation uses only active samples and is
        # compatible with the patch. A global rigid rotation would violate
        # stationary outer/obstacle traces and is not claimed as a null mode.
        rotation = ((0, -2, 3), (2, 0, -4), (-3, 4, 0))
        field = [sum(rotation[f.axis][d] * f.position[d] for d in range(3)) for f in faces]
        self.assertEqual(apply(patch, field)[1], 0)
        self.assertGreater(apply(rows, field)[1], 0)

    def test_unsupported_geometry_refuses(self):
        for kwargs in ({'lower': (0, 1, 1)}, {'upper': (3, 2, 2)},
                       {'lower': (F(3, 2), 1, 1)}, {'spacing': (0, 1, 1)},
                       {'counts': (2, 3, 3)}, {'origin': (1e30, 0, 0)}):
            with self.subTest(kwargs=kwargs), self.assertRaises(Refusal):
                geometry(**kwargs)


class FailClosedDumpTest(unittest.TestCase):
    def test_native_dump_if_qualification_supplies_one(self):
        path = os.environ.get('RHEON_ALIGNED_NATIVE_DUMP')
        if not path:
            self.skipTest('native dump is supplied by qualification, not synthesized')
        result = verify(path)
        self.assertGreater(result['active_faces'], 0)
        self.assertEqual(len(mutation_suite(path)), 11)

    def test_parser_fixture_and_adversarial_mutations(self):
        records = synthetic_dump(geometry())
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'unit.tsv'
            write_records(path, records)
            self.assertEqual(verify(path)['B_exact'], '17')
            for mutation in ('coefficient', 'row', 'zero_row', 'mass', 'B', 'matrix', 'action',
                             'nan', 'unknown', 'duplicate', 'missing_meta'):
                changed = deepcopy(records)
                with self.subTest(mutation=mutation):
                    if mutation == 'coefficient':
                        next(r for r in changed if r[0] == 'ROW' and int(r[10]) > 0)[12] = bits(123)
                    elif mutation in ('row', 'zero_row'):
                        index = next(i for i, r in enumerate(changed) if r[0] == 'ROW' and
                                     (mutation == 'row' and int(r[10]) > 0 or mutation == 'zero_row' and r[10] == '0'))
                        changed.pop(index)
                        # Renumbering prevents a trivial sequence-index refusal.
                        for i, r in enumerate(r for r in changed if r[0] == 'ROW'):
                            r[1] = str(i)
                    elif mutation == 'mass':
                        next(r for r in changed if r[0] == 'ACTIVE')[-1] = bits(2)
                    elif mutation == 'B':
                        next(r for r in changed if r[0] == 'LEDGER')[-1] = bits(18)
                    elif mutation == 'matrix':
                        next(r for r in changed if r[0] == 'MATRIX')[-1] = bits(999)
                    elif mutation == 'action':
                        next(r for r in changed if r[0] == 'ACTION')[-1] = bits(999)
                    elif mutation == 'nan':
                        next(r for r in changed if r[0] == 'ACTIVE')[-1] = '7ff8000000000000'
                    elif mutation == 'unknown':
                        changed.append(['UNKNOWN'])
                    elif mutation == 'duplicate':
                        changed.append(next(r for r in changed if r[0] == 'MATRIX'))
                    else:
                        changed = [r for r in changed if r[0] != 'META']
                    write_records(path, changed)
                    with self.assertRaises((Refusal, ValueError)):
                        verify(path)

    def test_scaled_sign_and_relative_corruptions_normal_and_optimized(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'scaled.tsv'
            for spacing in (1e-6, 1e12):
                records = synthetic_dump(geometry(spacing=(spacing,) * 3))
                write_records(path, records)
                verify(path)
                for tag, column in (('ACTIVE', 10), ('ACTIVE', 11), ('ACTIVE', 12),
                                    ('ROW', 8), ('ROW', 12)):
                    for factor in (-1, 2):
                        with self.subTest(spacing=spacing, tag=tag, column=column, factor=factor):
                            changed = deepcopy(records)
                            row = next(r for r in changed if r[0] == tag and
                                       (tag != 'ROW' or column != 12 or int(r[10]) > 0))
                            old = struct.unpack('>d', bytes.fromhex(row[column]))[0]
                            row[column] = bits(old * factor)
                            write_records(path, changed)
                            with self.assertRaises(Refusal):
                                verify(path)
                            result = subprocess.run([sys.executable, '-O', str(
                                Path(__file__).with_name('aligned_strain_oracle.py')), str(path)],
                                capture_output=True, text=True)
                            self.assertNotEqual(result.returncode, 0)

    def test_optimized_python_keeps_zero_row_gate(self):
        records = synthetic_dump(geometry())
        records = [r for r in records if not (r[0] == 'ROW' and r[10] == '0')]
        for i, row in enumerate(r for r in records if r[0] == 'ROW'):
            row[1] = str(i)
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'missing-zero.tsv'
            write_records(path, records)
            result = subprocess.run([sys.executable, '-O', str(Path(__file__).with_name('aligned_strain_oracle.py')), str(path)],
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('ROW geometry/count disagreement', result.stderr)


if __name__ == '__main__':
    unittest.main()
