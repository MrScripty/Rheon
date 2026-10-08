#!/usr/bin/env python3
"""Independent exact-rational lab for the bounded aligned-box strain operator.

Geometry is reconstructed from input grid planes and centers.  No Rust row,
weight, active index, mass, or matrix entry is used to construct the reference.
Binary64 coordinate evaluation is explicit; subsequent geometry/algebra uses
Fraction, so a represented center need not be a cell midpoint.
"""
from __future__ import annotations

from dataclasses import dataclass
from fractions import Fraction as F
from itertools import product
from pathlib import Path
import argparse
import json
import math
import struct


class Refusal(ValueError):
    """A missing, malformed, unsupported, or disagreeing lab observation."""


def require(condition, message):
    if not condition:
        raise Refusal(message)


def bits_value(token):
    require(len(token) == 16 and all(c in '0123456789abcdef' for c in token),
            'invalid binary64 bit token')
    value = struct.unpack('>d', bytes.fromhex(token))[0]
    require(math.isfinite(value), 'nonfinite binary64 observation')
    return F.from_float(value)


def represented(origin, spacing, index):
    return F.from_float(float(origin) + float(index) * float(spacing))


def flatten(p, dims):
    return p[0] + dims[0] * (p[1] + dims[1] * p[2])


def coordinates(dims):
    # Native MAC storage is x-fastest.
    for k in range(dims[2]):
        for j in range(dims[1]):
            for i in range(dims[0]):
                yield (i, j, k)


@dataclass(frozen=True)
class Geometry:
    counts: tuple[int, int, int]
    spacing: tuple[F, F, F]
    origin: tuple[F, F, F]
    lower: tuple[int, int, int]
    upper: tuple[int, int, int]
    density: F = F(1)
    viscosity: F = F(1)

    def __post_init__(self):
        require(all(type(n) is int and n >= 3 for n in self.counts), 'unsupported grid counts')
        require(all(h > 0 for h in self.spacing), 'nonpositive spacing')
        require(self.density > 0 and self.viscosity > 0, 'nonpositive material parameter')
        require(all(type(l) is int and type(u) is int and 1 <= l < u <= n - 1
                    for l, u, n in zip(self.lower, self.upper, self.counts)),
                'box must be aligned, internal, and have one fluid cell of padding')
        for d in range(3):
            for i in range(self.counts[d]):
                require(self.plane(d, i) < self.center(d, i) < self.plane(d, i + 1),
                        'unrepresentable grid coordinates')

    def plane(self, d, i):
        return represented(self.origin[d], self.spacing[d], i)

    def center(self, d, i):
        return represented(self.origin[d], self.spacing[d], F(i) + F(1, 2))

    def fluid(self, p):
        return all(0 <= p[d] < self.counts[d] for d in range(3)) and not all(
            self.lower[d] <= p[d] < self.upper[d] for d in range(3))

    def width(self, d, i):
        return self.plane(d, i + 1) - self.plane(d, i)

    def volume(self, p):
        return math.prod(self.width(d, p[d]) for d in range(3))

    def position(self, axis, p):
        return tuple(self.plane(d, p[d]) if axis == d else self.center(d, p[d]) for d in range(3))


@dataclass(frozen=True)
class Face:
    axis: int
    flat: int
    p: tuple[int, int, int]
    position: tuple[F, F, F]
    area: F
    distance: F
    mass: F


@dataclass(frozen=True)
class Row:
    family: tuple[int, int]
    p: tuple[int, int, int]
    quadrant: int
    weight: F
    terms: tuple[tuple[int, F], ...]
    closure: int = 0

    @property
    def key(self):
        return (*self.family, *self.p, self.quadrant)


def active_faces(g):
    faces = []
    for axis in range(3):
        dims = list(g.counts)
        dims[axis] += 1
        for p in coordinates(dims):
            below = list(p)
            below[axis] -= 1
            if not (g.fluid(tuple(below)) and g.fluid(p)):
                continue
            area = math.prod(g.width(d, p[d]) for d in range(3) if d != axis)
            distance = g.center(axis, p[axis]) - g.center(axis, p[axis] - 1)
            faces.append(Face(axis, flatten(p, dims), p, g.position(axis, p), area,
                              distance, g.density * area * distance))
    return faces


def canonical_terms(terms):
    result = {}
    for face, coefficient in terms:
        result[face] = result.get(face, F(0)) + coefficient
    return tuple(sorted((face, coefficient) for face, coefficient in result.items() if coefficient))


def matrix(rows, n):
    result = {}
    for row in rows:
        for i, a in row.terms:
            for j, b in row.terms:
                key = (i, j)
                result[key] = result.get(key, F(0)) + row.weight * a * b
    return {key: value for key, value in result.items() if value}


def bound(rows, faces):
    loads = [F(0)] * len(faces)
    for row in rows:
        norm = sum(abs(a) for _, a in row.terms)
        for i, a in row.terms:
            loads[i] += row.weight * abs(a) * norm
    return max((load / face.mass for load, face in zip(loads, faces)), default=F(0))


def apply(rows, field):
    result = [F(0)] * len(field)
    dissipation = F(0)
    for row in rows:
        strain = sum(a * field[i] for i, a in row.terms)
        dissipation += row.weight * strain * strain
        for i, a in row.terms:
            result[i] += row.weight * a * strain
    return result, dissipation


def close(actual, expected, label, tolerance=F(1, 10**11)):
    # Absolute floor accommodates a computed exact zero, relative term scales
    # only with the independently reconstructed expected quantity.
    require(abs(actual - expected) <= tolerance * max(F(1), abs(expected)),
            f'{label}: expected {expected}, observed {actual}')


def strain_rows(g, faces):
    """Derive normals and actual fluid sectors on strictly internal ab edges."""
    lookup = {(face.axis, face.p): i for i, face in enumerate(faces)}
    rows = []

    def term(axis, p, coefficient):
        i = lookup.get((axis, tuple(p)))
        return [] if i is None else [(i, coefficient)]

    for p in coordinates(g.counts):
        if not g.fluid(p):
            continue
        for a in range(3):
            hi = list(p)
            hi[a] += 1
            distance = g.width(a, p[a])
            terms = term(a, p, -1 / distance) + term(a, hi, 1 / distance)
            rows.append(Row((a, a), p, 0, 2 * g.volume(p), canonical_terms(terms), 4))
    for a, b in ((0, 1), (0, 2), (1, 2)):
        c = 3 - a - b
        dims = list(g.counts)
        dims[a] += 1
        dims[b] += 1
        for p in coordinates(dims):
            if p[a] in (0, g.counts[a]) or p[b] in (0, g.counts[b]):
                continue
            sectors = []
            for sa, sb in product((-1, 1), repeat=2):
                cell = list(p)
                cell[a] += (sa - 1) // 2
                cell[b] += (sb - 1) // 2
                if g.fluid(tuple(cell)):
                    sectors.append((sa, sb, tuple(cell)))
            require(len(sectors) in (0, 2, 3, 4), 'unsupported internal edge sector topology')
            if not sectors:
                continue
            full_terms = []
            if len(sectors) == 4:
                for axis, transverse in ((a, b), (b, a)):
                    lo = list(p)
                    lo[transverse] -= 1
                    distance = g.center(transverse, p[transverse]) - g.center(transverse, p[transverse] - 1)
                    full_terms += term(axis, lo, -1 / distance) + term(axis, p, 1 / distance)
            elif len(sectors) == 2:
                require(sectors[0][0] == sectors[1][0] or sectors[0][1] == sectors[1][1],
                        'unsupported diagonal fluid sectors')
                common_a = sectors[0][0] == sectors[1][0]
                axis, transverse, side = (b, a, sectors[0][0]) if common_a else (a, b, sectors[0][1])
                face = list(p)
                face[transverse] += (side - 1) // 2
                distance = abs(g.center(transverse, face[transverse]) - g.plane(transverse, p[transverse]))
                full_terms = term(axis, face, F(side) / distance)
            else:
                sa, sb = next((sa, sb) for sa, sb in product((-1, 1), repeat=2)
                              if not any(sa == ta and sb == tb for ta, tb, _ in sectors))
                ua = list(p)
                ua[b] += (-sb - 1) // 2
                ub = list(p)
                ub[a] += (-sa - 1) // 2
                ell_b = abs(g.center(b, ua[b]) - g.plane(b, p[b]))
                ell_a = abs(g.center(a, ub[a]) - g.plane(a, p[a]))
            for ta, tb, cell in sectors:
                weight = (abs(g.center(a, cell[a]) - g.plane(a, p[a]))
                          * abs(g.center(b, cell[b]) - g.plane(b, p[b]))
                          * g.width(c, cell[c]))
                terms = full_terms
                if len(sectors) == 3:
                    terms = (term(a, ua, F(-sb) / ell_b) if tb == -sb else [])
                    terms += term(b, ub, F(-sa) / ell_a) if ta == -sa else []
                q = (ta + 1) // 2 + 2 * ((tb + 1) // 2)
                rows.append(Row((a, b), p, q, weight, canonical_terms(terms),
                                {4: 0, 2: 1, 3: 2}[len(sectors)]))
    return rows


def reference(g):
    faces = active_faces(g)
    rows = strain_rows(g, faces)
    return faces, rows, matrix(rows, len(faces)), bound(rows, faces)


@dataclass
class Dump:
    geometry: Geometry
    faces: list[Face]
    rows: list[Row]
    matrix: dict[tuple[int, int], F]
    field: list[F]
    action: list[F]
    ledger: tuple[F, ...]


def parse_dump(path):
    records = {}
    for line in Path(path).read_text().splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        items = line.split('\t')
        require(items[0] in ('META', 'ACTIVE', 'ROW', 'MATRIX', 'FIELD', 'ACTION', 'LEDGER'),
                f'unknown record {items[0]}')
        records.setdefault(items[0], []).append(items[1:])
    require(len(records.get('META', [])) == 1, 'exactly one META required')
    m = records['META'][0]
    require(len(m) == 17, 'META shape')
    g = Geometry(tuple(map(int, m[:3])), tuple(map(bits_value, m[3:6])),
                 tuple(map(bits_value, m[6:9])), tuple(map(int, m[9:12])),
                 tuple(map(int, m[12:15])), bits_value(m[15]), bits_value(m[16]))
    faces = []
    for f in records.get('ACTIVE', []):
        require(len(f) == 12 and int(f[0]) == len(faces), 'ACTIVE shape/index')
        faces.append(Face(int(f[1]), int(f[2]), tuple(map(int, f[3:6])),
                          tuple(map(bits_value, f[6:9])), *map(bits_value, f[9:12])))
    rows = []
    for r in records.get('ROW', []):
        require(len(r) >= 10 and int(r[0]) == len(rows), 'ROW shape/index')
        count = int(r[9])
        require(count >= 0 and len(r) == 10 + 2 * count, 'ROW term shape')
        terms = tuple((int(r[10 + 2 * k]), bits_value(r[11 + 2 * k])) for k in range(count))
        require(all(0 <= i < len(faces) and a != 0 for i, a in terms), 'invalid ROW term')
        require(len({i for i, _ in terms}) == len(terms), 'duplicate ROW term')
        rows.append(Row(tuple(map(int, r[1:3])), tuple(map(int, r[3:6])),
                        int(r[6]), bits_value(r[7]), tuple(sorted(terms)), int(r[8])))
    entries = {}
    for r in records.get('MATRIX', []):
        require(len(r) == 3, 'MATRIX shape')
        key = tuple(map(int, r[:2]))
        require(key not in entries and all(0 <= i < len(faces) for i in key), 'MATRIX key')
        value = bits_value(r[2])
        require(value != 0, 'explicit zero MATRIX entry')
        entries[key] = value
    vectors = []
    for tag in ('FIELD', 'ACTION'):
        vec = []
        for r in records.get(tag, []):
            require(len(r) == 2 and int(r[0]) == len(vec), f'{tag} shape/index')
            vec.append(bits_value(r[1]))
        require(len(vec) == len(faces), f'{tag} missing entries')
        vectors.append(vec)
    require(len(records.get('LEDGER', [])) == 1, 'exactly one LEDGER required')
    ledger = tuple(map(bits_value, records['LEDGER'][0]))
    require(len(ledger) == 4, 'LEDGER shape')
    return Dump(g, faces, rows, entries, *vectors, ledger)


def verify(path):
    dump = parse_dump(path)
    faces, rows, stiffness, coefficient_bound = reference(dump.geometry)
    require(len(faces) == len(dump.faces), 'ACTIVE count disagreement')
    for i, (expected, actual) in enumerate(zip(faces, dump.faces)):
        require((actual.axis, actual.flat, actual.p) == (expected.axis, expected.flat, expected.p),
                f'ACTIVE {i} geometry identity')
        for label in ('area', 'distance', 'mass'):
            close(getattr(actual, label), getattr(expected, label), f'ACTIVE {i} {label}')
        require(actual.position == expected.position, f'ACTIVE {i} represented position')
    expected_rows = {r.key: r for r in rows}
    actual_rows = {r.key: r for r in dump.rows}
    require(len(actual_rows) == len(dump.rows), 'duplicate ROW key')
    require(actual_rows.keys() == expected_rows.keys(), 'ROW geometry/count disagreement')
    for key, expected in expected_rows.items():
        actual = actual_rows[key]
        require(actual.closure == expected.closure, f'ROW {key} closure disagreement')
        close(actual.weight, expected.weight, f'ROW {key} weight')
        require(tuple(i for i, _ in actual.terms) == tuple(i for i, _ in expected.terms),
                f'ROW {key} support disagreement')
        for (_, a), (_, b) in zip(actual.terms, expected.terms):
            close(a, b, f'ROW {key} coefficient')
    require(dump.matrix.keys() == stiffness.keys(), 'MATRIX support disagreement')
    for key, expected in stiffness.items():
        close(dump.matrix[key], expected, f'MATRIX {key}')
    action, dissipation = apply(rows, dump.field)
    for i, (actual, expected) in enumerate(zip(dump.action, action)):
        close(actual, expected, f'ACTION {i}')
    work = sum(v * f for v, f in zip(dump.field, action))
    for label, actual, expected in zip(('dissipation', 'work', 'identity', 'B'), dump.ledger,
                                      (dump.geometry.viscosity * dissipation, -dump.geometry.viscosity * work, F(0), coefficient_bound)):
        close(actual, expected, f'LEDGER {label}')
    return {'active_faces': len(faces), 'rows': len(rows), 'zero_rows': sum(not r.terms for r in rows),
            'matrix_nonzero': len(stiffness), 'B_exact': str(coefficient_bound),
            'dissipation_exact': str(dissipation), 'dump': str(Path(path).resolve())}


def mutation_suite(path):
    """Reject deliberately corrupted copies of a freshly verified native dump."""
    from copy import deepcopy
    from tempfile import TemporaryDirectory
    verify(path)
    records = [line.split('\t') for line in Path(path).read_text().splitlines()
               if line.strip() and not line.startswith('#')]
    variants = ('coefficient', 'row', 'zero_row', 'mass', 'B', 'matrix', 'action',
                'nan', 'unknown', 'duplicate', 'missing_meta')
    rejected = []
    with TemporaryDirectory(prefix='rheon-aligned-negative-') as temp:
        target = Path(temp) / 'mutated.tsv'
        for variant in variants:
            changed = deepcopy(records)
            if variant == 'coefficient':
                next(r for r in changed if r[0] == 'ROW' and int(r[10]) > 0)[12] = struct.pack('>d', 123.0).hex()
            elif variant in ('row', 'zero_row'):
                index = next((i for i, r in enumerate(changed) if r[0] == 'ROW' and
                              (variant == 'row' and int(r[10]) > 0 or variant == 'zero_row' and r[10] == '0')), None)
                require(index is not None, f'no {variant} mutation target')
                changed.pop(index)
                for i, r in enumerate(r for r in changed if r[0] == 'ROW'):
                    r[1] = str(i)
            elif variant == 'mass':
                next(r for r in changed if r[0] == 'ACTIVE')[-1] = struct.pack('>d', 123.0).hex()
            elif variant == 'B':
                next(r for r in changed if r[0] == 'LEDGER')[-1] = struct.pack('>d', 123.0).hex()
            elif variant in ('matrix', 'action'):
                next(r for r in changed if r[0] == variant.upper())[-1] = struct.pack('>d', 999.0).hex()
            elif variant == 'nan':
                next(r for r in changed if r[0] == 'ACTIVE')[-1] = '7ff8000000000000'
            elif variant == 'unknown':
                changed.append(['UNKNOWN'])
            elif variant == 'duplicate':
                changed.append(next(r for r in changed if r[0] == 'MATRIX'))
            else:
                changed = [r for r in changed if r[0] != 'META']
            target.write_text('\n'.join('\t'.join(r) for r in changed) + '\n')
            try:
                verify(target)
            except (Refusal, ValueError):
                rejected.append(variant)
            else:
                raise Refusal(f'mutated {variant} dump was accepted')
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('dump', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--negative-tests', action='store_true')
    args = parser.parse_args()
    result = verify(args.dump)
    if args.negative_tests:
        result['rejected_mutations'] = mutation_suite(args.dump)
    encoded = json.dumps(result, indent=2) + '\n'
    if args.output:
        args.output.write_text(encoded)
    print(encoded, end='')


if __name__ == '__main__':
    main()
