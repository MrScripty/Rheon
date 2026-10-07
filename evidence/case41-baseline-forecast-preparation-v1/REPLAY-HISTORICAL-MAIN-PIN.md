# Historical origin/main is a local replay fixture

The unchanged old verifier checks its local `origin/main` ref against
`9cd4587a54befa61bdfddc8e35014bd3c34f02fb`. That is the historical research pin,
not a claim that this must remain the current server main forever. The reviewed
external replay command is on `research/case41-external-cache-replay`, frozen
result `d1f2309d1e74775dd26237edb0655e3fb20df50f`.

If a later remote main has moved, replay in a separate disposable clone. Fetch
and verify the historical commit normally, then pin only that clone's local
tracking ref before the unchanged verifier runs:

```bash
TASK_CASE41_REPLAY_ROOT=$(mktemp -d /tmp/case41-historical-replay.XXXXXX)
git clone --branch research/case41-external-cache-replay \
  https://github.com/MrScripty/Rheon.git "$TASK_CASE41_REPLAY_ROOT/repo"
git -C "$TASK_CASE41_REPLAY_ROOT/repo" checkout --detach \
  d1f2309d1e74775dd26237edb0655e3fb20df50f
git -C "$TASK_CASE41_REPLAY_ROOT/repo" cat-file -e \
  9cd4587a54befa61bdfddc8e35014bd3c34f02fb^{commit}
git -C "$TASK_CASE41_REPLAY_ROOT/repo" update-ref refs/remotes/origin/main \
  9cd4587a54befa61bdfddc8e35014bd3c34f02fb
(cd "$TASK_CASE41_REPLAY_ROOT/repo" && python3 -B \
  evidence/case41-retained-cache-replay-v1/replay_external.py \
  --output "$TASK_CASE41_REPLAY_ROOT/results")
```

This does not push, change server main, change authentication or replace the
tracking ref in a working user checkout. If the historical object is absent,
fetch that exact authorized commit into the disposable clone first. The old
verifier/preflight still need the exact retained ELF at their historical path.
Tracked result directories are preserved; all new output is external.
