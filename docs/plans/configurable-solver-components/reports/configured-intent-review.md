# Independent configured frozen-intent source review

Reviewed 2026-10-15 by design_review (GPT-6.1 Sol), read-only review of
`/workspace/rheon-configured-run-sets`, uncommitted slice over
`45fc1e92332bbbece3501c9327b920bf57ce5e37`.

## Outcome and scope

No remaining material source or test findings in the exact final files below.
Source-approved for this bounded pure-intent slice and its explicit parent
registration, subject to exact-tree execution/Clippy and independent source
admission. This is not A4 completion, archive binding/readiness, actual runtime
execution, persistence qualification, model parity, or a hosted test receipt.

The final lint delta boxes only the large provider-pin field in PressureIntent.
Serde serializes that Box through its contained pin, so the serialized schema
and payload digest framing are unchanged. The fixture constructor boxes the pin;
the final borrowed if-let pattern binds the Box normally and uses autoderef.
This changes metadata storage, not numerical arrays or execution semantics.

Native carrier schema one, its serialized types/digests, archive binding and
reader remain unchanged. The parent file only registers the configured child.
Configured specs/sets use schema two and explicit distinct kinds; native schema
one readers reject the new version. The configured child executes no interval,
loads no provider, adds no numerical store and returns no archive Ready state.

## Contract assessment

Constructor capture uses actual immutable Session configuration and actual
constructor state. It checks constructor stage/frame/generation/time, absent
accepted receipts/interval start, sealed geometry, exact positive-zero rest fields
and full fractions. Raw logical constructor budgets, separate boundary cap,
quota, native method and volume/carrier identities are preserved rather than
reconstructed from adjusted internal carrier budget or Debug text. Equal material
and carrier density is an intentional restriction of this closed format.

Future intent explicitly separates constant requested dt/count, force units and
regions, volume gates, disabled versus Newtonian viscosity (including its separate
workspace cap), native-only versus native-prefix/configured pressure, provider
pins/locator, recording/constructor-capture policy and host executor declaration.
Cell/interval/force/member/text/document bounds and checked numerical constructors
remain distinct from actual later physical/allocation admission. Finite scalar
and region checks are appropriate declaration validation; they do not promise
that every declared force or viscosity is numerically executable.

The frozen digest binds version/kind, constructor and future payload. Exact
serialized comparison preserves signed-zero distinctions in constructor binding
and difference declarations. Mutating a public copy requires an explicit new
digest; saved publication remains create-new. Spec/set envelopes use the original
bounded reader, preserving duplicate-key rejection before generic JSON rewriting.
The corrected set-kind regressions isolate both envelope and revision kinds.

Provider identity is separately bounded and declared, with exact resolved-identity
comparison. Host executor identity is not conflated with model source. Historical
open never reads package files or compares against this executor. Actual execution
must explicitly invoke validate_execution and independently resolve/verify package
files and their physical/context support before model code. create_session only
constructs fresh rest state; it deliberately does not grant execution admission.

Project-relative deployment locators reject traversal, while external absolute
locators have a distinct bounded-text/path contract. Archive references retain
the stricter protected relative-path policy. This module does not resolve either
locator or archive during cold intent open and therefore establishes no filesystem
containment, package availability or artifact readiness.

Run-set checks cover baseline/member identity, recomputed variant differences,
unique attempt/archive references, expected host source and bounded terminal
declarations. FinalizedArchive values are declarations only. Failed/Cancelled
unrecorded accepted tails remain representable; none is promoted to a verified
archive. A later configured codec must verify actual fields, lineage, status,
counts, budgets and closed-file identity before providing any Ready result.

## Tests and remaining integration obligations

The nine public tests meaningfully cover both native implementations, every raw
constructor control and fresh constructor reconstruction; exact signed-zero and
disabled/zero-viscosity distinctions; historical reading with an unavailable
deployment; independent executor/provider drift; immutable revisions/native-v1
separation; versions/unknown fields/duplicate keys/digest tampering; locator and
file limits; invalid domain/resource/prefix declarations; actual accepted-owner
capture refusal; differences/attempt bounds; and terminal declarations without an
archive verification claim.

One low test defect was found and fixed during review: the unsupported set-kind
test originally supplied a raw revision where an envelope was expected. The
final test saves and positively opens a valid envelope, mutates each kind
independently, and requires the intended unknown-variant Codec diagnostic.

Before actual execution/readiness, the owner must additionally bind the recording
worker's concrete configured profile, authoritative accepted interval starts,
actual constructor and per-call controls, provider selections and terminal tails.
Constant requested dt must preserve legitimate shortened actual dt. Runtime
admission must compare the actual loaded package's prefix/support policy, not
only the declared identity strings. Fresh-run identity allocation/reruns, the
production adapter/worker/UI route, numerical comparison profile admission and
all cold archive states remain future integration work. No native profile may be
relabelled to bypass those dependencies.

Public API source has no warning suppression. Both dedicated integration-test
targets apply dead-code allowances only to their path-imported parent module:
each target exercises its own native/configured API while the other target owns
the complementary contracts. The existing saved-run test change adds only that
annotation, with no assertion removal or numerical change. These test-only
allowances do not substitute for production desktop Clippy. Exact affected
compilation remains the lead's gate.

## Reviewer actions

Reviewer inspected the complete new module/test, parent registration diff and
relevant numerical ownership contracts; SHA256 pins and git diff --check passed.
Reviewer ran no Cargo command, test, model or numerical experiment and changed no
repository file. Only this external review document was created. Owner-reported
test results are not independent reviewer execution evidence.

## Exact reviewed SHA256

| File | SHA256 |
| --- | --- |
| `src/bin/desktop/run_sets/configured.rs` | `606e435e27f6138df93c7d7ddc205e921c066da70c476a4a40232ac0c72c53a4` |
| `tests/configured_run_sets_contract.rs` | `29b29888112126677262bdd05792cad70170338eaac8c4767fc2ab0292a9e927` |
| `src/bin/desktop/run_sets.rs` (registration-only diff) | `405332f953a31e476a704ce189b85781faf19620a2c54e5b4cb077fdd95f2a5f` |
| `tests/saved_run_set_contract.rs` (test-import annotation only) | `6e001075a4307da0e51df04c4521bc8599194ce06fbf010c08e34bab8f693ef9` |
