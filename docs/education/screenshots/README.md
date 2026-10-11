# Research-book layout captures

These unaltered browser captures document the native timestep/computed flow,
sampling/transport lessons and earlier six-reference checkpoint separately.
The user explicitly requested repository-hosted screenshots; these documentation images are
a narrow exception to the normal external-only generated-output policy.
Numerical data, PDFs and disposable test output remain external. The native-app
captures in PR42 are separate and unchanged.

All sets use Chromium 151.0.7922.173 with software WebGL. Phone views are
browser emulation, not physical-device captures. All images retain their
original JPEG quality 85 bytes, with no cropping, resizing or other edits.

## Native timestep and computed flow

These eight captures use clean implementation commit
[`db9781a1a26e43bd73fb08995fdc855d42978c12`](https://github.com/MrScripty/Rheon/commit/db9781a1a26e43bd73fb08995fdc855d42978c12),
tree `9e4d034fcb29163fd92ceb5119d97747bf57c04d`.
The [provenance record](research-book-native-flow/provenance.json) binds each
image to its actual computed state, source, packet and browser qualification.
The later documentation commit adds these images without changing runtime sources.

The source pulse runs the existing native timestep in a WASM worker on a fixed
24³ grid. It ends after 80 accepted steps, at approximately 1.44536 s. The two
compact desktop views have identical field bits and accepted time; only the
camera differs. The amber outline marks injection, not collision geometry.
The tracer loses about 16.35% of its integral after injection stops. This scene
does not claim an internal obstacle, free surface, physical buoyancy or viscosity.

### Desktop: reading beside the computed volume

![Chapters, explanation and sticky accepted 3D tracer volume](research-book-native-flow/desktop-flow-reading.jpg)

### Compact view: the same computed state from two angles

![Actual Jacobi step 80 in the compact volume view](research-book-native-flow/desktop-flow-compact.jpg)

![The same fields and time after manual camera orbit](research-book-native-flow/desktop-flow-orbit.jpg)

### Phone: example, split and compact views

![390px Example view of accepted SGS step 80](research-book-native-flow/phone-flow-example.jpg)

![320px Split view retaining computed volume and readable text](research-book-native-flow/phone-flow-split.jpg)

![320px compact view with the complete domain in frame](research-book-native-flow/phone-flow-compact.jpg)

### One timestep: accepted speeds and measured ledger

![Desktop native force/projection lesson after accepted reset/retry](research-book-native-flow/desktop-timestep.jpg)

![320px phone with all face-table columns and actual timestep metrics](research-book-native-flow/phone-timestep-metrics.jpg)

## Current sampling and transport lessons

These four captures came from clean local source commit
`ae06472f96d568d2f7077561e7b0080763510366`, tree
`4f96d068eb780fa22b6eca2481504596d8db75c0`. This source tree is byte-identical to
the published implementation
[`1c14e5787420b4db305abed96863e15535ca8e21`](https://github.com/MrScripty/Rheon/commit/1c14e5787420b4db305abed96863e15535ca8e21).
GitHub App publication changed commit metadata only. The
[current provenance record](research-book-transport/provenance.json) retains
the actual capture identity, pixel sizes and file hashes.

### Desktop: inspect the sample and its eight donors

![Desktop sampling lesson with the deliberate x-face sampling-offset error and expanded donor table](research-book-transport/desktop-sampler.jpg)

### Desktop: compare transport beside the explanation

![Desktop reading view with the asymmetric periodic-transport fixture, three numerical methods and reference curves](research-book-transport/desktop-transport.jpg)

### Phone: compact split view at 320 pixels

![320px emulated phone Split view with transport curves, visible legend and readable comparison text](research-book-transport/phone-split.jpg)

### Phone: controls and measured results at 390 pixels

![390px emulated phone Example view showing asymmetric-fixture controls, fallback count and the nonzero limited sample-sum change](research-book-transport/phone-example.jpg)

## Historical six-reference checkpoint

These four captures preserve the reviewed teaching edition at
[`540bdef8fcbc0d0a55567325eba570589666bf80`](https://github.com/MrScripty/Rheon/commit/540bdef8fcbc0d0a55567325eba570589666bf80).
They show the six-reference-lesson checkpoint before the subsequent transport
lessons. Pixel sizes and SHA256 hashes remain in
[the historical provenance record](research-book-540bdef/provenance.json).

### Desktop: chapters, reading, and a sticky example

![Desktop research book with chapter navigation, readable lesson, and pressure-correction reference](research-book-540bdef/desktop-reading.jpg)

### Phone: full-width text

![390px phone text view of the wetting chapter](research-book-540bdef/phone-text.jpg)

### Phone: compact split view

![320px phone split view with an equilibrium cap reference above readable chapter text](research-book-540bdef/phone-split.jpg)

### Phone: complete example controls

![390px phone example view with stationary-wall slip controls and numerical readouts](research-book-540bdef/phone-example.jpg)
