"""Compare a real finer endpoint to a fit excluding that endpoint."""
import hashlib
import json
from pathlib import Path
import numpy as np

P = Path(__file__).resolve().parent
D = P.parent / 'forcing-temporal-diagnosis'
reference = json.loads((D / 'first-reference-diagnosis.json').read_text())
probe = json.loads((D / 'first-probe-replay.json').read_text())
fit = next(row for row in reference['rows'] if row['kind'] == 'initial' and row['load'] == 'reversed')
actual = next(row for row in probe['rows'] if row['kind'] == 'initial' and row['load'] == 'reversed' and row['probe_status'] == 'COMPLETE')
h = actual['h']
if h in fit['component_fit']['training_h'] or actual['accepted_steps'] != 64:
    raise ValueError('actual finer endpoint must be excluded from the fit')
c1, c2, c3 = np.array(fit['component_fit']['coefficients_c1_c2_c3'])
predicted = h * c1 + h * h * c2 + h * h * h * c3
gap = float(np.max(abs(predicted - actual['error_components'])))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
print(json.dumps(dict(status='DESCRIPTIVE_ACTUAL_FINER_HOLDOUT_COMPARISON',
    training_h=fit['component_fit']['training_h'], actual_h=h,
    predicted_error_components=predicted.tolist(), actual_error_components=actual['error_components'],
    max_prediction_gap=gap, actual_geometry_max_error=actual['geometry_max_error'],
    additional_actual_halving_ratio=actual['additional_halving_ratio'],
    original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND',
    scope='An actual endpoint excluded from the fit; no extrapolated qualification or formal order theorem.',
    source_sha256=sha(Path(__file__)),
    input_sha256={path.name: sha(path) for path in [D / 'first-reference-diagnosis.json', D / 'first-probe-replay.json']}), indent=2))
