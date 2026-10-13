"""Authored teaching stops; live steps require an explicit control action."""
import html
import re

# Heading IDs come from the unchanged authored chapters. Presets select actual
# stored cases (projection alone computes an explicitly labelled field blend).
CHAPTER_STOPS = {
    '03': [('divergence-from-flux-balance', 'projection', 'before'),
           ('the-adjoint-identity', 'projection', 'after'),
           ('layout-and-indexing', 'sampler', 'correct')],
    '04': [('the-pressure-quadratic-form', 'projection', 'before'),
           ('residual-as-a-diagnostic', 'projection', 'half'),
           ('acceptance-in-physical-space', 'projection', 'after')],
    '05': [('jacobi-as-a-reference', 'projection', 'before'),
           ('residual-and-warm-starts', 'projection', 'after')],
    '06': [('numerical-diffusion', 'transport', 'smooth'),
           ('component-aware-trilinear-sampling', 'sampler', 'correct'),
           ('failure-fixtures', 'sampler', 'wrong')],
    '07': [('overshoot-and-limiting', 'transport', 'symmetric'),
           ('executed-refinement', 'transport', 'smooth'),
           ('velocity-and-conservation', 'transport', 'asymmetric')],
    '08': [('two-cells-and-boundary-work', 'faceflux', 'work'), ('balanced-flow-is-not-zero-boundary-work', 'faceflux', 'balanced')],
    '09': [('volume-fractions', 'volume', 'left'),
           ('piecewise-linear-interface-reconstruction', 'volume', 'right')],
    '10': [('hydrostatic-balance', 'hydrostatic', 'light'),
           ('diffusion', 'viscous', 'late')],
    '12': [('restrictions', 'decay', 'threshold'),
           ('error-accounting', 'decay', 'refined'),
           ('long-horizons-and-outcomes', 'decay', 'unstable'),
           ('one-mode-three-time-updates', 'decay', 'threshold'),
           ('implicit-decay-and-fixed-time-refinement', 'decay', 'implicit'),
           ('an-energy-growing-step', 'decay', 'unstable')],
    '14': [('surface-and-volume-outputs', 'flow', 'volume'),
           ('metrics-and-integration', 'flow', 'metrics')],
    '17': [('core-ownership', 'timestep', 'ready'),
           ('storage-and-invariants', 'timestep', 'reject')],
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
CHAPTER_STOPS.setdefault('08', []).extend([('skew-samples-and-normal-gradients','reconstruction','skew'),('an-affine-correction-with-a-third-sample','reconstruction','corrected')])
CHAPTER_STOPS.setdefault('04', []).extend([('continuity-is-not-momentum-convergence','coupling','iterate'),('an-overrelaxed-two-path-counterexample','coupling','unstable')])
CHAPTER_STOPS.setdefault('05', []).extend([('two-disconnected-pressure-regions','gauges','compatible'),('global-balance-can-hide-component-errors','gauges','incompatible'),('fixed-cell-and-mean-zero-gauges','gauges','mean'),('volume-weighted-pressure-gauges','gauges','volume'),('projecting-a-source-changes-the-problem','gauges','projected'),('gauge-shifts-and-boundary-work','gauges','shifted'),('a-pressure-error-is-not-a-gauge','gauges','perturbed')])
CHAPTER_STOPS.setdefault('04', []).extend([('a-collocated-checkerboard', 'checker', 'alternating'), ('constant-pressure-control', 'checker', 'constant')])
CHAPTER_STOPS.setdefault('22', []).extend([('affine-rotation-and-shear', 'stressprobe', 'rotation'), ('boundary-work-supplies-shear', 'stressprobe', 'shear'), ('extension-control', 'stressprobe', 'extension')])

CHAPTER_STOPS.setdefault('01', []).extend([('gating-a-pressure-proposal', 'proposal', 'reject'), ('a-gauge-shift-is-not-an-error', 'proposal', 'gauge')])
CHAPTER_STOPS.setdefault('02', []).extend([('wall-forces-and-body-work', 'wallload', 'work')])
CHAPTER_STOPS.setdefault('05', []).extend([('two-unknowns-and-a-real-iteration', 'jacobi', 'convergent'), ('a-positive-matrix-with-an-unstable-iteration', 'jacobi', 'unstable')])
CHAPTER_STOPS.setdefault('11', []).extend([('one-grid-sample-two-particles', 'particletransfer', 'pic'), ('preserving-an-unresolved-transfer-mode', 'particletransfer', 'flip')])
CHAPTER_STOPS.setdefault('13', []).extend([('counting-faces-and-sparse-arrays', 'memory', 'small'), ('doubling-resolution-and-scalar-width', 'memory', 'large')])
CHAPTER_STOPS.setdefault('15', []).extend([('a-unit-lost-in-a-reduction', 'roundoff', 'lost'), ('cancel-the-large-pair-first', 'roundoff', 'cancel')])
CHAPTER_STOPS.setdefault('16', []).extend([('inspect-the-premise-before-the-result', 'proposal', 'reject')])
CHAPTER_STOPS.setdefault('18', []).extend([('midpoint-refinement-with-a-known-answer', 'refinement', 'clean'), ('refining-a-biased-model', 'refinement', 'biased')])
CHAPTER_STOPS.setdefault('19', []).extend([('signed-work-at-a-moving-wall', 'wallload', 'reverse')])
CHAPTER_STOPS.setdefault('22', []).extend([('a-specified-sst-2003-limiter', 'sstprobe', 'limited'), ('the-zero-k-constitutive-limit', 'sstprobe', 'laminar')])
CHAPTER_STOPS.setdefault('A-', []).extend([('execute-the-iteration-contract', 'jacobi', 'convergent')])
CHAPTER_STOPS.setdefault('B-', []).extend([('reproduce-an-arithmetic-counterexample', 'roundoff', 'lost')])
CHAPTER_STOPS.setdefault('C-', []).extend([('read-a-model-variant-before-using-it', 'sstprobe', 'limited')])
CHAPTER_STOPS.setdefault('D-', []).extend([('units-in-a-force-and-power-ledger', 'wallload', 'work')])
CHAPTER_STOPS.setdefault('E-', []).extend([('a-dirichlet-matrix-and-its-error-modes', 'jacobi', 'unstable')])
CHAPTER_STOPS.setdefault('F-', []).extend([('proof-assumptions-and-runtime-admission', 'proposal', 'nonfinite')])

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
TRANSPORT_LESSONS = [
    ('sampler', 'Put the sample on its own lattice', 'correct', 'wrong',
     'Inspect eight donors of q(x,y,z) = 1.2 + 0.7x − 1.1y + 0.4z. Select cell, x-, y- or z-face storage, then move the sample. The physical grid spacing stays (0.2, 0.3, 0.4).',
     'Matching lattice offsets reproduce the direct affine value to roundoff. Now keep x-face storage and use a cell-centre sampling offset: the signed error becomes −0.07. Positive weights summing to one do not detect this geometry mistake. Orange marks the requested point; the teal point is the weighted donor position. They coincide only with the matching offset.',
     'This browser recomputes the existing companion affine fixture with a complete regular stencil. It does not test obstacle extensions, native sampling execution or IEEE proof obligations. The donor table exposes all eight weights and values.', '06-transport'),
    ('transport', 'Bound the result, then check its total', 'symmetric', 'asymmetric',
     'Compare the linear departure sample, an unlimited forward/backward correction, and replacement by the linear value whenever the correction leaves its two-donor bounds. With the six-sample symmetric edge and Courant 0.5, predict the overshoot before inspecting the curves.',
     'The raw result reaches −0.125 and 1.125; two cells fall back. Now use the asymmetric edge at Courant 0.25: one fallback restores bounds, but the sum changes from 1.6 to 1.61875. Range preservation is not conservation. Try smooth sine, 64 samples, Courant 0.5 and 128 steps for one period; linear RMS error is about 0.101090 and limited error about 0.009022.',
     'Periodic 1D scalar reference with prescribed constant velocity, independently recomputed from the companion equations. The 3D depth axis only separates curves; it is not a third fluid dimension. Every method has the same fixed value scale. RMS uses analytic sine translation, or translation of the initial piecewise-linear interpolant for edge fixtures. Sum is unweighted; divide its change by sample count for a unit-period integral. No native midpoint/clamped transport or corrected native API is claimed.', '07-corrected-advection'),
]


TIMESTEP_LESSONS = [
    ('timestep', 'Publish one complete timestep', 'ready', 'reject', '', '', '', '17-rust-implementation'),
]


PRACTICAL_LESSONS = [(key,title,'','','','','',target) for key,title,target in [('wallload','Close wall force and work','02-continuum-equations'),('jacobi','Execute a sparse iteration','05-linear-solvers'),('memory','Count allocation inputs','13-memory-and-parallelism'),('roundoff','Inspect reduction order','15-floating-point'),('particletransfer','Inspect PIC/FLIP transfer','11-particle-methods'),('sstprobe','Specify the constitutive limiter','22-viscosity-and-stress'),('refinement','Separate mesh error and bias','18-validation-program'),('proposal','Gate a pressure proposal','01-purpose-and-model')]]

ENGINEERING_LESSONS = [(key,title,'','','','','',target) for key,title,target in [('reconstruction','Reconstruct a skew gradient','08-boundary-geometry'),('coupling','Separate continuity from momentum','04-pressure-projection'),('gauges','Separate compatibility from pressure gauge','05-linear-solvers'),('faceflux','Balance face flux and boundary work','08-boundary-geometry'),('checker','Expose collocated checkerboarding','04-pressure-projection'),('stressprobe','Separate rotation from strain','22-viscosity-and-stress')]]

DECAY_LESSONS = [('decay', 'Separate stability from decay accuracy', 'threshold', 'refined', '', '', '', '12-time-and-error')]

VOLUME_LESSONS = [('volume', 'Intersect liquid with a swept face slab', 'left', 'right', '', '', '', '09-liquid-surfaces')]

FLOW_LESSONS = [
    ('flow', 'Follow a 3D source pulse', 'volume', 'metrics', '', '', '', '14-guidance-rendering'),
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


def workspace(prefix='', lessons=None):
    options = ''.join(f'<option value="{key}">{i+1} · {html.escape(title)}</option>' for i, (key, title, *_) in enumerate(lessons or LESSONS + TRANSPORT_LESSONS + TIMESTEP_LESSONS + FLOW_LESSONS + VOLUME_LESSONS + DECAY_LESSONS + ENGINEERING_LESSONS + PRACTICAL_LESSONS))
    return f'''<section class="example-workspace" aria-label="Interactive example">
<div class="example-toolbar"><label for="lab">Example</label><select id="lab">{options}</select>
<button id="follow-reading" aria-pressed="true">Follow reading</button></div>
<p id="reading-context" role="status">Chapter introduction</p>
<div id="scene" tabindex="0" aria-label="3D example. Drag or use arrow keys to rotate. Use zoom buttons to zoom."><div id="plot-legend" hidden></div></div>
<div class="camera-toolbar"><button id="zoom-in" aria-label="Zoom in">+</button><button id="zoom-out" aria-label="Zoom out">−</button><button id="reset-view">Reset camera</button><span id="render-status" role="status">Loading 3D…</span></div>
<div class="lab-panel"><p class="example-kind">Reference model · no fluid timesteps</p><h2 id="lab-title"></h2><p id="lab-description"></p>
<div id="controls"></div><dl id="metrics" aria-live="polite"></dl>
<details class="example-notes"><summary>Expected result & limits</summary><p id="lab-expected"></p><p id="lab-limit"></p></details>
<a id="chapter-target" href="{prefix}chapters/04-pressure-projection.html">Read the mechanism →</a>
<details class="example-sources"><summary>Source & evidence</summary><span id="reference-sources"><a href="{prefix}reference.py">Reference code</a> · <a href="{prefix}reference-data.json">Stored fields</a> · <a href="{prefix}reference-qualification.json">Numerical receipt</a> · </span><span id="practical-sources" hidden><a href="{prefix}practical_reference.js">Computed mechanisms</a> · <a href="{prefix}chapters/18-validation-program.html#refining-a-biased-model">Worked error scope</a> · </span><span id="completion-sources" hidden><a href="{prefix}completion_reference.js">Completion mechanisms</a> · <a href="{prefix}chapters/08-boundary-geometry.html#skew-samples-and-normal-gradients">Worked scope</a> · </span><span id="gauge-sources" hidden><a href="{prefix}gauge_reference.js">Gauge equations</a> · <a href="{prefix}chapters/05-linear-solvers.html#two-disconnected-pressure-regions">Worked scope</a> · </span><span id="engineering-sources" hidden><a href="{prefix}engineering_reference.js">Engineering equations</a> · <a href="{prefix}chapters/08-boundary-geometry.html#two-cells-and-boundary-work">Worked scope</a> · </span><span id="decay-sources" hidden><a href="{prefix}decay_reference.js">Modal equations</a> · <a href="{prefix}chapters/12-time-and-error.html#checked-modal-amplification-proof-bridge">Research and proof scope</a> · </span><span id="volume-sources" hidden><a href="{prefix}volume_reference.js">Volume equations</a> · <a href="{prefix}chapters/09-liquid-surfaces.html#checked-volume-flux-proof-bridge">Proof scope and research</a> · </span><span id="transport-sources" hidden><a href="{prefix}transport_reference.js">Browser equations</a> · <a href="{prefix}transport-companion.py">Transport companion</a> · <a href="{prefix}sampling-companion.py">Sampling companion</a> · </span><span id="timestep-sources" hidden><a href="{prefix}solver-source.rs">Rust adapter</a> · <a href="{prefix}timestep-contract.rs">Existing native fixture</a> · <a href="{prefix}solver-presentation.json">Build identity</a> · <a href="{prefix}BROWSER_TIMESTEP.md">Capability audit</a> · </span><span id="flow-sources" hidden><a href="{prefix}flow-source.rs">Rust 3D fixture</a> · <a href="{prefix}flow_volume.js">Volume equations</a> · <a href="{prefix}flow_field.js">Velocity & lighting equations</a> · <a href="{prefix}solver-presentation.json">Build identity</a> · </span><a href="{prefix}chapters/F-expansion-contracts-and-sources.html">Primary sources & checked claims</a></details></div></section>'''


def tour():
    body = '<article class="reading-text"><p class="eyebrow">SIX MECHANISMS</p><h1>Read. Predict. Compare.</h1><p>Each example isolates one question. Scrolling selects a saved reference state; controls let you test the comparison. No example advances a live fluid solver.</p>'
    for i, (key, title, start, end, teaches, expected, limit, target) in enumerate(LESSONS):
        body += f'''<section class="teaching-lesson" id="{key}" data-lesson="{key}" data-preset="{start}"><p class="eyebrow">{i+1:02d} / 06</p><h2>{title}</h2><p>{teaches}</p>
<h3 id="{key}-compare" data-lesson="{key}" data-preset="{end}">Predict, then compare</h3><p>{expected}</p><p><strong>Scope.</strong> {limit}</p><p><a href="chapters/{target}.html">Read the derivation →</a></p></section>'''
    body += '<h2>Continue with recomputed mechanisms</h2><p><a href="transport-labs.html">Sampling and transport</a>: inspect lattice offsets, overshoot and the limits of fallback.</p><h2>Continue with qualified native records</h2><p><a href="native-labs.html">Wall and force sequence</a> · <a href="obstacle-flow-lab.html">Obstacle pressure and reduced shear</a> · <a href="aligned-strain-lab.html">Finite strain</a>. Each page states whether its required qualified packet is included.</p></article>'
    return body + workspace(lessons=LESSONS)


def transport_tour():
    body = '<article class="reading-text"><p class="eyebrow">SAMPLING → TRANSPORT</p><h1>Check the weights. Check the total.</h1><p>Two browser-recomputed references connect chapters 3, 6 and 7. Each control evaluates the displayed equations immediately; no animation advances a hidden fluid solver.</p>'
    for i, (key, title, start, end, teaches, expected, limit, target) in enumerate(TRANSPORT_LESSONS):
        body += f'''<section class="teaching-lesson" id="{key}" data-lesson="{key}" data-preset="{start}"><p class="eyebrow">{i+1:02d} / 02</p><h2>{title}</h2><p>{teaches}</p>
<h3 id="{key}-compare" data-lesson="{key}" data-preset="{end}">Predict, then compare</h3><p>{expected}</p><p><strong>Scope.</strong> {limit}</p><p><a href="chapters/{target}.html">Read the derivation →</a></p></section>'''
    return body + '<p><a href="labs.html">Return to the six physical reference mechanisms →</a></p></article>' + workspace(lessons=TRANSPORT_LESSONS)


def timestep_tour():
    body = '''<article class="reading-text"><p class="eyebrow">RUST → WEBASSEMBLY → ACCEPTED STATE</p><h1>Compute a step. Publish it whole.</h1>
<section class="teaching-lesson" id="timestep" data-lesson="timestep" data-preset="ready"><h2>Begin with a prediction</h2><p>Four unit cells form a 2 × 2 × 1 sealed box, initially at rest, with density 1 kg/m³. A prescribed acceleration of 2 m/s² acts in the x direction on the lower row for 0.25 s. Only its interior x face receives an impulse. Without pressure correction, that face would reach 0.5 m/s and the cell fluxes would not balance.</p><p>Predict the circulation that can remain after pressure correction. The orange region marks the force support. Teal arrows show accepted velocity at a fixed scale; there is no interpolated animation.</p></section>
<section class="teaching-lesson" id="timestep-accept" data-lesson="timestep" data-preset="accept"><h2>Run one actual timestep</h2><p>Press <strong>Run one step</strong>. The existing Rust <code>Simulation::step_with_forces</code> executes in a browser worker through WebAssembly: velocity advection, force application, pressure solve, corrected-field acceptance and tracer update precede publication. Advection starts from rest and tracer stays zero in this fixture.</p><p>The forced interior vector is q = (½, 0, 0, 0). The divergence-free cycle is c = (1, −1, −1, 1). Its projection is c(q·c)/(c·c) = c/8. Compare all four speeds with (+⅛, −⅛, −⅛, +⅛) m/s. Force-stage work is ⅛ J; accepted kinetic energy is 1/32 J. The pressure report and stored-field divergence measure different stages.</p><p>Both listed pressure methods use the existing settings and acceptance gates. Changing the method starts a new run. Only one step may be accepted per reset: repeating the force would activate nontrivial advection and invalidate this first-step oracle.</p></section>
<section class="teaching-lesson" id="timestep-transaction" data-lesson="timestep" data-preset="reject"><h2>A rejected candidate is not a new state</h2><p>Press <strong>Reject before commit</strong>, before or after acceptance. The real native callback rejects at <code>BeforeCommit</code>. The candidate is discarded; every accepted field bit, time and generation must stay unchanged. This is a scheduled rejection probe, not a promise that JavaScript can interrupt a synchronous WASM call.</p><p>After rejection from rest, a retry should match an uninterrupted first step. <strong>Reset state</strong> uses the native reset operation: it clears the fields and time while incrementing generation. Scrolling, switching phone layouts and moving the camera preserve the simulation state.</p><p><strong>Scope.</strong> This is one fixed-box force/projection integration fixture. It demonstrates no evolving liquid surface, viscosity, buoyancy, variable density, moving geometry or contact physics. The root solver and pressure API are unchanged. Native and browser agreement for this fixture does not qualify arbitrary grids or all WebAssembly engines.</p><p><a href="chapters/17-rust-implementation.html#storage-and-invariants">Read state ownership and publication →</a> · <a href="BROWSER_TIMESTEP.md">Inspect the capability audit and theory visualization backlog</a></p></section></article>'''
    return body + workspace(lessons=TIMESTEP_LESSONS)


def flow_tour():
    body = '''<article class="reading-text"><p class="eyebrow">COMPUTED IN THREE DIMENSIONS</p><h1>Follow the pulse.</h1><p><a href="flow-lab.html?view=example">Open compact example →</a></p>
<section class="teaching-lesson" id="flow" data-lesson="flow" data-preset="volume"><h2>A force, a source, a sealed box.</h2><p>A compact source injects tracer for the first 30 accepted steps. A localized oblique force drives flow through a unit box. Press <strong>Run</strong> to advance the existing Rust solver in a Web Worker. Each visible state contains a complete 24 × 24 × 24 concentration field and three staggered velocity components.</p><p>Predict where the pulse travels after injection stops. The amber outline marks the injection region. It is a source, not a solid obstacle. The box walls block normal flow. Accepted simulation time advances only when a native step passes its pressure, divergence and Courant checks; wall-clock playback speed depends on the device.</p></section>
<section class="teaching-lesson" id="flow-depth" data-lesson="flow" data-preset="volume"><h2>Rotate the data, not a flat image.</h2><p>Orbit the volume to inspect different depths of the same accepted field. Perspective rays sample the actual three-dimensional tensor. Trilinear interpolation smooths its display; it does not add numerical resolution. The fixed opacity law integrates concentration along each ray. Overhead lighting uses that same concentration field as a depth cue without changing opacity. Camera motion and display rendering never advance the simulation.</p><p>The curves trace the current staggered velocity field from six fixed seeds. Their colour uses a fixed 0–1 m/s speed scale. They are instantaneous streamlines: each curve follows the velocity at this accepted time, not the past or future path of a particle. Toggle lighting or curves in the inspector to compare the same data. No procedural smoke, synthetic particles or liquid surface is added.</p></section>
<section class="teaching-lesson" id="flow-limits" data-lesson="flow" data-preset="metrics"><h2>Measure what the picture cannot tell you.</h2><p>Open the inspector to compare accepted time, divergence, Courant number, force work and tracer amount. After the source switches off, the tracer amount can decline: bounded appearance transport is not a conservative liquid-volume update. The run ends after 80 accepted steps; reset starts a new experiment.</p><p><strong>Scope.</strong> Density is constant. The force is prescribed, with no viscosity, physical buoyancy, free surface or moving geometry. The fixed-obstacle pressure operator exists separately, but is not composed into this timestep. An impermeable internal obstacle therefore requires further solver integration and qualification. No coupled obstacle collision is depicted.</p><p><a href="chapters/14-guidance-rendering.html#surface-and-volume-outputs">Read rendering and output →</a> · <a href="timestep-lab.html">Inspect the exact first-step contract →</a></p></section></article>'''
    return body + workspace(lessons=FLOW_LESSONS)


def scripts(prefix=''):
    # Import-map addresses must be URLs, including at the site root.
    prefix = prefix or './'
    return f'''<script type="importmap">{{"imports":{{"three":"{prefix}vendor/three.module.js","three/addons/controls/OrbitControls.js":"{prefix}vendor/OrbitControls.js"}}}}</script><script type="module" src="{prefix}reader.js"></script>'''
