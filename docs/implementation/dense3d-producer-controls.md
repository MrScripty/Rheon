# Bounded accepted-interval producer metadata

This follow-on adds an explicit proposed-v2 metadata export around the accepted
sequence owner at public base `fee7b4a139574f87b259796b1ba8698a41d31ac1`.
It leaves the original v1 importer unchanged. The default example/export route
continues to produce v1. The new route requires `--producer-controls-v2`.
It changes metadata ownership and publication; it adds no numerical method,
training qualification, general source/force support or model-accuracy claim.

The consumer proposal is Tuldok source
`29240e4d2a9a51c5883b92bffa5a9ec9455ae64a`,
`docs/plans/sequence-field-controls/contract.md`, 5113 UTF-8 bytes, SHA256
`621500f8e5729cca52ee975b01cff9676c96c7f20550ba3b952e76c366905b7c`.
The parent supplied its exact text after independently recovering the archive
and checking all 615 payload hashes. The original local Library transfer failed;
this implementation does not claim that failed transfer provided readable bytes.
Tuldok's pinned inspector still refuses producer v2 and `producer_emitted`.
Implementing this proposed contract does not claim consumer adoption.

## Accepted ownership

The optional capture belongs to `Dense3dSequence`, which already owns the
simulation and frame writer. It observes actual call controls, samples the
previous accepted clocks/stamps, and calls physics once. When physics returns
`Ok`, it stores the actual new accepted interval before writing frame bytes.
Thus a subsequent writer error leaves the committed physics and corresponding
capture inspectable, while poisoning export and preventing completion.
Rejected/cancelled calls publish neither a new state nor an interval record.
Constructor frame zero has no interval. Requested dt is not a substitute for
accepted dt; the fixed-case capture requires the actual accepted dt to match
the proposed 0.0625 s case, otherwise export is poisoned.
That late metadata refusal occurs after physics publication: the actual
shortened interval and new owner stamps remain captured, while frame output
stays at the prior accepted frame and further stepping/output refuses.

Only the actual fixed-case inputs are admitted: boundary stamp 47/0, native f32
outward speeds [[-0.25,0.25],[0,0],[0,0]], inlet stamp 53/0 and six zero inlet
fractions. The smoke source must be absent, external force slice empty and
distributed liquid source `None`. A `Some` source containing all zero rates is
refused because a zero aggregate is not absence of a distributed control.
Unsupported requests and record/byte budgets are checked before physics.
The accepted report is checked against the current owner and copied controls.

Capture has eight inline slots. A conservative 2048-byte reservation per
interval plus array punctuation governs the bounded intermediate writer.
The actual inline optional-capture object storage is reported separately from
the existing simulation Vec-capacity payload. Neither number claims process
RSS, caller allocations, complete owner stack size or allocator overhead.
Writing/flush failure aborts export. An intermediate array may contain fewer
than eight accepted entries; it is staging, never a completed sequence.

## Exact proposed-v2 files

The complete manifest schema remains `rheon.dense3d.accepted-sequence` but uses
integer version 2. It retains every v1 manifest member and adds exactly
`controls_file` = `controls.json`, `controls_bytes` in 1..65536 and the lowercase
raw-file `controls_sha256`. Original frame bytes, their byte count and hash
remain bound. Old v1 readers explicitly refuse the new manifest; the source
pinned v1 importer SHA256 remains
`063d17a3bbc92a27c26c0f5dd4588a9478091ebbf53ce0ccb818cee6a9393b98`.

`controls.json` uses `rheon.dense3d.interval-controls`, integer version 2 and
exactly these root members: schema, version, run_binding, frames_sha256,
axis_order, side_order, speed_sign, units, provenance, intervals. `run_binding`
contains exactly schema, version, geometry, field_types, units,
pressure_semantics, config, limitations and provenance copied from the actual
manifest. Recursive comparison includes numeric types; it cannot equate a
boolean with an integer or silently change an integer copy into a float.
It excludes the raw manifest hash to avoid a circular binding.

Axes are x/y/z, sides low/high and the speed sign is positive outward normal.
The exact unit map is time:s, outward_speed:m/s, inlet_fraction:dimensionless,
source_rate:m^3/s and body_acceleration:m/s^2. Sidecar provenance has exactly
origin=`producer_emitted`, author and source_note; the two text values contain
1..1000 codepoints and no control characters. This declaration and hashes do
not prove authenticity. Independent verification of producer source, binary
and actual accepted-step evidence is still required.

There are exactly eight ordered interval objects. They contain start_frame,
end_frame, start_time_s, end_time_s, dt_s, carrier_before, carrier_after,
liquid_before, liquid_after, boundary_stamp, inlet_stamp, outward_speed_m_s,
inlet_fraction, source_mode, source_rate_m3_s and body_acceleration_m_s2.
Before/after frame numbers, clock values and canonical decimal-u64 stamps
match the adjacent raw accepted frames. Controls retain the fixed values above;
source_mode is none, source_rate_m3_s is numeric zero and body acceleration is
three numeric zeros. All values are finite; booleans are not numbers.

The native example writes a private `intervals.json` staging array directly
from the owner's captures. The Python exporter never constructs interval
controls from the old global configuration or reconstructs them from fields.
It binds the actual captured records to source/binary provenance and original
frame bytes. The independent v2 validator first applies the unchanged v1
numerical contract to an explicit in-memory v1 manifest view, then checks the
new raw hashes/byte counts, exact binding and every interval association.
No v1 file is written or silently upgraded by validation.

Frames and controls are flushed/synced before completion. The exporter removes
private staging, syncs the containing directory, independently validates the
whole proposed-v2 output and publishes `run.json` atomically with no overwrite,
using the existing publication helper. Failure cannot mark a partial run
complete and does not rerun physics. All three finished files are bounded and
bound by their counts/hashes where applicable; the manifest remains at most
64 KiB and frame/line limits remain 2 MiB/256 KiB.

## Evidence scope and source families

Focused native tests use the existing small fixture to check changing control
ownership, before/after snapshots, shortened accepted dt, preflight refusals,
cancelled/rejected calls and post-commit output failures. Actual tiny-frame
and capture comparisons establish interval serialization at that small
geometry. They do not masquerade as the fixed 16x8x4 completed v2 dataset.
Static authored fixtures exercise strict full-file validation and publication
failure paths without executing the sequence example or exporter pilot.
Authored fixtures are identified as tests; they are not producer execution
evidence or relabeled stored v1 data.

No new full pilot, held case41/geometry-band simulation, broad refinement,
training or model-quality campaign is part of this work. Existing frame tests
remain available; scoped qualification avoids launching the full pilot.
The existing repository Rust CI may run its ordinary public native regression
fixtures and bounded smoke regression. Those existing checks are retained;
they do not invoke the dataset exporter or constitute a held simulation campaign.

The proposal specifies no producer source-family wire member. It cannot be
extended silently without changing the exact consumer contract. The existing
nine frames remain one correlated trajectory, not nine independent samples or
independent train/test sets. Tuldok's source-family graph and its treatment of
unselected/deleted bridges remain consumer responsibilities. Exporting explicit
producer group identifiers requires a separately coordinated contract extension;
implementation provenance alone does not establish independent families.

The relayed phrase `source_rate_m3_s0` lacks a separator. The source currently
isolates `source_rate_m3_s` as the candidate key with numeric zero; exact fixture
or key confirmation is required before ordinary draft publication.
