# Native recording functionality on a software display

Qualified bounded gate at clean source
`4bcdb7f49225cd645a0934953ba7d339ff5e168d`, desktop binary SHA256
`664b3f73b63a9251e85d896e95350a74c35b6b6654ea056624c6a80018b0932e`.
External receipt `/tmp/rheon-native-display/qualification.json` SHA256
`f7a70734ec3835e0d9505f2b4f425404f8f39ed69d976c75ee50bc95e84895a1`
binds 27 artifact hashes, screenshots, interaction commands, CSV, SQLite and VTK.
This is an actual native eframe window; headless egui tests remain separate.

## Supported setup and claim boundary

No existing DISPLAY/WAYLAND/Xvfb was present. The configured Debian snapshot
endpoint returned HTTP 403. The official live Debian trixie repository was accessible;
APT verified its signed InRelease and package indexes. Six official packages were
downloaded and unpacked into `/tmp/rheon-native-display/root`, with no global
installation or configuration change. The independent reviewer additionally
verified the Debian signature, authenticated index hash and all package hashes.
An unreadable system APT retries file emitted a warning; no security setting was
changed to bypass it. The blocked snapshot endpoint was not retried.

Xvfb 21.1.16 used a private mode 0600 MIT-MAGIC-COOKIE authority, display 93 and
`-nolisten tcp`. The native window was 1250 × 820 on a 1440 × 1000 framebuffer.
Mesa 25.0.7 / LLVM 19.1.7 reported llvmpipe and `accelerated=no`; software rendering
was selected explicitly. Browser sandbox settings were unchanged. These observations
qualify display functionality only, never GPU/hardware/frame-time/RSS/real-time
performance. Owned app and display processes were stopped after the smoke.

## Real producers, selection and consumers

`recorded_carrier` created four completed publications and exited before GUI
opening. The actual app opened its SQLite archive, showed constructor ordinal 0
with four fields, then selected historical ordinal 1 among generations 0–3.
The native clipboard tracer CSV declared run 613, generation/ordinal 1,
publication 0.003 s, binary32, unit 1, accepted cell-center association. Its sample
round-trips to the exact stored f32 bits: `.006000000052154064`; latest generation 3
is `.017999999225139618`, so the historical value is observably distinct.

The GUI displayed interval pressure as f64 Pa, cell0 gauge 0, interval start 0 and
publication 0.003 s. Returning to constructor ordinal 0 while this field stayed
selected showed field-unavailable/absent labels and removed the value/CSV controls.
The GUI did not invent constructor pressure or substitute the latest publication.

GUI constructor and step 1 VTI exports pass the independent VTK 9.3.1 consumer's
all-native-bit, physical geometry, offset, association and interval checks. Step 1
bytes equal a separate cold CLI/API export and differ from latest generation 3.
A second native process reopened the same disk archive and exported step 1 with
the same bytes. This is inspection persistence, not integrator restart.

`recorded_material` separately produced six completed publications and exited.
The native app opened that archive, selected generation 1 among 0–5 and displayed
publication 1e-5 s, diagnostic evaluation 0 s, particle 0 and 64 retained particles.
Every copied particle f64 value—position, velocity, mass, temperature, density,
pressure and acceleration—matches SQLite bits; category/run/particle/time match.
GUI VTP bytes equal a separate cold API export and differ from latest generation 5.
Independent review used VTK 9.3.1 to verify all 64 particles' attributes/diagnostics.

## Independent review and honest harness failures

GPT-6.1 Sol High independent recording reviewer verified the receipt and current
binary, all 27 artifact hashes, clipboard values/metadata, selected/API byte
equality and latest distinctions, second-process reopening and five decisive
screenshots. No clipping or stale-data issue was found. It independently reran
carrier and material VTK consumers and verified package provenance. The bounded
functional native-display gate is qualified at the exact subject above.

An initial keyboard attempt lacked window focus and left the path empty; native
opening correctly refused it with unavailable selected data. Explicit focus was
established and the positive flow repeated. That attempt is not counted as a pass.

The first harness `xdotool windowclose` uses XDestroyWindow, not the normal close
protocol. External destruction exposed a winit 0.30.13 TranslateCoordinates Window
error panic, exit 101. This failure is retained in `app.log` and the receipt.
The second native process was closed with a standard WM_DELETE_WINDOW ClientMessage;
it exited 0. Independent review classified forced external destruction as outside
the chosen cold-inspection/export and normal-close contract. No source/dependency
patch or arbitrary X-server/window-loss resilience claim is made. Power-loss,
hardware performance and broader GUI resilience remain unqualified.
