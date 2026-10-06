# Constructor fixture check stopped before all finite steps

Formatting, release compilation, Clippy, scalar/affine/selector preflight and fresh 67088-byte memory accounting passed. The native experiment then failed its initial bit assertion before call1: the new check used CASES[1].initial, which belongs to another family, rather than the archived pressure_state/nonconstant/reversed case47 constructor. This is a harness fixture-selection error, not an original solver refusal or a changed accepted initial state. Actual ordinary constructor output remains unchanged. No public finite call, accepted advance, Newton correction or cancellation job ran; no numerical trajectory result is claimed.

All source, first reader failures, parser correction, exact preflight and native assertion failure remain preserved. A separately frozen v2 will use the exact original case47 constructor bits from forcing-case47-public-call-v1/E2-native.log; it will not alter constructor/solver/state source, retry any finite step, or change a gate.

Compiled source: 27b9d521ab4f6ea12658a4bd7e4c0085a6ce8136; tree0414732ea85aa11e23a90780b2b6e2a61bfb818d. Passed preflight: ebc620a07a93f381dbf1dd67b96207022a3658a6; tree65e4e493d89db44c0e7f1b5032f294febc1e371b. Actual ELF SHA-256 ff7931a683e1e5c7e39cf890dec7930280b15f9b5eacef7256cc665fa71e2f4b. Native job exited101 at the fixture assertion; zero step attempts.
