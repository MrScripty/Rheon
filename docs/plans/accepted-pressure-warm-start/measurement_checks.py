"""Strict, optimization-safe validation for retained measurement records."""
import csv
import io
import math
from pathlib import Path

COLUMNS = {
    'step', 'accepted', 'startup_s', 'step_s', 'pressure_s', 'capture_s',
    'iterations', 'time', 'dt', 'residual_l2', 'residual_max', 'divergence',
    'mass_error', 'energy', 'pressure_error', 'max_speed', 'bytes',
    'fingerprint', 'error',
}
TIMINGS = {'startup_s', 'step_s', 'pressure_s', 'capture_s'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def finite(value, label, nonnegative=False):
    number = float(value)
    require(math.isfinite(number), f'{label}: nonfinite number')
    require(not nonnegative or number >= 0.0, f'{label}: negative number')
    return number


def read_rows(text, steps, mode):
    reader = csv.DictReader(io.StringIO(text))
    require(reader.fieldnames is not None and set(reader.fieldnames) == COLUMNS,
            'unexpected CSV columns')
    require(len(reader.fieldnames) == len(COLUMNS), 'duplicate CSV column')
    rows = list(reader)
    require(0 < len(rows) <= steps, 'empty or oversized trajectory')
    time = 0.0
    for index, row in enumerate(rows):
        require(set(row) == COLUMNS and all(v is not None for v in row.values()),
                'malformed CSV row')
        require(row['step'] == str(index), 'wrong step sequence')
        require(row['accepted'] in {'true', 'false'}, 'invalid accepted flag')
        for metric in TIMINGS:
            finite(row[metric], metric, nonnegative=True)
        require(int(row['iterations']) >= 0 and int(row['bytes']) > 0,
                'invalid iterations or managed bytes')
        require(len(row['fingerprint']) == 16 and
                all(c in '0123456789abcdef' for c in row['fingerprint']),
                'invalid fingerprint')
        finite(row['max_speed'], 'max_speed', nonnegative=True)
        if row['accepted'] == 'true':
            dt = 1/128 if mode == 'switch' and index % 2 else 1/64
            require(finite(row['dt'], 'dt') == dt, 'changed accepted timestep')
            time += dt
            require(row['error'] == '', 'accepted row contains error')
            for metric in ['residual_l2', 'residual_max', 'divergence', 'energy']:
                finite(row[metric], metric, nonnegative=True)
            finite(row['mass_error'], 'mass_error')
            if mode != 'localized':
                finite(row['pressure_error'], 'pressure_error', nonnegative=True)
        else:
            require(index == len(rows)-1, 'failure is not terminal')
            require(bool(row['error']), 'failure lacks diagnostic')
            require(finite(row['dt'], 'dt') == 0.0, 'failed step advanced dt')
            if index:
                for field in ['fingerprint', 'bytes', 'max_speed']:
                    require(row[field] == rows[index-1][field],
                            f'failed step changed accepted {field}')
        require(finite(row['time'], 'time') == time, 'changed physical horizon')
    require(rows[-1]['accepted'] == 'false' or len(rows) == steps,
            'incomplete successful trajectory')
    require(sum(float(r['step_s']) + float(r['capture_s']) for r in rows) > 0,
            'nonpositive attempt cost')
    return rows


def validate_receipt(root, prefix, records, counts, workers, mode, method,
                     baseline=None, candidate=None):
    require(isinstance(records, list) and len(records) == 3, 'bad receipt shape')
    require({r['variant'] for r in records} == {'base', 'cold', 'warm'},
            'missing or duplicate receipt variant')
    data = {}
    for record in records:
        variant = record['variant']
        expected = [*map(str, counts), str(workers), '16',
                    'warm' if variant == 'warm' else 'cold', mode, method]
        require(len(record['argv']) == 9 and record['argv'][1:] == expected,
                'receipt command differs from declared case')
        binary = baseline if variant == 'base' else candidate
        if binary is not None:
            require(Path(record['argv'][0]).resolve() == Path(binary).resolve(),
                    'receipt binary differs from manifest')
        stem = root / f'{prefix}-{variant}'
        rows = read_rows(stem.with_suffix('.csv').read_text(), 16, mode)
        require(stem.with_suffix('.stderr').is_file(), 'missing stderr record')
        require(int(stem.with_suffix('.rss').read_text()) > 0, 'invalid peak RSS')
        total = sum(float(r['step_s']) + float(r['capture_s']) for r in rows)
        require(math.isclose(finite(record['full_s'], 'receipt total'), total,
                             rel_tol=1e-12, abs_tol=1e-12), 'receipt total mismatch')
        require(record['accepted'] == sum(r['accepted'] == 'true' for r in rows),
                'receipt accepted count mismatch')
        data[variant] = rows
    return data
