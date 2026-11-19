# Pre-timing artifact inspection correction

No timing has run. Primary-worker inspection mistakenly invoked GNU objcopy with
`--dump-section` and only an input object; without an output object it rewrites
that input. Consequently the independent execution-image review binds diagnostic
rewritten files, not raw Cargo outputs. Its exact hashes/comparison remain useful
for those inspected bytes, but cannot establish untouched compiler-artifact
provenance. Preserve the report, hashes and binaries; do not silently substitute.

Both unremapped and remapped initial inspection inputs were affected. The printed
pre-objcopy raw SHA256 values were:

- unremapped base92836290b3d17942e5c87af6a3bc0fb15f1316cacc27ddc57454325adaced72a;
- unremapped candidate30b43b7a8ef9d0003507a66716c697e682fcee26901a7ec9bd821de1c1aa38b1;
- remapped based1473cb7b75965d489d7cc59c6dec3529890d7978b0584f3fe08f31d2cdda4a4;
- remapped candidate7264603aa88c22073a7c58e7a294a992f0cf4459a750d7b1d968fe575a851bb9.

These were observations, not a guarantee that later inspected paths retained them.
Diagnostic remapped files are retained under /workspace/pr59-diagnostic-rewritten;
unremapped diagnostic copies under /workspace/pr59-initial-binaries. No outcome
or timing is discarded. No strip/build-ID adjustment/layout tuning is authorized.

Regenerate raw artifacts with unchanged source bytes and the identical frozen
recipe. Touch only harness source mtimes to make Cargo rebuild the example; retain
source hashes and compare to committed bytes. New read-only review must bind actual
rebuilt outputs. Use direct read-only ELF parsing/readelf; never objcopy the original
input. Final ordered-source control cohort remains unrun; stopping rules unchanged.

The first correction command used the base worktree as cwd, so repository-relative
report writes failed with FileNotFoundError; binary preservation and raw base rebuild
did succeed. Correction report writes were retried in the actual feature checkout.
