# Independent verifier review and repair

Parent requested independent review of admitted verifier/export proposal commit
`4621d69747740a950a90d78755fe3bb4e220efe4`. A separate read-only reviewer inspected
the two tools, oracle scope and capture path. It ran the pinned 19-test suite
normally and under `-O`, and found no cell-average formula defect. CodeRabbit
skipped this draft PR; that successful status did not constitute review.

## Findings

| ID | Finding at reviewed pin | Evidence and disposition |
| --- | --- | --- |
| R1 / P1 | Capture treated every ancestor `.git` path as a checkout, rejecting permitted `/tmp` and `/workspace` output because their sandbox markers are empty directories | Reviewer reproduced `origin/output-inside-checkout` while Git reported `/tmp` is not a repository. Replaced existence-only detection with actual Git checkout identity; markers are preserved. Independent follow-up confirmed closure. |
| R2 / P1 | Clean Git status and selected tracked inputs did not exclude ignored compilation inputs | Reviewer's clean-commit Cargo probe added ignored `build.rs`; the exact build command executed it while status stayed empty and inventory omitted it. Capture now builds a fresh tracked-only Git archive, hashes all exported tracked files/archive, and rejects unsupported Cargo configuration/compiler overrides. Independent follow-up confirmed closure. |
| R3 / scope limitation | The admitted <8% isolated phase-speed gate covers `(1,1,1)` only; mode two enters aggregate density error and is reported | A synthetic 20%-slow second mode can pass the aggregate gates. Report explicitly names the gated mode and a test preserves the counterexample. A separate mode-two speed requirement needs parent replanning; no threshold changed. |
| R4 / P1, repair follow-up | `CARGO_BUILD_RUSTFLAGS` bypassed the first override guard | Reviewer ran the real capture-protocol test with `--cfg unbound_probe`; Cargo changed the marker binary. Added this variable to rejected inputs and exact-diagnostic tests alongside ordinary/target-specific flags. Independent reviewer confirmed closure with the focused test. |

## Capture invariant and bounded repair

Origin authority belongs to the immutable tree, declared build recipe, locked
dependencies and observed executable. A clean status alone cannot supply it.
Blacklisting only ignored `build.rs` would leave the defect through `include!`,
embedded data or local configuration. A tracked-only export closes that input
population without deleting ignored work or changing the authoritative checkout.
Capture refuses unsupported build configuration rather than claiming its effects
came from the commit. Conventional `CARGO_PROFILE_*` settings are recorded build
inputs. The host/toolchain remain trusted prerequisites; this is not malicious
compiler/host attestation or a cryptographic authenticity guarantee.

The native example's admitted CSV/JSON format and numerical gates are unchanged.
Only verifier-owned capture mechanics/origin metadata were strengthened. Old
prototype origin records lacking the tracked-export identity are rejected; no
receipt was automatically refreshed. No actual native S1 run has been captured.

## Focused evidence

After the tracked-export/marker repair, 23 tests passed normally and under `-O`
in both worker and independent review, with pinned Rust 1.92.0 and NumPy 2.3.5.
Tests distinguish empty markers from real checkouts, reject Cargo config/flags,
and run a tiny original Rust program through the actual fresh build/run capture
protocol. Its original clean checkout contains ignored `build.rs`; the captured
binary reports that script did not run, the export excludes it, and the ignored
original file is preserved. This probe writes no CFD state and never enters
`assess_suite`; it is not native S1 numerical evidence. Probe files/logs/origin
records stay in temporary storage, outside commits.

The final R4 guard repair passed all 23 tests normally and under `-O`, Python
compilation and diff checks; independent focused review confirmed closure. All
review findings are resolved. The real S1 gate remains pending the implementation
owner's clean source pin.

## Licensing and CI disposition

Parent admits existing NumPy only as a separately installed validation tool.
No wheel/runtime packaging or distribution is proposed or authorized. Component
license disclosure remains; Rheon's Apache-2.0 source license is unchanged.
Report any future distribution before proceeding.

Both hosted push/PR dense3D workflows passed at 4621d69. They do not execute this
verifier and supply compatibility observations only. Report repair-head CI at
its actual commit rather than projecting success from a prior head.

## Native-failure diagnosis review

After native 1b62ae3 capture, a separate read-only follow-up checked the narrow
Sod t=0 repair and independently reproduced the acoustic symbol calculation.
It found no defect: exact constant-state integration preserves the tolerance,
native initial differences become zero and focused tests pass normally/-O.
Its scalar Fourier calculation reproduces order 0.561438498 versus native
0.561438404. It agrees 16/24/32 is a defensible prospective replan but the new
10% resolution heuristic requires explicit parent admission and guarantees no
order. No threshold change approved or native acceptance established. Historical
source-pending statements above describe earlier review checkpoints only.
