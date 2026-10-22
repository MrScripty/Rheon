# M3 actual native software-rendered observation

2026-10-22, source commit `8a5fdcbd850e184c53c9fe6bacaa04777c2aea76`, unchanged native binary SHA-256 `731b13b7bed5657968c38a692490892ed4345a42442053a933382be56d722b14`. This supplements the earlier truthful unavailable-display attempt. It does not rewrite that failure as success or claim physical GPU/vendor verification, simulation, persistence or whole-feature acceptance.

## Supported saved-environment capability

Parent requested inspection of already-available supported display capability, explicitly forbidding retry/bypass of the proxy403 package request. Read-only inspection found installed X.Org1.21.1.16, `dummy_drv.so`, GLX and Mesa swrast libraries. No package request, install or alternate metadata route was used. A new private dummy Xorg display `:97` was started with its own mode1600×1000, caller-owned config/logs and a random0600 Xauthority file under `/workspace/.voxel-display`. It uses `-nolisten tcp`, `-auth`, `-novtswitch`, no auto-added devices and no user desktop. No access-control or sandbox relaxation was used; the cookie is not published or logged. Xorg confirms `Loaded and initialized swrast` and `GLX: Initialized DRISWRAST GL provider`. Missing optional physical input/DBus drivers and VGA-arbiter warnings were not retried or escalated; XTest virtual input and GLX are available.

Actual application command: `DISPLAY=:97 XAUTHORITY=/workspace/.voxel-display/m3.xauth LIBGL_ALWAYS_SOFTWARE=1 target/debug/rheon-desktop`. It creates the actual1250×820 `Rheon / Live 3D` native window. Shader-cache writes under the read-only home fail and Mesa disables them; no protected-path modification/retry was used. Native input was delivered using installed libX11/libXtst's standard XTest extension. Screens were captured from this native window using installed ImageMagick and actually inspected by root. The helper/config/images/logs are verification outputs outside Git, not a replacement application or widget-only harness.

## Bounded actual observations

The app's existing default32³,1m document was used (32768 cells), rather than the prospective8³ example; no numerical owner/worker/step, recording, benchmark, emitter or provider was started. The controls' initial lattice was left fixed. This is a bounded idle authoring observation, not performance evidence.

- New Initial contents shows32768 Vacuum cells and an actionable full-occupancy refusal. Explicit Fill domain with Air yields32768 occupied Air cells, zero Water/Vacuum, revision1 and positive displayed gas mass1.204129kg.
- Radius was keyboard edited to0.18m. Z layer16 is physically0.515625m. Real numeric Center X/Y were edited to0.4/0.6; Tab from the Y edit reached the actual Apply dab button and Space activated it. Native Layer result:106 Water,32662 Air, zero Vacuum, revision2. An independent cell-center disc count gives106 and3.2290405273437504kg Water. Named Undo gives0 Water/revision3; Redo returns106/revision4.
- Pixel inputs have finite resolution: a click at601,394 corresponds approximately to(.3872512342,.5780217230,.515625) and produces103 Water cells, matching the independent inverse-camera/disc count. A click604,389 corresponds approximately to(.40163598830363423,.5999272048450459,.515625) and produces105. These were not falsely equated to the exact(.4,.6) keyboard center. Typing the latter inverse-camera coordinates through the native fields and activating Apply gives105 and an identical viewport Water appearance mask to its pointer dab:3278 pixels, mask SHA-256 `0b48cab664915fa8aed31feae3f0e11c7ce47abeeb85def84b08dfa9ea959f46`. Pixel-mask equality is appearance evidence; exact cell-ID ownership is separately covered by core/in-process contracts.
- Escape during a live candidate preserves committed105 cells and revision10. Remove at that support creates105 Vacuum cells, zero Water and revision11, with first Vacuum[12,13,16] and explicit launch refusal. Replace Air on Vacuum is an exact no-op/revision11; Add Air explicitly fills the holes, restoring32768 Air and revision12. No ambient refill occurs.
- Volume Replace Water at the same center/radius yields803 Water,31965 Air, zero Vacuum and revision13. Independent three-dimensional center-distance counting gives803. Great-circle depth preview and an inside-domain zoom were viewed; slice polygons remain visibly clipped to the viewport/near region. These views do not prove arbitrary camera/geometry robustness.
- Actual native numeric typing of0.1357 does not move the camera or change contents; text-local Ctrl+Z restores radius0.18 without undoing contents. Native invalid terminal release into the panel cancels the entire candidate. Committed803/revision13 and the3278-pixel Water mask remain unchanged through text editing/undo and invalid release.
- Actual physical window sizes1250×820 and900×650 were inspected at100% and150% egui UI zoom, using the built-in Ctrl+Plus five increments of0.1 and Ctrl+0 reset. This is verified application UI zoom, not a claim about physical monitor DPI. In compact layouts the existing top run strip replaces the settings sidebar. The Scene scroll region exposes operations, numeric fields, totals and disabled Simulate in separate scroll positions; no horizontal control clipping was observed in those inspected positions. At150% long phase totals wrap to additional lines. World radius remains0.18 and total803/revision13 through resizing/zooming/scrolling.

All retained results/recording/native numerical execution, fresh-process saved input and end-to-end A6 remain unimplemented/pending. The native idle check is evidence for M3 only. Root awaits independent evidence/source disposition before labelling M3 accepted; the parent prospectively permits independent M4 data/persistence implementation without pretending M3 acceptance.

## Local artifact identities

Actual native screenshots remain `/workspace/.voxel-display/`, outside tracked generated outputs. They contain this application and verification input only, no proprietary reference assets.

| File | SHA-256 |
| --- | --- |
| `m3-native-water-dab.png` | `8eac98e49811906f9b66afd9ba2d63dc29eaa51086d0861c74851577c2fa128b` |
| `m3-native-undo.png` | `b4e7345f3d53e9a31e3049b51ca340d1e0c4bd6978a4f16b5c8dca1b844bbe45` |
| `m3-native-redo.png` | `e193245435969c0aa2103bc63235dd91272d0a8eadee51a834a4046340463288` |
| `m3-pointer-mask.png` | `21c461334b0ae4fabb75655c35285034d32eb195af60eff92fb5077516cfcf98` |
| `m3-keyboard-mask.png` | `6331fb14086ba57a7281eca371820971ecc2b2ba21356f9b88316f613c03e7ae` |
| `m3-native-volume.png` | `b61d4e256ef60c8599f8b913c41ef1a0455589bcf0bd9d4147e8c334fb24125e` |
| `m3-inside-native.png` | `526874d1c6b7e71dadb26e8b1067af43c1b481a3b62a0c556a73c662575bc64c` |
| `m3-native-900-scrolled.png` | `d31b1b0ea04ebe31a292fd537ffc3ae0ca48f973404f16ef2668ae69be14f8d6` |
| `m3-native-900-150.png` | `839aefa4b162a44f3b1a8f7f3b79fd6cf14f07a454ed4110097720185fc1bb30` |
| `m3-native-1250-150-top.png` | `4b97f5ee51f8c79ad8f39bffab23454d06de2440b03b767e9db03c71977c3681` |
| `m3-native-1250-150-details.png` | `893e4b6cc5025d20dac5c511c743eee11685874ca7ef38105a4d7f9aa57aee94` |
| `m3-cancelled-native.png` | `fdb89d9b9d764f66b23c72c526a8e8dd3ca75a6754350fc9b059f6d095a71230` |
| `m3-replace-vacuum-noop.png` | `cc2d46ee00b6cb69b4217e66578d59570f7057b113272a624caa1ce15650a04a` |
| `m3-add-air-filled.png` | `1aca2fe3e53c37f262a6aabf57a14bc1afbf05c109aaf514f4bab6a4614e4bf0` |
| `m3-invalid-release-native.png` | `bfa34a0868956066ed337d9a463665447e733c7fb37b13a2239127deb44b4e59` |

The actual application was closed through WM_DELETE_WINDOW and exited0. After confirming the sole Xorg process command exactly matched the private display/config, root terminated its PID38201; Xorg exited0. No operator desktop session or unrelated process was closed. The subsequent [separate evidence review](m3-native-evidence-review.md) recommends bounded M3 component acceptance with its stated limits.
