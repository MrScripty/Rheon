# Enabled viewport context-lock repair

The focus guard introduced in425a986 called Response::has_focus and Context's
wants_keyboard_input inside Ui::input. In pinned egui0.33.3, input holds an exclusive
Context write guard; both focus queries acquire that same context. This is a
source-proven nonreentrant lock re-entry even on ordinary hover without Home/F.
The parent native diagnosis found sleeping/low-CPU stalls, but had no live stack;
attribution of those recorded stalls remains qualified until exact native repair
verification. Keep all failed native captures/logs intact.

Production repair: snapshot the keyboard-ownership boolean before Ui::input and
use that value inside. response.dragged inside the callback reads flags only.
Other camera math, event processing, input/focus behavior and gesture ownership
are unchanged. Independent inspection of eleven production read-input callbacks
and one input_mut in desktop/main found no additional nested context acquisition.
This is a bounded source inspection, not a general static-analysis certificate.

The new exact test paints the enabled production Appearance/Scene path with empty
canonical scene, no worker/frame/Inspector/model/solver; it exercises plain hover,
Home framing and F while TextEdit owns focus. It does not use the disabled synthetic
canvas. Existing exact primary drag and native-touchpad input probes also passed,
each individually, once, after list-only199-name enumeration and helper review.

Each binary invocation ran under the independently reviewed external wrapper:
10s wall,5s CPU,512MiB address space,1MiB log/file,core0; own child process group,
TERM then1s grace/KILL. All3 exited0, no timeout/signal action, captured225/190/222
bytes. Child PIDs were absent afterward. Wrapper/rawlogs/JSON/source/binary seals
are preserved. The wrapper is scoped to these no-spawn, no-fixture paths, not a
new security sandbox or live-cancellation supervisor. No pre-fix local timeout
reproduction was run; source inspection and the parent's native diagnosis motivated
the repair, and these logs establish post-fix enabled-path behavior only.

Locked desktop check1.22s, strict production Clippy2.81s, fmt/diff passed. No native
app, numerical/model workload, archive fixture or cleanup occurred. No JPEG or
Library identity is claimed. Production authored-force launches remain gated.

Native handoff: build the exact repair tip on PR146 including separate parent
scaled-timeline repaird6e23b3. First confirm idle enabled canvas accepts hover,
resize and close. Then inspect actual picking/outlines/handles/Escape/camera isolation
and property synchronization in the existing native owner's session. The earlier
8148fa2 disabled synthetic playback passes do not cover this enabled input path;
all previously unverified native gesture claims remain unverified until that owner
reports exact-source outcomes. No solver run is needed for idle force authoring.
Runtime/SQLite test preparation is a separate subsequent slice.
