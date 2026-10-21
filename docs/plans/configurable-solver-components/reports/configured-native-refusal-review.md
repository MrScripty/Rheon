# Independent audit: bc6 live native-baseline refusal

**The retained run shows an honest numerical refusal, with no invented accepted step or external-runtime candidate. No wiring defect is established by this evidence.** Read-only audit of the first run set `configured-set-1792117883590932511-353008-0.runset` from the actual native session declaring `bc6df137a6aee97be63e317f7d742b6077034a36`. No production source/threshold change, solver rerun, model execution, Cargo build or GUI launch by this reviewer.

Frozen controls declare4³ cells,0.25m spacing/zero origin (1m domain),rho1000,dt0.02,three intervals, acceleration[0.125,0,0],Disabled viscosity and defaultJacobi settings. Member0 is NativeOnly; member1 requests a native prefix then the separately pinned CPU fixture. These are constructor/future declarations, not proof of a successful interval.

## Actual retained custody

The existing generic Reader.verify, configured profile/history verifier, actual configured row decoder and public RCT1 auditor all pass on the real SQLite artifact. For this audit I compiled a tiny temporary cold reader against the already qualified existing core rlib; it creates no numerical/session/runtime owner. Its assertions/output establish:

- Exactly one Constructor row at accepted generation0/time exact+0,dtNone; all velocity/tracer values remain exact zero and all64 fractions exact1.
- No accepted receipt, runtime observation or interval pressure. The native pressure stage may have prepared tentative scratch before volume rejection; these absences are not a claim no native solve occurred.
- Typed terminal NumericalFailure with VerifiedLastPublication coverage,accepted generation/time0 and observationNone. Detail: `Step(Volume(FractionBounds { cell: 0, fraction: 1.0000000000000009 }))` plus acknowledged publications1. Although generic display detail conservatively says a tail may be unrecorded, the available typed audit verifies this exact constructor tail.
- Finished manifest pins failed/count1/gen0 with matching whole-file SHA. Configured member attempt remainsNone, and no completed.json exists. Declared intent→started→finished parent links are consistent. The SQLite hash was unchanged after both readonly SQL inspection and the actual public cold-reader audit.

SQLite SHA256 `11474e7ea8f9fef5e8f04003e47c53427e83d17ccc4f00c6a2e79b9c5e092d94`; finished manifest `1bbece42b884c52f8e6d521bd1f026a9eb4f184c52d7dbb59b144b2f6bc69ec7`. Exact artifact directory: `/tmp/rheon-configured-controls-native-bc6/.generated/recordings/configured-set-1792117883590932511-353008-0.runset`.

## Source-backed interpretation and limits

At bc6, src/liquid_volume.rs621–623 computes the candidate fraction and rejects any value outside exact[0,1]. The reported value is four representable f64 increments above1 (about8.88e-16), so rejection follows the existing policy even if input controls are statically admissible. That volume module is byte-identical to pre-A2 main739c1ae, SHA256 `24623038f62261ce6a17b034e9b6d82c5f0aba02dd0df7aad174c885ed2a8c96`.

LiquidStep's fallible volume preparation733–739 precedes the single carrier/liquid commit772/776. Session configured.rs99–114 publishes history,quota and reports only after successful full-step return. NativeOnly start_provider returns inertly at configured_track.rs150–157; outer Worker closes/pins the failed native archive then propagates result at rheon-desktop.rs685 before starting member1. Runtime absence alone would not prove no dispatch; together this actual NativeOnly/NotRun control flow supports no configured-package startup for this failed run.

Thus this is a practical numerical acceptance/robustness limit for the chosen valid controls, with correct rollback and failure custody. It does not demonstrate that every statically admitted point steps successfully. The constructor-only artifact lacks failed tentative velocities/RHS, so it cannot establish whether the last-bit excess arose solely from floating-point roundoff, finite native pressure residual or another numerical interaction; an equal-input numerical investigation would be separate. No exact legacy-path reproduction was run, so no assertion that an older stepping path would fail identically is made.

A stationary zero-force/zero-RHS live fixture can separately validate form launch, actual process lifecycle, cold inspection and controls. It must remain explicitly separate from this failed nonzero4³ scenario and cannot replace or be relabelled as the existing successful nonzero2³ analytical-pressure evidence. No threshold relaxation or clamp is implied.

The two09/10 PNGs are identical, SHA256 `474567762e0c74dd4336e8df2bb2fa57c3eeff68db01f28d80f93ee3566a903b`. I visually inspected10: it shows the inspected package and initial Step0/time0 preview, not a visible numerical-success result or readable failure banner in that viewport. The archive/typed terminal is the deciding failure evidence; do not count identical images as two distinct observed transitions.

Cold-reader source `/tmp/rheon-bc6-failed-native-inspect.rs` SHA256 `375d9ab91816910260a0d2d102cbc14fe5487bad66ca53001729c75eb289802f`; binary `63f720871ec78eb7879e729e57b1d8be16483722f823442d75b83513de4399e3`; actual output `/tmp/rheon-bc6-failed-native-inspect.log` SHA256 `3db1a634c707373142c32aba2ec03e49b8c169e0b74813a38692942c8fe7ee05`.

## PR112 snapshot

Read live PR112 at exact `ad8564c452d29d1ab2d858e08fd6fa6079ba3972`: default/core feature jobs, executable-contracts, bounded-lessons and both reported scoped checks pass; desktop feature and book build remain IN_PROGRESS, so draft/UNSTABLE is not merge-ready. CodeRabbit success is separate. Snapshot `/tmp/rheon-pr112-native-audit-checks.json`, SHA256 `504b75d52ab3cb35b33d71234076121b033b49b48f0ec8b262f212382223803b`. No merge or workflow mutation performed; root must refresh final statuses.
