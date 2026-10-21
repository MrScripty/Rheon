# Independent prospective qualification protocol review

Reviewed without execution on 2026-10-21 after the complete 152-path source
review. Subject protocol is `../protocol.md`, SHA256
`bce8489f4890076bc4ec4534f809a88972ce82bb31cb43aa67d6b04477fb68ec`.
External driver is `/workspace/pr142-run-qualification.py`, SHA256
`136201ef09fb74cbe9ea43138ea8b7f354d2412737431314627829ce4009fb9b`.
The driver was read in full and parsed as Python AST; it was not executed.

No material source/protocol mismatch found. The source review and final active
v3 metadata comparison remain prerequisites to running this exact driver on a
clean committed tree. Its output directory is exclusive and outside Git.
Preparatory target reuse is disclosed and does not establish a cold build.
It clears ambient Python optimization, records material Cargo/Rust/Python
overrides, uses locked Cargo commands and the declared two-job profile, and
requires clean unchanged HEAD before each top-level command.

The native qualifier is unchanged and independently owns its source/receipt
guard, 18-command native/rational recipe and 1800-second subprocess timeouts.
The driver checks its passed status, source HEAD and command cardinality,
binds the actual exporter bytes, and builds the packet with that fresh receipt.
Both packet suites must report exactly 26 tests and final OK without skips.
Strict all-target Clippy and ordinary tests cover core-only, default and
desktop configurations. No `--ignored`, `--allow-dirty`, older receipt input,
threshold change, cloud-task action or earlier artifact retrieval occurs.

Raw logs and a progressive command journal retain exit/failure context.
Cargo `Running ... (path)` entries identify retained test executables, and
the exporter is retained separately. Source hashes and retained executable
copies are rechecked before success. These checks do not claim exhaustive
identity of every operating-system process (for example the Python interpreter,
rustdoc's temporary executables or a child adapter), process/RSS resource bounds
or native graphical behavior. The outer driver supplies no additional timeout
for feature suites; the protocol does not select a numerical campaign or rely
on such a bound for scientific authorization.

Final log/ELF archive sealing, CRC/member audit, independent actual-result
review, hosted checks and exact-head normal merge verification are subsequent
protocol steps, not results established by this prospective review. A driver
exception must remain a failed attempt with its retained journal/logs; no
partial run may be promoted using a later run's outcomes. No execution or merge
acceptance is conferred by this document.
