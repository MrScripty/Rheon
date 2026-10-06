cargo run --locked --release --bin rheon -- --size 16 --steps 12 --source-off-at 8 --output "$RUNNER_TEMP/rheon-smoke"
python3 - <<'PY'
import csv, json, os, pathlib, struct
p = pathlib.Path(os.environ['RUNNER_TEMP']) / 'rheon-smoke'
result = json.loads((p / 'run.json').read_text(encoding='utf-8'))
assert result['size'] == 16 and result['steps'] == 12
assert result['managed_simulation_bytes'] == 333824
assert result['last_divergence_max'] <= 1e-5
assert result['whole_process_memory_cap_claimed'] is False
assert result['real_time_performance_claimed'] is False
rows = list(csv.DictReader((p / 'steps.csv').read_text(encoding='utf-8').splitlines()))
assert len(rows) == 12
assert all(float(row['actual_divergence_max']) <= 1e-5 for row in rows)
image = (p / 'opacity.png').read_bytes()
assert image[:8] == b'\x89PNG\r\n\x1a\n'
assert struct.unpack('>II', image[16:24]) == (16, 16)
print('PASS executable completion manifest, all-step divergence and PNG dimensions')
PY
