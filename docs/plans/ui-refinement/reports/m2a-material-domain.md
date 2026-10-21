# Containment preflight and M2a material/domain refinement

Base: `7318023f04946171d7909119300af76e455c2926`, tree
`258d1c744f046a1b56b4ec6dab2e4edb3e9133cb`. This is an uncommitted local
continuation. Existing commits, remote branch and retained native artifacts are
preserved. No new commit/publication or native session. An unintended broad test
filter did execute numerical fixtures; its exact scope is recorded below.

## Read-only containment result: live cancellation parked

Inspected `/proc/self/{cgroup,mountinfo}`, `/proc/swaps`, visible cgroup files,
filesystem capacity/inodes and inherited rlimits with ordinary read-only calls.
No write probe, cgroup migration, mount, provisioning or permission expansion.

| Prerequisite | Observed | Result |
| --- | --- | --- |
| Linux cgroup v2 and cpu/memory/pids controllers | `/sys/fs/cgroup` is cgroup2; controllers visible | Present, insufficient alone |
| Exclusive empty writable delegated leaf, writable procs/kill | Mount is **read-only**; root and sole visible child `init` not writable. `init` is populated, has no children and no enabled child controllers | Missing; no usable delegated leaf or exclusive reservation |
| Required CPU/memory/process caps | Root cpu.max `400000 100000`, memory.max `17179869184`, swap.max `max`, pids.max `max`; `init` cpu/memory/swap/pids are `max`. cpu.max.burst is 0 at root | Required contract caps absent; do not change them |
| Dedicated empty tmpfs, `noswap`, <=32 MiB and <=1,024 inodes | Existing `/tmp` and `/dev/shm` are tmpfs, capacity **9,441,337,344 bytes**, **2,305,014 inodes**, no `noswap` mount option. `/tmp` is shared; no exclusive reservation supplied for either | Missing; spare capacity is not a bounded dedicated mount |
| Existing swap configuration | `/proc/swaps` lists no active swap | Does not satisfy the required mount-level `noswap` condition |
| Per-process limits can remain unchanged or be lowered | CPU/AS/FSIZE unlimited; NOFILE 16,384; CORE soft 0. No lowering attempted | No tighter inherited-limit conflict observed; genuine execution not admitted |
| Exact source/test contract and sealed developer artifact | Existing source7318023, ignored genuine case, reviewed supervisor/seal retained from previous checkpoint | Preserved; no executable invoked by this preflight |

Concrete blocker: this environment supplies neither the writable exclusive
cgroup leaf nor the dedicated bounded tmpfs required by the reviewed contract.
**Only live cancellation is parked.** No alternative supervisor or weaker
fallback is designed. It can resume only in an already authorized environment
with those exact resources supplied and verified. M1 remains unaccepted.

`DISPLAY` and `WAYLAND_DISPLAY` are unset. No current native inspection or new
JPEG is claimed; the existing426fd25 native evidence retains its original scope.

## Independent M2a slice

Only production file: `src/bin/desktop/settings.rs`. Material/domain preset
selectors retain their IDs, full meanings and options while bounding selected
text and opened menus to available width. Temperature editors move under named
default-closed disclosures with current °C in their stable-ID headers. Model
limits and gas reference pressure stay outside. Property help exposes existing
hover explanations through an ordinary named disclosure, with actual keyboard
input covered independently of native accessibility claims.

All units, values, ranges, default physics, conversions and `.changed()` guards
remain owned by their existing controls. The shared quantity helper, launch,
worker, cancellation, recording, Inspector and D1 are unchanged. No generic form
framework or smaller typography is introduced.

## Verification and native handoff

Five focused `settings::tests::` tests and nine `setup_ui_tests::ui_setup_` tests
pass. Locked affected desktop compilation, strict affected desktop/test Clippy,
Rust formatting and diff whitespace checks pass. Final logs are
`/tmp/rheon-ui-m2a-final-{material-tests,setup-tests,clippy,check,fmt,diff-check}.log`.
The reviewed production/test file SHA256 is
`0937b5e293fdd024641f46382900debac0b6f8c263d2a3c5d61fc0f8191dc102`.
[Independent admission review](m2a-plan-review.md) and
[source review](m2a-source-review.md) found no remaining newly introduced material
source issue under the explicit evidence limits below. Neither review accepts
native/general keyboard usability or excuses the test-selection incident.

Initial narrow-form regressions exposed the gas density-unit selector's
100px default minimum; reducing that minimum to80px retains complete unit labels
and fits the row. All three unit choices and both form widths are covered.
Keyboard focus traversal needs a settling frame in egui; actual input remains
the test oracle, without forced focus. Failure logs are preserved.

### Open numeric-keyboard defect (UI-5)

Forward Tab through the Air material editors without typing changed density
from `1.2041288413018594` to `1.2041300000000001` kg/m³ and molar mass from
`0.02896546` to `0.028965499999999998` kg/mol in the original forward-traversal
test (`/tmp/rheon-ui-m2a-tests-4.log`). The pre-slice source already used the same
eight-decimal formatting/conversion path; pinned egui0.33.3 `drag_value.rs`
reparses cached formatted text on lost focus. This is an existing numeric-editor
defect, not deliberate editing or a harmless test artifact. It remains open;
repair needs an explicit shared-editor admission and independent regression.

The focused help test now uses actual Tab to the first preset selector, then
Shift+Tab to the final Property help disclosure, Enter to open and Space to close,
then Tab away. This proves that isolated form route and unchanged configuration
for it; it does not prove full-application focus order, arbitrary traversal value
preservation, native keyboard behavior or accessibility. Exact strict equality
is retained; Air values were not normalized and tolerances were not weakened.

### Test-selection incident

The lead accidentally used the substring filter `ui_`, which matches full test
paths/names, including existing numerical fixture tests. It completed before
detection:19 passed,1 ignored,5.85s. The genuine supervised cancellation test was
ignored and did not run. The numerical fixtures selected unintentionally were:

- `archive_view::tests::cold_open_real_egui_navigation_and_invalid_reload_clear_selected_identity`:
  3×4×5 carrier, two0.003s steps, pressure cap2000 iterations.
- `inspector::tests::historical_selection_survives_capture_reload_and_egui_export_uses_that_step`:
  4³ particle samples, three1e-5s steps, max64 particles.
- `inspector::tests::carrier_field_selection_absence_real_egui_vti_export_and_metadata_errors`:
  3×2×4 carrier, two0.01s steps, pressure cap1000 iterations.
- `engineering::tests::egui_controls_pick_masked_and_fluid_cells_and_keep_vectors_above_tiles`:
  default12×8×6 static-obstacle pressure projection,0.05s interval,2000-iteration cap.
- `configured_comparison_tests::actual_egui_configured_page_compute_exports_and_edits_invalidate_both_linked_and_unverified_sources`:
  existing native/configured2³ comparison worker fixture, three1/16s requested
  steps per member, stdlib Python analytic pressure-adapter fixture.

This violated the no-numerical-execution instruction. The lead disclosed it
promptly, inspected the selected source paths, and did not rerun the broad
filter. Existing successful tests performed their own disposable-output teardown;
already removed fixture bytes cannot be claimed preserved. The complete test log
is retained at `/tmp/rheon-ui-m2a-all-ui-tests.log`. No manual cleanup, new
supervisor, live cancellation claim or acceptance credit comes from this run.
Subsequent runtime checks use only the explicitly inspected inert module/test
prefixes named above; compile/lint commands do not execute tests.

For native validation, apply the exact final working patch to7318023 and verify
all file hashes before building. No workload is required. At1250×820 and900×650,
100% and150% UI scale: open Fluid material and Domain & environment, inspect all
long preset/fill dropdowns, open/close both thermal sections, edit/reopen °C
values, inspect gas reference pressure, and Tab/Shift+Tab to Property help and
activate it using Enter/Space. Verify values/units, scroll access, full captions
and persistent main actions. Preserve/report known UI-5 if reproducing general
numeric traversal; do not treat it as a new M2a value-preservation pass. Preserve
actual quality85 JPEGs labelled with exact
source/patch identity and state. Do not substitute historical captures, screenshots
from another revision or headless tests. Active cancellation is a separate parked
test and is not part of this idle native handoff. No 40% word reduction or
representative user success is asserted.
