# Final column artifacts and the scope of the older gates

This narrow qualification successor is based on the exact PR18 candidate
`35b00247e68f04e6db539c365298862f82e1c2d0`. It changes no Rust physics or native
outputs. Historical receipts, examples, figures, Lean files and published book
inputs remain frozen. External review and PR advancement stay with the parent.

## What the old gate omitted and what is repaired

The column-interface companion promised finite fields, accepted geometry and
native pixels. Its reader checked initial/frame exports and final cells, but
never read coupled `final-faces.csv`, `final-geometry.csv` or `final.png`.
The old plot generator read two of those unchecked artifacts to display the
accepted eight-interval result. Fresh frozen-reader controls therefore accepted
nonfinite final velocity/geometry and a corrupt final PNG in normal and optimized
Python. This was a reader coverage defect; the stored original artifacts remain
valid and are not replaced by the corrupt witnesses.

The successor parses every coupled final face/geometry field through the same
explicit finite-field checks, verifies exact identity with the last accepted
frame, and rederives every final grayscale pixel from that frame's liquid
fractions. Integer face indices are required before conversion to dictionary
keys, preventing fractional indices from aliasing valid faces. Missing finals,
finite changed fields, nonfinite fields and malformed/wrong-pixel PNGs reject.
Imposed fixtures export no final face/geometry/PNG triplet: their single imposed
face field and final fractions retain their separate manufactured oracle.

New controls exercise ten corrupt-final variants on all six coupled scenarios,
in both Python modes, alongside the twenty original controls. The native
thirteen-scenario packet and sixty-four coupled intervals must still pass.
The guarded successor plotting run copies immutable inputs into temporary
storage, verifies that copy, regenerates the independent summary and invokes
the frozen plot code there. It saves the new figure separately and never writes
over the historical figure or receipts.

## CI changes

Both pull-request and main-push path lists cover `examples/**`, replacing the
incomplete explicit example list. They also cover both column-interface reader
files and `evidence/column-interface/**`. A direct test checks each of the
candidate's twenty-three example paths, both verifier paths and a final native
PNG as individually changed files for each event. This closes the reported
eleven-example omission and also covers future examples. It changes triggering,
not solver settings or the CI job's existing checks.

The parent reports that CI run `37424773368` ran from 06:38:53 to 06:54:12 and
ended cancelled. Formatting and all strict lints passed, 237 default and 231
core-only tests completed, desktop dependencies compiled for 3m16s, and 85
desktop tests ran before cancellation. Smoke export and lifecycle checks were
skipped. The log says operation cancelled; exhaustion of the 15-minute job
budget is a supported diagnosis, not a confirmed terminal timeout label.
Direct API retrieval is forbidden in this environment, so these run facts are
preserved with their parent-reported provenance and the frozen workflow source.

The smallest configuration repair retains the shared serial job/cache and raises
its budget to thirty minutes. The original serial compilation/testing already
exhausted fifteen minutes before completion of desktop tests and the separate
release CLI build/smoke/lifecycle stages. Thirty minutes reserves another old
job budget for the remaining builds/checks and ordinary runtime variation. A
byte comparison of the whole job after substituting the old timeout proves that
no feature mode, command, assertion or evidence-upload step was removed. Local
remaining-stage runs are measured separately; they are warm-cache execution,
not evidence that a fresh hosted runner will finish in the same time. A new
hosted run must complete every stage before CI qualification can be claimed.

## Limits this repair does not close

`tools/verify_viscosity.py` validates isolated manufactured modal behavior and
the coupled demonstration's finite fields, energy, phase ledger, divergence,
versions and pixels. After the first symmetric force pulse it does not
independently replay coupled vector transitions through advection, forcing,
viscosity and pressure. The coupled energy comparison is consequently not a
complete dynamics oracle: a same-energy divergence-free field change can evade
those checks if corresponding pixels are changed. The existing isolated
viscosity tests do not supply that missing coupled transition check.

The column-interface reader independently checks the initial pulse/activation
pressure and all mixed-rest pressures, later held-wet divergence, atmospheric
air storage and held/end geometry. It does not reconstruct the later changing
wet pressure solve or pressure/velocity correction from the prior frame. A
finite fabricated later wet pressure can therefore evade the gate. This fails
to cover an inference that all later pressure values are numerically correct;
classification activation and accepted publication are narrower evidence.

These limits remain explicit scope witnesses, not positive physical
qualification. The newer coupled-discrete/third-component replay applies to a
different declared model and does not repair either older reader. No new Lean,
moving-cap pressure accuracy, general liquid or continuum validation claim is
made by this artifact/CI repair.
