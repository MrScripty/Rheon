# Comparison harness repair

Base: `0f2db35009624467a1f338700d8967a350443c93`; isolated branch
`repair/comparison-harness-contracts`. Qualification evidence from `7e1a76dd`
is on its separate branch and is not mixed into this repair.

Verified current PR3 findings:

- [Requested timestep](https://github.com/MrScripty/Rheon/pull/3#discussion_r4175917552):
  the old child argv omitted dt and its manifest value was never checked.
- [Measurement deadline](https://github.com/MrScripty/Rheon/pull/3#discussion_r4175917556):
  the old harness could poll a stalled child indefinitely.

The repair adds explicit dt passing/validation and an optional per-child wall
deadline, disabled by default. Timeout shares the TERM/KILL cleanup mechanism
with cancellation but retains a separate terminal reason. It reaps the direct
child, preserves logs/status, and stops the comparison without publication or
starting the next job. See `docs/COMPARISON.md` for API/CLI behavior and limits.

Validation command, run from repository root:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

`before.log` records regression failures against the unchanged harness (seven
tests then existed). `after.log` records all eight passing tests, including the
subsequently added real-CLI timeout exit-code test. Coverage includes explicit
separate dt argv tokens; missing/wrong manifest rejection; invalid deadlines
before output creation; real sleeping children that terminate on TERM or require
KILL; child reaping and no later jobs; preserved diagnostics; distinct user
cancellation; real CLI success with a configured deadline and failure on timeout;
and the pre-existing success/output/selection checks.

The existing release solver binary was reused, without recompiling or modifying
Rust, dependencies, numerical fixtures, book/proof artifacts, or stable solver
IDs. `receipt.json` records its hash and Python/source identities. Tests ran on
Linux; POSIX signal assertions are skipped on other OSes. No Windows process
termination or hard real-time deadline claim is made. Hosted exact-head CI and
independent native review remain integration gates, not locally claimed results.

Only trailing whitespace and surplus terminal blank lines in test logs are removed for Git whitespace
checks. No PR head, review thread, review request, or merge is changed here.
