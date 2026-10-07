# Cache-free, external-output scientific replay

From this published branch's repository root, with Python and mpmath installed:

```bash
TASK_CASE41_ATTR_DIR=$(mktemp -d /tmp/case41-primitive-attribution.XXXXXX)
python3 -B evidence/case41-stored-primitive-attribution-v1/analyze.py 80 \
  > "$TASK_CASE41_ATTR_DIR/80-normal.json"
python3 -B -O evidence/case41-stored-primitive-attribution-v1/analyze.py 80 \
  > "$TASK_CASE41_ATTR_DIR/80-optimized.json"
python3 -B evidence/case41-stored-primitive-attribution-v1/analyze.py 120 \
  > "$TASK_CASE41_ATTR_DIR/120-normal.json"
python3 -B -O evidence/case41-stored-primitive-attribution-v1/analyze.py 120 \
  > "$TASK_CASE41_ATTR_DIR/120-optimized.json"
cmp "$TASK_CASE41_ATTR_DIR/80-normal.json" \
  evidence/case41-stored-primitive-attribution-v1/analysis-80-normal.json
cmp "$TASK_CASE41_ATTR_DIR/120-normal.json" \
  evidence/case41-stored-primitive-attribution-v1/analysis-120-normal.json
python3 -B evidence/case41-stored-primitive-attribution-v1/verify.py \
  > "$TASK_CASE41_ATTR_DIR/verification.json"
```

The reader only writes stdout. Bytecode writes are disabled. Outputs go to a new
external temporary directory, leaving all tracked evidence intact. No exact ELF
or ephemeral historical cache file is needed for this scientific reader. Full
original preflight/verifier replay remains a separate retained-ELF protocol.
