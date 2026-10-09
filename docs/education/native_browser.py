"""Qualify only recorded native playback and its rendered data, never integrate."""
import csv
import hashlib
import io
import json
import math
from pathlib import PurePosixPath


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def qualify(page, mobile, load, check_errors, expected, base_url):
    def get(name):
        response = page.request.get(base_url.rstrip('/')+'/'+name)
        require(response.ok, 'Native asset HTTP failure: '+name)
        return response.body()

    load(page, 'native-labs.html')
    require(page.locator('main article').count() == 3, 'Missing three-step native hub')
    require('Snapshot playback' in page.locator('main').inner_text(), 'Missing native playback scope')
    presentation = json.loads(get('native-presentation.json'))
    require(presentation['accepted_inputs'] == expected['labs'], 'Native presentation input provenance changed')
    require(presentation['numerical_calls'] == 0, 'Native build claims numerical calls')
    load(mobile, 'native-labs.html')
    require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'), 'Mobile native hub overflow')
    result = {'recorded_bundles_included': presentation['recorded_bundles_included'],
              'hub_models': 3, 'mobile_no_overflow': True, 'native_snapshot_checks': 0,
              'wall_final_profile_checks': 0, 'native_numerical_calls': 0}
    if not presentation['recorded_bundles_included']:
        require(page.locator('main').inner_text().count('not included') == 3,
                'Source-only native hub must state all three absent bundles')
        require(page.locator('main a[href^="native-labs/"]').count() == 0,
                'Source-only native hub links missing playback')
        return result

    files = presentation['files']
    for name, digest in files.items():
        require(hashlib.sha256(get(name)).hexdigest() == digest, 'Published native bytes changed: '+name)
    for lab in expected['labs']:
        prefix = 'native-labs/'+lab['id']+'/'
        require({PurePosixPath(n).name for n in files if n.startswith(prefix)} == set(lab['files']),
                'Native file inventory changed: '+lab['id'])
        for name, digest in lab['files'].items():
            if lab['id'] != 'wall-friction' or not name.endswith('.html'):
                require(files[prefix+name] == digest, 'Original native record changed: '+name)
        if lab['id'] == 'wall-friction':
            load(page, prefix+'index.html')
            for name in sorted(n for n in lab['files'] if n.endswith('.csv')):
                rows = list(csv.DictReader(io.StringIO(get(prefix+name).decode())))
                times = sorted({float(r['time']) for r in rows})
                require(len(times) == 4, 'Wall CSV must retain four saved times')
                final = [r for r in rows if float(r['time']) == times[-1]]
                load(page, prefix+name.replace('.csv','.html'))
                require('Snapshot playback' in page.locator('[role=note]').inner_text(), 'Wall plot missing playback notice')
                require(page.locator('a').filter(has_text='Native transient CSV').get_attribute('href') == name,
                        'Wall CSV target changed')
                actual = page.locator('polyline[stroke="#087eae"]').get_attribute('points').split()
                require(len(actual) == len(final), 'Wall plot dropped native rows')
                for point, row in zip(actual, final):
                    x, y = map(float, point.split(','))
                    require(abs(x-(40+420*float(row['velocity']))) < 1e-8 and
                            abs(y-(290-280*float(row['y']))) < 1e-8,
                            'Wall final plot differs from frozen CSV: '+name)
                load(mobile, prefix+name.replace('.csv','.html'))
                require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'), 'Mobile wall profile overflow')
                result['wall_final_profile_checks'] += 1
            continue
        load(page, prefix+'index.html')
        require('Snapshot playback' in page.locator('[role=note]').inner_text(), 'Native page missing playback notice')
        data = json.loads(page.locator('#native-data').inner_text())
        require(len(data['cases']) == lab['counts']['cases'], 'Native case roster changed')
        common = {'Kinetic energy (J)':'energy', 'Bulk loss (J)':'bulk', 'Bulk viscous loss (J)':'bulk',
                  'Explicit-step energy correction (J)':'update', 'Explicit increment energy (J)':'update',
                  'Rounding work (J)':'rounding', 'Energy ledger residual (J)':'energy_residual',
                  'Momentum ledger residual (kg m/s)':'momentum_residual', 'Actuator work (J)':'work',
                  'Navier relative loss (J)':'wall_dissipation', 'Body old-speed work (J)':'body_work',
                  'Stationary-wall work (J)':'wall_work', 'Body impulse (N s)':'body_impulse',
                  'Momentum (kg m/s)':'momentum', 'Actual flow per width (m²/s)':'flow_per_width',
                  'Continuum steady deviation (m/s)':'continuum_deviation',
                  'Discrete equilibrium deviation (m/s)':'discrete_deviation',
                  'Discrete startup deviation (m/s)':'transient_deviation'}
        for case_index, case in enumerate(data['cases']):
            require(len(case['snapshots']) == 5, 'Native saved-time roster changed')
            page.select_option('#case', str(case_index))
            for index, snapshot in enumerate(case['snapshots']):
                page.locator('#snapshot').fill(str(index))
                page.locator('#snapshot').dispatch_event('input')
                require(f"step {snapshot['step']}" in page.locator('#time').inner_text(), 'Native snapshot control is unresponsive')
                require(page.locator('#csv').get_attribute('href') == case['name']+'.csv', 'Native CSV link selects the wrong case')
                circles = page.locator('#chart circle').evaluate_all('(nodes)=>nodes.map(n=>[+n.getAttribute("cx"),+n.getAttribute("cy")])')
                require(len(circles) == len(snapshot['profiles']), 'Native plot dropped recorded nodes')
                if lab['id'] == 'poiseuille':
                    peak = case['q']/(8*case['mu']); lo=min(0,peak*1.08); hi=max(0,peak*1.08)
                else:
                    lo,hi=0,1
                for (x,y),row in zip(circles,snapshot['profiles']):
                    require(math.isclose(x,65+420*(row[1]-lo)/(hi-lo),abs_tol=1e-8) and
                            math.isclose(y,365-340*row[0],abs_tol=1e-8), 'Native plotted profile differs from saved data')
                ledger = page.locator('#ledger tr').evaluate_all('(rows)=>rows.map(r=>[r.cells[0].textContent,r.cells[1].textContent])')
                require(len(ledger) == (17 if lab['id']=='poiseuille' else 13), 'Native ledger rows are missing')
                for label, rendered in ledger:
                    if label in common:
                        value = snapshot[common[label]]
                        require(abs(float(rendered)-value) <= max(abs(value)*5e-6,1e-300), 'Native ledger changed: '+label)
                check_errors()
                result['native_snapshot_checks'] += 1
            page.locator('#previous').click()
            require(page.locator('#snapshot').input_value() == '3', 'Earlier-snapshot button failed')
            page.locator('#next').click()
            require(page.locator('#snapshot').input_value() == '4', 'Later-snapshot button failed')
        load(mobile,prefix+'index.html')
        mobile.select_option('#case',str(len(data['cases'])-1))
        mobile.locator('#snapshot').fill('0');mobile.locator('#snapshot').dispatch_event('input')
        require('step 0' in mobile.locator('#time').inner_text(), 'Mobile native controls failed')
        require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'), 'Mobile native playback overflow')
    require(result['native_snapshot_checks'] == 50 and result['wall_final_profile_checks'] == 6,
            'Incomplete native browser roster')
    check_errors()
    return result
