# Explicit runtime/book admission review

Reviewed runtime input: PR65 `a3a66b18b34bef2a32ce1b658fce952115b1f7f4`.
Preserved book input: PR66 proof-corrected continuation `ae4fff0`.
Integration branch: `research/book-runtime-qualified-integration`, isolated from main.
This record authorizes attempting fresh qualification of the exact pinned inputs;
it is not evidence that qualification passed or permission to merge PR59.

## Root cause and policy

The old historical gate deliberately admitted only `f75cd66` and its narrow
dense3D root extension. Main `2209012` and later runtime sources changed build
inputs. Fresh native/rational execution could pass before the educational exporter
refused Cargo.lock. That mismatch is in source admission, not demonstrated
numerical disagreement. PR43's complete reviewed 55-path extension permits its
own book source, but the runtime composition adds 18 modules and changes eleven
existing inputs. Its stale record must refuse the combined tree.

Retain the original 55-path inventory bytes exactly (SHA256
`84c036618b32de66623d8ded24ae8cf95194457e36b9a130087a9c2bf3bd3f6a`).
Introduce a separately named version-two runtime epoch pinned to PR65 ancestry
and all 73 native/build/independent-qualification paths. A present stale new epoch
never falls back to historical admission. Fresh receipts must bind both epoch
records, every actual source digest, and the corresponding committed blobs.
No inventory is refreshed automatically. Source-only builds continue to identify
absent native packets; numerical publication still requires fresh qualification.

## Changed inputs reviewed for bounded admission

| Inputs | Review and claim boundary |
| --- | --- |
| Cargo.toml/Cargo.lock | Native mesh loader adds pinned glTF/base64/capability-filesystem dependencies. Reviewed downloaded dependency metadata gives MIT/Apache alternatives, not GPL. Locked builds remain required. |
| lib.rs | New session/material/pressure/provider, collision, VTK and emitter exports. Native exports preserved; source admission now includes every added module. |
| simulation.rs/liquid_step.rs | Opt-in provider workspace follows original commit gates; unsupported boundary/surface/viscosity combinations refuse. Corrected candidate velocity and liquid/accepted pressure publish after acceptance; full-step publication remains distinct from a pressure solve. |
| particle_fluid.rs | Separate candidate/accepted diagnostics, static collision mesh handling and translated emitter births. Retained byte accounting adds diagnostics and collider ownership. These changes do not certify continuum SPH or moving-solid coupling. |
| emitter.rs/emitter_controls.rs | Vector jet and independent emitter translation; deterministic perturbations are inlet controls, not an SST turbulence solver. Interval/domain checks precede acceptance. |
| session/material_session/pressure_session/provider_step/solver_provider/modal | Bounded snapshots, finite/gauge/full residual gates, explicit reference fallback, fixed model provenance and capability metadata. PressureSession publication and full-step transactional publication remain distinct. |
| mesh_collision/mesh_import | Static convex-solid or explicitly two-sided-surface policy; finite bounded geometry, resource/transform/primitive validation and capability-relative file reads. This is collision representation, not a volume mesher or conservative moving-boundary CFD assembly. |
| vtk_export | Immutable snapshot stages, physical geometry/unit metadata and writer validation. Export is interchange evidence, not a solver validation result. |
| Desktop README/history/viewport and added collision/emitter/engineering/export/identity/inspector/particle/transport modules | Native inspector, bounded obstacle/transport views, VTK and emitter/collision controls. Preserve book-owned accepted-speed versus force-pressure labels. View/export capability is distinct from numerical/model qualification. |

The aligned-strain operator, grid geometry, pressure operator, SGS module,
aligned-strain example, exact rational oracle and qualification runner are
byte-identical to the admitted book inventory. The book-owned scheduler/SGS tests
and desktop diagnostic label clarification survive the merge. The unchanged
bounded qualification path still must be rerun in the complete combined crate;
no historical numerical receipt is reused. PR65's owner reports combined runtime
checks, but they apply to its earlier tree and are not transferred here.

## Browser dependency repair supported by observed failures

The nested browser lockfile first failed a `--locked` check because its dependency
graph lacked the newly introduced root dependencies. Regenerate that lockfile
using the declared exact direct versions and review the diff; retain `--locked`.
The subsequent WASM check exposed `errno`: target OS unknown is unsupported.
The cause is unconditional native filesystem imports through cap-std/rustix.

Restrict glTF/base64/cap-std and the `mesh_import` module to native targets.
The pure collision module and fixed browser solver remain available; native
mesh-loader exports and behavior are preserved. This changes platform build
availability only, not a numerical solver. The locked wasm32 browser check then
passes. Native import/collision tests and the full desktop feature checks remain
required before merge. All relevant final native bytes are pinned in the new
inventory. Changes after this review require another explicit review and fresh
qualification.

## Evidence to complete

Local gate checks are preliminary. Require a clean committed source for the
locked native/rational runner, all 22 source/receipt/adversarial tests (including
six real-packet cases), browser adapter build/identity checks, native contracts,
Lean kernel/axiom audit, actual desktop/mobile teaching checks, PDF/local links
and full publication probes. Main merge order must be coordinated with the
runtime owner. Merge/Pages authorization does not convert any pending item into
a pass. Human screenshot review remains blocked if supported transfer returns403.

## Completed local qualification checkpoint

Clean committed source `d35967ec69018af707bc5353ee0bffa0b4934f38` passed the
18-command native/rational runner, including five geometric specimen/oracle
comparisons and mutation/rejection checks. The exported real native packet then
passed all 22 source/receipt/adversarial tests normally and under Python -O,
with zero skips. The compact authored [evidence record](INTEGRATION_EVIDENCE.json)
binds the external receipt digest; generated receipts and logs are not Git payloads.
The aligned native qualifier did not run Lean. Separately, corrected PR66 Lean
run38478346517 passed 330 declarations and its rejection probes; proof bytes are
identical in this integration. Final hosted publication/runtime checks remain
required on the final source after this metadata-only update.
