# Saved comparisons for configured sessions

The existing desktop comparison page accepts native carrier run sets and
configured accepted-Session run sets. Both use the same recorded selections,
physical-time alignment, pressure gauge policy, field reducers, linked slices
and probes. The configured path does not relabel a recording as native-v1 or
construct a Session or model while reading it.

## Sources and artifact versions

`run_sets::source::SavedRunSet` selects the existing manifest owner. Its
borrowed member view delegates archive readiness to that owner's `open_member`;
an embedded pin alone does not grant Ready. Native source serialization and
digests retain their original bytes. Configured intent, executor declarations,
provider pins and terminal outcomes retain their separate meanings.

The existing `SavedRecipe` and verified/unverified result types are extended
with this source union. Native recipe and result envelopes remain schema1;
configured envelopes use schema2. Decode explicitly checks the envelope version
against the validated source variant, in addition to kind, digest, analysis
version and selection coverage. A newly hashed schema1 configured payload or
schema2 native payload is refused. Unknown versions and malformed inner
declarations do not acquire meaning through untagged decoding.

Opening a saved recipe or result checks its bounded representation and digest
without opening source archives. A stored result remains unverified until its
exact pinned sources are recomputed. Verification compares the complete stored
coverage digest with that recomputation, including numerical metrics. Finite,
freshly hashed invented metrics therefore remain unverified and fail source
verification. Export uses the existing immutable publication owner and requires
a verified result.

`SavedRecipe::capture` preserves the native caller entry point;
`capture_source` accepts the explicit native/configured union. The shared source
materializer validates member/selection identity and retains the existing
combined64MiB selected-field limit. No alternate SQL store, metric engine or
numerical loop is introduced.

## Quantitative and operational presentation

The page aligns exact selected publications, shows matched and unmatched
coverage and reports actual source endpoint, stage, dt and recorded projection
interval. Configured pressure interval starts come from their owner; missing
starts are not reconstructed as endpoint minus dt. Equal physical time does
not establish equal RHS, operator or trajectory. Field units, gauges and
unavailable reasons remain those of the shared comparison library.

For configured selections, explicit computation also extracts the existing
typed receipt/runtime facts and bounded pressure-call summaries. The page
caches metadata for rendering and keeps arrays in the existing field-view
owner only after an explicit load. It distinguishes accepted pressure,
candidate rejection, provider method entry, runtime outcome and observed writer
dispatch. Missing runtime observations remain unavailable. Authoritative RHS
payload hash relations are shown only from retained stamps; equal hashes are
not execution authentication or proof of equal requests.

Summary counts and durations describe exactly the selected subset, including
gaps. Core total/inference/qualification/fallback durations retain their
pressure-call scope; they do not measure full-step wall time, isolated model
compute, startup/warmup, throughput or speedup. The page does not invent facts
about unselected history or an unrecorded accepted tail.

Linked fields retain the existing explicit-load, bounded native-sample,
shared-scale and probe controls. Preflight includes configured recordings'
additional f64 fraction payload before opening either archive. Selection,
recipe, source location or verification changes clear derived capabilities and
the old linked cache. Painting performs no archive I/O. Existing Inspector CSV
exports retain the selected run/publication/evaluation time and field units;
no second export store is added.

## Acceptance and handoff

Actual production Worker contracts compose full-pressure and correction
packages with saved pressure comparisons, independent hydrostatic expectations,
canonical gauges, linked physical probes and Inspector CSV. They distinguish
actual host quality fallback from an invalid external reply with its complete
payload digest, and exercise real egui controls, invalidation and resealed
version/kind/metric counterexamples. A separate producer process exits before
a fresh reader with no model runtime on PATH verifies saved results, linked samples and CSV after
the package is removed. These analytic fixtures establish integration behavior,
not learned-model accuracy or performance.

Final execution receipts, whole-source qualification and native-window evidence
must name their exact composed revision. Earlier component receipts remain
bound to their own sources. The existing recording, manifest, runtime and build
owners remain the implementation handoff boundaries; new physical semantics
still require qualification.
