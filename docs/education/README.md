# Rheon GitHub Pages teaching edition

## Reading with examples

The desktop book places the chapter list at left, bounded-width text in the
middle and a sticky 3D reference at right. The six-part laboratory page is a
reading sequence: pressure correction, first mesh crossing, layered hydrostatics,
viscous decay, wall slip, then equilibrium capillarity. Each lesson states a
prediction, comparison and measurable limitation. The authored chapter text,
citations, figures, Lean source and numerical generators remain unchanged.

Scroll-linked examples cover chapters **3–7, 10, 14, 17, 19–25**: 15 of 25 chapters,
with no scroll-linked appendix. This is partial topic coverage. The
[coverage ledger](COVERAGE.md) lists every chapter and appendix, each source
kind, and the remaining science and UI gaps. Heading-to-case bindings live in
`teaching.py` and fail the build if an expected heading moves. The remaining
chapters/appendices retain their text and source links. The native, obstacle and
aligned-strain pages retain their existing qualified-packet gates. No absent
packet is replaced with synthetic data or claimed as a live solver.

In the original six lessons, scrolling selects an explicit saved case;
projection recomputes the labelled blend. The sampling/transport sequence below
recomputes its bounded teaching mechanisms. Changing a control pauses following.
**Follow reading** resumes at the current section. Camera drag/arrow keys, zoom
buttons and reset are independent of section changes. The wheel scrolls the
page rather than zooming.
On phones, **Text**, **Split** and **Example** select a readable text view,
compact sticky example above the text, or the complete controls. In Split, the
example panel scrolls internally to its controls and readouts.

Checked proof bridges are compact disclosures. Opening their direct section
links reveals the complete original statement. The print/PDF and Markdown
editions keep the full source text. GUI renderer/orbit/lifecycle and scoped
control theme reuse the pinned real browser viewer documented in
[`browser/viewer/README.md`](../../browser/viewer/README.md).

`teaching_browser.py` adds numerical and interaction comparisons to the existing
browser qualification. New source files and shared viewer sources are bound by
the browser receipt; changing them rejects stale qualification. Generated test
screenshots, PDF and receipts stay outside Git. The explicitly requested
[native timestep/computed-flow captures](screenshots/README.md#native-timestep-and-computed-flow)
are a documented exception. They use clean implementation commit
`db9781a1a26e43bd73fb08995fdc855d42978c12`, tree
`9e4d034fcb29163fd92ceb5119d97747bf57c04d`; their
[provenance](screenshots/research-book-native-flow/provenance.json) records
actual state, source, packet, browser and image hashes. The compact desktop
angles share identical computed fields and accepted time.
The [earlier sampling/transport captures](screenshots/README.md#current-sampling-and-transport-lessons)
and [historical six-reference captures](screenshots/README.md#historical-six-reference-checkpoint)
remain unchanged with their original provenance. Phone captures are browser
emulation, not physical devices. All images retain unaltered JPEG 85 bytes.
PR42's native-app screenshots remain separate and unchanged. No publishing
action is part of the build.

The 25-chapter book, six appendices, implementation guides, locally rendered equations, displayed Lean source, six interactive 3D reference laboratories and the connected PR22–24 native wall/force sequence. Owner: Puma. The destination is repository GitHub Pages.

The three native labs are recorded playback with distinct models: two finite Navier walls, compatible no-slip/mixed boundaries, and uniform tangential forcing with stationary no-slip walls. Their controls select saved cases/times; they do not recompute parameters or advance a live liquid simulator. The original six 3D labs remain separate analytical/stored mechanisms. The projection control displays an algebraic blend of two saved fields; camera motion only changes the view.

## Current native material and CPU contracts

The current native desktop has a separate bounded SPH material preview. Chapters
11, 13, 16 and 17 connect its equations, accepted-state lifecycle, preview/history
sampling and CPU dependency order to the actual implementation. The
[material guide](../research-book/implementation/material-preview-contracts.md)
and [CPU scheduling guide](../research-book/implementation/cpu-scheduling-contracts.md)
state the predictions, independent checks and limits. No browser material solver,
CAD wall load, GPU performance or global stability claim is added.

The reviewed Lean inventory now contains 138 public theorem statements and 193
explicit audit entries. The new material and scheduler statements require fresh
pinned kernel qualification; `native-sequence.json` records its current status
separately from the unchanged historical 106/161 reconstruction receipt.
The page builder never turns inventory matching into a kernel pass.

The aligned-strain admission path also has an explicit reviewed current-source
inventory, `aligned-strain-native-sources.json`. It covers crate/build inputs and
the independent qualification path, rejects changed/added/missing files, and
requires a fresh clean native/rational receipt whose committed sources and
inventory match. Historical native admission remains exact; changing a manifest
is not numerical qualification or authority to reuse a previous packet.

## Recomputed sampling and transport lessons

The generated `transport-labs.html` page adds a two-lesson sequence; `labs.html`
retains the original six reference mechanisms. The browser kernel is
[`transport_reference.js`](transport_reference.js), based on the existing
[component-aware sampling fixture](../research-book/companion/depth_experiments.py)
and [periodic transport formulas](../research-book/companion/experiments.py).

First select cell/x/y/z sample locations, inspect eight donor weights and
compare affine interpolation with direct evaluation. Deliberately using the
cell offset for the original x-face fixture exposes its −0.07 bias. Then
compare periodic linear transport, raw forward/backward correction and
donor-bound fallback. Inspect profile error, extrema, sample-sum change and
fallback activity; bounds alone do not establish conservation or convergence.
Chapter 3 expands its layout discussion, while chapters 6 and 7 gain linked
sampling, numerical-diffusion, limiting and refinement examples.

**Sine · one period at Courant 0.5** sets the step count to twice the selected
sample count, keeping the travelled period fixed for refinement comparisons.
The six-sample witnesses display their fixed size explicitly. A compact plot
legend remains visible in phone Split mode.

`test_transport_reference.py` compares the actual JavaScript kernel with an
independent Python/Fraction oracle, including reversion versus clamping and the
four historical sine refinements. `transport_browser.py` compares 12 sampling
states (96 donors), 10 transport states (30 fixed-scale curves), four tour stops,
seven chapter stops, 11 desktop deep links and 390/320-pixel phone modes. Its
receipt binds both published JavaScript modules and both companion source
copies; the publication gate rejects missing checks or stale sources.

These are browser-recomputed educational mechanisms. The native
[tracer transport](../../src/advection.rs) uses midpoint sampling with a
[fixed-box extension](../../src/sampling.rs); it has no corrected-advection
fallback API. Native smoke/liquid stepping exists. The separate fixed force/projection
lesson below now binds one existing native timestep; it does not turn these
scalar teaching mechanisms into native advection. The existing aligned-strain page also recomputes an operator without
advancing a fluid. See [precise proof boundaries and blockers](COVERAGE.md#proof-boundaries-and-remaining-blockers).

## One real browser timestep

`timestep-lab.html` and chapter 17 now share one fixed-box Rust integration
fixture. A separate [adapter crate](../../browser/solver/README.md) compiles the
unchanged core to WebAssembly and runs it in a Web Worker. **Run one step** calls
`Simulation::step_with_forces`; scrolling, camera motion and reading modes never
advance or reset native state. The fixed 2 × 2 × 1 force cycle starts from rest
and admits one interval per reset. It compares four interior speeds against
(⅛, −⅛, −⅛, ⅛) m/s, force-stage work ⅛ J and accepted energy 1/32 J.

**Reject before commit** invokes the real native cancellation callback at a
predetermined stage. It preserves all accepted field/time/generation bits,
including after a nonzero accepted state. It is not an asynchronous interrupt.
The UI displays actual residual/divergence reports, retains accepted values on
rejection, and uses native reset semantics. No pressure samples are fabricated.

The [capability audit and theory visualization roster](BROWSER_TIMESTEP.md)
distinguish the now-supplied build/binding path from unavailable general physics.
First-step advection acts on rest and tracer stays zero. No viscosity, interface,
contact-line dynamics, parameterized grid or repeated trajectory is advertised.
A source-only build states that the compiled packet is absent and disables its
step controls. The browser gate compares actual WASM state with the independently
derived cycle and a fresh host execution of the existing fixture for both methods.

## Computed 3D source pulse

`flow-lab.html` (or `flow-lab.html?view=example` for a compact embeddable view)
and chapter 14 add a bounded 24³ native flow in the same worker
build. **Run** advances accepted states asynchronously; **Pause** stops scheduling
new steps (one already running step may finish). **Reset** clears native state.
The source injects tracer for 30 steps and the experiment ends at 80 accepted
steps. Actual time, grid size and progress remain visible; the numerical ledger
and source links share one collapsed inspector. The camera survives accepted updates and reading-mode
changes. No step starts on page load or section scroll.

The volume uses the full accepted three-dimensional concentration tensor,
trilinear display interpolation and a fixed opacity law. It contains no procedural
noise, synthetic particle trajectory or inferred water surface. Inspect it from
two angles at the same accepted time. The renderer reuses the real GUI camera,
WebGL renderer and lifecycle without its normalized one-dimensional profile adapter.

An optional overhead depth cue attenuates colour using the same concentration
tensor; it leaves the fixed opacity integral unchanged. The optical-depth prefix
is exact at cell centres for the vertically interpolated tracer and is interpolated
between those centres for display. It is an appearance model, not calibrated
light scattering. Six fixed seeds trace the instantaneous staggered velocity
field, with a fixed 0–1 m/s colour scale. These frozen-field streamline overlays are not
particle histories or physically scattered light. Both display layers can be switched off in the inspector
without advancing or resetting native state. The responsive scene remains 24³;
interpolation, lighting and canvas pixel density add no numerical resolution.

The amber outline is the injection region. Fixed sealed domain walls are the
only impermeable geometry in this timestep. Static-obstacle pressure is available
separately but requires further composition before an advancing obstacle-flow
demo is scientifically valid. Appearance tracer can lose total amount after the
source turns off; inspect the measured change in the disclosure. No viscosity,
physical buoyancy, free-surface or obstacle-coupled evolution is claimed.

The host exporter retains full tensor references for both pressure methods.
These external numerical artifacts are downloaded by qualification only; the
interactive view computes its own states and does not load recorded fields.

## Build and preview outside Git

Requires Python 3.12, Pandoc 3.1.11.1, Node 24, the npm lock and `requirements.txt`. Three.js 0.180.0 and KaTeX 0.16.22 are pinned and copied with licenses; runtime assets are local. From the repository root, choose a fresh output path and asset directories outside every checkout. The builder refuses existing output directories and checkout ancestors:

```bash
export RHEON_EDITION=/tmp/rheon-edition
export RHEON_ASSETS=/tmp/rheon-assets
export RHEON_SOLVER=/tmp/rheon-solver
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$RHEON_ASSETS"
cp docs/education/package.json docs/education/package-lock.json "$RHEON_ASSETS/"
npm ci --prefix "$RHEON_ASSETS" --ignore-scripts --no-audit --no-fund
python3 -m pip install -r docs/research-book/expansion/requirements.txt -r docs/research-book/companion/requirements.txt -r docs/education/requirements.txt
python3 docs/education/generate_references.py
rustup target add --toolchain 1.92.0 wasm32-unknown-unknown
cargo +1.92.0 install wasm-bindgen-cli --version 0.2.105 --locked
python3 docs/education/build_solver.py --output-dir "$RHEON_SOLVER"
python3 docs/education/build.py --output-dir "$RHEON_EDITION" --asset-dir "$RHEON_ASSETS/node_modules" --solver-dir "$RHEON_SOLVER"
python3 docs/education/verify_browser.py --render-pdf --output-dir "$RHEON_EDITION"
python3 docs/education/verify_site.py --output-dir "$RHEON_EDITION"
python3 docs/education/verify_browser.py --check --output-dir "$RHEON_EDITION"
python3 -m http.server 8765 --bind 127.0.0.1 --directory "$RHEON_EDITION"
```

Install Chromium with `python3 -m playwright install --with-deps chromium`, or use installed Chromium. All qualification launchers explicitly enable the Chromium sandbox; a host without a working sandbox fails before qualification. No automatic unsandboxed fallback is provided. The browser verifier uses a private local server and checks the six reference labs, the native hub, navigation/search, math, page/network errors and mobile layout. When native bundles are supplied, it checks every saved native profile, plot, ledger and CSV link. `--check` is read-only: it cannot regenerate a PDF or bless stale receipts. `--url` selects an existing preview server.

`--render-pdf` intentionally renders the new reading PDF and writes its source/book/PDF-bound receipts in the output directory. Inspect that PDF before publication. HTML, PDF, Markdown ZIP, copied evidence, browser receipts, assets and build artifacts stay outside Git. Historical PDFs and receipts remain available in Git history with their original source identities. Generated references use the existing checked generators and ignored output locations; PDFs and browser receipts are rendered directly into the external edition before verification.

## Optional original native bundles

`native-sequence.json` declares exact accepted source identities and file hashes. Supply an outside-Git directory with `wall-friction/`, `no-slip/` and `poiseuille/` containing those exact accepted HTML/JSON/CSV files, then add `--native-labs-dir /path/to/frozen-labs` to the build command. Admission checks every file/hash; changed data or an unexpected file is refused. The builder never invokes an example or integration to create replacement records. The PR22 HTML gets a derived playback notice and responsive styling; all original input files and numerical records remain unchanged.

A source-only CI build without these packets states that the recorded bundles are absent and links their model/proof guides. It does not claim browser qualification of omitted playback. A separately qualified local preview can include all three original bundles. Workflow outputs live under `RUNNER_TEMP`; the review artifact is retained by Actions. Pages deployment runs only from main, so an unmerged source draft is not a live deployment claim.

The illustrated `Rheon-expanded-markdown.zip` includes companion figures, implementation guides and Lean sources. Extract it with those relative paths intact. The reading PDF/Markdown includes all 31 original sections and selected implementation guides explaining the native progression and requirements roadmap; the site renders the full implementation-guide collection. Site verification checks actual local HTML and packaged Markdown links, copied source hashes and PDF portability. PDF links use internal reading destinations or immutable source URLs; they cannot point to the temporary preview server.

## Proof and implementation boundaries

`proofs/source-inventory-pr24.json` preserves the accepted PR24 kernel proof bytes: 57 public theorems and 77 audited declarations. The historical reconstruction at `f75cd66` included AlignedStrain. Its fresh pinned normal `lake build`, allowed-axiom audit and rejection/source-policy probes checked 106 public theorem statements, 144 explicitly expected declarations and 161 audited declarations. `native-sequence.json` binds the immutable reconstruction source and local qualification receipt, and links the associated hosted Lean run without asserting its checkout identity. Earlier source qualifications and recorded native packet identities remain historical. The book builder checks current bytes; it does not rerun Lean or prove assembly, IEEE arithmetic, transient reference evaluation or global solver convergence.

The expansion's older 42-public/60-audit receipt remains explicitly historical. Its exact-real finite-strain and planar-contact contracts retain their assumptions. The original Python reference source/data receipts are checked before building. Source-only/data-only changes, stale PDF/browser inputs, missing assets, broken controls and page errors reject publication.

The [native progression](../research-book/implementation/native-wall-force-sequence.md) connects equations, units, conditional Lean claims and demos. The [requirements map](../research-book/implementation/requirements-roadmap.md) distinguishes existing box/liquid APIs from missing general mesh/interface composition. The bounded static closed box owner supplies shared fluid volumes, open areas, connectivity and conservative flux. [Sealed obstacle pressure and reduced periodic shear](../research-book/implementation/static-obstacle-flow.md) consume that owner with distinct boundary models; they do not compose a general moving-interface tensor-stress solver.


## Recorded static geometry laboratory

Generate only the geometry controls in `examples/static_obstacle.rs`, with a fresh
JSON path outside Git, then qualify using `static_obstacle.py --records PATH
--executable ELF --receipt PATH`. A packet consists exactly of `records.json`
and `qualification.json`. Pass `--obstacle-records-dir PACKET` to the book builder;
it checks the frozen clean source, exact record bytes and independent rational
controls before publishing `obstacle-lab.html`. Without a packet the page states
absence. The CI review artifact constructs this bounded geometry packet only;
it does not run any fluid integration or the denied research campaigns.

## Recorded obstacle pressure and reduced shear

`examples/obstacle_flow.rs` writes four native pressure projections and 48
reduced-shear updates to a fresh JSON path outside Git. Qualify them using
`obstacle_flow.py --records PATH --executable ELF --receipt PATH`. The independent
oracle uses analytic affine pressures and exact rational dense solves for the
shear fixtures, with force/work/impulse checks. This is not a historical numerical
campaign or arbitrary embedded tensor-viscosity qualification.

Pass the two-file packet with `--obstacle-flow-records-dir PACKET` when building
the book. `obstacle-flow-lab.html` exposes actual saved field responses and
clearly distinguishes sealed pressure from fully developed periodic shear.
Browser checks cover 24 pressure slice/state views and 54 saved shear profiles,
including mobile layout. Without a packet the page explicitly states absence.


## Interactive aligned strain laboratory

The new `aligned-strain-lab.html` page recomputes finite strain, transpose action,
viscous force and dissipation for velocity samples on one native unit 3×3×3 grid
with a padded stationary center cube. It includes all 210 rows, including six
zero rows, and retains the 48-face pressure space and stored face masses.
Normal strain, cross-component engineering shear, the twelve reflected corner
edges and one local shear-cancelling rotation patch have separate controls.
The sliders change face samples and viscosity; there is no timestep or fluid
solve. The displayed nearest-rounded coefficient estimate cannot authorize a
step. Exact Lean links and original research are beside the controls and in
[the laboratory guide](../research-book/implementation/aligned-strain-laboratory.md).

Capture a fresh native packet after committing a clean source tree. All output
directories below must be new and outside every Git checkout:

```sh
python3 tools/qualify_aligned_strain.py /absolute/external/native-qualification
python3 docs/education/aligned_strain_packet.py --output /absolute/external/strain-packet \
  --executable "$CARGO_TARGET_DIR/debug/examples/aligned_strain" \
  --primary-qualification /absolute/external/native-qualification/qualification.json
RHEON_ALIGNED_PACKET=/absolute/external/strain-packet python3 -m unittest discover -s docs/education -p test_aligned_strain_packet.py
python3 docs/education/build.py --output-dir /absolute/external/edition \
  --asset-dir /absolute/external/assets/node_modules \
  --aligned-strain-records-dir /absolute/external/strain-packet
python3 docs/education/verify_browser.py --render-pdf --output-dir /absolute/external/edition
python3 docs/education/verify_site.py --output-dir /absolute/external/edition
python3 docs/education/verify_browser.py --check --output-dir /absolute/external/edition
```

The packet binds immutable qualified native source bytes, fresh executable and
TSV hashes, and an independent rational re-export. Chromium verification
compares live browser arrays and diagnostics with independently rederived
rational controls. It saves JPEG evidence at quality 85 outside Git. Read-only
publication checks retain the PDF and receipts. The research-book workflow
rebuilds and qualifies its own native executable and packet; it does not reuse
a machine-specific historical binary or infer current checks from old counts.
