"""Authored teaching stops over the existing reference data; no solver calls."""
import html
import re

# Heading IDs come from the unchanged authored chapters. Presets select actual
# stored cases (projection alone computes an explicitly labelled field blend).
CHAPTER_STOPS = {
    '03': [('divergence-from-flux-balance', 'projection', 'before'),
           ('the-adjoint-identity', 'projection', 'after')],
    '04': [('the-pressure-quadratic-form', 'projection', 'before'),
           ('residual-as-a-diagnostic', 'projection', 'half'),
           ('acceptance-in-physical-space', 'projection', 'after')],
    '05': [('jacobi-as-a-reference', 'projection', 'before'),
           ('residual-and-warm-starts', 'projection', 'after')],
    '10': [('hydrostatic-balance', 'hydrostatic', 'light'),
           ('diffusion', 'viscous', 'late')],
    '19': [('units-before-parameters', 'projection', 'before'),
           ('pressure-and-external-work', 'projection', 'after')],
    '20': [('admission-and-representation', 'collision', 'crossing'),
           ('earliest-path-collision', 'collision', 'miss')],
    '21': [('constant-material-and-multiple-phases', 'hydrostatic', 'light'),
           ('hydrostatic-oracle', 'hydrostatic', 'heavy')],
    '22': [('symmetric-strain', 'viscous', 'initial'),
           ('shared-shear-mode-oracle', 'viscous', 'late')],
    '23': [('equilibrium-surface-energy', 'cap', 'neutral'),
           ('constant-volume-cap-geometry', 'cap', 'spread')],
    '24': [('dissipative-navier-law', 'slip', 'no-slip'),
           ('couette-reference', 'slip', 'slip')],
    '25': [('pressure-force-balance', 'cap', 'neutral'),
           ('time-scales-and-resolution', 'cap', 'tension')],
}
LESSONS = [
    ('projection', 'Pressure removes incompatible flux', 'before', 'after',
     'Compare the original face field with the complete pressure correction. Cell markers show incidence; arrows keep the same speed scale.',
     'At 100%, maximum incidence approaches the stored solve residual and the unit-weight kinetic norm falls. At 50%, incidence is approximately half its starting value.',
     'The slider blends two saved fields. It is not a PCG iteration history or a moving-wall solve. The incidence has graph units, not physical divergence in s⁻¹.', '04-pressure-projection'),
    ('collision', 'Find the first crossing', 'crossing', 'miss',
     'Select a crossing, a near miss and an inside start. The orange marker is the earliest triangle intersection along the finite segment.',
     'A crossing can start and end outside yet still have a hit with 0 ≤ t ≤ 1. A near miss reports no intersection.',
     'These are static translated mesh poses with query tolerance 10⁻¹². The wall-speed arrow is prescribed metadata; no moving-boundary interval is solved. The Lean claim is for an infinite stationary plane.', '20-collision-mesh-pipeline'),
    ('hydrostatic', 'Density changes the pressure gradient', 'light', 'heavy',
     'Compare two half-metre layers at rest. All cases use one pressure-arrow scale, so a larger bottom pressure makes a longer arrow.',
     'Bottom gauge pressure equals 9.81 × (ρlower / 2 + ρupper / 2) Pa. Compare the measured readout with this independent formula.',
     'The reported face balance error checks the discrete increments against a piecewise analytic oracle. No interface transport or stratification evolution is computed.', '21-density-and-liquid-volume'),
    ('viscous', 'Measure damping, not just motion', 'initial', 'late',
     'Choose viscosity and a saved time. Velocity arrows retain one scale; increasing time damps the periodic sine mode.',
     'Energy decreases. Compare the dense implicit result with the discrete eigenmode and inspect the gap from continuum decay.',
     'This is a scalar periodic shear reference extruded into 3D. The amplitude gap is a discretization effect, not a certified error bound for a general tensor or free-surface solver.', '22-viscosity-and-stress'),
    ('slip', 'Separate fluid slip from wall motion', 'no-slip', 'slip',
     'Keep the lower wall stationary and the upper wall at 1 m/s. Increase only the lower Navier slip length.',
     'Fluid slip speed rises while wall shear falls. The lower wall itself remains at 0 m/s; the upper wall remains no-slip.',
     'The analytic steady Couette model has one slipping wall. It does not stand in for the separately recorded two-wall, transient or forced native cases.', '24-slip-and-wall-friction'),
    ('cap', 'Separate wetting shape from tension', 'neutral', 'tension',
     'At fixed volume, vary contact angle to change contact radius. Then hold the angle fixed and change surface tension.',
     'Tension changes pressure jump and adhesion work, while radius and volume stay fixed. The volume integral error measures quadrature agreement.',
     'A spherical cap is a zero-gravity equilibrium oracle. It does not demonstrate relaxation, hysteresis, contact-line dynamics or stable liquid stepping.', '23-wetting-and-adhesion'),
]


def compact_proofs(body):
    """Keep exact source/proof text and anchors, collapse only checked bridges."""
    pattern = r'(<h2\b[^>]*>Checked[^<]*</h2>)(.*?)(?=<h2\b|\Z)'
    def fold(match):
        heading, content = match.groups()
        label = re.sub('<[^>]+>', '', heading)
        return f'<details class="proof-disclosure"><summary>{label}</summary>{heading}{content}</details>'
    return re.sub(pattern, fold, body, flags=re.S)


def chapter(body, slug):
    stops = CHAPTER_STOPS.get(slug[:2], [])
    body = compact_proofs(body)
    if not stops:
        return f'<article class="reading-text">{body}</article>', False
    first = stops[0]
    body = f'<div data-lesson="{first[1]}" data-preset="{first[2]}" data-label="Chapter introduction"></div>' + body
    for anchor, lab, preset in stops:
        pattern = rf'(<h2\b[^>]*id="{anchor}"[^>]*)(>)'
        if not re.search(pattern, body):
            raise ValueError(f'Missing teaching heading: {slug}#{anchor}')
        body = re.sub(pattern, rf'\1 data-lesson="{lab}" data-preset="{preset}"\2', body)
    return f'<article class="reading-text">{body}</article>{workspace("../")}', True


def workspace(prefix=''):
    options = ''.join(f'<option value="{key}">{i+1} · {html.escape(title)}</option>' for i, (key, title, *_) in enumerate(LESSONS))
    return f'''<section class="example-workspace" aria-label="Interactive example">
<div class="example-toolbar"><label for="lab">Example</label><select id="lab">{options}</select>
<button id="follow-reading" aria-pressed="true">Follow reading</button></div>
<p id="reading-context" role="status">Chapter introduction</p>
<div id="scene" tabindex="0" aria-label="3D example. Drag or use arrow keys to rotate. Use zoom buttons to zoom."></div>
<div class="camera-toolbar"><button id="zoom-in" aria-label="Zoom in">+</button><button id="zoom-out" aria-label="Zoom out">−</button><button id="reset-view">Reset camera</button><span id="render-status" role="status">Loading 3D…</span></div>
<div class="lab-panel"><p class="example-kind">Reference model · no fluid timesteps</p><h2 id="lab-title"></h2><p id="lab-description"></p>
<div id="controls"></div><dl id="metrics" aria-live="polite"></dl>
<details class="example-notes"><summary>Expected result & limits</summary><p id="lab-expected"></p><p id="lab-limit"></p></details>
<a id="chapter-target" href="{prefix}chapters/04-pressure-projection.html">Read the mechanism →</a>
<details class="example-sources"><summary>Source & evidence</summary><a href="{prefix}reference.py">Reference code</a> · <a href="{prefix}reference-data.json">Stored fields</a> · <a href="{prefix}reference-qualification.json">Numerical receipt</a> · <a href="{prefix}chapters/F-expansion-contracts-and-sources.html">Primary sources & checked claims</a></details></div></section>'''


def tour():
    body = '<article class="reading-text"><p class="eyebrow">SIX MECHANISMS</p><h1>Read. Predict. Compare.</h1><p>Each example isolates one question. Scrolling selects a saved reference state; controls let you test the comparison. No example advances a live fluid solver.</p>'
    for i, (key, title, start, end, teaches, expected, limit, target) in enumerate(LESSONS):
        body += f'''<section class="teaching-lesson" id="{key}" data-lesson="{key}" data-preset="{start}"><p class="eyebrow">{i+1:02d} / 06</p><h2>{title}</h2><p>{teaches}</p>
<h3 id="{key}-compare" data-lesson="{key}" data-preset="{end}">Predict, then compare</h3><p>{expected}</p><p><strong>Scope.</strong> {limit}</p><p><a href="chapters/{target}.html">Read the derivation →</a></p></section>'''
    body += '<h2>Continue with qualified native records</h2><p><a href="native-labs.html">Wall and force sequence</a> · <a href="obstacle-flow-lab.html">Obstacle pressure and reduced shear</a> · <a href="aligned-strain-lab.html">Finite strain</a>. Each page states whether its required qualified packet is included.</p></article>'
    return body + workspace()


def scripts(prefix=''):
    # Import-map addresses must be URLs, including at the site root.
    prefix = prefix or './'
    return f'''<script type="importmap">{{"imports":{{"three":"{prefix}vendor/three.module.js","three/addons/controls/OrbitControls.js":"{prefix}vendor/OrbitControls.js"}}}}</script><script type="module" src="{prefix}reader.js"></script>'''
