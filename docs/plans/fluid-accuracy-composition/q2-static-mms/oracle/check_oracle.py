#!/usr/bin/env python3
"""Prospective CV3-A evaluator checks; NO science is run by importing this file.

Future separately admitted command: python check_oracle.py NATIVE_ROOT FRESH_OUTPUT
Runs the frozen independent analyzer, never the native executable. All selected
case/reference/mutation calls are behind main(). Source preparation is not launch
authority. Fixed resource limits are prospective, unmeasured hard-stop limits.

Adapter contract coordinated with the independent owner:
  analyze.py case CASE FRESH_OUTPUT
  analyze.py audit CASE FRESH_OUTPUT
  analyze.py aggregate NATIVE_ROOT CASE_REPORT_ROOT FRESH_OUTPUT
Successful/negative numerical outputs are deterministic analysis.json files;
gates[NAME] contains value, limit and pass; all_required_gates_pass summarizes
the mathematical disposition. Early identity refusals use exactly
"CV3 input guard: NAME" (optionally a RuntimeError traceback prefix).
"""
import csv
from fractions import Fraction as F
import hashlib
import io
import json
import math
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import sys
import time


CASES = tuple(f'{family}-n{n}' for family in (
    'rectangle-origin', 'square-17-origin', 'square-17-quarter') for n in (4, 8, 16))
MUTANT_CASE = 'square-17-origin-n4'
MODES = (('normal', ()), ('O', ('-O',)), ('OO', ('-OO',)))
CHILD_SECONDS = 1800
MODE_SECONDS = 10800
COMPLETE_SECONDS = 32400
CONTROL_SECONDS = 10800
ADDRESS_BYTES = 8 << 30
COMPLETE_OUTPUT_BYTES = 8 << 30
CONTROL_OUTPUT_BYTES = 2 << 30
INTEGER_DECIMAL_CHAR_LIMIT = 1_000_000
INTEGER_BIT_LIMIT = 3_000_000


def require(condition, message):
    if not condition:
        raise RuntimeError('CV3 harness guard: ' + message)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def identical_files(left, right):
    with left.open('rb') as a, right.open('rb') as b:
        while True:
            x, y = a.read(1 << 20), b.read(1 << 20)
            if x != y:
                return False
            if not x:
                return True


def inventory(root):
    require(root.is_dir() and not root.is_symlink(), 'owned ordinary corpus directory')
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), 'no corpus symlinks')
        if path.is_file():
            result[str(path.relative_to(root))] = digest(path)
        else:
            require(path.is_dir(), 'ordinary corpus entries only')
    require(result, 'nonempty corpus')
    return result


def output_size(root):
    total = 0
    for path in root.rglob('*'):
        require(not path.is_symlink(), 'no output symlinks')
        if path.is_file():
            total += path.stat().st_size
    return total


def child_cpu():
    used = resource.getrusage(resource.RUSAGE_CHILDREN)
    return used.ru_utime + used.ru_stime


def observed_rss(pid):
    try:
        lines = Path(f'/proc/{pid}/status').read_text().splitlines()
    except FileNotFoundError:
        return 0
    for line in lines:
        if line.startswith('VmRSS:'):
            return int(line.split()[1]) * 1024
    return 0


def check_budget(root, deadline, limit=CONTROL_OUTPUT_BYTES):
    require(time.monotonic() < deadline, 'fixed total wall limit')
    require(output_size(root) <= limit, 'fixed aggregate scratch limit')


def load_csv(path):
    with path.open(newline='') as stream:
        reader = csv.DictReader(stream)
        fields = reader.fieldnames
        require(fields is not None, 'mutation requires existing CSV header')
        rows = list(reader)
    require(rows and all(None not in r for r in rows), 'mutation requires complete source rows')
    return fields, rows


def save_csv(path, fields, rows):
    require(path.is_file() and not path.is_symlink(), 'rewrite only copied ordinary file')
    text = io.StringIO()
    writer = csv.DictWriter(text, fieldnames=fields, lineterminator='\n')
    writer.writeheader()
    writer.writerows(rows)
    path.write_text(text.getvalue())


def number(text):
    value = float(text)
    require(math.isfinite(value), 'finite source mutation primitive')
    return value


def represented(text):
    return F.from_float(number(text))


def render(value):
    value = float(value)
    require(math.isfinite(value), 'finite mutated value')
    return format(value, '.17e')


def input_rows(case):
    fields, rows = load_csv(case / 'input.csv')
    values = {r['key']: r['value'] for r in rows}
    require(len(values) == len(rows), 'source input keys unique')
    return fields, rows, values


def copied_case(raw, out, tag, deadline):
    """Independent fresh bytes, never symlinks or writes into the source corpus."""
    parent = out / ('fixture-' + tag)
    parent.mkdir()
    target = parent / MUTANT_CASE
    target.mkdir()
    source = raw / MUTANT_CASE
    for path in sorted(source.rglob('*')):
        dest = target / path.relative_to(source)
        require(not path.is_symlink(), 'mutation source symlink refused')
        if path.is_dir():
            dest.mkdir()
        elif path.is_file():
            check_budget(out, deadline)
            require(output_size(out) + path.stat().st_size <= CONTROL_OUTPUT_BYTES,
                    'fixture copy fits fixed aggregate scratch limit')
            shutil.copyfile(path, dest)
        else:
            require(False, 'ordinary mutation source entry')
    return target


def rewrite_residuals(case):
    """Recompute b-Cz for EVERY retained state from actual full exported blocks.

    KKT is [A,-B^T;-B,0], b=[rhs,0]. Lower b-Cz is +Bu;
    it is not the correction API's desired constraint -Bu.
    """
    fields, rows = load_csv(case / 'states.csv')
    _, loads = load_csv(case / 'loads.csv')
    rhs = [represented(r['rhs']) for r in sorted(loads, key=lambda r: int(r['index']))]
    nv = len(rhs)
    _, factors = load_csv(case / 'factors.csv')
    _, matrices = load_csv(case / 'operators' / 'matrices.csv')
    b_rows = [r for r in matrices if r['block'] == 'B']
    nc = 1 + max(int(r['row']) for r in b_rows)
    a = [[F(0)] * nv for _ in range(nv)]
    b = [[F(0)] * nv for _ in range(nc)]
    for r in factors:
        if r['block'] == 'A':
            a[int(r['row'])][int(r['column'])] = represented(r['value'])
    for r in b_rows:
        b[int(r['row'])][int(r['column'])] = represented(r['value'])
    for iteration in range(4):
        state = [r for r in rows if r['method'] == 'selected' and int(r['iteration']) == iteration]
        u = {int(r['index']): represented(r['value']) for r in state if r['kind'] == 'u'}
        p = {int(r['index']): represented(r['value']) for r in state if r['kind'] == 'p'}
        require(set(u) == set(range(nv)) and set(p) == set(range(nc)), 'complete mutation state')
        ru = [rhs[i] - sum((a[i][j] * u[j] for j in range(nv)), F(0))
              + sum((b[c][i] * p[c] for c in range(nc)), F(0)) for i in range(nv)]
        rp = [sum((b[c][j] * u[j] for j in range(nv)), F(0)) for c in range(nc)]
        for r in state:
            r['original_residual'] = render((ru if r['kind'] == 'u' else rp)[int(r['index'])])
    save_csv(case / 'states.csv', fields, rows)


def rewrite_rhs(case, *, body_factor=None, boundary=None):
    fields, rows = load_csv(case / 'loads.csv')
    for r in rows:
        index = int(r['index'])
        if body_factor is not None:
            r['body'] = render(represented(r['body']) * body_factor)
        if boundary is not None:
            r['boundary'] = render(boundary[index])
        r['rhs'] = render(represented(r['body']) + represented(r['boundary']))
    save_csv(case / 'loads.csv', fields, rows)
    rewrite_residuals(case)


def boundary_load_from_rows(case, points, basis):
    _, nodes = load_csv(case / 'operators' / 'nodes.csv')
    active = {int(r['active']): int(r['axis']) for r in nodes if r['active'] != ''}
    weighted = {int(r['q']): (represented(r['gx_weight']), represented(r['gy_weight'])) for r in points}
    boundary = [F(0)] * len(active)
    for r in basis:
        i = int(r['index'])
        boundary[i] += represented(r['phi']) * weighted[int(r['q'])][active[i]]
    return boundary


def pressure_traction(case, pressure_multiplier):
    """Keep the viscous contribution; change only the analytic -p n term."""
    fields, points = load_csv(case / 'traction-points.csv')
    for r in points:
        x, y = represented(r['x']), represented(r['y'])
        p = 1 + x + 2 * y + x * y  # fixed L=U=tau=1 continuum problem
        for value, normal in (('gx_weight', 'nx_weight'), ('gy_weight', 'ny_weight')):
            r[value] = render(represented(r[value]) + (1 - pressure_multiplier) * p * represented(r[normal]))
    save_csv(case / 'traction-points.csv', fields, points)
    _, basis = load_csv(case / 'traction-basis.csv')
    rewrite_rhs(case, boundary=boundary_load_from_rows(case, points, basis))


def two_point_degree4(case):
    """Three rows, degree0..3 Gauss2 moments, but incorrect degree4 moments.

    Use t=(-a,+a,+a) mapped onto [0,1], weights=(.5,.25,.25).
    Rebuild points, normal weights, continuum g, basis phi and boundary/RHS.
    No row-count/parser defect is introduced; the degree-four guard must reject.
    """
    pf, points = load_csv(case / 'traction-points.csv')
    bf, _ = load_csv(case / 'traction-basis.csv')
    _, segments = load_csv(case / 'operators' / 'segments.csv')
    _, node_rows = load_csv(case / 'operators' / 'nodes.csv')
    _, _, meta = input_rows(case)
    mu, depth = number(meta['mu']), number(meta['Href'])
    n = int(meta['n'])
    h = number(meta['extent']) / n
    active = {int(r['active']): r for r in node_rows if r['active'] != ''}
    by_record = {}
    for r in points:
        by_record.setdefault(int(r['record']), []).append(r)
    t_minus, t_plus = (1 - 1 / math.sqrt(3)) / 2, (1 + 1 / math.sqrt(3)) / 2
    basis = []
    require(by_record, 'nonempty copied physical traction segments')
    for record, group in by_record.items():
        segment = segments[record]
        require(int(segment['segment']) == record and segment['kind'] == 'global' and len(group) == 3,
                'three existing rows per physical segment')
        ax, ay, bx, by = (number(segment[k]) for k in ('ax', 'ay', 'bx', 'by'))
        for r, t, weight in zip(group, (t_minus, t_plus, t_plus), (.5, .25, .25)):
            x, y = ax + t * (bx - ax), ay + t * (by - ay)
            nx, ny = (by - ay) * weight * depth, (ax - bx) * weight * depth
            p = 1 + x + 2 * y + x * y
            xx, xy, yy = -p + 4 * mu * (x + y), 2 * mu * (x - y), -p - 4 * mu * (x + y)
            for key, value in zip(('x', 'y', 'nx_weight', 'ny_weight', 'gx_weight', 'gy_weight'),
                                  (x, y, nx, ny, xx * nx + xy * ny, xy * nx + yy * ny)):
                r[key] = render(value)
            for i, node in sorted(active.items()):
                phi = max(0., 1 - abs((x - number(node['x'])) / h)) * max(0., 1 - abs((y - number(node['y'])) / h))
                if phi != 0:
                    basis.append({'q': r['q'], 'index': str(i), 'phi': render(phi)})
    save_csv(case / 'traction-points.csv', pf, points)
    save_csv(case / 'traction-basis.csv', bf, basis)
    rewrite_rhs(case, boundary=boundary_load_from_rows(case, points, basis))


def wrong_phase(case):
    fields, rows, _ = input_rows(case)
    for r in rows:
        if r['key'] == 'phase':
            require(r['value'] == 'origin', 'chosen origin-phase source')
            r['value'] = 'quarter'
    save_csv(case / 'input.csv', fields, rows)


def wrong_source(case):
    fields, rows = load_csv(case / 'operators' / 'source.csv')
    rows[0]['x'] = render(represented(rows[0]['x']) + F(1, 1024))
    save_csv(case / 'operators' / 'source.csv', fields, rows)
    fields, meta, _ = input_rows(case)
    value = number(rows[0]['x'])
    import struct  # stdlib bit identity, never an oracle/scientific import
    for r in meta:
        if r['key'] == 'vertex_0_0':
            r['value'] = render(value)
            r['bits'] = str(struct.unpack('Q', struct.pack('d', value))[0])
    save_csv(case / 'input.csv', fields, meta)


def wrong_p0_trace(case):
    fields, rows = load_csv(case / 'operators' / 'cells.csv')
    active = [r for r in rows if r['pressure'] != '']
    require(len(active) >= 2, 'two physical P0 pressure cells')
    active[0]['pressure'], active[1]['pressure'] = active[1]['pressure'], active[0]['pressure']
    save_csv(case / 'operators' / 'cells.csv', fields, rows)


def wrong_final_velocity(case):
    fields, rows = load_csv(case / 'states.csv')
    chosen = [r for r in rows if (r['method'], r['iteration'], r['kind'], r['index']) == ('selected', '3', 'u', '0')]
    require(len(chosen) == 1, 'one final chosen velocity coefficient')
    old = number(chosen[0]['value'])
    step = max(.125, 4 * math.ulp(old))
    bad = old + step
    if not math.isfinite(bad):
        bad = old - step
    require(math.isfinite(bad) and abs(F.from_float(bad) - F.from_float(old)) >= F(1, 8),
            'material representable wrong final coefficient')
    chosen[0]['value'] = render(bad)
    save_csv(case / 'states.csv', fields, rows)
    rewrite_residuals(case)


def nan_body(case):
    fields, rows = load_csv(case / 'loads.csv')
    rows[0]['body'] = 'NaN'
    save_csv(case / 'loads.csv', fields, rows)


def duplicate_load_index(case):
    fields, rows = load_csv(case / 'loads.csv')
    rows.append(dict(rows[0]))
    save_csv(case / 'loads.csv', fields, rows)


def missing_zero_matrix_entry(case):
    path = case / 'operators' / 'matrices.csv'
    fields, rows = load_csv(path)
    chosen = next((i for i, row in enumerate(rows) if represented(row['value']) == 0), None)
    require(chosen is not None, 'existing zero dense entry for targeted malformed control')
    del rows[chosen]
    save_csv(path, fields, rows)


def missing_final_iteration(case):
    path = case / 'states.csv'
    fields, rows = load_csv(path)
    rows = [row for row in rows if not (row['method'] == 'selected' and row['iteration'] == '3')]
    save_csv(path, fields, rows)


def failed_guards(report):
    gates = report.get('gates')
    require(isinstance(gates, dict) and (gates or report.get('status') == 'native-refused'),
            'frozen analyzer named mathematical gates')
    for name, gate in gates.items():
        require(isinstance(name, str) and isinstance(gate, dict) and isinstance(gate.get('pass'), bool),
                'named gate result schema')
        decoded = {}
        for key in ('value', 'limit'):
            rational = gate.get(key)
            require(isinstance(rational, dict) and set(rational) == {'numerator', 'denominator'}
                    and all(isinstance(rational[k], str) for k in rational), 'exact rational gate schema')
            require(all(len(text) <= INTEGER_DECIMAL_CHAR_LIMIT for text in rational.values()),
                    'bounded rational decimal text')
            integers = {part: int(text) for part, text in rational.items()}
            require(all(value.bit_length() <= INTEGER_BIT_LIMIT for value in integers.values()),
                    'bounded rational integer bits')
            require(integers['denominator'] > 0, 'positive rational gate denominator')
            decoded[key] = F(integers['numerator'], integers['denominator'])
        value, limit = (decoded[key] for key in ('value', 'limit'))
        require(gate['pass'] == (value <= limit), 'gate boolean matches exact value and limit')
    return [name for name, gate in gates.items() if not gate['pass']]


def main():
    # Bounded report codec policy, scoped to this explicitly invoked CLI.
    # It changes neither scientific tolerances nor the represented fractions.
    sys.set_int_max_str_digits(INTEGER_DECIMAL_CHAR_LIMIT)
    require(len(sys.argv) == 3, 'usage check_oracle.py NATIVE_ROOT FRESH_OUTPUT')
    raw = Path(sys.argv[1]).resolve()
    out = Path(sys.argv[2]).absolute()
    require(raw.is_dir() and not out.exists(), 'existing raw / fresh retained output')
    require(not out.resolve().is_relative_to(raw), 'output outside immutable raw corpus')
    require({p.name for p in raw.iterdir() if p.is_dir()} == set(CASES), 'exact nine native case directories')
    before = inventory(raw)
    analyzer = Path(__file__).with_name('analyze.py').resolve()
    require(analyzer.is_file(), 'completed separately reviewed analyzer required')
    source_freeze = {str(p): digest(p) for p in sorted(analyzer.parent.glob('*.py'))}
    out.mkdir(parents=True)
    budget_root = out / 'complete'
    budget_root.mkdir()
    output_limit = COMPLETE_OUTPUT_BYTES
    started = time.monotonic()
    deadline = started + COMPLETE_SECONDS
    complete_cpu_start = child_cpu()
    mode_cpu_start = complete_cpu_start
    control_cpu_start = None
    runs = []
    record = {'status': 'preparing', 'source_sha256': source_freeze,
              'native_before_sha256': before, 'limits': {'child_cpu_seconds': CHILD_SECONDS,
              'child_wall_seconds': CHILD_SECONDS, 'complete_mode_wall_seconds': MODE_SECONDS,
              'complete_three_mode_wall_seconds': COMPLETE_SECONDS,
              'control_total_wall_seconds': CONTROL_SECONDS,
              'address_bytes': ADDRESS_BYTES, 'complete_output_bytes': COMPLETE_OUTPUT_BYTES,
              'control_output_bytes': CONTROL_OUTPUT_BYTES, 'workers': 1},
              'runs': runs, 'resource_feasibility': 'unmeasured; hard stop on limit'}

    def retain():
        (budget_root / 'report.json').write_text(json.dumps(record, indent=2, sort_keys=True, allow_nan=False) + '\n')

    def check_stage_budget():
        check_budget(budget_root, deadline, output_limit)
        if control_cpu_start is None:
            require(child_cpu() - complete_cpu_start < COMPLETE_SECONDS, 'complete total child CPU limit')
            require(child_cpu() - mode_cpu_start < MODE_SECONDS, 'complete mode child CPU limit')
        else:
            require(child_cpu() - control_cpu_start < CONTROL_SECONDS, 'control total child CPU limit')

    def execute(tag, mode, flags, command_args, expected_guard=None, target_result=None,
                input_refusal=False, numerical_failure=False):
        check_stage_budget()
        require({str(p): digest(p) for p in sorted(analyzer.parent.glob('*.py'))} == source_freeze,
                'oracle source set and bytes unchanged during checks')
        result = target_result or budget_root / (tag + '-' + mode + '-result')
        require(result.resolve().is_relative_to(budget_root.resolve()), 'result/logs inside accounted stage root')
        require(not result.exists(), 'fresh analyzer output')
        argv = [sys.executable, *flags, str(analyzer), *map(str, command_args), str(result)]
        entry = {'tag': tag, 'mode': mode, 'argv': argv, 'expected_guard': expected_guard,
                 'status': 'starting', 'peak_observed_rss': 0}
        runs.append(entry)
        retain()
        previous_signal_mask = None
        def limits():
            resource.setrlimit(resource.RLIMIT_AS, (ADDRESS_BYTES, ADDRESS_BYTES))
            resource.setrlimit(resource.RLIMIT_CPU, (CHILD_SECONDS, CHILD_SECONDS))
            resource.setrlimit(resource.RLIMIT_FSIZE, (output_limit, output_limit))
            resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
            # The child must not carry the parent's creation-window signal mask.
            signal.pthread_sigmask(signal.SIG_SETMASK, previous_signal_mask)
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', OPENBLAS_NUM_THREADS='1',
                   OMP_NUM_THREADS='1', MKL_NUM_THREADS='1', BLIS_NUM_THREADS='1', NUMEXPR_NUM_THREADS='1')
        child = None
        child_started = time.monotonic()
        stdout = budget_root / (tag + '-' + mode + '.stdout')
        stderr = budget_root / (tag + '-' + mode + '.stderr')
        try:
            with stdout.open('xb') as so, stderr.open('xb') as se:
                previous_signal_mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT, signal.SIGTERM})
                try:
                    child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=so, stderr=se,
                                             start_new_session=True, preexec_fn=limits, env=env)
                finally:
                    # Pending signals are delivered only after child is owned.
                    signal.pthread_sigmask(signal.SIG_SETMASK, previous_signal_mask)
                while child.poll() is None:
                    check_stage_budget()
                    resident = observed_rss(child.pid)
                    entry['peak_observed_rss'] = max(entry['peak_observed_rss'], resident)
                    require(resident <= ADDRESS_BYTES, 'observed child RSS limit')
                    require(time.monotonic() - child_started < CHILD_SECONDS, 'fixed child wall limit')
                    time.sleep(.1)
                child.wait()
            check_stage_budget()
            require(time.monotonic() - child_started < CHILD_SECONDS, 'final child wall limit')
            entry.update(returncode=child.returncode, status='returned')
            report_path = result / 'analysis.json'
            if report_path.is_file():
                require(not input_refusal, 'malformed control reaches intended typed input refusal')
                report = json.loads(report_path.read_text())
                bad = failed_guards(report)
                passed = report.get('all_required_gates_pass')
                require(report.get('status') in ('numerical-pass', 'numerical-fail', 'native-refused'),
                        'declared scientific report disposition')
                require(isinstance(passed, bool) and passed == (not bad and report['status'] != 'native-refused'),
                        'scientific disposition matches named guards')
                require(child.returncode == (0 if passed else 1), 'exit agrees with mathematical disposition')
                if expected_guard is not None:
                    require(not passed and expected_guard in bad, 'intended mathematical guard rejects ' + tag)
            else:
                require(not numerical_failure, 'mathematical control retains complete negative numerical analysis')
                require(expected_guard is not None and child.returncode == 1, 'complete deterministic report required')
                lines = stderr.read_text().splitlines()
                accepted = {'CV3 input guard: ' + expected_guard,
                            'RuntimeError: CV3 input guard: ' + expected_guard}
                require(any(line in accepted for line in lines),
                        'exact named mathematical refusal, not parse/nonzero-exit alone')
                report = None
            entry['status'] = 'expected-disposition'
            retain()
            return result, report
        except BaseException as error:
            if child is not None and child.poll() is None:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait()
            entry.update(status='refused', reason=str(error))
            if child is not None:
                entry['returncode'] = child.returncode
            retain()
            raise

    try:
        retain()
        complete_reports = {}
        baseline_case = None
        original_scientific = {}
        record['original_scientific_dispositions'] = original_scientific
        for mode, flags in MODES:
            deadline = min(started + COMPLETE_SECONDS, time.monotonic() + MODE_SECONDS)
            mode_cpu_start = child_cpu()
            per_case = budget_root / ('complete-' + mode + '-cases')
            per_case.mkdir()
            for case in CASES:
                _, report = execute('complete-' + case, mode, flags, ('case', raw / case), target_result=per_case / case)
                require(report is not None and report.get('case') == case, 'full original case identity retained')
                original_scientific[mode + '/' + case] = report['all_required_gates_pass']
                if mode == 'normal' and case == MUTANT_CASE:
                    baseline_case = report
                check_budget(budget_root, deadline, output_limit)
            result, report = execute('complete-aggregate', mode, flags, ('aggregate', raw, per_case))
            original_scientific[mode + '/aggregate'] = report['all_required_gates_pass']
            complete_reports[mode] = {'aggregate': result / 'analysis.json',
                                     **{case: per_case / case / 'analysis.json' for case in CASES}}
        require(all(identical_files(complete_reports['normal'][key], complete_reports[mode][key])
                    for mode in ('O', 'OO') for key in (*CASES, 'aggregate')),
                'all nine full case and aggregate scientific reports byte-identical')
        record.update(status='complete-modes-finished', complete_modes_byte_identical=True)
        record['original_scientific_dispositions'] = original_scientific
        record['original_scientific_gates_pass'] = all(original_scientific.values())
        retain()
        budget_root = out / 'controls'
        budget_root.mkdir()
        output_limit = CONTROL_OUTPUT_BYTES
        deadline = time.monotonic() + CONTROL_SECONDS
        control_cpu_start = child_cpu()
        record['status'] = 'checking-retained-mutations'
        retain()
        recipes = (
            ('omitted-pressure-traction', 'audit', 'exact-pressure-traction', lambda c: pressure_traction(c, F(0))),
            ('wrong-pressure-traction', 'audit', 'exact-pressure-traction', lambda c: pressure_traction(c, F(5, 4))),
            ('reversed-pressure-traction', 'audit', 'exact-pressure-traction', lambda c: pressure_traction(c, F(-1))),
            ('omitted-body-force', 'audit', 'exact-body-load', lambda c: rewrite_rhs(c, body_factor=F(0))),
            ('wrong-body-force', 'audit', 'exact-body-load', lambda c: rewrite_rhs(c, body_factor=F(5, 4))),
            ('wrong-phase', 'audit', 'phase-identity', wrong_phase),
            ('wrong-source', 'audit', 'source-identity', wrong_source),
            ('wrong-P0-trace', 'audit', 'pressure-trace-binding', wrong_p0_trace),
            ('degree3-oblique-rule', 'audit', 'traction-degree4-moments', two_point_degree4),
            ('self-consistent-wrong-final-u', 'case', 'solution-R1-velocity', wrong_final_velocity),
        )
        malformed = (
            ('NaN-body-field', 'audit', 'nonfinite field', nan_body),
            ('duplicate-load-index', 'audit', 'duplicate index', duplicate_load_index),
            ('missing-zero-dense-entry', 'audit', 'complete dense blocks including zeros', missing_zero_matrix_entry),
            ('missing-final-state-iteration', 'audit', 'all four selected states', missing_final_iteration),
        )
        identity_guards = {'phase-identity', 'source-identity', 'pressure-trace-binding'}
        require(baseline_case is not None and baseline_case['status'] != 'native-refused',
                'complete structurally valid original mutant-case report is required')
        for tag, kind, guard, mutate in recipes + malformed:
            is_malformed = (tag, kind, guard, mutate) in malformed
            if not is_malformed and guard not in identity_guards:
                require(guard in baseline_case['gates'] and baseline_case['gates'][guard]['pass'],
                        'untouched case target guard must pass before mutation: ' + guard)
            fixture = copied_case(raw, budget_root, tag, deadline)
            mutate(fixture)
            after = inventory(fixture)
            reference = {str(p.relative_to(raw / MUTANT_CASE)): digest(p)
                         for p in sorted((raw / MUTANT_CASE).rglob('*')) if p.is_file()}
            require(after != reference, 'mutation changed independent copied bytes')
            (budget_root / (tag + '-fixture-sha256.json')).write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
            for mode, flags in MODES:
                execute(tag, mode, flags, (kind, fixture), guard, input_refusal=is_malformed,
                        numerical_failure=not is_malformed and guard not in identity_guards)
            require(inventory(fixture) == after, 'analyzer leaves retained mutant unchanged')
        record.update(status='completed', all_tooling_contracts_pass=True,
                      complete_modes_byte_identical=True, mutation_fixtures=len(recipes),
                      malformed_fixtures=len(malformed), subprocess_checks=len(runs),
                      scientific_scope='fixed nine cases only')
        require(len(runs) == 72, 'fixed 30 complete plus 42 control subprocess dispositions')
    except BaseException as error:
        record.update(status='refused', reason=str(error), all_tooling_contracts_pass=False)
        raise
    finally:
        try:
            after = inventory(raw)
        except BaseException as error:
            record.update(original_corpus_unchanged=False, original_corpus_inventory_error=str(error),
                          status='refused', all_tooling_contracts_pass=False)
            retain()
            raise
        record['native_after_sha256'] = after
        record['original_corpus_unchanged'] = after == before
        if after != before:
            record.update(status='refused', all_tooling_contracts_pass=False)
        retain()
        require(after == before, 'all original raw file hashes unchanged after checks/refusal')
    print(json.dumps({'all_tooling_contracts_pass': True,
                      'original_scientific_gates_pass': record['original_scientific_gates_pass'],
                      'subprocess_checks': len(runs)}, sort_keys=True))


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise InterruptedError('CV3 harness interrupted by SIGTERM')

    previous_sigterm = signal.signal(signal.SIGTERM, interrupted)
    try:
        try:
            main()
        except Exception as error:
            print(str(error), file=sys.stderr)
            sys.exit(1)
    finally:
        signal.signal(signal.SIGTERM, previous_sigterm)
