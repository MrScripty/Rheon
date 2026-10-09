# Independent pre-implementation design review

Reviewed the proposed `dense3d-sequence-design.md` against public base
`9cd4587a54befa61bdfddc8e35014bd3c34f02fb`, particularly
`src/liquid_step.rs`, `src/simulation.rs`, and `examples/liquid_step.rs`.
Review performed before exporter implementation. No numerical changes proposed.

## Decision

Architecture approved. The design's independent-review resolution records the
main cap, failure, importer-boundedness, provenance, and publication requirements
before implementation. The following remain implementation acceptance criteria;
they require no redesign or new dependencies.

1. Check frame capacity before calling `step_with_box_flux`. Reaching the cap must
   not advance physics to an additional state that cannot be exported. Check all
   fields, report scalars, time, dt, and stamp continuity before writing any byte
   of a frame; constructor admission must fit the declared transport-only mode.
2. A write can leave a partial last JSONL line. This is acceptable only as a failed,
   incomplete export without `run.json`. Poison the exporter after any output or
   finalization error, including flush; retain read-only inspection of the already
   accepted state, and never retry physics or silently resume the stream.
3. The importer must cap manifest bytes, total JSONL bytes, line bytes, frame count,
   and expected scalar counts before allocating/parsing large input. Require a
   final newline, exactly nine frames, no trailing data, canonical decimal u64
   stamp strings, and strict finite native-float conversion. Validate pressure
   semantics, geometry offsets, field shapes/types/units, and manifest config as
   well as axis names. Corruption controls must rehash changed frames so semantic
   validation is exercised instead of merely rejecting the checksum.
4. Provenance must hash the candidate exporter, fixture, importer/orchestrator,
   Cargo.toml, Cargo.lock, and rust-toolchain.toml. Record base commit separately
   from actual source commit and dirty status. A base commit plus dirty=true is
   insufficient to identify added or modified code. Record the exact core-only
   build/run command and executable digest. Freeze/verify source hashes across
   build and run so the manifest identifies the code that produced the output.
5. Publish the completion manifest only after successful simulation, stream
   flush/sync, independent import validation, and digest computation. Reserve a
   fresh directory and use no-overwrite creation/publication. Check the absence
   of a manifest after injected writer failure and rejected/cancelled steps.

## Basis and limits

`LiquidTransportSimulation::state` exposes borrowed accepted carrier, liquid and
held pressure arrays plus publication stamps. Existing `step_impl` prepares both
owners and publishes only after the cancellation checkpoint, so writing after
`Ok(report)` correctly excludes failed/cancelled candidates. Pressure already has
the documented last-accepted-interval interpretation; no endpoint law is needed.
The fixed fixture agrees with the smallest existing `liquid_step` scenario and
can avoid the example's comparison loop and PNG dependency. Nine round-tripping
JSONL frames plus a completion manifest are sufficient for a bounded consumer.

Implementation validation must still demonstrate exact accepted-view field bits,
actual clocks/dt and stamps, rejection/poisoning/write-failure behavior, independent
MAC divergence, volume/mass/centroid, and the existing binomial donor oracle. This
design review is not evidence that those implementation checks have passed and
does not qualify transport output for scientific or training use.

## Independent implementation review

Reviewed `src/dense3d_sequence.rs`, `examples/dense3d_sequence.rs`,
`tools/export_dense3d_sequence.py`, and `tests/dense3d_sequence_contract.rs` after
implementation. The sequence owns the simulation, admits constructor state only,
preflights frame and conservative byte capacity before stepping, writes only after
the existing successful publication, and poisons itself after step, serialization,
write, or flush failure. Its inspection API cannot mutate or retry physics. Field
and report finiteness/shape/stamp checks precede serialization. Formatting preserves
native round trips without an intermediate JSON document or new dependency.

The runner uses a fresh directory, create-new frames, bounded one-fixture execution,
source/executable consistency checks across build/run, independent import validation,
and a synced temporary manifest followed by an exclusive hard link. Publication
failure removes its own completion marker and never calls physics a second time.
The contract tests exercise exact accepted-view bits, u64 maximum identities,
rejection/cancellation/caps, post-commit output failure, malformed views and terminal
flush failure. Publication tests cover overwrite and write/sync/link failure paths.

Two findings were raised and resolved before the capped pilot:

* The initial runner/importer names and strict manifest keys disagreed. The runner
  now uses `contract.parse` and removes the extra report field. The runner records
  `provenance.build_command`, and the importer requires the exact locked core-only
  Cargo build command alongside the run command.
* `BufWriter` attempts buffered output on drop. Returning through `?` on a failed
  write/flush can therefore silently retry output. The example now uses an
  unbuffered `&File` sink, so dropping it cannot retry buffered output. The
  underlying accepted physics and absent-manifest behavior remain correct.

Implementation review approved after re-reading both fixes and the importer schema
update. No pressure/controller/scientific-law changes are needed. The planned capped
pilot and focused checks provide execution evidence separately; this review alone
does not certify their results.

## Independent build-artifact selection review (PR20 follow-up)

Pre-edit design reviewed and approved at candidate `fee7b4a`. The defect was the
runner deriving `target_directory/debug/examples/dense3d_sequence`, which can
select an older executable when Cargo uses a configured target subdirectory.
No simulation, sequence schema, or numerical changes are required for the repair.

Final implementation reviewed in `tools/export_dense3d_sequence.py`,
`tools/import_dense3d_sequence.py`, and both Python contract test files. Approved:
the runner obtains the exact local Rheon package and example identity from Cargo
metadata, then accepts exactly one compiler-artifact message matching package ID,
manifest path, example name/kind, binary crate type, and example source path. It
requires an explicitly false profile test flag, no features, a successful final
build-finished message, and an absolute reported executable path. It hashes and
executes that path, with no derived-path fallback. Missing/ambiguous artifacts,
foreign identities, harnesses, and invalid executable reports fail before export.

The controlled command selects the dev build; optimization/debug defaults are
deliberately not hardcoded because legitimate dev configuration overrides can
change them. Cargo artifact JSON reports compiler settings and a test flag,
without a profile name, as documented in the
[Cargo artifact-message contract](https://doc.rust-lang.org/cargo/reference/external-tools.html#artifact-messages).

The importer permits exactly the original provenance build command and that same
command with the appended JSON message-format flag. All existing manifest keys,
version, field contracts, and validation gates remain unchanged. Tests inspect
wrong/missing/ambiguous artifact, package/target/source, profile, feature, and path
controls, plus both provenance command variants. Three dependency-free Cargo
example tests exercise default output, `CARGO_BUILD_TARGET`, and external
`CARGO_HOME` target configuration; the latter two plant executable stale decoys
and check that only Cargo's current artifact runs. These fixtures run no Rheon
simulation or dataset export.

No remaining implementation-review blockers. Execution results are recorded by
the task owner separately; this independent review verifies the implementation
and test coverage without rerunning simulation or creating a second pilot.
