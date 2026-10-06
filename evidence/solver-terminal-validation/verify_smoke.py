"""Exact existing hosted executable smoke assertions on actual new-source export."""
import csv,json,struct
from pathlib import Path
p=Path(__file__).resolve().parent/'smoke'
result=json.loads((p/'run.json').read_text());assert result['size']==16 and result['steps']==12
assert result['managed_simulation_bytes']==333824
assert result['last_divergence_max']<=1e-5
assert result['whole_process_memory_cap_claimed']is False
assert result['real_time_performance_claimed']is False
rows=list(csv.DictReader((p/'steps.csv').read_text().splitlines()));assert len(rows)==12
assert all(float(row['actual_divergence_max'])<=1e-5 for row in rows)
image=(p/'opacity.png').read_bytes();assert image[:8]==b'\x89PNG\r\n\x1a\n';assert struct.unpack('>II',image[16:24])==(16,16)
print('PASS executable completion manifest, all-step divergence and PNG dimensions')
