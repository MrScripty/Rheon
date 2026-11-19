# Independent M3 native evidence review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer and native operator. Reviewed [actual native software-rendered evidence](m3-native-software-evidence.md), selected actual images under `/workspace/.voxel-display`, the local oracle/input-helper/display log, existing M3 contracts, and the previously corrected core/UI source identities. Only this report was written. No source, test, status, ledger or plan was changed. No native session, test, simulation, build, benchmark or external action was executed by this reviewer.

**Disposition: the bounded M3 native idle-authoring gate is supported.** Combined with the resolved [core](m3-core-independent-review.md) and [UI](m3-ui-independent-review.md) reviews and the implementer's executed contract fixtures, this evidence is sufficient to recommend M3 component acceptance for the observed Linux software-rendered environment and bounded authoring behavior. The parent/plan owner records actual admission. There is no remaining required M3 source correction from this evidence review. This does not accept M4 persistence/execution, A6, the composed feature, physical GPU behavior, numerical accuracy, performance or every camera/display/keyboard environment.

The parent's prospective work-order adjustment allowed independent M4 data/persistence work; it was not used as native evidence or retroactive acceptance. New evidence now closes the particular missing native idle workflow rather than renaming the previous headless tests. The earlier no-display startup and proxy-403 package request remain truthful historical failures, unretried by this reviewer. Nothing here authorizes retrying the denied coordination comment, uncertain Library writes or shared-main integration.

## Evidence inspected and identity

The reported source is local commit `8a5fdcbd850e184c53c9fe6bacaa04777c2aea76`. A read-only hash of `target/debug/rheon-desktop` matches the report's `731b13b7bed5657968c38a692490892ed4345a42442053a933382be56d722b14`. The actual core, contents UI and viewport identities still match the corrected source reviewed earlier. The native evidence report SHA-256 at review is `641e84ffd69a53eb8f88811b6e4c6f5593262bc432f5da3ccf63226a07346954`.

I opened these 21 actual PNGs using `view_image`, rather than inferring appearance from source, a widget harness or screenshot filenames:

- `m3-native-vacuum`, `m3-air-details`, `m3-native-water-dab`, `m3-native-undo`, `m3-native-redo`;
- `m3-pointer-mask`, `m3-keyboard-mask`, `m3-live-candidate`, `m3-cancelled-native`;
- `m3-remove-vacuum`, `m3-replace-vacuum-noop`, `m3-add-air-filled`, `m3-native-volume`;
- `m3-numeric-text-native`, `m3-text-undo-native`, `m3-invalid-release-native`, `m3-inside-native`;
- `m3-native-900-scrolled`, `m3-native-900-150`, `m3-native-1250-150-top`, `m3-native-1250-150-details`.

All names above have `.png` suffix and reside outside tracked generated outputs under `/workspace/.voxel-display`. All 15 artifact hashes listed in the native report were recomputed read-only and match that report. The extra inspected images are supporting observations, not a new test cohort. I did not read the authentication cookie, change the display or invoke its input helper.

The Xorg log identifies the installed dummy driver and explicitly records `Loaded and initialized swrast` and `GLX: Initialized DRISWRAST GL provider`. The configuration has no auto-added physical devices and uses the dummy screen. The existing helper calls standard XTest motion/button/key APIs against the real window; it does not replace the app or construct an HTML/egui-only mock. The report states the authenticated, TCP-disabled private display command and software-rendering environment. These facts support an actual software-rendered native session. They are not proof of physical input-device behavior, hardware acceleration or a user desktop session. Missing optional device/DBus support and unavailable shader-cache writes are reported without a bypass claim.

## Claims checked against visible evidence and contracts

| Observed state | Independent evidence assessment |
| --- | --- |
| New document and explicit Air fill | Vacuum image shows the actionable 32768-cell full-occupancy refusal. Air image shows 32768 Air, zero Water/Vacuum, revision 1 and positive displayed Air mass 1.204129 kg. The old setup sidebar's “Empty / vacuum” is not this document's material authority. The canonical Initial contents selection, document totals and explicit fill action are the authoring state. |
| Exact numeric Layer dab | Radius is visibly 0.180 m; Z layer is 16; displayed position is 0.51562 m, the five-decimal rendering of 0.515625 m. Numeric Centers show 0.400/0.600 m and Apply dab has focus. Result is 106 Water, 32662 Air, zero Vacuum, revision 2. The reported unrounded Water mass is consistent with cell-volume times captured density; the screenshot displays rounded 3.229041 kg. |
| Undo and Redo | Native images show the Layer patch removed with zero Water/revision 3, then restored to 106 Water/revision 4. This demonstrates the visible action sequence, while exact original IDs and atomic transaction semantics come from the independently reviewed core/input contracts. |
| Paired pointer and numeric dabs | Both paired images show 105 Water, but the numeric center for the second is approximately 0.40164/0.59993 m, not the nominal 0.4/0.6 m. Their revisions differ (8 and 10), so whole images are not claimed identical. The report correctly compares only the viewport Water appearance mask and does not call that exact document-ID equality. |
| Live candidate and Escape | Candidate image explicitly shows 88 pending cell changes while committed totals remain 105 Water/revision 10. Cancelled image returns to the committed patch and keeps those totals/revision. This matches private candidate ownership rather than incremental committed edits. |
| Remove, Replace and explicit refill | Remove image shows a dark support, 105 Vacuum, zero Water/revision 11 and the first-Vacuum refusal at [12,13,16]. Replace Air retains the hole and revision 11. Add Air shows all 32768 Air, zero Vacuum and revision 12, plus the explicit Add-fills-Vacuum hint. No automatic ambient refill is inferred. |
| Volume edit | Volume mode image shows 803 Water, 31965 Air, zero Vacuum/revision 13. The visible slice is a cross-section, so its blue area does not itself expose all 803 cells. The operator reports an independent 3D cell-center count; the existing core contract supplies the reviewed full-volume predicate. |
| Numeric typing/local Undo and invalid terminal release | Typing image shows radius 0.1357 with the same view and 803 Water/revision 13; text-Undo image restores radius 0.180 with those committed totals unchanged. Invalid-release image retains 803/revision 13 and displays `Initial contents: InvalidPoint`. Together with the operator's native event sequence and the reviewed cancellation/key consumers, these support the claimed bounded native behavior. |

I separately checked the arithmetic underlying the nominal numeric Layer count, without running a test or authoring operation. Cell spacing is 1/32 m, radius in cell units is 5.76, and the Z cell center is (16+1/2)/32 = 0.515625 m. With X/Y center (0.4,0.6), the disc predicate is

\[
(i-12.3)^2+(j-18.7)^2\leq 5.76^2.
\]

For integer X indices 7 through 18, the accepted Y populations are 4, 8, 10, 10, 11, 12, 12, 11, 10, 9, 7 and 2, summing to 106. This checks the units and nominal count independently of the screenshot mask and app loop. The remaining Air count is consequently 32768−106 = 32662. For each occupied phase, volume is its cell count divided by 32768 m³, and mass is that volume times its captured phase density. These are initial-state quantities, not evidence of runtime mass conservation. Air's positive mass is consistent with material, rather than Vacuum.

I did not reexecute the operator's inverse-camera count, full 3D count or pixel-mask calculation. Their results are reported operator calculations and match the inspected displayed counts; the paired screenshot files and binary are identified. `m3-native-oracle.json` captures the nominal geometry/center/106 expectation, not a complete rerun recipe for every reported calculation. The report must retain this provenance distinction. Exact authoritative voxel IDs are established by the in-process data contracts, not by counting blue pixels. A single slice, an antialiased silhouette, an SDF/mesh or a matching mask cannot independently qualify a 3D solver initial state.

## Layout, interior access and rendering limits

The 1250×820 images at normal UI zoom show concise brush operations, material/radius, slice/depth, named numeric centers, Undo/Redo, totals, actionable refusal and disabled Simulate contents. The 900×650 normal-scale scrolled image exposes the lower controls/totals without horizontal truncation in that inspected position. At 150% application UI zoom, the 900×650 image shows a narrower remaining viewport and lower fields extending below the visible scroll area; that is evidence that scrolling is necessary, not evidence that every control is simultaneously visible. The separate 1250×820 150% top/detail positions show operation controls and wrapped phase totals/disabled launch. The report's bounded “inspected positions” wording is appropriate. This is built-in egui zoom, not monitor-DPI or universal screen-size acceptance.

Layer 16 demonstrably provides interior domain access independently of a surface hit. The close-zoom image shows slice cells and volume-preview circles confined to the viewport boundary, consistent with the inspected painter clipping. The camera origin is not recorded as an authoritative numeric artifact, so this reviewer cannot independently certify that the eye itself was inside the domain, or that this screenshot crosses the near plane. Treat “inside-domain zoom” as the operator's observation, not a separate proof of every inside/near-camera case. Prior source review covers the ray/near-distance convention and polygon clipping; neither it nor this image establishes arbitrary camera/geometry robustness. This limitation does not prevent the bounded interior-slice idle gate from being satisfied.

The actual native checks use a fixed 32³, 1 m document rather than the earlier prospective 8³ example. That deviation is explicit and still bounded; no resolution change of a nonempty document or performance extrapolation is claimed. The sampled layout and input behavior do not qualify every axis, arbitrary anisotropic native geometry, all accessibility technology, physical keyboard layout or OS focus-delivery stream. Core/input fixtures cover those admitted data/ownership cases separately. Changes to the camera/control composition require affected requalification.

## Admission boundary and remaining work

The original unavailable-display blocker is now superseded for this specific native software-rendered workflow by actual successful evidence. Keep the earlier failure as history and update current status through the plan owner, with this supplement linked. The prospective M4 work-order change remains distinct from that evidence disposition. No status file was changed by this reviewer.

M3 component acceptance here means binary initial material authoring, exact core transactions/history, bounded slice/preview interaction and the observed actual idle native workflow. It does not mean the displayed masses reach a running solver. Simulate contents is visibly disabled throughout; the old run strip reports Initial contents unavailable. No numerical worker, physics step, recording or saved-result publication was observed or claimed.

M4 must still prove its versioned saved input, immutable accepted run intent, actual voxel worker/recorder/result consumers, failure/cancellation behavior and cold rerun. A6 and whole-feature admission remain pending. Physical GPU/vendor testing, performance, thermal/phase change, arbitrary fluids and production/shared-main integration remain outside this review. There is no new required simulation or benchmark cohort to close M3's idle gate.

Only this new review report was written. No reviewer action was denied, pending or uncertain. Previously denied metadata/comment actions and uncertain Library writes were untouched.
