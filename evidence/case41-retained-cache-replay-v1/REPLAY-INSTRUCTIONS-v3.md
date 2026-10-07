# Fresh external replay output

The old `replay_v2.py` command in `RESULTS.md` is not usable from the published
result tree: its exclusive output directory is already tracked. That instruction
and all old evidence are preserved; this additive entry point corrects the
reproduction command without deleting or overwriting them.

From the repository root, with Python/mpmath and the exact retained ELF available:

```bash
TASK_CASE41_REPLAY_DIR=$(mktemp -d /tmp/case41-replay.XXXXXX)
python3 -B evidence/case41-retained-cache-replay-v1/replay_external.py \
  --output "$TASK_CASE41_REPLAY_DIR/results"
```

Each invocation must use a new external directory. The script refuses an existing
output or a path inside this repository. It creates its detached historical
checkout beside the output, overlays only the exact archived cache bytes there,
and disables bytecode writes. It runs the unchanged archived capture reader,
read-only preflight and verifier, plus the four original negative hash controls.
No native executable is launched. The tracked `replay-v2-results` is untouched.

The original verifier/preflight still require the exact retained ELF at the
historical absolute path. Its SHA-256 is checked, not executed. On another host,
full preflight/verifier reproduction needs that ELF; capture analysis only does
not. The historical memory certificate remains incomplete.
