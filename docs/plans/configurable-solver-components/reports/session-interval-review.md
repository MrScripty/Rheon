# Independent accepted Session interval-start review

Reviewed 2026-10-15 by design_review (GPT-6.1 Sol), source-only review of
`/workspace/rheon-accepted-session`, four-file slice over
`a161844bd73f01c85935873a8cfcf4cf3ab09acd`.

## Outcome

No material findings in the exact source, regression and API addition below.
Source-approved for exact-tree validation and rebinding the existing selected
Session source entries. This review does not establish a persistence codec,
runtime/model parity or complete milestone acceptance.

`SessionSnapshot::accepted_interval_start()` returns None at construction and
the exact pre-step carrier time for the most recently completely accepted
interval. All three Session paths capture that time before calling the existing
numerical owner, then publish it only after the call succeeds: legacy pressure
provider, boundary/transport, and configured pressure with optional viscosity.
The configured path uses that same local pre-step value for its pressure context.
Failure, cancellation, deadline, exhausted quota and stop do not overwrite the
stored interval start. The new scalar introduces no array allocation, numerical
kernel change or reconstructed clock, and no new fallible operation follows the
physics commit.

This closes a recording-authority gap: native-only configured calls have no
provider context receipt, and accumulated floating endpoints cannot safely
reconstruct exact interval starts by subtracting dt. Recorders must capture the
owner-provided value while holding the accepted snapshot, before another call.
It is the last accepted interval start, not a restart cursor or a future schedule.

## Regression assessment

The public regression covers all three Session paths and both native pressure
implementations. It performs 64 genuine accepted intervals at 1/60 seconds,
compares each returned start by IEEE bits with the actual previous owner time,
and requires a real counterexample to end-minus-dt reconstruction in every
path/method combination. This guards against replacing the getter with a
superficially equivalent subtraction.

It then cancels at the actual final commit checkpoint, checks unchanged encoded
publication/frame/start and receipt sentinels, injects invalid viscosity, retries
the same next interval successfully, and verifies quota and stop preserve the
final publication/start. The resting two-cell fixture deliberately isolates
clock/publication semantics without claiming nontrivial flow or learned-model
qualification. Existing receipt-address checks are supplemented by retained
duration/configured facts; addresses alone are not treated as content identity.

The API paragraph accurately describes owner-captured time, constructor absence,
failure retention and the nonbinary floating-point subtraction hazard.

## Reviewer verification

Reviewer inspected the complete two-source-file diff, public regression and API
addition, checked exact SHA256 pins and ran `git diff --check` (passed).
Reviewer ran no Cargo command, test, model or numerical experiment and made no
repository edits. Only this external review document was created. Reported test
and Clippy results remain the lead's execution evidence, not reviewer reruns.

## Exact reviewed SHA256

| File | SHA256 |
| --- | --- |
| `src/session.rs` | `367c370e2257e35de09943e8dc3e1257b994c5e01d1409af1a989682f41c583e` |
| `src/session/configured.rs` | `d99ae990e9c7ddc956f95f414a25832411a4dc84ab434df95ff9f8f68cf52a83` |
| `tests/session_interval_start_contract.rs` | `4cdc0a85d21d77784ef142ba04f1ef20fb0333555d7bbb15b43465f475934769` |
| `docs/api/configured-pressure-session.md` (added interval-start paragraph) | `43fcfcfcee3e41bcc90c0bd79e0b55de2238a7f1e126729fc44f47909375e170` |
