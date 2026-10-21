# Narrow scaled timeline repair

Native validator reported clipping of the track selector and time captions at
900×650physical/150%, with the setup280logical px and collapsed Scene64px panels.
This separate repair follows5eab3ac and changes only timeline layout plus its inert
regression. Runtime-test preparation remains separate.

The selected track text is laid out against the full available row minus the
button decoration budget; ComboBox moves to a new row before its inner horizontal
layout begins when needed. Captions are prewrapped to the full row and placed as
whole widgets. Full labels, units, fonts, timeline-track identity, popup choices,
keyboard/input and playback-state transitions remain. No additional fixed height.

A corrected exact UI regression reproduced real track text clipping before the
fix and passes after it for both tracks at100%/150%, cold and second paint. It uses
native scale via RawInput and literal presentation data, not App.update/history/
Worker/physics/SQLite. An earlier scheduled-zoom setup failure is preserved but is
not the deciding production regression. Final five individually exact inert timeline
probes pass against198-name enumeration; locked desktop check1.15s, strict production
Clippy2.82s, formatter and diff checks pass. Independent source review and exact
source/log seals are committed. Manifest SHA256:
`1bbfbc8deeb2c40f149af2d9b073640ac74734bc75647ab44efca54ecd50e0cb`.

Native repair remains UNVERIFIED. Existing validator owns the next native check;
no competing session was launched. At the exact repair revision, inspect both track
labels, selected time/cost captions, speed editing and popup at900×650/150% and
1250×820/100%, with Scene hidden and the ordinary setup sidebar present. Preserve
compact content-driven height while permitting necessary row growth. No simulation
is required for the already labelled workerless synthetic playback harness; it is
presentation data, not force/worker evidence. Any actual JPEG must carry exact source,
state and synthetic annotation at quality85; no new JPEG/Library ID is claimed here.

Parent-reported8148fa2 synthetic playback/tracks/speed precision, Draft/Retained
restoration, scale and close checks passed. Production native sessions twice stalled
at initial frame: picking/outlines/handles/Escape/camera isolation/property sync remain
UNVERIFIED. Parent says owned processes stopped and older databases/evidence unchanged.
The native owner diagnoses those stalls separately. No numerical/model workload,
archive fixture, broad test filter, gate removal, cleanup or merge occurred here.
