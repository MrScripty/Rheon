# Separately frozen paired observation candidate

This research candidate replaces the observation wrapper only. It appends one
identical `paired_probe.rs` include to separately copied candidate/reference
modules. Their equation, point, partition, arithmetic, scalar, frozen accepted
state, material parameters, h, order16, original unknowns and pressure columns
are unchanged. The reference uses its original binary64 equation; the candidate
uses the original E2 working-affine chart. The small adapters retain those
original setup differences. Production files are restored after isolated builds.

The numerical boundary returns the original `Result<Equation, Error>` into one
caller return slot. Its match and serializer borrow that slot. Serialization is
a separate non-inlined function after the numerical call returns; no full
Equation or Point observation copy is introduced. The baseline rate, perturbed
unknown and native column are the three original 176-byte buffers. Every field
from the old FD observation is retained, with additional stored equation fields
borrowed directly. No point/partition replay, Newton correction or owner exists.

The future roster is exactly baseline then j=0..5, each once. The original
checked perturbation graph and nominal FD denominator are unchanged. Baseline
refusal stops immediately; column refusals are recorded and the remaining
distinct prescribed columns remain in the roster. No retry exists. Candidate
baseline rate bits must equal the archived order16 E2 residual exactly before
the first perturbation is attempted. `baseline-binding.json` identifies that
old record. Neither numerical test has an execution runner in this packet.

Only formatting, release compilation, Clippy, scalar/affine conformance, type
layout and static ELF preflight are authorized now. Independent acceptance of a
**complete** memory preflight is required before either roster can execute. A
known subtotal, matching symbols or this worker's own review cannot supply it.

The original failed reader, incomplete 66,816-byte subtotal, all previous
evidence and this candidate's failed preparation attempts remain unchanged.
The unchanged additional-memory cap is 67,584 bytes. This is neither a whole
process cap nor a production change or main merge.
