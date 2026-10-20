# Bounded dimension-guard replan, before source edits

PR141 explicitly anticipates the Layout n2..8 ceiling and authorizes a bounded
n16 extension. Actual source has two further independent limits:

- `assembly.rs::Operators::build`: positive active basis length at most256.
- `solver.rs::validate` and `solver.rs::qr`: velocity row dimension at most256.

The n16 rectangle alone needs275 coefficients with unchanged support rules.
For X velocity, positive hat support gives i=2..14 (13) and j=3..13 (11),
or143. For Y velocity, i=2..13 (12) and j=3..13 (11), or132. These strict
support inequalities are separated from the represented polygon endpoints;
no equality, floor or ambiguous tiny support is involved. Thus changing only
Layout cannot implement the admitted nine cases. This is source reasoning,
not a selected assembly, solve or numerical probe.

Proposed smallest correction: retain the algorithms and formulas, extend the
private layout ceiling to16, the active-velocity and solver velocity ceilings
to the admitted full-grid upper bound544, and bound pressure rows by256. Add
checked n/node/dense-size arithmetic and above-ceiling refusal tests. Keep
the same Cholesky, QR, original full target and exactly three compensated
corrections. The full-grid KKT upper bound is800; no baseline KKT solve is
required for this pilot. Extend no production owner and select no new backend.

Alternatives: silently coarsening to8 violates admission; copying a second
solver creates divergent ownership; deleting supports/pressures changes the
method. All are rejected. Frozen CV2 code remains at b0a9d52 and prior branch
head902f609; this child will receive a distinct source freeze if admitted.
It must not claim that the modified solver source is byte-identical to CV2.

Disposition: pending independent prospective review. No dimension guard has
been changed at the time this replan was recorded. A review rejecting this
bounded extension stops source implementation and returns the exact write set
and reason to the repository lead. No numerical execution follows from writing
or committing this document.
