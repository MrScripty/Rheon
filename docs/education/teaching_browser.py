"""Qualify the six reference examples and the research-book reading surface.

This bounded browser check evaluates displayed saved/reference states. It does
not advance a fluid solver, qualify a pressure API, or certify continuum error.
"""
from pathlib import Path
import hashlib
import json
import math
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LABS = ('projection', 'collision', 'hydrostatic', 'viscous', 'slip', 'cap')


def require(condition, message):
    if not condition:
        raise RuntimeError('Teaching browser: ' + message)


def close(actual, expected, label, *, scale=None, tolerance=1e-11):
    require(type(actual) in (int, float) and math.isfinite(actual), 'nonfinite ' + label)
    magnitude = max(abs(expected), abs(scale)) if scale is not None else abs(expected)
    require(abs(actual - expected) <= tolerance * magnitude,
            f'{label}: observed {actual!r}, expected {expected!r}')


def metrics(page):
    return page.locator('#metrics').evaluate('''node => Object.fromEntries(
        [...node.querySelectorAll('dt')].map(dt => [dt.textContent, dt.nextElementSibling.textContent]))''')


def number(values, key):
    require(key in values, 'missing displayed metric ' + key)
    match = re.match(r'^\s*([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?)', values[key], re.I)
    require(match is not None, 'nonnumeric displayed metric ' + key)
    return float(match[1])


def display(values, key, expected):
    # The GUI intentionally rounds to five significant figures. No unit-sized
    # absolute floor is allowed: tiny residual/gap readouts must remain tiny.
    close(number(values, key), expected, 'display ' + key, tolerance=5.1e-5)


def settle(page):
    page.evaluate('new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))')


def snapshot(page):
    return page.evaluate(r'''async () => {
      const reader = [...document.scripts].find(s => /\/reader\.js$/.test(s.src));
      return (await import(new URL('labs.js', reader.src).href)).snapshot();
    }''')


def settled_camera(page):
    # RecordedView uses inertial OrbitControls. Wait for measured convergence,
    # not a wall-clock guess that depends on the software renderer frame rate.
    return page.evaluate(r'''async () => {
      const reader = [...document.scripts].find(s => /\/reader\.js$/.test(s.src));
      const lab = await import(new URL('labs.js', reader.src).href);
      let previous = lab.snapshot(), stable = 0;
      const end = performance.now() + 30000;
      while (performance.now() < end) {
        await new Promise(resolve => requestAnimationFrame(resolve));
        const current = lab.snapshot();
        const distance = Math.hypot(...current.camera.map((v,i) => v-current.target[i]));
        const movement = Math.hypot(...current.camera.map((v,i) => v-previous.camera[i]));
        stable = movement <= distance*1e-10 ? stable+1 : 0;
        if (stable >= 6) return current;
        previous = current;
      }
      throw new Error('Teaching browser: camera damping did not settle');
    }''')


def range_value(page, value):
    control = page.locator('#controls input[type=range]')
    control.fill(str(value))
    control.dispatch_event('input')
    settle(page)


def choose(page, value):
    page.locator('#controls select').select_option(str(value))
    settle(page)


def arrows(actual, expected, label):
    # Subthreshold vectors are deliberately absent from THREE.ArrowHelper.
    expected = [(origin, vector) for origin, vector in expected if math.hypot(*vector) >= 1e-8]
    require(len(actual) == len(expected), label + ' arrow count')
    for index, (observed, (origin, vector)) in enumerate(zip(actual, expected)):
        length = math.hypot(*vector)
        for axis, coordinate in enumerate(origin):
            close(observed['origin'][axis], coordinate, f'{label} arrow {index} origin {axis}')
            close(observed['direction'][axis], vector[axis]/length,
                  f'{label} arrow {index} direction {axis}', scale=1)
        close(observed['length'], length, f'{label} arrow {index} fixed-scale length')


def same_camera(before, after, label):
    for key in ('camera', 'target'):
        require(len(before[key]) == len(after[key]), label + ' ' + key + ' shape')
        for a, b in zip(before[key], after[key]):
            close(a, b, label + ' ' + key, scale=math.dist(before['camera'], before['target']), tolerance=1e-8)


def evidence_path(directory, name):
    directory = Path(directory).resolve()
    require(not directory.is_relative_to(ROOT) and not ROOT.is_relative_to(directory),
            'screenshots must be outside the checkout')
    ancestor = directory
    while not ancestor.exists():
        ancestor = ancestor.parent
    require(subprocess.run(['git', '-C', str(ancestor), 'rev-parse', '--is-inside-work-tree'],
                           capture_output=True).returncode != 0, 'screenshots must be outside every Git checkout')
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / name
    require(not destination.exists(), 'screenshot destination must be fresh: ' + str(destination))
    return destination


def numerical_checks(page, data, check_errors):
    counts = {lab: 0 for lab in LABS}
    arrow_states = 0
    cap_images = 0

    def inspect(lab):
        nonlocal arrow_states
        settle(page)
        state = snapshot(page)
        require(state['lab'] == lab and state['webgl'], lab + ' scene selection/WebGL')
        require(page.locator('#lab-description').inner_text().strip(), lab + ' teaching description')
        require(page.locator('#lab-expected').text_content().strip(), lab + ' expected result')
        require(page.locator('#lab-limit').text_content().strip(), lab + ' limitation')
        counts[lab] += 1
        if lab != 'cap':
            arrow_states += 1
        check_errors()
        return state, metrics(page)

    page.select_option('#lab', 'projection')
    p = data['projection']
    initial_incidence = max(map(abs, p['divergence_before']))
    energies = []
    for percent in (0, 50, 100):
        range_value(page, percent)
        state, values = inspect('projection')
        fraction = percent / 100
        velocity = [(1-fraction)*a + fraction*b for a, b in zip(p['before'], p['after'])]
        incidence = [0.0] * len(p['cells'])
        for (left, right), speed in zip(p['edges'], velocity):
            incidence[left] -= speed
            incidence[right] += speed
        stored = [(1-fraction)*a + fraction*b for a, b in zip(p['divergence_before'], p['divergence_after'])]
        for observed, expected in zip(incidence, stored):
            close(observed, expected, 'projection independent edge incidence', scale=initial_incidence)
        energy = 0.5 * sum(speed*speed for speed in velocity)
        display(values, 'Correction fraction', percent)
        display(values, 'Max incidence', max(map(abs, stored)))
        display(values, 'Graph kinetic norm', energy)
        display(values, 'Incidence / initial', max(map(abs, stored)) / initial_incidence)
        display(values, 'Full solve residual', p['residual_max'])
        arrows(state['arrows'], [(origin, [speed*.16 if axis == direction else 0 for axis in range(3)])
                                for origin, speed, direction in zip(p['positions'], velocity, p['axes'])], 'projection')
        energies.append(energy)
    require(energies[0] > energies[1] > energies[2], 'projection norm reduction')
    close(energies[0], p['energy_before'], 'projection initial norm')
    close(energies[-1], p['energy_after'], 'projection corrected norm')

    page.select_option('#lab', 'collision')
    for index, case in enumerate(data['collision']['cases']):
        choose(page, index)
        state, values = inspect('collision')
        require(values['Trace'] == case['label'], 'collision trace label')
        display(values, 'Mesh translation', case['shift'])
        display(values, 'Wall-speed annotation', .2)
        if case['t'] is None:
            require(values['Earliest hit t'] == 'no intersection', 'collision miss reported as hit')
            arrows(state['arrows'], [], 'collision miss')
        else:
            display(values, 'Earliest hit t', case['t'])
            require(0 <= case['t'] <= 1, 'collision hit outside segment')
            hit = [a + case['t']*(b-a) for a, b in zip(case['start'], case['end'])]
            for a, b in zip(hit, case['hit']):
                close(a, b, 'collision segment hit')
            arrows(state['arrows'], [(hit, case['wall_velocity'])], 'collision annotation')

    page.select_option('#lab', 'hydrostatic')
    h = data['hydrostatic']
    pressure_scale = max(max(case['pressure']) for case in h['sets'])
    for index, case in enumerate(h['sets']):
        choose(page, index)
        state, values = inspect('hydrostatic')
        expected = h['gravity'] * h['height'] * .5 * (case['lower_density'] + case['upper_density'])
        display(values, 'Bottom gauge pressure', expected)
        display(values, 'Analytic bottom pressure', expected)
        display(values, 'Lower density', case['lower_density'])
        display(values, 'Upper density', case['upper_density'])
        display(values, 'Face balance error', case['balance_error'])
        close(case['pressure'][0], expected, 'hydrostatic reference bottom pressure')
        arrows(state['arrows'], [([.42, y-.5, .2], [pressure/pressure_scale*.5, 0, 0])
                                for j, (y, pressure) in enumerate(zip(case['y'], case['pressure'])) if j % 2 == 0],
               'hydrostatic common pressure scale')

    page.select_option('#lab', 'viscous')
    v = data['viscous']
    eigenvalue = 4 * math.sin(math.pi/v['n'])**2 / v['h']**2
    close(v['lambda_discrete'], eigenvalue, 'independent periodic eigenvalue')
    for index, case in enumerate(v['sets']):
        choose(page, index)
        prior_energy = prior_arrows = None
        for step in (0, 50, 100):
            range_value(page, step)
            state, values = inspect('viscous')
            q = case['states'][step]
            amplitude = (1 + v['dt']*case['nu']*eigenvalue)**(-step)
            continuum = math.exp(-4*math.pi**2*case['nu']*q['time'])
            close(q['amplitude'], amplitude, 'implicit scalar-mode amplitude')
            close(q['exact_continuum_amplitude'], continuum, 'continuum scalar-mode amplitude')
            display(values, 'Kinematic viscosity', case['nu'])
            display(values, 'Time', q['time'])
            display(values, 'Discrete amplitude', amplitude)
            display(values, 'Continuum amplitude', continuum)
            display(values, 'Amplitude gap', abs(amplitude-continuum))
            display(values, 'Kinetic energy', .5*v['h']*sum(speed*speed for speed in q['velocity']))
            arrows(state['arrows'], [([-.25, y-.5, z], [speed*.4, 0, 0])
                                    for z in (-.35, 0, .35) for y, speed in zip(v['x'], q['velocity'])], 'viscous damping')
            maximum = max(arrow['length'] for arrow in state['arrows'])
            if prior_energy is not None:
                require(q['energy'] < prior_energy and maximum < prior_arrows, 'viscous arrows and energy must damp')
            prior_energy, prior_arrows = q['energy'], maximum

    page.select_option('#lab', 'slip')
    s = data['slip']
    prior_speed, prior_shear = -1, math.inf
    for index, case in enumerate(s['sets']):
        choose(page, index)
        state, values = inspect('slip')
        speed = s['U']*case['length']/(s['height']+case['length'])
        shear = s['mu']*s['U']/(s['height']+case['length'])
        display(values, 'Slip length', case['length'])
        display(values, 'Fluid slip speed', speed)
        display(values, 'Lower-wall speed', 0)
        display(values, 'Wall shear', shear)
        display(values, 'Wall dissipation', shear*speed)
        require(speed > prior_speed and shear < prior_shear, 'slip speed/shear direction')
        arrows(state['arrows'], [([-.3, y-.5, z], [speed*.5, 0, 0])
                                for z in (-.3, 0, .3) for j, (y, speed) in enumerate(zip(s['y'], case['velocity']))
                                if j % 2 == 0], 'slip fluid profile')
        close(case['velocity'][-1], s['U'], 'upper wall no-slip')
        prior_speed, prior_shear = speed, shear

    page.select_option('#lab', 'cap')
    for angle in (0, 12, 24):
        range_value(page, angle)
        q = data['cap']['sets'][angle]
        cosine = math.cos(math.radians(q['theta']))
        radius = (3*q['volume']/(math.pi*(2-3*cosine+cosine**3)))**(1/3)
        close(q['R'], radius, 'independent spherical cap radius')
        close(q['contact_radius'], radius*math.sin(math.radians(q['theta'])), 'independent contact radius')
        picture = None
        for tension, material in enumerate(q['tensions']):
            choose(page, tension)
            _, values = inspect('cap')
            display(values, 'Contact angle', q['theta'])
            display(values, 'Contact radius', q['contact_radius']*1000)
            display(values, 'Liquid volume', q['volume']*1e6)
            display(values, 'Surface tension', material['sigma'])
            display(values, 'Pressure jump', 2*material['sigma']/radius)
            display(values, 'Adhesion work', material['sigma']*(1+cosine))
            display(values, 'Volume integral error', abs(q['integrated_volume']-q['volume']))
            pixels = hashlib.sha256(page.locator('#scene canvas').screenshot()).hexdigest()
            if picture is not None:
                require(pixels == picture, 'surface tension altered the rendered equilibrium geometry')
                cap_images += 1
            picture = pixels
    return {'states': counts, 'total_states': sum(counts.values()), 'arrow_states': arrow_states,
            'cap_render_invariance_comparisons': cap_images,
            'independent_equations': ['graph incidence and kinetic norm', 'segment interpolation',
                                     'two-layer hydrostatic pressure', 'implicit periodic eigenmode and continuum decay',
                                     'one-slip-wall Couette speed, traction and dissipation',
                                     'spherical cap volume, Young–Laplace pressure and adhesion work']}


def scroll_to(page, selector):
    page.locator(selector).evaluate('''node => {
        const extra = innerWidth <= 760 && document.body.dataset.readingMode === 'split'
          ? document.querySelector('.example-workspace').getBoundingClientRect().height : 0;
        window.scrollTo({top: window.scrollY + node.getBoundingClientRect().top - extra -
          document.querySelector('header').getBoundingClientRect().bottom - 24, behavior: 'instant'});
    }''')
    settle(page)


def no_overflow(page, label):
    require(page.evaluate('document.documentElement.scrollWidth <= innerWidth + 1'), label + ' horizontal overflow')


def qualify(page, mobile, load, check_errors, artifact_dir=None):
    reference = ROOT / 'docs/research-book/expansion/reference-data.json'
    data = json.loads(reference.read_text())
    screenshots = []

    def picture(name, target=page):
        if artifact_dir is not None:
            path = evidence_path(artifact_dir, name)
            settle(target)
            target.screenshot(path=str(path), type='jpeg', quality=85, full_page=False)
            screenshots.append(str(path))

    load(page, 'labs.html')
    page.wait_for_selector('#metrics dd')
    observed = page.evaluate('async () => (await fetch("reference-data.json")).json()')
    require(observed == data, 'served reference data differs from qualified source')
    numerical = numerical_checks(page, data, check_errors)
    load(page, 'labs.html')
    page.wait_for_selector('#metrics dd')
    require(page.locator('.reading-text').count() == 1 and page.locator('.example-workspace').count() == 1,
            'missing reading/example columns')
    navigation = page.locator('#navigation').bounding_box()
    article = page.locator('.reading-text').bounding_box()
    workspace = page.locator('.example-workspace').bounding_box()
    require(navigation['x'] + navigation['width'] <= article['x'] and article['x'] + article['width'] <= workspace['x'],
            'desktop chapter/text/example ordering')
    require(article['width'] >= 360 and workspace['width'] >= 320, 'desktop readable column widths')
    require(page.locator('.example-workspace').evaluate("node => getComputedStyle(node).position") == 'sticky',
            'desktop example is not sticky')
    no_overflow(page, 'desktop')
    initial = snapshot(page)
    stop_count = 0
    preset_metrics = {
        'projection': [('Correction fraction', 0), ('Correction fraction', 100)],
        'collision': [('Trace', 'crossing'), ('Trace', 'miss')],
        'hydrostatic': [('Upper density', 800), ('Upper density', 1000)],
        'viscous': [('Time', 0), ('Time', 1)],
        'slip': [('Slip length', 0), ('Slip length', .5)],
        'cap': [('Surface tension', .072), ('Surface tension', .12)],
    }
    for lab in LABS:
        for index, selector in enumerate(('#'+lab, '#'+lab+'-compare')):
            scroll_to(page, selector)
            require(snapshot(page)['lab'] == lab, 'scroll failed to select ' + selector)
            key, expected = preset_metrics[lab][index]
            if isinstance(expected, str):
                require(metrics(page)[key] == expected, 'scroll preset ' + selector)
            else:
                display(metrics(page), key, expected)
            same_camera(initial, snapshot(page), 'scroll must preserve camera')
            require(page.locator('#follow-reading').get_attribute('aria-pressed') == 'true', 'scroll follow disabled itself')
            current = page.locator(selector)
            require('current-lesson' in (current.get_attribute('class') or ''), 'scroll active lesson ' + selector)
            require(abs(page.locator('.example-workspace').bounding_box()['y'] - 84) <= 2, 'example did not stay pinned')
            stop_count += 1
    scroll_to(page, '#projection')
    picture('teaching-desktop-reading.jpg')

    # Real drag and keyboard interaction use the embedded shared RecordedView.
    canvas = page.locator('#scene canvas').bounding_box()
    page.mouse.move(canvas['x']+canvas['width']*.45, canvas['y']+canvas['height']*.5)
    page.mouse.down()
    page.mouse.move(canvas['x']+canvas['width']*.65, canvas['y']+canvas['height']*.6, steps=8)
    page.mouse.up()
    manual = settled_camera(page)
    require(math.dist(manual['camera'], initial['camera']) > .05, 'manual camera drag did not rotate')
    page.locator('#scene').press('ArrowLeft')
    keyboard = settled_camera(page)
    require(math.dist(manual['camera'], keyboard['camera']) > .01, 'keyboard camera did not rotate')
    scroll_to(page, '#hydrostatic')
    same_camera(keyboard, snapshot(page), 'scroll after manual camera')
    page.select_option('#lab', 'projection')
    require(page.locator('#follow-reading').get_attribute('aria-pressed') == 'false', 'manual selection did not pause follow')
    # Resume, then isolate range input from selection as the pause trigger.
    scroll_to(page, '#projection')
    page.locator('#follow-reading').click()
    settle(page)
    range_value(page, 50)
    same_camera(keyboard, snapshot(page), 'manual controls must preserve camera')
    require(page.locator('#follow-reading').get_attribute('aria-pressed') == 'false', 'manual control did not pause follow')
    scroll_to(page, '#cap')
    require(snapshot(page)['lab'] == 'projection' and number(metrics(page), 'Correction fraction') == 50,
            'scroll overwrote manual comparison')
    same_camera(keyboard, snapshot(page), 'paused scroll camera')
    page.locator('#follow-reading').click()
    settle(page)
    require(snapshot(page)['lab'] == 'cap' and page.locator('#follow-reading').get_attribute('aria-pressed') == 'true',
            'follow did not resume current section')
    same_camera(keyboard, snapshot(page), 'resume follow camera')
    before_zoom = snapshot(page)
    page.locator('#zoom-in').click()
    settle(page)
    zoomed = snapshot(page)
    require(math.dist(zoomed['camera'], zoomed['target']) < math.dist(before_zoom['camera'], before_zoom['target']),
            'zoom-in button')
    page.locator('#zoom-out').click()
    settle(page)
    zoom_out = snapshot(page)
    require(math.dist(zoom_out['camera'], zoom_out['target']) > math.dist(zoomed['camera'], zoomed['target']),
            'zoom-out button')
    page.locator('#reset-view').click()
    settle(page)
    same_camera(initial, snapshot(page), 'reset camera')
    wheel_start = snapshot(page)
    scroll_to(page, '#slip')
    wheel_position = page.evaluate('window.scrollY')
    canvas = page.locator('#scene canvas').bounding_box()
    page.mouse.move(canvas['x']+canvas['width']/2, canvas['y']+canvas['height']/2)
    page.mouse.wheel(0, 350)
    page.wait_for_timeout(200)
    same_camera(wheel_start, snapshot(page), 'scroll wheel must not zoom camera')
    require(page.evaluate('window.scrollY') > wheel_position, 'wheel over scene did not scroll reading text')
    picture('teaching-desktop-comparison.jpg')

    load(page, 'chapters/20-collision-mesh-pipeline.html')
    proof = page.locator('.proof-disclosure').first
    require(proof.count() == 1 and proof.get_attribute('open') is None, 'proof should start compact')
    heading = proof.locator('h2').first.get_attribute('id')
    proof.locator('summary').click()
    require(proof.get_attribute('open') is not None and proof.locator('h2').first.is_visible(), 'proof disclosure expansion')
    load(page, 'chapters/20-collision-mesh-pipeline.html#'+heading)
    require(page.locator('.proof-disclosure').first.get_attribute('open') is not None, 'proof deep link failed to reveal claim')
    require(page.locator('#'+heading).is_visible(), 'proof deep link target invisible')
    links = set(page.locator('.reading-text a[href]').evaluate_all('nodes => nodes.map(node => node.getAttribute("href"))'))
    require({'https://graphics.stanford.edu/papers/thin_shells_fluid_coupling-sig05/',
             'https://basilisk.fr/src/embed.h', '../proofs/StaticObstacle.lean'} <= links,
            'chapter primary-source or checked-claim links missing')
    picture('teaching-desktop-proof.jpg')

    mobile_receipt = []
    for width in (390, 320):
        mobile.set_viewport_size({'width': width, 'height': 844})
        load(mobile, 'labs.html')
        mobile.wait_for_selector('#metrics dd', state='attached')
        require(mobile.locator('body').get_attribute('data-reading-mode') == 'text', 'phone defaults to text')
        require(mobile.locator('.reading-text').is_visible() and not mobile.locator('.example-workspace').is_visible(),
                'phone text mode visibility')
        require(mobile.locator('.reading-text').evaluate('node => parseFloat(getComputedStyle(node).fontSize)') >= 16,
                'phone text too small')
        require(mobile.locator('.reading-text').bounding_box()['width'] >= width-48, 'phone text width')
        no_overflow(mobile, f'phone {width} text')
        mobile.locator('#menu').click()
        require(mobile.locator('#navigation').is_visible() and mobile.locator('#menu').get_attribute('aria-expanded') == 'true',
                'phone chapter menu open')
        mobile.locator('#search').fill('wetting')
        require(mobile.locator('.chapter-link:visible').count() == 1, 'phone chapter search')
        mobile.locator('.chapter-link:visible').click()
        mobile.wait_for_load_state('networkidle')
        require('23-wetting-and-adhesion' in mobile.url and not mobile.locator('#navigation').is_visible(),
                'phone chapter navigation/close')
        mobile.locator('#menu').click()
        mobile.locator('#search').press('Escape')
        require(not mobile.locator('#navigation').is_visible(), 'phone Escape closes chapter menu')
        picture(f'teaching-phone-{width}-text.jpg', mobile)
        # Keep a real reading position across a hidden article. Its collapsed
        # element coordinates must never select the chapter's final preset.
        mobile.evaluate("window.scrollTo({top: 200, behavior: 'instant'})")
        settle(mobile)
        reading_position = mobile.evaluate('window.scrollY')
        reading_state = snapshot(mobile)
        reading_metrics = metrics(mobile)
        mobile.locator('[data-reading-mode=example]').click()
        settle(mobile)
        require(snapshot(mobile)['lab'] == reading_state['lab'] and metrics(mobile) == reading_metrics,
                'phone view switch changed the reading example')
        mobile.locator('[data-reading-mode=text]').click()
        settle(mobile)
        require(abs(mobile.evaluate('window.scrollY')-reading_position) <= 2,
                'phone return to text lost reading position')
        require(metrics(mobile) == reading_metrics, 'phone return to text changed the reading example')
        mobile.locator('[data-reading-mode=split]').click()
        settle(mobile)
        require(mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),
                'phone split shows text and example')
        workspace = mobile.locator('.example-workspace').bounding_box()
        require(workspace['height'] <= 844*.4 and workspace['width'] <= width, 'phone split example consumes reading area')
        split_camera = snapshot(mobile)
        scroll_to(mobile, '#constant-volume-cap-geometry')
        display(metrics(mobile), 'Contact angle', 30)
        same_camera(split_camera, snapshot(mobile), 'phone split scroll camera')
        no_overflow(mobile, f'phone {width} split')
        picture(f'teaching-phone-{width}-split.jpg', mobile)
        mobile.locator('[data-reading-mode=example]').click()
        settle(mobile)
        require(not mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),
                'phone example mode visibility')
        follow_state, follow_metrics = snapshot(mobile), metrics(mobile)
        mobile.select_option('#lab', 'slip')
        choose(mobile, 2)
        display(metrics(mobile), 'Fluid slip speed', 1/3)
        no_overflow(mobile, f'phone {width} example')
        picture(f'teaching-phone-{width}-example.jpg', mobile)
        mobile.locator('#follow-reading').click()
        settle(mobile)
        require(mobile.locator('body').get_attribute('data-reading-mode') == 'example',
                'phone follow unexpectedly changed viewing mode')
        require(snapshot(mobile)['lab'] == follow_state['lab'] and metrics(mobile) == follow_metrics,
                'phone follow in example mode failed to restore reading preset')
        mobile.locator('[data-reading-mode=text]').click()
        require(mobile.locator('.reading-text').is_visible(), 'phone return to reading')
        mobile_receipt.append({'width': width, 'modes': ['text', 'split', 'example'],
                               'chapter_search_and_navigation': True, 'no_horizontal_overflow': True,
                               'readable_text': True, 'example_control_comparison': True,
                               'mode_preserves_lesson_and_scroll': True,
                               'split_scroll_selects_preset': True,
                               'follow_in_example_restores_reading_preset': True})
        check_errors()
    mobile.set_viewport_size({'width': 390, 'height': 844})
    check_errors()
    return {'schema': 'rheon-teaching-browser-v1', 'reference_sha256': hashlib.sha256(reference.read_bytes()).hexdigest(),
            'numerical': numerical, 'desktop_columns': True, 'desktop_sticky_example': True,
            'scroll_stops': stop_count, 'manual_comparison_pauses_follow': True, 'follow_resumes_current_section': True,
            'camera_drag_and_keyboard': True, 'camera_preserved_during_scroll_and_controls': True,
            'camera_reset_and_zoom': True, 'wheel_does_not_zoom': True,
            'compact_proof_disclosure_and_deep_link': True, 'chapter_source_links': True,
            'mobile': mobile_receipt, 'fluid_solves': 0, 'fluid_advances': 0, 'screenshots': screenshots}
