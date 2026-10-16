# Independent isolated CI Rust setup review

Reviewed exact commit `50732f5c0f4c9729dcb0a7696e5731d085e80a49` on
`fix/isolated-ci-rustup-20261016`, based on current main `8bf7a3ef`.
Four allocated files only. Source approved; hosted readiness remains pending.

Both retrieved failing job logs (107/116742623356 and108/116746445138)
show first-install partial-toolchain recovery followed by a cargo-clippy
conflict before repository compilation. They do not establish its deeper
runner cause. The helper avoids that state rather than repairing an existing
installation: exclusive job-owned directories, explicit newly installed
absolute tools, a pinned official bootstrap digest, explicit official servers,
repository-pinned minimal toolchain/components and checked versions before
environment publication. Existing feature tests, strict lints and numerical
qualification remain unchanged. The new contracts execute before setup in the
executable-contracts job. Other workflows retain their existing setup.

Independent actual execution on the final commit:

- `python3 -m unittest discover -s tools -p test_setup_ci_rust.py`:7passed.
- The same command under `python3 -O`:7passed.
- Final diff and exact four-file scope inspected; no credentials, native
  artifacts, compiler caches or production numerical sources added.
- Independently fetched official archive checksum and matched the helper's
  `20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c`.

The seven tests exercise an explicit fake distribution: polluted inherited
paths/selectors, fresh homes, size/checksum refusal, failed install and wrong
components/versions. They do not claim a real Rust installation. The author
separately executed an actual fresh official1.28.2 bootstrap and Rust1.92
install, then formatting, strict all-target Clippy and compilation/test-command
smoke on a dependency-free crate with zero unit tests. That evidence is not
Rheon behavior or native-window qualification. The reviewer did not rerun that
download/install. No hermeticity or reproducibility claim follows.

Official route: https://rust-lang.github.io/rustup/installation/other.html
(versioned archives and adjacent SHA-256 checksums); exact checksum endpoint:
https://static.rust-lang.org/rustup/archive/1.28.2/x86_64-unknown-linux-gnu/rustup-init.sha256.

Exact SHA-256 pins:

| Path | SHA-256 |
| --- | --- |
| `.github/workflows/rust-rheon.yml` | `769e89aadb3ed2abbcf2e72a8a722ab0de5165976de0f8f35eca0ad37e63ea76` |
| `.github/workflows/research-book-pages.yml` | `a4ab0fdddbd141d0323db8a2dd800c484c235dd23121028f5b9d14bfe20acc67` |
| `tools/setup_ci_rust.py` | `57ccd44c75d3a834034e98d934efabe4586c90ed302aa8ad64d3b202a67fc172` |
| `tools/test_setup_ci_rust.py` | `4640698e95804086a3e487088c09dac709c913f61d6b2c6c5dd3fc6178e704b7` |

Logs: `/tmp/rheon-ci-toolchain-independent-final-tests.log` and
`/tmp/rheon-ci-toolchain-independent-final-tests-optimized.log`.
No material source finding remains. Exact-head hosted gates and current-main
integration checks are still required before merge.
