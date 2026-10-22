#!/usr/bin/env python3
"""Render descriptive tables from analyze_matrix.py's derived summary.
Usage: write_tables.py summary.json output.md
"""
import json
from pathlib import Path
import statistics
import sys

summary = json.loads(Path(sys.argv[1]).read_text())
if summary['violations'] or not summary.get('qualified'):
    raise SystemExit('refusing to render unqualified or incomplete final tables')
lines = [
    '# Corrected CPU measurement tables',
    '',
    'Six balanced permutations per case. Seconds are medians across runs; percent',
    'changes are medians of paired ratios with the full observed range. No ratios',
    'are admitted for incomplete horizons. Base/cold/warm are distinct variants.',
    'Dataset qualification is distinct from the performance admission decision.',
    'Full cost includes the native liquid step and borrowed-field inspection,',
    'excluding allocation, process/pool startup, rendering and export.',
    '',
    '| Case | n | Accepted base/cold/warm | Base s | Cold s | Warm s | Warm/base % [range] | Cold/base % [range] |',
    '|---|---:|---|---:|---:|---:|---|---|',
]

def median(case, metric, variant):
    return statistics.median(t[metric][variant] for t in case['trials'])

def percent(case, metric):
    value = case.get(metric)
    if value is None:
        return '—'
    return f"{value['median']:+.2f} [{value['min']:+.2f}, {value['max']:+.2f}]"

for name, case in summary['cases'].items():
    accepted = '/'.join(','.join(map(str, sorted({t['accepted'][v] for t in case['trials']}))) for v in ['base','cold','warm'])
    seconds = [median(case, 'full_s', v) for v in ['base','cold','warm']]
    lines.append(f"| {name} | {case['repetitions']} | {accepted} | "
                 + ' | '.join(f'{x:.6f}' for x in seconds)
                 + f" | {percent(case, 'warm_vs_base_pct')} | {percent(case, 'cold_vs_base_pct')} |")

lines += [
    '', '## Phase, iterations and memory', '',
    'Pressure phase includes pressure correction. Inspection scans/hashes fields;',
    'it does not copy a preview or serialize an export. Iterations are reported',
    'only for complete horizons; a failed attempt has no available iteration count.',
    'Managed bytes are array capacity accounting. RSS is the maximum per-process',
    'peak across the six runs, in KiB; it is not a whole-process memory-cap promise.',
    '',
    '| Case | Pressure base/warm s | Inspection base/warm s | Iterations base/warm | Managed bytes | Peak RSS base/cold/warm KiB |',
    '|---|---|---|---|---:|---|',
]
for name, case in summary['cases'].items():
    pressure = '/'.join(f'{median(case,"pressure_phase_s",v):.6f}' for v in ['base','warm'])
    capture = '/'.join(f'{median(case,"capture_s",v):.6f}' for v in ['base','warm'])
    complete = all(all(n == 16 for n in t['accepted'].values()) for t in case['trials'])
    iterations = '/'.join(f'{median(case,"iterations",v):.0f}' for v in ['base','warm']) if complete else '—'
    managed = int(median(case, 'managed_bytes', 'cold'))
    rss = '/'.join(str(max(t['rss_kib'][v] for t in case['trials'])) for v in ['base','cold','warm'])
    if not complete:
        pressure = '—'
    lines.append(f'| {name} | {pressure} | {capture} | {iterations} | {managed} | {rss} |')

lines += [
    '', '## Allocation, initial and sustained cost', '',
    'Allocation is owner construction inside an existing worker pool; process/pool',
    'startup is excluded. Step0 and sustained steps include inspection. Incomplete',
    'horizons are shown as attempt costs in the first table, not as sustained gains.',
    '',
    '| Case | Allocation base/cold/warm s | Step0 base/cold/warm s | Steps1–15 base/cold/warm s |',
    '|---|---|---|---|',
]
for name, case in summary['cases'].items():
    complete = all(all(n == 16 for n in t['accepted'].values()) for t in case['trials'])
    values = []
    for metric in ['startup_s','step0_s','sustained_s']:
        values.append('/'.join(f'{median(case,metric,v):.6f}' for v in ['base','cold','warm'])
                      if metric != 'sustained_s' or complete else '—')
    lines.append(f'| {name} | ' + ' | '.join(values) + ' |')

lines += ['', f"Analyzer violations: {len(summary['violations'])}.", '']
Path(sys.argv[2]).write_text('\n'.join(lines))
