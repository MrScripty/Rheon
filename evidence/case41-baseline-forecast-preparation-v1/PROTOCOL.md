# Preparation only: original baseline and one frozen forecast

This is a separate fixed-observation experiment, pending independent review and
later explicit execution authorization. Exactly two order16 evaluator calls are
proposed: original case41 baseline, followed only on success by the single
rounded coordinate forecast from captured-matrix prediction result
def4ec2238405155c728c98cb1a76929723b9a01. All original accepted input bits,
geometry/material/load/h parameters, equations, arithmetic, quadrature order
and gates remain unchanged. No original finite-difference column/denominator is
altered; the prescribed captured matrix is not recomputed here.

`forecast-inputs.json` binds the original candidate, exact projected displacement,
its RN64 conversion and each RN64(baseline + rounded displacement) coordinate.
The observer constructs only these 22 declared bits. It calls the unchanged
`Work::equation` through the same non-inline boundary for both observations.
Every chart, geometry, mass, force, pressure and donor field is recomputed by
that evaluator at its actual input. Nothing is supplied from counterfactual
latent or previously stored forecast fields. It constructs no accepted owner.

The native baseline assertion compares all 22 rate bits with frozen E2 before
any forecast call. A baseline refusal or mismatch stops. A forecast refusal is
terminal and retained. No search, retry, additional matrix, Newton loop,
controller correction, owner advance, third solve or publication is permitted.
This experiment is not an eighth correction and cannot qualify the exhausted
historical controller run. A below-gate observation would still be a diagnostic,
not a completed nonlinear step or public acceptance.

The return destination is one reused Result<Equation> slot. Full observations
are serialized by borrowing that result outside the equation boundary. One
176-byte forecast array is declared; all original observations remain present.
The immutable baseline and forecast bits have read-only storage. No full
Equation/Point copies are added for observation retention.

## Resource protocol proposed for review

The old 66,368/67,584 additional-memory certificate remains incomplete. Unknown
allocator/library/I/O costs are not assigned zero and no new certificate is
claimed. These separate whole-child limits bound a later observation job:

| Resource | Proposed enforcement |
|---|---|
| Address space | 256 MiB hard RLIMIT_AS |
| Main and Rust test thread stack | 8 MiB RLIMIT_STACK / RUST_MIN_STACK |
| CPU / wall | 60 s hard RLIMIT_CPU / 120 s parent supervisor |
| Stdout / stderr | 2 MiB / 1 MiB bounded parent reads/writes |
| Core / open files / file output | RLIMIT_CORE=0 / NOFILE=32 / FSIZE=8 MiB |

The preceding seven-observation job used 10.375 MiB peak child-lifetime RSS,
148,516,864 sampled VmPeak, 0.047627 CPU seconds and 286,936 stdout bytes. A
256 MiB address limit covers that observed virtual footprint with headroom;
8 MiB stacks exceed retained fixed-frame routes. Two full records are expected
to be about 82 KiB; 2 MiB allows over 20 times this output, with an actual hard
supervisor limit. These are conservative fixture-specific operational limits,
not a proof of an additional-memory difference or an arithmetic error budget.
Host/cgroup availability and actual fresh-ELF frames/layout are recorded during
preflight. Limits are not reduced to inferred unknown external costs.

The frozen future runner uses a closed environment, exact test name and ELF
hash, exclusive before-marker, isolated process group, bounded output, wait4
peak RSS and /proc samples. Crash, allocation failure, limit/truncation,
assertion mismatch, numerical refusal or incomplete observation roster is
incomplete evidence, with no automatic retry or fallback. It refuses before any
native launch unless a later exact authorization binds reviewed protocol,
preflight and ELF hashes. No such authorization is created in this preparation.
