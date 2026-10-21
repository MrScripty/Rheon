#!/usr/bin/env python3
"""Independent stdlib SQLite/native-bit oracle for diagnostic pressure summaries.
Checks pinned source/selection/time and derives norms with fsum, independently of
Rust's streaming hypot. This verifies reduction/custody, not model accuracy or
archive admissibility; use DiagnosticReader.verify for full lineage validation.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import sqlite3
import struct


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return list(hashlib.sha256(path.read_bytes()).digest())


def number(raw):
    require(len(raw) == 8, "scalar width")
    return struct.unpack('<d', raw)[0]


def l2_norm(values):
    """Scaled fsum is independent of streaming hypot and preserves finite range."""
    require(all(math.isfinite(value) for value in values), 'finite norm input')
    scale = max(map(abs, values), default=0.0)
    if scale == 0.0:
        return 0.0
    norm = scale * math.sqrt(math.fsum((value / scale) ** 2 for value in values))
    require(math.isfinite(norm), 'representable norm')
    return norm


def verify(path):
    summary = json.loads(path.read_bytes())
    require(summary['schema'] == 'rheon-diagnostic-pressure-comparison-v1' and summary['analysis_version'] == 1, 'analysis schema')
    manifest = json.loads((path.parent / summary['run_set_file']).read_bytes())
    require(manifest['sha256'] == summary['run_set_sha256'], 'manifest identity')
    member = next(m for m in manifest['run_set']['members'] if m['member_id'] == summary['member_id'])
    archive = path.parent / member['archive_file']
    require(digest(archive) == summary['archive_sha256'] == member['finalized']['sha256'], 'archive identity')
    with sqlite3.connect(archive.resolve().as_uri() + '?mode=ro', uri=True) as db:
        require(db.execute('PRAGMA application_id').fetchone()[0] == 0x52484450, 'diagnostic application ID')
        require(db.execute('PRAGMA user_version').fetchone()[0] == 2, 'diagnostic schema version')
        run_id, geometry = db.execute('SELECT run_id,geometry FROM diagnostic_run WHERE id=1').fetchone()
        row = db.execute('SELECT start,dt,request_digest FROM attempts WHERE attempt_index=?', (summary['attempt_index'],)).fetchone()
        samples = dict(db.execute('SELECT role,pressure FROM pressure_samples WHERE attempt_index=?', (summary['attempt_index'],)))
        request_pins = [v[0] for v in db.execute('SELECT request_digest FROM pressure_samples WHERE attempt_index=?', (summary['attempt_index'],))]
    require(run_id == summary['run_id'] == member['run_id'], 'run identity')
    require(list(struct.unpack('<3Q', geometry[:24])) == summary['counts'], 'native cell geometry')
    start, dt, request_pin = row
    require(list(request_pin) == summary['request_sha256'] and all(pin == request_pin for pin in request_pins), 'request identity')
    require(number(start) == summary['interval_start'] and number(dt) == summary['dt'] and number(start) + number(dt) == summary['interval_end'], 'physical interval')
    require(0 in samples and 1 in samples, 'candidate/native unavailable')
    x = [v[0] for v in struct.iter_unpack('<d', samples[1])]
    y = [v[0] for v in struct.iter_unpack('<d', samples[0])]
    n = math.prod(summary['counts'])
    require(len(x) == len(y) == n and n > 0 and all(math.isfinite(v) for v in x + y), 'finite native support')
    require(x[0] == y[0] == 0.0, 'canonical pressure gauge')
    delta = [b - a for a, b in zip(x, y, strict=True)]
    error = l2_norm(delta)
    norm = l2_norm(x)
    expected = dict(support_count=n, l2=error, rmse=error / math.sqrt(n), linf=max(map(abs, delta)), relative_l2=None if norm == 0 else error / norm)
    comparison = summary['comparison']
    require(summary['unavailable'] is None and comparison['units'] == 'Pa', 'pressure units/availability')
    require(comparison['gauge'] == dict(reference_canonical=True, candidate_canonical=True, removed_offset_pa=0.0, diagnostic=False), 'direct canonical gauge')
    require(comparison['color_bounds'] == dict(min=min(x + y), max=max(x + y)), 'common native bounds')
    for key, value in expected.items():
        actual = comparison['metrics'][key]
        require(actual is None if value is None else actual is not None and math.isfinite(actual) and math.isclose(actual, value, rel_tol=2e-13, abs_tol=1e-300), 'metric ' + key)
    return n


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('summary', type=Path)
    args = parser.parse_args()
    print(f'independent diagnostic pressure oracle passed: {verify(args.summary)} cells')
