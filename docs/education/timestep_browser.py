"""Independent qualification of the bounded native-Rust WASM timestep lesson.

The oracle derives the four-edge circulation with exact rational arithmetic.
Browser and fresh host execution are checked separately against it; host output
is never used to manufacture a browser result. This is a single accepted step
of the existing sealed-box Simulation API, not a liquid/tensor/contact model.
"""
from fractions import Fraction as F
from pathlib import Path
import hashlib
import json
import math
import struct
from urllib.parse import urljoin

from teaching_browser import (arrows, evidence_path, metrics, no_overflow,
                              same_camera, scroll_to, settle, settled_camera,
                              snapshot)

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = 'force-cycle-2x2x1-v1'
METHODS = ('jacobi-pcg-v1', 'sgs-pcg-v1')
FIELDS = ('x', 'y', 'z', 'tracer')
COUNTS = {'x': (3, 2, 1), 'y': (2, 3, 1), 'z': (2, 2, 2), 'tracer': (2, 2, 1)}
GRID = {'counts': [2, 2, 1], 'spacing': [1, 1, 1], 'origin': [0, 0, 0]}
SOURCE_PATHS = {'timestep_labs.js': 'docs/education/timestep_labs.js',
                'timestep_worker.js': 'docs/education/timestep_worker.js',
                'solver-source.rs': 'browser/solver/src/lib.rs',
                'timestep-contract.rs': 'tests/forces_contract.rs'}
SOURCE_FILES = tuple(SOURCE_PATHS)
PACKET_FILES = ('rheon_solver.js', 'rheon_solver_bg.wasm', 'native-reference.json', 'build-receipt.json')


def source_hashes(repo=ROOT):
    return {name: hashlib.sha256((Path(repo)/source).read_bytes()).hexdigest()
            for name, source in SOURCE_PATHS.items()}


def require(condition, message):
    if not condition:
        raise RuntimeError('Timestep browser: ' + message)


def close(actual, expected, label, *, scale=None, tolerance=2e-12):
    require(type(actual) in (float, int) and math.isfinite(actual), 'nonfinite ' + label)
    magnitude = abs(float(expected)) if scale is None else max(abs(float(expected)), abs(float(scale)))
    require(abs(actual - float(expected)) <= tolerance * magnitude,
            f'{label}: observed {actual!r}, expected {float(expected)!r}')


def float_bits(value):
    return struct.unpack('>I', struct.pack('>f', value))[0]


def time_bits(value):
    return struct.pack('>d', value).hex()


def index(counts, i, j, k):
    return i + counts[0] * (j + counts[1] * k)


def cycle_oracle(accepted=True):
    """Project (1/2,0,0,0) onto the unit-square graph's cycle nullspace."""
    # Columns are oriented lower/upper x edges, then left/right y edges;
    # rows are cells in x-fastest order. Incidence is outgoing minus incoming.
    incidence = ((1, 0, 1, 0), (-1, 0, 0, 1), (0, 1, -1, 0), (0, -1, 0, -1))
    circulation = (F(1), F(-1), F(-1), F(1))
    require(all(sum(a*b for a, b in zip(row, circulation)) == 0 for row in incidence),
            'independent circulation is not divergence free')
    impulse = (F(1, 4)*2, F(0), F(0), F(0))
    coefficient = sum(a*b for a, b in zip(impulse, circulation)) / sum(v*v for v in circulation)
    edge = [coefficient*v if accepted else F(0) for v in circulation]
    fields = {name: [F(0)]*math.prod(counts) for name, counts in COUNTS.items()}
    for name, coordinates, value in zip(('x', 'x', 'y', 'y'),
                                        ((1, 0, 0), (1, 1, 0), (0, 1, 0), (1, 1, 0)), edge):
        fields[name][index(COUNTS[name], *coordinates)] = value
    return {**fields, 'divergence': [sum(a*b for a, b in zip(row, edge)) for row in incidence],
            'kinetic_energy': sum(v*v for v in edge)/2,
            'courant': F(1, 4)*(max(map(abs, fields['x']))+max(map(abs, fields['y']))),
            'force_work': sum(v*v for v in impulse)/2 if accepted else F(0)}


def field_diagnostics(state):
    """Direct x-fastest face-flux traversal; no production divergence routine."""
    divergence = []
    for k in range(1):
        for j in range(2):
            for i in range(2):
                value = 0.0
                for axis, name in enumerate(('x', 'y', 'z')):
                    lower = [i, j, k]
                    upper = lower.copy()
                    upper[axis] += 1
                    value += state[name][index(COUNTS[name], *upper)]-state[name][index(COUNTS[name], *lower)]
                divergence.append(value)
    return {'divergence': divergence,
            'actual_divergence_max': max(map(abs, divergence)),
            'kinetic_energy': .5*sum(v*v for name in ('x', 'y', 'z') for v in state[name]),
            'courant': .25*sum(max(map(abs, state[name])) for name in ('x', 'y', 'z')),
            'tracer_integral': sum(state['tracer'])}


def check_state(state, method, accepted, generation):
    require(isinstance(state, dict), 'missing native snapshot')
    require(state.get('schema') == 1 and state.get('fixture') == FIXTURE, 'native snapshot identity')
    require(state.get('method') == method and method in METHODS, 'native pressure method identity')
    require(state.get('grid') == GRID, 'native fixed grid identity')
    require(state.get('accepted_since_reset') is accepted, 'accepted-since-reset marker')
    require(type(state.get('generation')) is int and state['generation'] == generation and
            state.get('generation_u64') == str(generation), 'accepted generation identity')
    close(state.get('time'), .25 if accepted else 0, 'accepted physical time')
    require(state.get('time_bits') == time_bits(state['time']), 'accepted time bit mirror')
    expected = cycle_oracle(accepted)
    for name, counts in COUNTS.items():
        actual = state.get(name)
        require(isinstance(actual, list) and len(actual) == math.prod(counts), 'native field shape '+name)
        bit_values = state.get('bits', {}).get(name)
        require(isinstance(bit_values, list) and len(bit_values) == len(actual), 'native field bit shape '+name)
        for offset, (value, truth, bits) in enumerate(zip(actual, expected[name], bit_values)):
            # Match the original force contract's 1e-8 face tolerance; walls,
            # z faces and the zero tracer must remain exactly zero.
            close(value, truth, f'cycle {name}[{offset}]', scale=F(1, 8) if truth else 0, tolerance=8e-8)
            require(type(bits) is int and bits == float_bits(value), f'native float32 bit mirror {name}[{offset}]')
            require(struct.unpack('>f', struct.pack('>I', bits))[0] == value,
                    f'native field is exactly binary32 {name}[{offset}]')
    derived = field_diagnostics(state)
    require(max(map(abs, derived['divergence'])) <= 4e-8, 'independent cell divergence bound')
    close(derived['kinetic_energy'], expected['kinetic_energy'], 'independent cycle energy', tolerance=2e-7)
    close(derived['courant'], expected['courant'], 'independent cycle Courant', tolerance=2e-7)
    require(derived['tracer_integral'] == 0, 'zero tracer transport')
    return derived


def check_report(report, state):
    require(isinstance(report, dict), 'accepted step report is missing')
    derived = field_diagnostics(state)
    for key, value in [('dt', .25), ('time', state['time']), ('generation', state['generation']),
                       *[(key, derived[key]) for key in ('actual_divergence_max', 'courant',
                                                        'tracer_integral', 'kinetic_energy')]]:
        close(report.get(key), value, 'reported '+key)
    force = report.get('forces', {})
    for key, value in [('kinetic_energy_before', 0), ('kinetic_energy_after', F(1, 8)), ('applied_work', F(1, 8))]:
        close(force.get(key), value, 'reported force '+key)
    pressure = report.get('pressure', {})
    require(type(pressure.get('iterations')) is int and 0 < pressure['iterations'] <= 1000,
            'native pressure iterations')
    for key, limit in [('true_residual_l2', 1e-9), ('true_residual_max', 1e-9), ('predicted_divergence_max', 1e-9)]:
        value = pressure.get(key)
        require(type(value) in (float, int) and math.isfinite(value) and 0 <= value <= limit,
                'native pressure acceptance '+key)


def check_outcome(outcome, method, accepted, generation, code='accepted', before=None):
    require(isinstance(outcome, dict), 'missing native step outcome')
    require(outcome.get('status') == ('accepted' if code == 'accepted' else 'rejected') and
            outcome.get('code') == code, 'native outcome code '+code)
    if code == 'accepted':
        require(outcome.get('detail') is None, 'accepted outcome has no failure detail')
    else:
        require(isinstance(outcome.get('detail'), str) and outcome['detail'], 'native rejection explanation')
    check_state(outcome.get('state'), method, accepted, generation)
    if code == 'accepted':
        check_report(outcome.get('report'), outcome['state'])
    else:
        require(outcome.get('report') is None, 'rejection publishes no accepted report')
        require(before is not None and outcome['state'] == before, 'rejected candidate changed accepted state or bits')
    return outcome['state']


def check_native_reference(packet):
    require(isinstance(packet, dict) and packet.get('schema') == 1 and packet.get('fixture') == FIXTURE and
            packet.get('dt') == .25, 'fresh host reference identity')
    runs = packet.get('runs')
    require(isinstance(runs, list) and [run.get('method') for run in runs] == list(METHODS),
            'fresh host reference includes both original pressure methods')
    for run in runs:
        method = run['method']
        baseline = run.get('baseline')
        check_state(baseline, method, False, 0)
        check_outcome(run.get('invalid_dt'), method, False, 0, 'invalid_time_step', baseline)
        check_outcome(run.get('cancelled_rest'), method, False, 0, 'cancelled_before_commit', baseline)
        accepted = check_outcome(run.get('accepted'), method, True, 1)
        check_outcome(run.get('cancelled_nonzero'), method, True, 1, 'cancelled_before_commit', accepted)
        check_outcome(run.get('second_accept'), method, True, 1, 'reset_required', accepted)
        check_state(run.get('reset'), method, False, 2)
        retried = check_outcome(run.get('after_reset'), method, True, 3)
        require(all(accepted[name] == retried[name] for name in (*FIELDS, 'bits', 'time', 'time_bits')),
                'fresh host reset/retry differs from original accepted fields')
    return {run['method']: run for run in runs}


def wait_ready(page):
    # Import once through evaluate (which awaits the module), then poll a
    # synchronous predicate. An async predicate can itself be truthy before
    # its Promise has resolved in the supported browser automation runtime.
    page.evaluate(r'''async () => {
      const reader = [...document.scripts].find(s => /\/reader\.js$/.test(s.src));
      window.__timestepQualification = await import(new URL('labs.js', reader.src).href);
    }''')
    result = page.wait_for_function('''() => {
      const observed = window.__timestepQualification.snapshot();
      const reference = observed.reference;
      return reference && !reference.busy &&
        (reference.state || reference.error || reference.included === false) ? observed : false;
    }''').json_value()
    settle(page)
    require(result['lab'] == 'timestep', 'native timestep lesson selection')
    require(result.get('webgl'), 'native timestep shared WebGL view')
    reference = result.get('reference')
    require(isinstance(reference, dict) and reference.get('busy') is False and not reference.get('error'),
            'native worker failed or did not settle: '+str(reference))
    return result


def readout(page, reference):
    state, report = reference['state'], reference['lastReport']
    values = metrics(page)
    for label, expected in [('Accepted time', state['time']), ('State generation', state['generation']),
                            ('Accepted kinetic energy', report['kinetic_energy'] if report else 0),
                            ('Stored divergence max', report['actual_divergence_max'] if report else 0)]:
        close(float(values[label].split()[0]), expected, 'visible '+label, tolerance=5.1e-6)
    expected_operation = reference['outcome']['code'] if reference['outcome'] else 'Ready at rest'
    require(values.get('Last operation') == expected_operation, 'visible native outcome')
    if report:
        for label, expected in [('Accepted dt', report['dt']), ('Pressure iterations', report['pressure']['iterations']),
                                ('Pressure true residual max', report['pressure']['true_residual_max']),
                                ('Force-stage work', report['forces']['applied_work']),
                                ('Accepted Courant', report['courant'])]:
            close(float(values[label].split()[0]), expected, 'visible '+label, tolerance=5.1e-6)
    else:
        require(values.get('Pressure iterations') == 'No accepted solve yet', 'rest claims no accepted pressure solve')
    expected = cycle_oracle()
    faces = [('x', 1), ('x', 4), ('y', 2), ('y', 3)]
    rows = page.locator('#timestep-faces tbody tr')
    require(rows.count() == 4, 'four native interior-face readout rows')
    for row, (name, offset) in enumerate(faces):
        cells = rows.nth(row).locator('td').all_text_contents()
        require(len(cells) == 3, 'native face readout columns')
        close(float(cells[1]), state[name][offset], 'visible native interior face', tolerance=5.1e-6)
        if state['accepted_since_reset']:
            close(float(cells[2]), expected[name][offset], 'visible cycle reference', tolerance=5.1e-6)
        else:
            require(cells[2] == 'after step', 'reference is explicitly distinct from unstepped state')
    if state['accepted_since_reset']:
        error = max(abs(state[name][offset]-float(expected[name][offset])) for name, offset in faces)
        close(float(values['Error against c/8'].split()[0]), error, 'visible cycle error', tolerance=5.1e-6)


def check_scene(observed):
    state = observed['reference']['state']
    positions = ((1, .5, .5), (1, 1.5, .5), (.5, 1, .5), (1.5, 1, .5))
    speeds = (state['x'][1], state['x'][4], state['y'][2], state['y'][3])
    expected = []
    for i, (position, speed) in enumerate(zip(positions, speeds)):
        origin = [(p-centre)*.6 for p, centre in zip(position, (1, 1, .5))]
        vector = [0, 0, 0]
        vector[0 if i < 2 else 1] = speed*1.6
        expected.append((origin, vector))
    arrows(observed['arrows'], expected, 'native accepted velocities at fixed scale')


def check_face_table_layout(layout):
    """Require visible face/reference columns, not merely no page overflow."""
    require(isinstance(layout, dict) and layout.get('headers') == ['Face', 'Accepted', 'Reference'],
            'native face table has all three comparison columns')
    require(layout['scrollWidth'] <= layout['clientWidth']+1 and abs(layout['scrollLeft']) <= 1,
            'native face table must fit without horizontal scrolling')
    left = max(0, layout['left'])
    right = min(layout['viewportWidth'], layout['right'])
    require(right > left and len(layout.get('cells', [])) == 15,
            'native face table has a header and four complete rows')
    require(layout['top'] >= layout['headerBottom']-1 and layout['bottom'] <= layout['viewportHeight']+1,
            'native face table is vertically visible below phone controls')
    for cell in layout['cells']:
        require(cell['visible'] and cell['right'] > cell['left'] and
                left-1 <= cell['left'] and cell['right'] <= right+1,
                'native face table comparison column is clipped')
        require(cell['scrollWidth'] <= cell['clientWidth']+1, 'native face table value is clipped inside its cell')


def visible_face_table(page):
    """Inspect actual phone pixels' layout, then restore the reader's position."""
    before = snapshot(page)
    position = page.evaluate('''() => ({x:scrollX,y:scrollY,
      panel:document.querySelector('.example-workspace').scrollTop})''')
    try:
        table = page.locator('#timestep-faces table')
        table.scroll_into_view_if_needed()
        settle(page)
        layout = table.evaluate('''node => {
          const box=node.getBoundingClientRect();
          return {headers:[...node.querySelectorAll('th')].map(cell=>cell.textContent),
            left:box.left,right:box.right,top:box.top,bottom:box.bottom,
            viewportWidth:innerWidth,viewportHeight:innerHeight,
            headerBottom:document.querySelector('header').getBoundingClientRect().bottom,
            scrollWidth:node.scrollWidth,clientWidth:node.clientWidth,scrollLeft:node.scrollLeft,
            cells:[...node.querySelectorAll('th,td')].map(cell=>{
              const b=cell.getBoundingClientRect(),s=getComputedStyle(cell);
              return {left:b.left,right:b.right,scrollWidth:cell.scrollWidth,clientWidth:cell.clientWidth,
                visible:b.height>0&&s.display!=='none'&&s.visibility==='visible'};
            })};
        }''')
        check_face_table_layout(layout)
        return layout
    finally:
        page.evaluate('''p=>{
          document.querySelector('.example-workspace').scrollTop=p.panel;
          scrollTo({left:p.x,top:p.y,behavior:'instant'});
        }''', position)
        settle(page)
        after = snapshot(page)
        require(after['reference'] == before['reference'], 'inspecting phone face table changed native state')
        same_camera(before, after, 'phone face table inspection preserves camera')


def inspect(page, method=None, accepted=None, generation=None):
    observed = wait_ready(page)
    reference = observed['reference']
    require(reference.get('included') is True and reference.get('runtime') == 'wasm-worker',
            'native timestep requires initialized Rust WebAssembly worker')
    require(len(page.workers) == 1 and page.workers[0].url.endswith('/timestep_worker.js'),
            'one owned solver Web Worker')
    for name in ('requestCount', 'acceptedCalls', 'rejectedCalls'):
        require(type(reference.get(name)) is int and reference[name] >= 0, 'worker operation count '+name)
    state = reference.get('state', {})
    check_state(state, method or state.get('method'),
                state.get('accepted_since_reset') if accepted is None else accepted,
                state.get('generation') if generation is None else generation)
    require(page.locator('#timestep-method').input_value() == state['method'], 'method control matches native owner')
    require(page.locator('#timestep-step').is_disabled() is state['accepted_since_reset'],
            'one accepted step per reset is enforced in controls')
    if state['accepted_since_reset']:
        check_report(reference.get('lastReport'), state)
    else:
        require(reference.get('lastReport') is None, 'reset/rest clears last accepted report')
    require(page.locator('#lab-expected').text_content().strip() and page.locator('#lab-limit').text_content().strip(),
            'native lesson prediction and capability limits')
    require(page.locator('#timestep-status').inner_text().strip(), 'native worker operation status')
    readout(page, reference)
    check_scene(observed)
    return observed


def qualify(page, mobile, load, check_errors, artifact_dir=None):
    """Exercise real UI commands, native bit preservation and layout isolation."""
    screenshots, wasm_responses = [], []
    initial_sources = source_hashes()
    def response_received(response):
        if response.url.endswith('/solver/rheon_solver_bg.wasm'):
            wasm_responses.append({'url': response.url, 'status': response.status})
    contexts = list(dict.fromkeys([page.context, mobile.context]))
    for context in contexts:
        context.on('response', response_received)
    numerical_states = accepted_steps = rejected_candidates = resets = 0
    def picture(name, target=page):
        if artifact_dir is not None:
            destination = evidence_path(artifact_dir, name)
            settle(target)
            target.screenshot(path=str(destination), type='jpeg', quality=85, full_page=False)
            screenshots.append(str(destination))
    def checked(target=page, method=None, accepted=None, generation=None):
        nonlocal numerical_states
        result = inspect(target, method, accepted, generation)
        numerical_states += 1
        check_errors()
        return result
    def perform(target, button, expected, method):
        nonlocal accepted_steps, rejected_candidates, resets
        before = checked(target)['reference']
        target.locator('#'+button).click()
        after = checked(target)['reference']
        require(after['requestCount'] == before['requestCount']+1, 'one worker request per explicit command')
        if button == 'timestep-reset':
            resets += 1
            require(after['outcome'] is None and after['state'] == expected, 'native reset matches fresh host output')
            require(after['acceptedCalls'] == before['acceptedCalls'] and after['rejectedCalls'] == before['rejectedCalls'],
                    'reset is not a fluid step')
        else:
            outcome = after['outcome']
            check_outcome(outcome, method, expected['state']['accepted_since_reset'], expected['state']['generation'],
                          expected['code'], before['state'] if expected['status'] == 'rejected' else None)
            require(outcome == expected, 'browser native outcome differs from fresh host execution')
            accepted = expected['status'] == 'accepted'
            accepted_steps += int(accepted)
            rejected_candidates += int(not accepted)
            require(after['acceptedCalls'] == before['acceptedCalls']+int(accepted) and
                    after['rejectedCalls'] == before['rejectedCalls']+int(not accepted), 'native command accounting')
            if not accepted:
                require(after['lastReport'] == before['lastReport'], 'rejection preserves accepted diagnostics')
                require('BeforeCommit' in target.locator('#timestep-status').inner_text(), 'visible native rejection stage')
        require(target.locator('#follow-reading').get_attribute('aria-pressed') == 'false',
                'explicit native command pauses reading follow')
        return after
    try:
        load(page, 'timestep-lab.html')
        initial = wait_ready(page)
        root_url = page.evaluate("new URL('.', [...document.scripts].find(s=>/\\/reader\\.js$/.test(s.src)).src).href")
        def fetch_bytes(path):
            response = page.request.get(urljoin(root_url, path))
            require(response.ok, 'actual served source/artifact is available: '+path)
            return response.body()
        served_sources = {name: hashlib.sha256(fetch_bytes(name)).hexdigest() for name in SOURCE_FILES}
        require(served_sources == initial_sources, 'actual served timestep modules/contracts differ from reviewed source')
        if initial['reference'].get('included') is False:
            require('no compiled solver packet' in page.locator('#timestep-status').inner_text(),
                    'source-only edition explains unavailable native timestep')
            require(all(page.locator('#'+name).is_disabled() for name in
                        ('timestep-step', 'timestep-reject', 'timestep-reset', 'timestep-method')),
                    'source-only edition disables native controls')
            require(initial['reference'].get('state') is None and not page.workers,
                    'source-only edition invents no native state')
            check_errors()
            return {'schema': 'rheon-timestep-browser-v1', 'included': False, 'fluid_advances': 0,
                    'rejected_candidates': 0, 'numerical_states': 0, 'unavailable_controls_disabled': True,
                    'no_synthetic_state': True, 'source_sha256': served_sources, 'packet_hashes': {}, 'screenshots': []}
        packet_bytes = {name: fetch_bytes('solver/'+name) for name in PACKET_FILES}
        packet_hashes = {name: hashlib.sha256(data).hexdigest() for name, data in packet_bytes.items()}
        build_receipt = json.loads(packet_bytes['build-receipt.json'])
        require(build_receipt.get('fixture') == FIXTURE, 'served solver build fixture identity')
        for name in PACKET_FILES[:-1]:
            require(packet_hashes[name] == build_receipt.get('artifact_sha256', {}).get(name),
                    'actual served solver artifact differs from its build receipt: '+name)
        packet = json.loads(packet_bytes['native-reference.json'])
        host = check_native_reference(packet)
        require(wasm_responses and all(item['status'] == 200 for item in wasm_responses),
                'actual worker WebAssembly asset loaded successfully')

        for method in METHODS:
            before = checked()['reference']
            page.select_option('#timestep-method', method)
            baseline = checked(method=method, accepted=False, generation=0)['reference']
            require(baseline['state'] == host[method]['baseline'] and baseline['outcome'] is None,
                    'method selection creates the native fixture at rest')
            require(baseline['requestCount'] == before['requestCount']+1 and
                    baseline['acceptedCalls'] == before['acceptedCalls'], 'method selection never advances fluid')
            perform(page, 'timestep-reject', host[method]['cancelled_rest'], method)
            accepted = perform(page, 'timestep-step', host[method]['accepted'], method)
            # A disabled button must not submit a hidden second command. The
            # native host sequence separately verifies reset_required refusal.
            page.locator('#timestep-step').evaluate('node=>node.click()')
            require(checked()['reference'] == accepted, 'disabled second step still changed native state')
            perform(page, 'timestep-reject', host[method]['cancelled_nonzero'], method)
            perform(page, 'timestep-reset', host[method]['reset'], method)
            perform(page, 'timestep-step', host[method]['after_reset'], method)

        stable = checked()
        scene = page.locator('#scene canvas').bounding_box()
        page.mouse.move(scene['x']+scene['width']*.4, scene['y']+scene['height']*.5)
        page.mouse.down()
        page.mouse.move(scene['x']+scene['width']*.65, scene['y']+scene['height']*.6, steps=8)
        page.mouse.up()
        dragged = settled_camera(page)
        require(math.dist(stable['camera'], dragged['camera']) > .05, 'native camera drag works')
        page.locator('#scene').press('ArrowRight')
        camera = settled_camera(page)
        require(math.dist(dragged['camera'], camera['camera']) > .01, 'native keyboard camera works')
        require(checked()['reference'] == stable['reference'], 'camera interactions advanced or reset fluid')
        for anchor in ('timestep', 'timestep-accept', 'timestep-transaction'):
            scroll_to(page, '#'+anchor)
            observed = checked()
            require(observed['reference'] == stable['reference'], 'scroll advanced or reset native fluid')
            same_camera(camera, observed, 'native scroll preserves manual camera')
        page.locator('#follow-reading').click()
        observed = checked()
        require(observed['reference'] == stable['reference'] and
                page.locator('#follow-reading').get_attribute('aria-pressed') == 'true',
                'resume reading preserves accepted state')
        for anchor in ('timestep', 'timestep-accept', 'timestep-transaction'):
            scroll_to(page, '#'+anchor)
            observed = checked()
            require(observed['reference'] == stable['reference'], 'reading stop submitted a native timestep')
            require(page.locator('#'+anchor).evaluate('node=>node.classList.contains("current-lesson")'),
                    'native reading stop activation '+anchor)
            same_camera(camera, observed, 'native following preserves camera')
            no_overflow(page, 'native desktop reading')
        page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
        picture('timestep-desktop-accepted.jpg')
        for button in ('zoom-in', 'zoom-out', 'reset-view'):
            page.locator('#'+button).click()
            require(checked()['reference'] == stable['reference'], 'camera button changed accepted fluid')

        chapter = 'chapters/17-rust-implementation.html'
        load(page, chapter+'#core-ownership')
        chapter_initial = checked(method=METHODS[0], accepted=False, generation=0)
        require(chapter_initial['reference']['acceptedCalls'] == 0, 'chapter deep link did not start at rest')
        perform(page, 'timestep-step', host[METHODS[0]]['accepted'], METHODS[0])
        chapter_state = checked()
        scroll_to(page, '#storage-and-invariants')
        require(checked()['reference'] == chapter_state['reference'], 'chapter scroll preserves accepted state')
        page.locator('#follow-reading').click()
        for anchor in ('core-ownership', 'storage-and-invariants'):
            scroll_to(page, '#'+anchor)
            observed = checked()
            require(observed['reference'] == chapter_state['reference'], 'chapter following changed native state')
            same_camera(chapter_state, observed, 'native chapter preserves camera')
        for destination in ['timestep-lab.html#'+anchor for anchor in ('timestep', 'timestep-accept', 'timestep-transaction')] + [
                chapter+'#core-ownership', chapter+'#storage-and-invariants']:
            load(page, destination)
            linked = checked(method=METHODS[0], accepted=False, generation=0)
            require(linked['reference']['acceptedCalls'] == linked['reference']['rejectedCalls'] == 0,
                    'native deep link must never execute a timestep')
            anchor = destination.split('#')[1]
            require(page.locator('#'+anchor).evaluate('node=>node.classList.contains("current-lesson")'),
                    'native deep link reading stop '+anchor)

        mobile_results = []
        for width in (390, 320):
            mobile.set_viewport_size({'width': width, 'height': 844})
            load(mobile, 'timestep-lab.html#timestep-accept')
            rest = checked(mobile, METHODS[0], False, 0)
            require(mobile.locator('body').get_attribute('data-reading-mode') == 'text' and
                    mobile.locator('.reading-text').is_visible() and not mobile.locator('.example-workspace').is_visible(),
                    'native phone readable text default')
            require(mobile.locator('.reading-text').evaluate('node=>parseFloat(getComputedStyle(node).fontSize)') >= 16 and
                    mobile.locator('.reading-text').bounding_box()['width'] >= width-48, 'native phone text dimensions')
            no_overflow(mobile, 'native phone text')
            reading_position = mobile.evaluate('scrollY')
            mobile.locator('[data-reading-mode=example]').click()
            require(checked(mobile)['reference'] == rest['reference'], 'phone Example mode started a timestep')
            mobile.locator('[data-reading-mode=text]').click()
            settle(mobile)
            require(abs(mobile.evaluate('scrollY')-reading_position) <= 2, 'native phone restores reading position')
            mobile.locator('[data-reading-mode=split]').click()
            require(mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),
                    'native phone Split visibility')
            scroll_to(mobile, '#timestep-transaction')
            require(checked(mobile)['reference'] == rest['reference'], 'phone Split scroll started a timestep')
            no_overflow(mobile, 'native phone Split')
            mobile.locator('[data-reading-mode=example]').click()
            perform(mobile, 'timestep-reject', host[METHODS[0]]['cancelled_rest'], METHODS[0])
            perform(mobile, 'timestep-step', host[METHODS[0]]['accepted'], METHODS[0])
            accepted = checked(mobile)
            visible_face_table(mobile)
            picture(f'timestep-phone-{width}-example.jpg', mobile)
            mobile.locator('[data-reading-mode=split]').click()
            scroll_to(mobile, '#timestep-accept')
            require(checked(mobile)['reference'] == accepted['reference'], 'phone Split changed accepted fluid')
            same_camera(accepted, snapshot(mobile), 'native phone preserves camera')
            picture(f'timestep-phone-{width}-split.jpg', mobile)
            mobile.locator('[data-reading-mode=text]').click()
            require(checked(mobile)['reference'] == accepted['reference'], 'phone Text changed accepted fluid')
            no_overflow(mobile, 'native phone accepted text')
            load(mobile, chapter+'#storage-and-invariants')
            linked = checked(mobile, METHODS[0], False, 0)
            require(linked['reference']['acceptedCalls'] == linked['reference']['rejectedCalls'] == 0,
                    'phone native chapter link submitted a timestep')
            mobile_results.append({'width': width, 'modes': ['text', 'split', 'example'], 'readable_text': True,
                                   'no_horizontal_overflow': True, 'mode_preserves_lesson_and_scroll': True,
                                   'layout_and_scroll_do_not_advance': True, 'rejection_then_acceptance': True,
                                   'chapter_direct_link': True, 'all_face_table_columns_visible': True})
        mobile.set_viewport_size({'width': 390, 'height': 844})
        check_errors()
        require(source_hashes() == initial_sources, 'reviewed timestep modules/contracts changed during qualification')
        require(accepted_steps == 7 and rejected_candidates == 6 and resets == 2, 'bounded native command coverage')
        return {'schema': 'rheon-timestep-browser-v1', 'included': True,
                'source_sha256': served_sources, 'packet_hashes': packet_hashes,
                'fixture': FIXTURE, 'methods': list(METHODS), 'native_reference_runs': len(host),
                'numerical_states': numerical_states, 'fluid_advances': accepted_steps,
                'rejected_candidates': rejected_candidates, 'native_resets': resets,
                'full_field_and_bit_comparison': True, 'independent_cycle_projection': True,
                'independent_cell_divergence': True, 'force_work_and_kinetic_energy': True,
                'fresh_host_native_comparison': True, 'actual_wasm_worker': True,
                'fixed_scale_velocity_arrows': True, 'desktop_scroll_stops': 3, 'chapter_scroll_stops': 2,
                'desktop_direct_links': 5, 'rejection_preserves_accepted_bits': True,
                'camera_and_reading_do_not_advance': True, 'manual_commands_pause_follow': True,
                'mobile': mobile_results, 'screenshots': screenshots}
    finally:
        for context in contexts:
            context.remove_listener('response', response_received)


def validate_receipt(observed, included):
    """Use the publication gate's ValueError boundary for malformed receipts."""
    try:
        _validate_receipt(observed, included)
    except RuntimeError as error:
        raise ValueError(str(error)) from error


def _validate_receipt(observed, included):
    """Refuse omitted numerical/transaction/UI coverage in publication receipts."""
    require(isinstance(observed, dict) and observed.get('schema') == 'rheon-timestep-browser-v1',
            'missing native timestep qualification receipt')
    require(observed.get('included') is included, 'native timestep receipt inclusion mismatch')
    require(observed.get('source_sha256') == source_hashes(), 'stale native timestep served module/contract bindings')
    require(isinstance(observed.get('screenshots'), list), 'native timestep screenshot list')
    if not included:
        require(observed.get('packet_hashes') == {}, 'unavailable native timestep cannot claim solver assets')
        for name in ('fluid_advances', 'rejected_candidates', 'numerical_states'):
            require(type(observed.get(name)) is int and observed[name] == 0,
                    'unavailable native timestep cannot claim '+name)
        for name in ('unavailable_controls_disabled', 'no_synthetic_state'):
            require(observed.get(name) is True, 'unavailable native timestep check '+name)
        return
    hashes = observed.get('packet_hashes')
    require(isinstance(hashes, dict) and set(hashes) == set(PACKET_FILES), 'missing served solver artifact bindings')
    require(all(isinstance(value, str) and len(value) == 64 and all(c in '0123456789abcdef' for c in value)
                for value in hashes.values()), 'malformed served solver artifact hashes')
    require(observed.get('fixture') == FIXTURE and observed.get('methods') == list(METHODS),
            'native timestep receipt fixed fixture and both methods')
    for name, expected in [('native_reference_runs', 2), ('numerical_states', 72), ('fluid_advances', 7),
                           ('rejected_candidates', 6), ('native_resets', 2), ('desktop_scroll_stops', 3),
                           ('chapter_scroll_stops', 2), ('desktop_direct_links', 5)]:
        require(type(observed.get(name)) is int and observed[name] == expected, 'native timestep coverage '+name)
    for name in ('full_field_and_bit_comparison', 'independent_cycle_projection', 'independent_cell_divergence',
                 'force_work_and_kinetic_energy', 'fresh_host_native_comparison', 'actual_wasm_worker',
                 'fixed_scale_velocity_arrows', 'rejection_preserves_accepted_bits',
                 'camera_and_reading_do_not_advance', 'manual_commands_pause_follow'):
        require(observed.get(name) is True, 'native timestep qualification check '+name)
    mobile = observed.get('mobile')
    require(isinstance(mobile, list) and [entry.get('width') for entry in mobile] == [390, 320],
            'native timestep requires both phone widths')
    for entry in mobile:
        require(entry.get('modes') == ['text', 'split', 'example'], 'native phone mode coverage')
        for name in ('readable_text', 'no_horizontal_overflow', 'mode_preserves_lesson_and_scroll',
                     'layout_and_scroll_do_not_advance', 'rejection_then_acceptance', 'chapter_direct_link',
                     'all_face_table_columns_visible'):
            require(entry.get(name) is True, 'native phone qualification check '+name)
