"""Real-browser playback, stored-field, mesh, camera and mobile evidence."""
from pathlib import Path
import hashlib
import json
import math
import re

from compressible_lab import require


SCHEMA = 'rheon-compressible-browser-v1'
COUNTS = ('dynamic_states', 'field_comparisons', 'metric_comparisons', 'camera_checks')


def validate_receipt(receipt, published):
    require(type(receipt) is dict and receipt.get('schema') == SCHEMA, 'browser receipt schema')
    included = published is not None
    require(receipt.get('included') is included, 'browser inclusion binding')
    require(receipt.get('browser_fluid_solves') == 0, 'browser solve claim')
    for key in COUNTS:
        require(type(receipt.get(key)) is int and (receipt[key] > 0 if included else receipt[key] == 0), 'browser observed ' + key)
    require(receipt.get('mobile_no_overflow') is True, 'mobile qualification')
    if included:
        require(receipt.get('records_sha256') == published['records_sha256'], 'browser field source binding')
        require(receipt.get('source_sha256') == published['sources'], 'browser numerical source binding')
        require(receipt.get('webgl') is True and receipt.get('fallback_checked') is True and receipt.get('playback_checked') is True and receipt.get('distinct_case_field_rasters') is True, 'browser render/playback checks')
        require(receipt['dynamic_states'] >= 84 and receipt['field_comparisons'] >= 84 * 144 and receipt['metric_comparisons'] >= 84 * 9 and receipt['camera_checks'] >= 4, 'incomplete observed browser evidence')
    else:
        require(receipt.get('records_sha256') is None and receipt.get('source_sha256') is None and receipt.get('screenshots') == [], 'source-only browser invented evidence')
        require(not receipt.get('webgl') and not receipt.get('fallback_checked') and not receipt.get('playback_checked') and not receipt.get('distinct_case_field_rasters'), 'source-only browser invented checks')
    return receipt


def close(actual, expected, label, allowance=1e-12, absolute=None):
    require(type(actual) in (int, float) and math.isfinite(actual), 'browser finite ' + label)
    require(math.isclose(actual, expected, rel_tol=allowance, abs_tol=allowance if absolute is None else absolute), 'browser mismatch ' + label)


def coordinates(index, shape):
    return [index // (shape[1] * shape[2]), (index // shape[2]) % shape[1], index % shape[2]]


def linear_channel(channel):
    value = channel / 255
    return value / 12.92 if value < 0.04045 else ((value + 0.055) / 1.055) ** 2.4


STOPS = [[linear_channel(v) for v in rgb] for rgb in ((35,72,161),(36,171,183),(237,196,89),(212,68,53))]


def inspect(page, packet):
    observation = page.evaluate("""() => ({
        snapshot: window.__compressible.snapshot(),
        metrics: Object.fromEntries(['mass','energy','min_density','min_pressure','relative_conservation_drift','momentum-x','momentum-y','momentum-z','error'].map(key => {
            const node = document.getElementById('compressible-' + key);
            return [key, {value: node.dataset.value, text: node.textContent}];
        })),
        probe: Array.from(document.querySelectorAll('#compressible-probe dd'), node => node.textContent),
        cells: document.getElementById('compressible-scene').dataset.cells
    })""")
    snapshot = observation['snapshot']
    case = next(c for c in packet['cases'] if c['id'] == snapshot['caseId'])
    frame = case['frames'][snapshot['frame']]
    shape, extent = case['grid']['shape'], case['grid']['extent']
    expected_indices = [i for i in range(math.prod(shape)) if snapshot['mode'] == 'volume' or coordinates(i, shape)[snapshot['axis']] == snapshot['slice']]
    require(snapshot['indices'] == expected_indices, 'browser displayed cell selection')
    values = [math.hypot(*v) for v in frame['velocity']] if snapshot['scalar'] == 'speed' else frame[snapshot['scalar']]
    require(len(snapshot['values']) == len(expected_indices) and len(snapshot['renderedPositions']) == len(expected_indices), 'browser field shape')
    lo, hi = case['field_ranges'][snapshot['scalar']]
    for position, (index, actual) in enumerate(zip(expected_indices, snapshot['values'])):
        close(actual, values[index], 'stored field')
        q = coordinates(index, shape)
        for d, coordinate in enumerate(snapshot['renderedPositions'][position]):
            close(coordinate, extent[d][0] + (q[d] + .5) * (extent[d][1] - extent[d][0]) / shape[d], 'rendered cell center', 1e-6)
        if snapshot['renderMode'] == 'webgl':
            scaled = (.5 if hi == lo else min(1, max(0, (values[index] - lo) / (hi - lo)))) * 3
            low = min(2, int(scaled))
            for d, channel in enumerate(snapshot['renderedColors'][position]):
                expected = STOPS[low][d] + (STOPS[low+1][d] - STOPS[low][d]) * (scaled - low)
                close(channel, expected, 'rendered scalar color', 1e-6)
    metric_values = {key: frame['diagnostics'][key] for key in ('mass','energy','min_density','min_pressure','relative_conservation_drift')}
    metric_values.update({'momentum-' + axis: value for axis, value in zip('xyz', frame['diagnostics']['momentum'])})
    metric_values['error'] = frame['diagnostics']['density_l1_reference_error']
    for key, value in metric_values.items():
        metric = observation['metrics'][key]
        require(float(metric['value']) == value, 'displayed ledger round-trip ' + key)
        match = re.search(r'[-+]?(?:[0-9]*\.)?[0-9]+(?:e[-+]?[0-9]+)?', metric['text'].split(': ')[-1], re.I)
        require(match is not None, 'visible number ' + key)
        close(float(match.group()), value, 'visible metric ' + key, 1e-5, 1e-30)
    probe = observation['probe']
    cell = snapshot['cell']
    require(probe[0] == ', '.join(map(str, coordinates(cell, shape))), 'browser inspector cell')
    for actual, expected in zip(probe[4].split(', '), frame['velocity'][cell]):
        close(float(actual), expected, 'visible velocity', 1e-5, 1e-30)
    for actual, expected in zip(probe[5].split(', '), frame['conserved'][cell]):
        close(float(actual), expected, 'visible conservative state', 1e-5, 1e-30)
    require(observation['cells'] == str(len(expected_indices)), 'browser scene field count')
    return snapshot, len(expected_indices), len(metric_values)


def qualify(page, mobile, load, check_errors, packet_dir=None, artifact_dir=None, repo=None):
    load(page, 'compressible-lab.html')
    presentation = page.evaluate("fetch('compressible-presentation.json').then(r=>r.json())")
    receipt = {'schema': SCHEMA, 'included': presentation['included'], 'browser_fluid_solves': 0,
               **{key: 0 for key in COUNTS}, 'mobile_no_overflow': True, 'screenshots': [],
               'webgl': False, 'fallback_checked': False, 'playback_checked': False, 'distinct_case_field_rasters': False,
               'records_sha256': None, 'source_sha256': None}
    if not presentation['included']:
        require(page.locator('#compressible-case').count() == 0 and 'fields are absent' in page.locator('main').inner_text(), 'source-only viewer notice')
        load(mobile, 'compressible-lab.html')
        require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth + 1'), 'source-only mobile overflow')
        check_errors()
        return validate_receipt(receipt, None)
    page.wait_for_function('window.__compressible !== undefined')
    # The packet is loaded independently of the viewer's closure and mesh.
    text = page.evaluate("fetch('compressible-records.json').then(r=>r.text())")
    packet = json.loads(text)
    require(hashlib.sha256(text.encode()).hexdigest() == presentation['qualification']['records_sha256'], 'browser downloaded field binding')
    if packet_dir is not None:
        require(packet == json.loads((Path(packet_dir)/'results.json').read_text()), 'browser independent packet file')
    receipt['records_sha256'] = presentation['qualification']['records_sha256']
    receipt['source_sha256'] = packet['provenance']['source_sha256']
    def check(target=page, **expected):
        snapshot, fields, metrics = inspect(target, packet)
        for key, value in expected.items():
            require(snapshot[key] == value, 'unresponsive browser control ' + key)
        receipt['dynamic_states'] += 1; receipt['field_comparisons'] += fields; receipt['metric_comparisons'] += metrics
        check_errors()
        return snapshot
    def picture(name, target=page):
        directory = Path(artifact_dir) if artifact_dir is not None else Path('/tmp/rheon-compressible-browser')
        if repo is not None:
            require(not directory.resolve().is_relative_to(Path(repo).resolve()), 'pixel evidence must be external')
        directory.mkdir(parents=True, exist_ok=True)
        path = directory / name
        target.screenshot(path=str(path), full_page=True, type='jpeg', quality=85)
        receipt['screenshots'].append(str(path))
    require(check()['renderMode'] == 'webgl' and page.locator('#compressible-scene canvas').count() == 1, 'real WebGL viewer')
    receipt['webgl'] = True
    raster_hashes = []
    for case in packet['cases']:
        page.select_option('#compressible-case', case['id'])
        for frame_id, frame in enumerate(case['frames']):
            page.locator('#compressible-frame').fill(str(frame_id)); page.locator('#compressible-frame').dispatch_event('input')
            for scalar in ('density', 'pressure', 'speed'):
                page.select_option('#compressible-scalar', scalar)
                for axis in (0, 2):
                    page.select_option('#compressible-axis', str(axis))
                    page.locator('#compressible-slice').fill(str((frame_id + axis) % case['grid']['shape'][axis])); page.locator('#compressible-slice').dispatch_event('input')
                    check(caseId=case['id'], frame=frame_id, scalar=scalar, axis=axis, slice=(frame_id + axis) % case['grid']['shape'][axis])
        page.select_option('#compressible-mode', 'volume'); check(mode='volume', caseId=case['id']); picture(case['id'] + '-volume.jpg')
        raster_hashes.append(hashlib.sha256(page.locator('#compressible-scene canvas').screenshot()).hexdigest())
        page.select_option('#compressible-mode', 'slice')
    require(len(set(raster_hashes)) == 2, 'case change did not alter rendered field pixels')
    receipt['distinct_case_field_rasters'] = True
    page.select_option('#compressible-case', 'oblique_acoustic'); page.select_option('#compressible-scalar', 'density')
    page.select_option('#compressible-axis', '1'); page.locator('#compressible-cell').fill('37'); page.locator('#compressible-cell').dispatch_event('input'); check()
    before = check()['camera']; bounds = page.locator('#compressible-scene').bounding_box()
    x, y = bounds['x'] + bounds['width']*.45, bounds['y'] + bounds['height']*.5
    page.mouse.move(x, y); page.mouse.down(); page.mouse.move(x+65, y+35, steps=6); page.mouse.up(); page.wait_for_timeout(120)
    require(check()['camera'] != before, 'pointer camera orbit'); receipt['camera_checks'] += 1
    page.locator('#compressible-scene').focus(); before = check()['camera']; page.keyboard.press('ArrowRight')
    require(check()['camera'] != before, 'keyboard camera orbit'); receipt['camera_checks'] += 1
    before = check()['camera']; page.locator('#compressible-reset').click()
    require(check()['camera'] != before, 'camera reset'); receipt['camera_checks'] += 1
    page.set_viewport_size({'width': 1180, 'height': 900}); page.wait_for_timeout(100)
    snap = check(); close(snap['cameraAspect'], snap['viewport'][0]/snap['viewport'][1], 'resized camera aspect')
    receipt['camera_checks'] += 1
    page.locator('#compressible-frame').fill('0'); page.locator('#compressible-frame').dispatch_event('input'); page.locator('#compressible-play').click()
    page.wait_for_function('window.__compressible.snapshot().frame > 0')
    page.locator('#compressible-play').click(); snap = check(); require(not snap['playing'], 'playback pause')
    paused = snap['frame']; page.wait_for_timeout(650); require(check()['frame'] == paused, 'paused playback timer')
    page.locator('#compressible-frame').fill(str(len(packet['cases'][1]['frames']) - 2)); page.locator('#compressible-frame').dispatch_event('input'); page.locator('#compressible-play').click()
    page.wait_for_function('!window.__compressible.snapshot().playing'); require(check()['frame'] == len(packet['cases'][1]['frames']) - 1, 'last-frame playback stop')
    receipt['playback_checked'] = True; picture('acoustic-slice-desktop.jpg')
    load(mobile, 'compressible-lab.html'); mobile.wait_for_function('window.__compressible !== undefined')
    mobile.select_option('#compressible-case', 'oblique_acoustic'); mobile.select_option('#compressible-axis', '1'); mobile.select_option('#compressible-scalar', 'pressure')
    mobile.locator('#compressible-frame').fill('5'); mobile.locator('#compressible-frame').dispatch_event('input'); check(mobile, caseId='oblique_acoustic', frame=5, scalar='pressure', axis=1)
    require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'), 'mobile field viewer overflow')
    picture('acoustic-mobile.jpg', mobile)
    # An independent page suppresses WebGL to exercise the declared fallback.
    fallback_context = page.context.browser.new_context()
    fallback = fallback_context.new_page()
    fallback_errors = []
    fallback.on('pageerror', lambda e: fallback_errors.append(str(e)))
    fallback.on('response', lambda r: fallback_errors.append(str(r.status) + ' ' + r.url) if r.status >= 400 else None)
    fallback.on('requestfailed', lambda r: fallback_errors.append(str(r.failure) + ' ' + r.url))
    fallback.add_init_script("const original = HTMLCanvasElement.prototype.getContext; HTMLCanvasElement.prototype.getContext = function(type,...args){return type.startsWith('webgl') ? null : original.call(this,type,...args)}")
    load(fallback, 'compressible-lab.html'); fallback.wait_for_function('window.__compressible !== undefined')
    fallback.select_option('#compressible-case', 'oblique_acoustic'); fallback.select_option('#compressible-axis', '1'); fallback.select_option('#compressible-scalar', 'pressure')
    snap = check(fallback); require(snap['renderMode'] == 'slice-fallback' and fallback.locator('#compressible-fallback rect').count() == 144, 'authoritative WebGL fallback')
    require(fallback.locator('#compressible-mode').is_disabled() and 'WebGL unavailable' in fallback.locator('#compressible-status').inner_text(), 'fallback capability notice')
    require(not fallback_errors, 'fallback browser errors: ' + str(fallback_errors))
    receipt['fallback_checked'] = True; fallback_context.close(); check_errors()
    return validate_receipt(receipt, presentation['qualification'])
