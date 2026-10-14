"""Publish the source-bound, stored three-dimensional Euler teaching fixtures."""
from pathlib import Path
import hashlib
import json
import math
import shutil
import subprocess
import sys

SOURCE_ROOT = 'docs/research-book/compressible'
SOURCE_FILES = ('reference.py', 'oracles.py', 'generate.py', 'test_reference.py')
PRESENTATION_SCHEMA = 'rheon-compressible-presentation-v1'


def require(condition, message):
    if not condition:
        raise RuntimeError('Compressible fixture: ' + message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_hashes(repo):
    return {name: sha(Path(repo) / SOURCE_ROOT / name) for name in SOURCE_FILES}


def _validate_packet(repo, directory, filename="results.json"):
    """Reject absent provenance, stale source, and invalid field/metric shapes."""
    if directory is None:
        return None
    path = Path(directory) / filename
    require(path.is_file() and not path.is_symlink(), 'results.json must be a regular file')
    packet = json.loads(path.read_text())
    require(type(packet.get('schema_version')) is int and packet['schema_version'] == 1, 'unsupported results schema')
    require(packet.get('provenance', {}).get('source_sha256') == source_hashes(repo), 'stale numerical source binding')
    require(type(packet.get('cases')) is list and len(packet['cases']) == 2, 'expected two qualified cases')
    require({c['id'] for c in packet['cases']} == {'planar_sod_periodic', 'oblique_acoustic'}, 'case identity')
    for case in packet['cases']:
        grid = case['grid']
        require(len(grid['shape']) == 3 and all(type(n) is int and n > 1 for n in grid['shape']), 'three-dimensional grid')
        require(grid['centering'] == 'cell' and grid['flatten_order'] == 'C (z fastest)', 'field layout')
        require(len(grid['extent']) == 3 and all(len(e) == 2 and e[1] > e[0] for e in grid['extent']), 'grid extent')
        require(grid['shape'] == [12, 12, 12] and grid['extent'] == [[0, 1], [0, 1], [0, 1]], 'qualified fixture grid')
        params = case['parameters']
        require(params['gamma'] == 1.4 and params['cfl'] == 0.25 and params['boundary'] == 'periodic' and params['seed'] == 0, 'qualified physical parameters')
        require(math.isfinite(params['dimensionless_end_time']) and params['dimensionless_end_time'] == 0.08, 'qualified end time')
        count = math.prod(grid['shape'])
        require(len(case['frames']) == 7, 'seven saved evolution frames')
        require(case['frames'][0]['time'] == 0 and all(a['time'] < b['time'] for a, b in zip(case['frames'], case['frames'][1:])), 'saved time order')
        initial_totals = None
        ranges = {'density': [], 'pressure': [], 'speed': []}
        for frame in case['frames']:
            require(type(frame['time']) in (int, float) and math.isfinite(frame['time']) and 0 <= frame['time'] <= 0.08, 'finite saved time')
            for name in ('density', 'pressure'):
                values = frame[name]
                require(len(values) == count and all(type(v) in (int, float) and math.isfinite(v) and v > 0 for v in values), name + ' field')
            require(len(frame['velocity']) == count and all(len(v) == 3 and all(type(x) in (int, float) and math.isfinite(x) for x in v) for v in frame['velocity']), 'three-component velocity')
            diagnostics = frame['diagnostics']
            for name in ('mass', 'energy', 'min_density', 'min_pressure', 'relative_conservation_drift'):
                require(type(diagnostics[name]) in (int, float) and math.isfinite(diagnostics[name]), 'finite ' + name)
            require(diagnostics['min_density'] == min(frame['density']) and diagnostics['min_pressure'] == min(frame['pressure']), 'minimum readouts')
            require(len(diagnostics['momentum']) == 3 and all(type(v) in (int, float) and math.isfinite(v) for v in diagnostics['momentum']), 'momentum readout')
            cell_volume = math.prod((e[1] - e[0]) / n for e, n in zip(grid['extent'], grid['shape']))
            require(len(frame['conserved']) == count and all(len(v) == 5 and all(type(x) in (int, float) and math.isfinite(x) for x in v) for v in frame['conserved']), 'conserved field shape')
            def close(a, b, label):
                require(math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-13), label)
            for rho, p, u, state in zip(frame['density'], frame['pressure'], frame['velocity'], frame['conserved']):
                close(state[0], rho, 'conserved density')
                for d in range(3):
                    close(state[d + 1], rho * u[d], 'conserved momentum')
                close(state[4], p / (params['gamma'] - 1) + 0.5 * rho * sum(v * v for v in u), 'conserved energy')
            totals = [math.fsum(state[d] for state in frame['conserved']) * cell_volume for d in range(5)]
            require(len(diagnostics['conserved_totals']) == 5, 'total vector shape')
            for a, b in zip(totals, diagnostics['conserved_totals']):
                close(a, b, 'conservative total')
            close(totals[0], diagnostics['mass'], 'mass ledger')
            close(totals[4], diagnostics['energy'], 'energy ledger')
            for a, b in zip(totals[1:4], diagnostics['momentum']):
                close(a, b, 'momentum ledger')
            if initial_totals is None:
                initial_totals = totals
            drift = max(abs(a - b) / max(abs(b), 1) for a, b in zip(totals, initial_totals))
            close(drift, diagnostics['relative_conservation_drift'], 'drift ledger')
            require(0 <= diagnostics['relative_conservation_drift'] <= 2e-13, 'periodic conservation limit')
            require(type(diagnostics['steps']) is int and diagnostics['steps'] >= 0, 'recorded step count')
            require(type(diagnostics['density_l1_reference_error']) in (int, float) and math.isfinite(diagnostics['density_l1_reference_error']) and diagnostics['density_l1_reference_error'] >= 0, 'reference error')
            speeds = [math.hypot(*v) for v in frame['velocity']]
            close(max(speeds), diagnostics['max_speed'], 'speed maximum')
            for name, values in (('density', frame['density']), ('pressure', frame['pressure']), ('speed', speeds)):
                ranges[name].extend((min(values), max(values)))
        for name, values in ranges.items():
            require(len(case['field_ranges'][name]) == 2, 'range shape')
            for a, b in zip(case['field_ranges'][name], (min(values), max(values))):
                require(math.isfinite(a) and math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-14), 'field range ' + name)
    require(packet['qualification'].get('status') == 'satisfied', 'missing numerical qualification')
    operation = subprocess.run([sys.executable, '-c', """import json,sys
import numpy as np
from generate import validate_results
from oracles import acoustic_density, sod_exact
packet = json.load(sys.stdin)
validate_results(packet)
for case in packet['cases']:
    shape = tuple(case['grid']['shape'])
    for frame in case['frames']:
        density = np.asarray(frame['density']).reshape(shape)
        if case['id'] == 'planar_sod_periodic':
            x = (np.arange(shape[0]) + .5) / shape[0]
            expected, _, _ = sod_exact(x, frame['time'])
            interior = (x > .2) & (x < .8)
            error = np.mean(np.abs(density[interior,0,0] - expected[interior]))
        else:
            expected = acoustic_density(shape, frame['time'], .02, case['parameters']['gamma'])
            error = np.mean(np.abs(density - expected))
        if not np.isclose(error, frame['diagnostics']['density_l1_reference_error'], rtol=1e-12, atol=1e-14):
            raise ValueError('density reference error mismatch')
"""],
                               cwd=Path(repo) / SOURCE_ROOT, input=json.dumps(packet, allow_nan=False), text=True, capture_output=True)
    require(operation.returncode == 0, 'numerical producer verification failed: ' + operation.stderr[-1200:])
    return {'schema': PRESENTATION_SCHEMA, 'records_sha256': sha(path), 'sources': source_hashes(repo), 'qualification': packet.get('qualification', {})}


def validate_packet(repo, directory, filename="results.json"):
    try:
        return _validate_packet(repo, directory, filename)
    except (KeyError, TypeError, ValueError, OverflowError) as error:
        raise RuntimeError('Compressible fixture: missing or invalid packet field: ' + str(error)) from error


def render_lab(included):
    if not included:
        return '<p class="eyebrow">COMPRESSIBLE FLOW / STORED FIXTURES</p><h1>See pressure carry a wave.</h1><p>Qualified compressible fields are absent in this source-only edition.</p><p><a href="chapters/26-compressible-state-and-waves.html">Read the compressible state and wave chapter →</a></p>'
    return '''<style>
.compressible-workbench{display:grid;grid-template-columns:minmax(0,1.65fr) minmax(220px,1fr);border:1px solid var(--line);background:white;border-radius:8px;overflow:hidden;margin:24px 0}.compressible-visual{min-width:0;background:#edf3f2;padding:16px}.compressible-visual h2{font-size:23px;margin:0 0 6px}#compressible-scene{height:440px;min-width:0;position:relative}#compressible-scene canvas,#compressible-fallback{width:100%;height:100%;display:block}.compressible-panel{min-width:0;padding:20px;font-size:14px}.compressible-panel label{display:block;margin-top:12px;font-weight:600}.compressible-panel select,.compressible-panel input{width:100%}.compressible-panel input{accent-color:var(--teal)}.compressible-buttons{display:flex;gap:8px;margin-top:14px;flex-wrap:wrap}.compressible-legend{display:flex;align-items:center;justify-content:space-between;gap:10px;font-size:12px}.compressible-ramp{height:10px;flex:1;background:linear-gradient(90deg,#2348a1,#24abb7,#edc459,#d44435);border-radius:4px}.compressible-note{font-size:13px;color:var(--muted)}.compressible-ledger{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}.compressible-ledger article{padding:14px;border:1px solid var(--line);background:white;min-width:0}.compressible-ledger span{display:block;font-size:11px;color:var(--muted)}.compressible-ledger strong{font-size:18px;font-variant-numeric:tabular-nums}.compressible-inspection{display:grid;grid-template-columns:1fr 1fr;gap:25px}.compressible-inspection h2{font-size:25px;margin-top:28px}#compressible-probe dd{overflow-wrap:anywhere}#compressible-status{min-height:3em}#compressible-error{font-weight:600}.compressible-links{overflow-wrap:anywhere}@media(max-width:1100px){.compressible-workbench{grid-template-columns:1fr}#compressible-scene{height:390px}.compressible-ledger{grid-template-columns:repeat(2,minmax(0,1fr))}}@media(max-width:760px){#compressible-scene{height:300px}.compressible-inspection{grid-template-columns:1fr}.compressible-panel{padding:16px}.compressible-ledger strong{font-size:15px}}
</style><p class="eyebrow">COMPRESSIBLE FLOW / COMPUTED RECORDS</p>
<h1>See pressure carry a wave.</h1>
<p class="lede">Explore two small, saved three-dimensional Euler experiments: a planar shock tube and acoustic waves crossing through the volume.</p>
<p role="note"><strong>Playback of computed fixtures.</strong> The controls select saved fields and times from a periodic, first-order local Lax–Friedrichs finite-volume reference. This teaching viewer does not run a fluid solver. The shock tube is extruded; the acoustic case varies along all three axes.</p>
<section class="compressible-workbench" aria-label="Saved compressible field viewer">
<div class="compressible-visual"><h2 id="compressible-title">Stored Euler fields</h2><p class="compressible-note" id="compressible-grid"></p>
<div id="compressible-scene" tabindex="0" aria-label="Three-dimensional saved field. Drag to rotate, scroll to zoom, or use arrow keys to orbit and plus and minus to zoom."></div>
<div class="compressible-legend"><output id="compressible-low"></output><span class="compressible-ramp" aria-hidden="true"></span><output id="compressible-high"></output></div><p class="compressible-note" id="compressible-caption"></p></div>
<div class="compressible-panel">
<label for="compressible-case">Computed case</label><select id="compressible-case"></select>
<label for="compressible-frame">Saved time <output id="compressible-time" for="compressible-frame"></output></label><input id="compressible-frame" type="range" min="0" max="1" step="1" value="0">
<label for="compressible-scalar">Color field</label><select id="compressible-scalar"><option value="density">Density ρ</option><option value="pressure">Pressure p</option><option value="speed">Speed |u|</option></select>
<label for="compressible-mode">Display</label><select id="compressible-mode"><option value="slice">One cell-center slice</option><option value="volume">Full voxel field</option></select>
<label for="compressible-axis">Slice normal</label><select id="compressible-axis"><option value="0">x</option><option value="1">y</option><option value="2" selected>z</option></select>
<label for="compressible-slice">Slice index <output id="compressible-slice-value" for="compressible-slice"></output></label><input id="compressible-slice" type="range" min="0" max="11" step="1" value="6">
<label for="compressible-cell">Inspect cell <output id="compressible-cell-value" for="compressible-cell"></output></label><input id="compressible-cell" type="range" min="0" max="1727" step="1" value="942">
<div class="compressible-buttons"><button id="compressible-play" type="button" aria-pressed="false">Play saved frames</button><button id="compressible-reset" type="button">Reset camera</button></div>
<p class="compressible-note">Camera: drag to orbit · scroll to zoom · focus scene for arrow keys and + / −. Playback stops at the last saved time.</p><p id="compressible-status" role="status">Loading source-bound computed fields…</p>
</div></section>
<section class="compressible-ledger" aria-label="Stored conservative ledger" aria-live="polite">
<article><span>TOTAL MASS</span><strong id="compressible-mass">—</strong></article><article><span>TOTAL ENERGY</span><strong id="compressible-energy">—</strong></article><article><span>MINIMUM DENSITY</span><strong id="compressible-min_density">—</strong></article><article><span>MINIMUM PRESSURE</span><strong id="compressible-min_pressure">—</strong></article><article><span>RELATIVE CONSERVATION DRIFT</span><strong id="compressible-relative_conservation_drift">—</strong></article><article><span>MOMENTUM x</span><strong id="compressible-momentum-x">—</strong></article><article><span>MOMENTUM y</span><strong id="compressible-momentum-y">—</strong></article><article><span>MOMENTUM z</span><strong id="compressible-momentum-z">—</strong></article>
</section>
<div class="compressible-inspection"><section><h2>Inspect the stored state.</h2><dl id="compressible-probe"></dl><p class="compressible-note">The conserved state is (ρ, ρuₓ, ρuᵧ, ρu_z, E). Pressure follows the ideal-gas equation of state. All values use nondimensional reference units.</p></section><section><h2>What was checked?</h2><p id="compressible-error"></p><p id="compressible-qualification" class="compressible-note"></p><p class="compressible-note">Periodic fluxes conserve the totals to measured floating-point error. Positive minima describe these retained runs; they are not a general positivity guarantee. Finite grid and time refinement checks do not certify an arbitrary Euler calculation.</p></section></div>
<p class="compressible-links"><a href="chapters/26-compressible-state-and-waves.html">26 · State and waves</a> · <a href="chapters/27-compressible-discretization.html">27 · Finite-volume discretization</a> · <a href="chapters/28-compressible-boundaries-and-validation.html">28 · Boundaries and validation</a></p>
<details><summary>Computed fields, source and limits</summary><p>The reference advances inviscid ideal-gas Euler equations on a Cartesian periodic grid. No pressure projection, obstacle, viscosity, surface tension, interface tracking or production compressible API is exercised here. Full volume view draws transparent cell boxes; a slice gives unoccluded cell values.</p><p id="compressible-provenance" class="compressible-note"></p><p><a href="compressible-records.json" download>Download saved fields and numerical qualification</a> · <a href="compressible-presentation.json">Publication source binding</a> · <a href="compressible-source/reference.py" download>Reference solver</a> · <a href="compressible-source/oracles.py" download>Independent oracles</a> · <a href="compressible-source/generate.py" download>Fixture generator</a> · <a href="compressible-source/test_reference.py" download>Numerical tests</a></p></details>'''


def publish(repo, output, directory=None):
    receipt = validate_packet(repo, directory)
    output = Path(output)
    presentation = {'schema': PRESENTATION_SCHEMA, 'included': receipt is not None, 'qualification': receipt, 'browser_fluid_solves': 0}
    (output / 'compressible-presentation.json').write_text(json.dumps(presentation, indent=2, allow_nan=False) + '\n')
    if receipt is not None:
        shutil.copy2(Path(directory) / 'results.json', output / 'compressible-records.json')
        shutil.copy2(Path(repo) / 'docs/education/compressible.js', output / 'compressible.js')
        sources = output / 'compressible-source'
        sources.mkdir(exist_ok=True)
        for name in SOURCE_FILES:
            shutil.copy2(Path(repo) / SOURCE_ROOT / name, sources / name)
    return render_lab(receipt is not None)


def verify_published(repo, output):
    output = Path(output)
    presentation = json.loads((output / 'compressible-presentation.json').read_text())
    require(presentation.get('schema') == PRESENTATION_SCHEMA and presentation.get('browser_fluid_solves') == 0, 'publication scope')
    require(type(presentation.get('included')) is bool, 'inclusion flag')
    if not presentation['included']:
        require(presentation.get('qualification') is None and not (output / 'compressible-records.json').exists(), 'source-only publication fields')
        return None
    receipt = presentation['qualification']
    require(validate_packet(repo, output, 'compressible-records.json') == receipt, 'published numerical revalidation')
    require(receipt['sources'] == source_hashes(repo), 'published numerical source identity')
    require(receipt['records_sha256'] == sha(output / 'compressible-records.json'), 'published records binding')
    packet = json.loads((output / 'compressible-records.json').read_text())
    require(packet['provenance']['source_sha256'] == receipt['sources'] and packet.get('qualification', {}) == receipt['qualification'], 'published packet provenance')
    require(sha(output / 'compressible.js') == sha(Path(repo) / 'docs/education/compressible.js'), 'published viewer source')
    for name in SOURCE_FILES:
        require(sha(output / 'compressible-source' / name) == receipt['sources'][name], 'published source ' + name)
    return receipt
