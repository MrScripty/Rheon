"""Independent full-field rollout qualification; no saved expected-result fixtures.

Usage: python3 tools/verify_ai_pressure_trajectory.py NEW_ROLLOUT_DIRECTORY
NumPy reads every captured accepted frame and recomputes pressure/velocity gates,
reference errors, exact unsupported fallback equality, selection and timing ledgers.
"""
import csv
import json
from pathlib import Path
import statistics
import sys

import numpy as np

N = 32
CELLS = N**3
FACES = (N+1)*N*N
FRAMES = 24
SEEDS = (900, 901)
MODES = ('jacobi', 'sgs', 'learned', 'injected-fallback', 'shifted-jacobi', 'unsupported-fallback')
CHECKPOINT = '25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_csv(path):
    with Path(path).open() as file:
        return list(csv.DictReader(file))


def relative(a, b):
    denominator = float(np.linalg.norm(b))
    numerator = float(np.linalg.norm(a-b))
    require(np.isfinite(denominator) and np.isfinite(numerator), 'nonfinite norm')
    return numerator/denominator if denominator else (0.0 if numerator == 0 else float('inf'))


def load_frame(path):
    raw = Path(path).read_bytes()
    require(len(raw) == 16*CELLS + 4*(3*FACES+CELLS), 'field size')
    double = np.frombuffer(raw, dtype='<f8', count=2*CELLS)
    single = np.frombuffer(raw, dtype='<f4', offset=16*CELLS).astype(np.float64)
    require(np.isfinite(double).all() and np.isfinite(single).all(), 'nonfinite field')
    return double[:CELLS], double[CELLS:], single[:3*FACES], single[3*FACES:]


def qualify_fields(fields, receipt):
    rhs, pressure, velocity, tracer = fields
    require(pressure[0] == 0, 'zero-cell gauge')
    field = pressure.reshape(N,N,N)
    product = np.zeros_like(field)
    for axis in range(3):
        lo, hi = [slice(None)]*3, [slice(None)]*3
        lo[axis], hi[axis] = slice(0,-1), slice(1,None)
        flux = (field[tuple(lo)]-field[tuple(hi)]) / (N*1.25)
        product[tuple(lo)] += flux
        product[tuple(hi)] -= flux
    residual = rhs-product.flatten()
    rhs_norm = float(np.linalg.norm(rhs))
    residual_norm = float(np.linalg.norm(residual))
    predicted = float(np.max(np.abs(residual)) * N**3 / 60)
    require(residual_norm <= max(1e-12,1e-8*rhs_norm), 'pressure residual gate')
    require(predicted <= 1e-7, 'predicted divergence gate')
    # Allow only roundoff in the independently assembled residual ledger.
    budget = 128*np.finfo(float).eps*(rhs_norm+float(np.linalg.norm(product)))
    require(abs(residual_norm-float(receipt['residual_l2'])) <= budget, 'residual receipt')
    require(abs(predicted-float(receipt['predicted_divergence'])) <= budget*N**3/60, 'predicted divergence receipt')
    x = velocity[:FACES].reshape(N,N,N+1)
    y = velocity[FACES:2*FACES].reshape(N,N+1,N)
    z = velocity[2*FACES:].reshape(N+1,N,N)
    require(np.all(x[:,:,(0,-1)] == 0) and np.all(y[:,(0,-1),:] == 0) and np.all(z[(0,-1),:,:] == 0), 'sealed wall velocity')
    divergence = N*(np.diff(x,axis=2)+np.diff(y,axis=1)+np.diff(z,axis=0))
    actual = float(np.max(np.abs(divergence)))
    courant = N/60*sum(float(np.max(np.abs(v))) for v in (x,y,z))
    require(actual <= 1e-5 and courant <= 1, 'actual divergence/Courant gate')
    require(np.all((tracer >= 0) & (tracer <= 1)), 'tracer range')
    integral = float(np.sum(tracer)/N**3)
    require(np.isclose(actual,float(receipt['actual_divergence']),rtol=1e-12,atol=0), 'actual divergence receipt')
    require(np.isclose(courant,float(receipt['courant']),rtol=1e-12,atol=0), 'Courant receipt')
    require(np.isclose(integral,float(receipt['tracer_integral']),rtol=1e-12,atol=0), 'tracer integral receipt')
    return {'relative_residual':residual_norm/rhs_norm if rhs_norm else residual_norm,
            'actual_divergence':actual,'predicted_divergence':predicted}


def expected_selection(mode, frame):
    if mode == 'unsupported-fallback':
        return 'jacobi-pcg-v1', 'unsupported'
    if mode == 'injected-fallback' and frame == 9:
        return 'jacobi-pcg-v1', 'quality-gate'
    if mode in ('learned', 'injected-fallback'):
        return 'lab-modal-pressure-v1', 'none'
    return ('sgs-pcg-v1' if mode == 'sgs' else 'jacobi-pcg-v1'), 'none'


def verify(directory):
    directory = Path(directory)
    complete = json.loads((directory/'complete.json').read_text())
    require(complete.get('schema') == 'rheon-ai-trajectory-v1' and complete.get('checkpoint_sha256') == CHECKPOINT,
            'complete manifest identity')
    require(complete.get('frames') == FRAMES and complete.get('seeds') == list(SEEDS) and
            complete.get('timing_repetitions') == 3 and complete.get('fault_frame') == 9, 'frozen rollout scope')
    for key in ('model_load_ms','process_ms'):
        require(type(complete.get(key)) in (float,int) and np.isfinite(complete[key]) and complete[key] >= 0, 'process timing')
    rows = read_csv(directory/'frames.csv')
    runs = read_csv(directory/'runs.csv')
    run_keys = {(('capture' if rep == 3 else 'timed'),str(rep),str(seed),mode)
                for rep in range(4) for seed in SEEDS for mode in MODES}
    key = lambda row: (row['phase'],row['repetition'],row['seed'],row['mode'])
    require(len(runs) == len(run_keys) and {key(r) for r in runs} == run_keys, 'incomplete/duplicate runs')
    expected = {(*k,str(frame)) for k in run_keys for frame in range(1,FRAMES+1)}
    require(len(rows) == len(expected) and {(*key(r),r['frame']) for r in rows} == expected, 'incomplete/duplicate frames')
    frames = {(*key(r),r['frame']):r for r in rows}
    for row in rows:
        frame = int(row['frame'])
        require((row['selected'],row['rejection']) == expected_selection(row['mode'],frame), 'provider selection/rejection')
        numbers = {name:float(row[name]) for name in ('time','step_ms','provider_ms','inference_ms','qualification_ms','fallback_ms',
                   'residual_l2','predicted_divergence','actual_divergence','courant','tracer_integral')}
        require(all(np.isfinite(v) and v >= 0 for v in numbers.values()), 'nonfinite/negative frame receipt')
        require(np.isclose(numbers['time'],frame/60,rtol=4*np.finfo(float).eps,atol=0), 'simulation time')
        require(numbers['provider_ms'] >= sum(numbers[k] for k in ('inference_ms','qualification_ms','fallback_ms')) and
                numbers['step_ms'] >= numbers['provider_ms'], 'frame timing scope')
        require(numbers['predicted_divergence'] <= 1e-7 and numbers['actual_divergence'] <= 1e-5 and numbers['courant'] <= 1,
                'timed frame physical gate')
    for run in runs:
        require(int(run['frames']) == FRAMES, 'run length')
        initial, steps, wall = (float(run[k]) for k in ('initialization_ms','steps_ms','rollout_wall_ms'))
        require(all(np.isfinite(v) and v >= 0 for v in (initial,steps,wall)), 'nonfinite/negative run timing')
        require(wall >= initial+steps and np.isclose(steps,sum(float(frames[(*key(run),str(i))]['step_ms']) for i in range(1,FRAMES+1)),rtol=1e-12), 'run timing scope')
    maximum = {'relative_residual':0.0,'actual_divergence':0.0,'predicted_divergence':0.0}
    errors = {mode:{'velocity_relative_l2':0.0,'tracer_relative_l2':0.0,'centered_pressure_relative_l2':0.0}
              for mode in ('learned','injected-fallback','sgs')}
    for seed in SEEDS:
        for frame in range(1,FRAMES+1):
            loaded = {}
            for mode in MODES:
                loaded[mode] = load_frame(directory/f'{seed}-{mode}'/f'{frame:03}.bin')
                receipt = frames[('capture','3',str(seed),mode,str(frame))]
                for metric,value in qualify_fields(loaded[mode],receipt).items():
                    maximum[metric] = max(maximum[metric],value)
            # Unsupported-domain inference is never called; complete fallback
            # evolution must match its own equally shifted reference bit for bit.
            require(all(np.array_equal(a,b) for a,b in zip(loaded['unsupported-fallback'],loaded['shifted-jacobi'])), 'unsupported fallback differs from reference')
            for mode in errors:
                _, p, v, t = loaded[mode]
                references = ('jacobi','sgs') if mode != 'sgs' else ('jacobi',)
                for reference in references:
                    _, rp, rv, rt = loaded[reference]
                    values = {'velocity_relative_l2':relative(v,rv),'tracer_relative_l2':relative(t,rt),
                              'centered_pressure_relative_l2':relative(p-p.mean(),rp-rp.mean())}
                    for metric,value in values.items():
                        require(value <= 1e-5, f'rollout error {mode}/{reference}/{metric}')
                        errors[mode][metric] = max(errors[mode][metric],value)
    timing = {mode:statistics.median(float(r['rollout_wall_ms']) for r in runs if r['phase']=='timed' and r['mode']==mode) for mode in MODES}
    return {'schema':'rheon-ai-trajectory-qualification-v1','timed_frames':864,'captured_frames':288,
            'frames_per_trajectory':FRAMES,'simulated_seconds':FRAMES/60,'public_lab_seeds':list(SEEDS),
            'maxima':maximum,'errors_against_numerical_references':errors,'median_complete_rollout_ms':timing,
            'model_load_ms':complete['model_load_ms'],'main_body_including_capture_ms':complete['process_ms'],
            'scope':'native single-thread CPU; fixed sealed lab scene; independent 24-frame trajectories; known public seeds; no generalization or analytic-inverse superiority claim'}


if __name__ == '__main__':
    print(json.dumps(verify(sys.argv[1]),indent=2,allow_nan=False))
