# Design observations

PR67 was read through gh and fetched, is OPEN at 24b3860, base main and explicit HOLD.
The saved local work branch 6ba41c7 was clean. A separate feature ref now protects the
PR67-dependent work; no owner branch or main was modified. MaterialSnapshot and
ParticleFluidSimulation expose exact accepted particles and committed pre-integration
diagnostics with distinct times. Existing write_particles_vtp validates this exact
association and does not invent constructor diagnostics. Inspector currently retains
only one latest snapshot plus 256 selected samples. Timeline PreviewFrame is appearance
only. Material configuration and controls implement Debug; their round-trip decimal
text can serve as opaque provenance, never executable integrator restart input.

SQLite is selected over a handwritten two-file data/index journal because step/field/
metadata publication needs one transaction and interruption-safe cold reopen. Native
libsqlite pkg-config is absent; bundled SQLite is selected explicitly, not ambient
linkage. Rusqlite 0.37.0 (MIT) and sha2 0.10.9 (MIT/Apache-2.0) are pinned native-only
recording dependencies. Cargo locks actual transitive identities. Bundled SQLite
uses its established transaction/checksum boundaries; SHA256 binds archive content
against accidental field/metadata corruption, not hostile database rewriting.
No trust/authenticity claim. API remains native-only as cap-std already is; browser
solver remains unchanged. Record payload limits exclude SQLite/allocator/runtime
workspace and reader/writer connection caches; whole-process cap is not claimed.
