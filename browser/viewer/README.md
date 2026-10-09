# Rheon browser motion workspace

A static shared viewer shell for recorded mechanics and independent simple-human
posing. It is an initial GUI foundation for inspecting biomechanical work, with
no biomechanics, muscle dynamics or coupled fluid/human simulation engine.
Existing computational systems remain separate. No solver or research harness
is imported or executed by the shell, adapters or packaging script.

The Kenoma editor is consumed unchanged at
`3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4`, protocol 1 and additive rig version 1, from
`MrScripty/Kenoma` branch `feat/simple-skin-graph`. Its source and build remain
Kenoma-owned. This packaging boundary copies that exact clean checkout into a
generated external folder, builds only its independent human WASM generator,
and keeps matched glue/WASM together. Upgrading requires changing the reviewed
pin in both adapters.js and build.py. The component uses gizmo-only controls
and a bound rest mesh whose vertex IDs and triangle indices stay fixed during
deformation. Its worker coalesces pose requests and rejects stale results after
undo/removal; the latest displayed mesh may temporarily lag the handles. The
underlying request API remains synchronous inside the worker. Contact can
interpenetrate and extreme bends can crease; there is no collision physics,
anatomical accuracy or muscle model. See the pinned Kenoma
[rig API](https://github.com/MrScripty/Kenoma/blob/3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4/browser/simple-graph/RIG_API.md).
The scene stays alive across workspace-tab switches. The unchanged embedded
editor supplies Save/Open for versioned local `.human.sqlite` files: IDs, poses,
transforms, colors and selection survive an explicit save and reopen after
reload. Opening replaces the scene atomically in one undoable edit. There is
no autosave; generated geometry, undo history and camera/gizmo selection are
omitted. Limits are 64 characters, a 4 MiB file and 1 MiB source text. The
session ID counter remains monotonic. The package includes local sql.js 1.14.2
WASM and its MIT license, with no CDN or storage service. See the pinned
[scene contract](https://github.com/MrScripty/Kenoma/blob/3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4/browser/simple-graph/SCENE_FILES.md).
No parent messaging or pose/force exchange is invented.

## Build outside Git

Use Node 24, Python 3.11+, Rust with `wasm32-unknown-unknown`, and the matching
`wasm-bindgen` CLI (0.2.129 at this pin). Install the existing locked Rheon
education assets outside Git, or reuse an unchanged installation of that lock:

```sh
mkdir -p /external/rheon-assets
cp docs/education/package.json docs/education/package-lock.json /external/rheon-assets/
npm ci --prefix /external/rheon-assets --ignore-scripts --no-audit --no-fund
git clone https://github.com/MrScripty/Kenoma.git /external/kenoma
git -C /external/kenoma checkout --detach 3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4
python3 browser/viewer/build.py --output /external/rheon-viewer \
  --kenoma-source /external/kenoma --asset-dir /external/rheon-assets/node_modules \
  --target-dir /external/kenoma-wasm-target --wasm-bindgen /path/to/wasm-bindgen
python3 -m http.server 8000 --directory /external/rheon-viewer
```

The output must be fresh and outside every Git checkout. Build failures remain
failures; no replacement WASM or synthetic recording is supplied. The output
contains local Three.js 0.180.0 modules/licenses, immutable component source,
WASM/glue, and a source/file-hash receipt. It needs no CDN, backend or isolation
headers. Publish the complete folder to a static host such as GitHub Pages when
separately authorized. This work adds no deployment workflow or deployment.

## Supported native records

Open one existing JSON file; parsing is bounded to 8 MiB. Adapters validate
finite fields and shapes and clone/freeze display data without modifying input.
SHA256 labels identify imported bytes; they are not trusted qualification
digests. The viewer does not reproduce original oracle/proof checks or accept
those hashes as physical evidence. Unsupported versions/formats refuse with
visible feedback and preserve the previously loaded recording.

- `examples/rigid_motion.rs` JSON: the documented eight-vertex, twelve-triangle
  fixture, initial state and at most 64 stored updates. Exact stored vertices
  drive the mesh. Loads belong to the preceding update; its force arrow is
  anchored at the previous recorded center. The initial load is unreported.
- `rheon-obstacle-flow-records-v1` from `examples/obstacle_flow.rs`: reduced shear
  cases, supplied centers, recorded velocity and energy ledgers. Pressure cases
  are not adapted in this slice. Displayed elapsed time is recorded step × dt.
- Native `column_no_slip` / `column_poiseuille` results.json: named cases and
  saved profiles/times/energy. Only the native profile column is shown; no
  analytical reference, interpolation or trajectory is fabricated.

Playback advances through saved indices every 500 ms, independently of physical
time. Scrub/previous/next/case/mode changes pause playback. Force and velocity
arrow lengths are normalized presentation scales; plotted speeds and metric
values retain their units. Camera movement changes only presentation.

## Embedding and adapter boundary

```html
<iframe title="Rheon motion workspace" src="/Rheon/viewer/index.html"
  style="width:100%;height:850px;border:0"></iframe>
```

All assets use relative URLs and support a project subpath. For a custom host,
define the Three.js import map to the packaged vendor module, import shell.js,
then insert `<rheon-viewer>`. Each element uses its own shadow DOM, camera,
recording, timeline and persistent Kenoma iframe. Multiple viewers are isolated.
`rheon-frame` events report selected case/index/time, without changing numerical
data. `adapters.js` handles native-to-display conversion; `view.js` owns only
graphics; `shell.js` owns controls/lifecycle. The Kenoma iframe consumes its
existing ordinary-page boundary. It is trusted packaged JavaScript, not a
security sandbox. Neither host nor child reinterprets the other's computation.

## Verification

```sh
node --test browser/viewer/tests/adapters.test.mjs
python3 browser/viewer/tests/browser.py --site /external/rheon-viewer \
  --rigid /existing/moving-force-torque.stdout.json \
  --shear /existing/obstacle-flow/records.json --output /external/viewer-check
```

The browser check uses actual installed Chromium/Playwright and existing records;
it never launches an example, oracle campaign or numerical provider. It checks
recorded vertices/metrics, navigation, play/pause, invalid imports, real Kenoma
WASM posing/undo, retained scene across tabs, iframe/subpath embedding and phone
layout, with screenshots and hash-bound receipts outside Git. Rig checks await
`renderer.whenIdle()` before inspecting or capturing worker-produced meshes,
then compare bound topology across pose/undo and rapid queued input. Scene-file
checks download real SQLite, reopen after reload, compare exact source state,
undo imports and reject future versions and stale reads atomically.

## Next integration gap in the existing plan

Rheon's root README calls for a standalone demonstration GUI plus a modular
framework usable by other apps. This shell supplies the presentation boundary;
it does not add a browser solver. Current supported native JSON outputs can be
opened directly with the file picker, unchanged and without repackaging: the
rigid-motion, reduced-shear and fixed-slab adapters consume their documented
formats. Static packaging is needed once for the GUI and pinned component.

The next real GUI gap is automatic discovery and admission of existing outputs.
There is no run catalog, output-folder importer or export-to-viewer link, so
users still locate each file manually. Aligned-strain TSV, full MAC fields,
pressure slices and the retained wrench JSONL also lack viewer adapters. A
small versioned output catalog should identify format, source/state, units,
recorded times, geometry and exact files, with reader-side limits and visible
unsupported states. It can index already-produced data without running physics,
changing acquisition digests, or requiring a fresh site build for each run.
The current physics roadmap's moving/interface/general-coupling gaps remain
separate computational prerequisites. The pinned Kenoma scene-file milestone closes explicit save/reopen for human
authoring; it adds no automatic scene storage or human/simulation exchange.
