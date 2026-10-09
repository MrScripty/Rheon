# Rheon GitHub Pages teaching edition

The 25-chapter book, six appendices, implementation guides, locally rendered equations, displayed Lean source, six interactive 3D reference laboratories and the connected PR22–24 native wall/force sequence. Owner: Puma. The destination is repository GitHub Pages.

The three native labs are recorded playback with distinct models: two finite Navier walls, compatible no-slip/mixed boundaries, and uniform tangential forcing with stationary no-slip walls. Their controls select saved cases/times; they do not recompute parameters or advance a live liquid simulator. The original six 3D labs remain separate analytical/stored mechanisms. The projection control displays an algebraic blend of two saved fields; camera motion only changes the view.

## Current and historical proof associations

The earlier aligned-strain reconstruction at `f75cd66` qualified inventory
`be5a1ebd…` with a normal lake build. That record is preserved under
`historical_reconstruction_proof` in `native-sequence.json`. Later additions
changed the inventory to `fdc589f6…`; the old presentation association had
remained at the reconstruction inventory and correctly refused a current build.

`current_proof_qualification` now binds the unchanged proof tree from clean
`7d3c8df2d9149e5cd643a2af1f29f2ccf94bfb7e` to its already retained owner and
independent viscous-wrench qualification receipts. Both runs compiled all twenty
modules and the root with pinned Lean 4.19.0 directly, then audited 342 actual
declarations, including 286 explicitly expected names. The source contains 216
public theorem statements. Their receipt digests, audit-log digest and all 33
proof-source/dependency-file digests are recorded in the association. This is
local direct-compiler evidence; the historically linked hosted workflow belongs
to the earlier reconstruction. Reconciliation executes no Lean or physics run.

The builder retains the exact inventory/source gate and additionally checks the
receipt association, dependency files and presentation counts. Original native
lab identities, historical receipts, PDFs and published GUI artifacts are
preserved. A successful local source build does not authorize merging or Pages
deployment; those remain separate approval steps.

## Build and preview outside Git

Requires Python 3.12, Pandoc 3.1.11.1, Node 24, the npm lock and `requirements.txt`. Three.js 0.180.0 and KaTeX 0.16.22 are pinned and copied with licenses; runtime assets are local. From the repository root, choose a fresh output path and asset directories outside every checkout. The builder refuses existing output directories and checkout ancestors:

```bash
export RHEON_EDITION=/tmp/rheon-edition
export RHEON_ASSETS=/tmp/rheon-assets
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$RHEON_ASSETS"
cp docs/education/package.json docs/education/package-lock.json "$RHEON_ASSETS/"
npm ci --prefix "$RHEON_ASSETS" --ignore-scripts --no-audit --no-fund
python3 docs/education/build.py --output-dir "$RHEON_EDITION" --asset-dir "$RHEON_ASSETS/node_modules"
python3 docs/education/verify_browser.py --render-pdf --output-dir "$RHEON_EDITION"
python3 docs/education/verify_site.py --output-dir "$RHEON_EDITION"
python3 docs/education/verify_browser.py --check --output-dir "$RHEON_EDITION"
python3 -m http.server 8765 --bind 127.0.0.1 --directory "$RHEON_EDITION"
```

Install Chromium with `python3 -m playwright install --with-deps chromium`, or use installed Chromium. The browser verifier uses a private local server and checks the six reference labs, the native hub, navigation/search, math, page/network errors and mobile layout. When native bundles are supplied, it checks every saved native profile, plot, ledger and CSV link. `--check` is read-only: it cannot regenerate a PDF or bless stale receipts. `--url` selects an existing preview server.

`--render-pdf` intentionally renders the new reading PDF and writes its source/book/PDF-bound receipts in the output directory. Inspect that PDF before publication. HTML, PDF, Markdown ZIP, copied evidence, browser receipts, assets and build artifacts stay outside Git. Historical tracked PDFs and receipts are preserved with their original source identities.

## Optional original native bundles

`native-sequence.json` declares exact accepted source identities and file hashes. Supply an outside-Git directory with `wall-friction/`, `no-slip/` and `poiseuille/` containing those exact accepted HTML/JSON/CSV files, then add `--native-labs-dir /path/to/frozen-labs` to the build command. Admission checks every file/hash; changed data or an unexpected file is refused. The builder never invokes an example or integration to create replacement records. The PR22 HTML gets a derived playback notice and responsive styling; all original input files and numerical records remain unchanged.

A source-only CI build without these packets states that the recorded bundles are absent and links their model/proof guides. It does not claim browser qualification of omitted playback. A separately qualified local preview can include all three original bundles. Workflow outputs live under `RUNNER_TEMP`; the review artifact is retained by Actions. Pages deployment runs only from main, so an unmerged source draft is not a live deployment claim.

The illustrated `Rheon-expanded-markdown.zip` includes companion figures, implementation guides and Lean sources. Extract it with those relative paths intact. The reading PDF/Markdown includes all 31 original sections and five selected guides explaining the native progression and requirements roadmap; the site renders the full implementation-guide collection. Site verification checks actual local HTML and packaged Markdown links, copied source hashes and PDF portability. PDF links use internal reading destinations or immutable source URLs; they cannot point to the temporary preview server.

## Proof and implementation boundaries

`proofs/source-inventory-pr24.json` preserves the accepted PR24 kernel proof bytes: 57 public theorems and 77 audited declarations. The reconstructed `source-inventory.json` includes AlignedStrain. Its fresh pinned normal `lake build`, allowed-axiom audit and rejection/source-policy probes checked 106 public theorem statements, 144 explicitly expected declarations and 161 audited declarations. `native-sequence.json` binds the immutable reconstruction source and local qualification receipt, and links the associated hosted Lean run without asserting its checkout identity. Earlier source qualifications and recorded native packet identities remain historical. The book builder checks current bytes; it does not rerun Lean or prove assembly, IEEE arithmetic, transient reference evaluation or global solver convergence.

The expansion's older 42-public/60-audit receipt remains explicitly historical. Its exact-real finite-strain and planar-contact contracts retain their assumptions. The original Python reference source/data receipts are checked before building. Source-only/data-only changes, stale PDF/browser inputs, missing assets, broken controls and page errors reject publication.

The [native progression](../research-book/implementation/native-wall-force-sequence.md) connects equations, units, conditional Lean claims and demos. The [requirements map](../research-book/implementation/requirements-roadmap.md) distinguishes existing box/liquid APIs from missing general mesh/interface composition. The bounded static closed box owner now supplies shared fluid volumes, open areas, connectivity and conservative flux. Obstacle pressure and wall stress must still consume that same owner before a fluid solve can be claimed.


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


## Shared motion workspace in Pages

The edition builder accepts `--viewer-dir /external/rheon-viewer`. This must be
a clean, exact-current-head package from `browser/viewer/build.py`, with its
reviewed Kenoma pin and all required runtime files. Admission checks current
Rheon runtime bytes, every package hash and every referenced producer blob
before copying it to `viewer/`. Navigation and the overview link
`viewer/index.html` only when it is included; source-only builds state absence.
The edition does not compile the component, run an output producer or rewrite
recorded data. The independent component and recorded playback remain separate.

The existing `research-book-pages.yml` source integration now packages exact
Kenoma `3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4`, installs matching
wasm-bindgen 0.2.129, adds the WASM target, indexes the flow packet already
produced by the existing workflow, verifies the HTTP catalog UI, and passes
`--viewer-dir` into the existing edition build. Generated tools, component
checkout, WASM, catalog and preview remain under `RUNNER_TEMP`. This adds no
numerical producer or campaign; the existing geometry/pressure/shear/strain
qualification steps remain unchanged. The build timeout accommodates the new
pinned human component/tool compilation.

To expose the reviewed GUI on the repository website, approve merging this
source into main, then let the existing main-only Pages workflow build and deploy
its complete edition. The workflow's Configure/Upload/Deploy conditions and
Pages permissions are unchanged. No merge, workflow dispatch or deployment is
performed by preparing this integration. A development branch preview is not
a live website publication. Historical browser/PDF receipts remain historical;
a new edition must pass its current rendering and publication checks.
