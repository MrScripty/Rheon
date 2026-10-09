"""HTML body for the qualified, immutable aligned-strain teaching packet.

The builder owns packet validation, source binding and publication. This module
only renders the supplied packet and exposes its native rows to the browser.
"""
import html
import json

OPERATOR_COMMIT = 'f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b'
PROOF = f'https://github.com/MrScripty/Rheon/blob/{OPERATOR_COMMIT}/proofs/Rheon/AlignedStrain.lean'


def render_lab(packet):
    """Return the body; the enclosing book shell loads aligned_strain.css/js."""
    if packet is None:
        return '<p class="eyebrow">FINITE STRAIN LABORATORY</p><h1>See strain become viscous force.</h1><p>A source-bound native packet is required for this laboratory. This edition has no qualified specimen.</p>'
    payload = json.dumps(packet, separators=(',', ':'), allow_nan=False).replace('<', '\\u003c').replace('>', '\\u003e').replace('&', '\\u0026')
    source = packet.get('source', {})
    commit = html.escape(str(source.get('commit', '')), quote=True)
    return f'''<div class="strain-lab" id="aligned-strain-lab">
<p class="eyebrow">FINITE STRAIN / INTERACTIVE OPERATOR LAB</p>
<h1>See strain become<br>viscous force.</h1>
<p class="lede">Change the velocity samples. Watch the same retained strain rows gather a deformation, then scatter a resisting force back to the faces.</p>
<div class="strain-scope"><span class="strain-dot"></span><strong>One stationary, aligned box.</strong> Sealed free-slip outer walls · no-slip obstacle · no time advancement</div>
<section class="strain-workbench" aria-label="Aligned strain interactive laboratory">
  <div class="strain-visual">
    <div class="strain-scene-heading"><div><span class="strain-kicker">THE ACTUAL MAC SPECIMEN</span><h2>Face samples, cell strains.</h2></div><label class="strain-slice-label" for="strain-slice">z layer <select id="strain-slice" aria-label="Displayed z cell-center layer"></select></label></div>
    <svg id="strain-view" viewBox="0 0 600 560" role="img" aria-labelledby="strain-view-title strain-view-description"><title id="strain-view-title">Aligned obstacle velocity and strain slice</title><desc id="strain-view-description">Velocity arrows sit on active grid faces. The filled box is stationary. A selected row shows its actual support and its fluid quadrants.</desc></svg>
    <div class="strain-legend"><span><i class="strain-key strain-key-x"></i>uₓ</span><span><i class="strain-key strain-key-y"></i>uᵧ</span><span><i class="strain-key strain-key-z"></i>u_z, near/far z face</span><span><i class="strain-key strain-key-heat"></i>selected sector loss</span></div>
    <p class="strain-caption" id="strain-scene-caption">Arrows show the active samples. Prescribed wall traces are eliminated zeros.</p>
  </div>
  <div class="strain-controls">
    <span class="strain-kicker">TRY A VELOCITY FIELD</span>
    <label for="strain-preset">Sample pattern</label><select id="strain-preset"></select>
    <p id="strain-preset-description" class="strain-control-note"></p>
    <label for="strain-corner">Actual corner patch</label><select id="strain-corner"></select>
    <label for="strain-amplitude"><span id="strain-amplitude-label">U · x-face speed</span><output id="strain-amplitude-value" for="strain-amplitude"></output></label>
    <input id="strain-amplitude" type="range" min="-2" max="2" step="0.05" value="1">
    <label for="strain-secondary"><span id="strain-secondary-label">V · y-face speed</span><output id="strain-secondary-value" for="strain-secondary"></output></label>
    <input id="strain-secondary" type="range" min="-2" max="2" step="0.05" value="0.5">
    <div class="strain-material"><label for="strain-mu">Dynamic viscosity μ <output id="strain-mu-value" for="strain-mu"></output></label><input id="strain-mu" type="range" min="0" max="2" step="0.05" value="1"><p>Viscosity scales the force and dissipation. It leaves the geometric rows unchanged.</p></div>
    <label class="strain-force-control" for="strain-show-force"><span>Overlay viscous force</span><input id="strain-show-force" type="checkbox"></label>
    <button id="strain-reset" type="button">Reset this pattern</button>
    <div class="strain-live-note" role="status" id="strain-status">Loading qualified native rows…</div>
  </div>
</section>
<section class="strain-metric-grid" aria-label="Live force and strain ledger" aria-live="polite">
  <article><span>NORMAL LOSS</span><strong id="strain-normalD" data-value="0">—</strong><small>μ Σ normal 2V (∂ₐuₐ)²</small></article>
  <article><span>ENGINEERING SHEAR LOSS</span><strong id="strain-shearD" data-value="0">—</strong><small>μ Σ sectors v_q γₐᵦ²</small></article>
  <article class="strain-total"><span>TOTAL DISSIPATION</span><strong id="strain-D" data-value="0">—</strong><small>D = normal + shear · watts</small></article>
  <article><span>VISCOUS FORCE WORK</span><strong id="strain-work" data-value="0">—</strong><small>u · f = −D · watts</small></article>
</section>
<section class="strain-inspection" aria-label="Inspect an actual retained row">
  <div class="strain-row-panel" id="strain-inspector">
    <span class="strain-kicker">FOLLOW ONE ROW</span><h2>Gather, weight, scatter.</h2>
    <label for="strain-row">Retained quadrature row</label><select id="strain-row"></select>
    <p class="strain-row-location" id="strain-row-location"></p>
    <div class="strain-formula" id="strain-row-formula"></div>
    <dl class="strain-row-values" id="strain-row-values"></dl>
    <div class="strain-table-wrap"><table class="strain-term-table"><thead><tr><th>Active face</th><th>E coefficient</th><th>Velocity</th><th>E u</th></tr></thead><tbody id="strain-row-terms"></tbody></table></div>
    <p id="strain-row-boundary" class="strain-control-note"></p>
  </div>
  <div class="strain-corner-panel">
    <span class="strain-kicker">WHY THE CORNER COUPLES COMPONENTS</span><h2>Three fluid quadrants.<br>Two velocity samples.</h2>
    <div class="strain-corner-matrix"><span>K<sub>corner</sub> =</span><span class="strain-matrix">2&nbsp;&nbsp;1<br>1&nbsp;&nbsp;2</span></div>
    <p>After reflecting local coordinates and components to put the solid southwest, the unit sector rows are <strong>2U + 2V</strong>, <strong>2U</strong> and <strong>2V</strong>, each weighted by ¼ m³.</p>
    <p class="strain-corner-equation">D<sub>corner</sub> / μ = ¼(2U + 2V)² + ¼(2U)² + ¼(2V)²<br>= 2U² + 2UV + 2V²</p>
    <p>The shared northeast sector creates the cross term. In those reflected local coordinates, the reconstruction <code>uₓ = U max(y,0)/ℓᵧ</code>, <code>uᵧ = V max(x,0)/ℓₓ</code> vanishes on the two solid half-walls.</p>
    <button type="button" id="strain-focus-corner">Inspect the coupled corner row</button>
    <p id="strain-corner-live" class="strain-control-note"></p>
  </div>
</section>
<section class="strain-explanation"><h2>The displayed algebra is the operator.</h2>
<div class="strain-explanation-grid"><article><h3>1. Gather a strain</h3><p>Each row evaluates <code>sᵣ = Σ Eᵣf u_f</code>. Normal rows use <code>∂ₐuₐ</code>. Engineering shear uses <code>γₐᵦ = ∂ᵦuₐ + ∂ₐuᵦ</code>; the symmetric tensor entry is <code>εₐᵦ = γₐᵦ/2</code>.</p></article>
<article><h3>2. Weight its square</h3><p>Normal rows carry <code>2V</code>. Each shear row carries its actual fluid sector volume <code>v_q</code>. Thus <code>D = μΣ wᵣsᵣ²</code>. The shear term is <code>μ v_q γ²</code>, including the two symmetric off-diagonal tensor entries.</p></article>
<article><h3>3. Scatter a force</h3><p>The same coefficients scatter <code>Ku = EᵀWEu</code>. The stationary viscous force is <code>f = −μKu</code> in newtons. Exact finite algebra gives <code>u·f = −D</code>.</p></article></div>
<p class="strain-numerical-ledger">Browser force/work identity <code>u·f + D</code>: <strong id="strain-identity" data-value="0">—</strong> W. Kinetic energy with stored <code>m_f = ρA_fd_f</code>: <strong id="strain-energy" data-value="0">—</strong> J. Coefficient estimate B: <strong id="strain-B" data-value="0">—</strong> m/kg, <strong>UNENCLOSED</strong>. These nearest-rounded diagnostics authorize no timestep.</p>
<p>Local rotation can cancel an interior shear row. The surrounding fixed traces can still produce global loss. The affine patterns sample the active faces; prescribed obstacle and outer normal speeds stay zero.</p>
</section>
<details class="strain-evidence"><summary>Assumptions, native source and exact Lean statements</summary>
<p>This specimen is one exactly aligned internal box with at least one fluid cell of padding. The immutable native operator supplies every row, active face and stored mass. Outer walls impose <code>uₙ = 0</code> and free-slip tangential traces. The obstacle imposes stationary no slip. Outer shear strains vanish analytically before enumeration; zero rows on included samples remain present.</p>
<p>These controls evaluate fixed finite rows. They do not advance a fluid state or invoke pressure projection. The exact Lean theorems treat stored coefficients, weights and masses as real inputs; they do not prove Rust assembly, JavaScript arithmetic, IEEE error enclosure or continuum convergence.</p>
<ul><li><a href="{PROOF}#L15">gather and strainOperator — finite sums</a></li><li><a href="{PROOF}#L54">transpose_work — uᵀKu = D</a></li><li><a href="{PROOF}#L59">loss_nonnegative — fixed nonnegative weights</a></li><li><a href="{PROOF}#L256">unit_corner_block — the coupled quadratic form</a></li><li><a href="proofs/AlignedStrain.lean">Download local Lean source</a> · <a href="implementation/reconstructed-aligned-strain.html">Read the reconstruction and boundary derivation</a></li></ul>
<p><a href="aligned-strain-packet.json">Inspect the source-bound specimen packet</a> · <a href="https://github.com/MrScripty/Rheon/blob/{OPERATOR_COMMIT}/src/aligned_strain.rs">Immutable native operator source</a>. Packet capture commit: <code>{commit}</code>.</p>
<p id="strain-provenance"></p>
</details>
</div><script>window.ALIGNED_STRAIN_PACKET={payload};</script>'''
