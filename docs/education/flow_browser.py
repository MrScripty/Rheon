"""Independent qualification for genuine 3D native smoke-tracer computation.

Every browser tensor comes from an observed Worker response.  The independent
checks traverse staggered faces/cells and compare a fresh native binary export;
no renderer, saved animation or interpolated picture substitutes for state.
"""
from array import array
from copy import deepcopy
from pathlib import Path
import hashlib
import json
import math
import struct
import sys
import time
from urllib.parse import urljoin

from flow_field_oracles import (JS_PROBE, check_native_curves,
                               check_results as check_field_results, fixture_cases)
from teaching_browser import (evidence_path, metrics, no_overflow, same_camera,
                              scroll_to, settle, settled_camera, snapshot)

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = 'rheon-flow-browser-v2'
FIXTURE = 'force-plume-24-v1'
METHODS = ('jacobi-pcg-v1', 'sgs-pcg-v1')
FIELDS = ('x', 'y', 'z', 'tracer')
GRID = {'counts': [24, 24, 24], 'spacing': [1/24]*3, 'origin': [0]*3}
DENSITY = 1.2
MAX_STEPS = 80
SOURCE_STEPS = 30
REQUESTED_DT = .02
PARAMETERS = {'density':1.2,'source_lower':[.30,.08,.38],'source_upper':[.50,.28,.58],
              'tracer_rate':2.,'vertical_acceleration':0.,'force_density':[.6,1.2,-.3],
              'force_lower':[.25,.10,.30],'force_upper':[.60,.45,.65],
              'memory_limit':64*1024*1024,'relative_residual':1e-9,'absolute_residual':1e-12,
              'predicted_divergence_limit':1e-7,'pressure_max_iterations':2000,
              'actual_divergence_limit':1e-5,'max_courant':1.}
SOURCE_PATHS = {'flow_labs.js': 'docs/education/flow_labs.js',
                'flow_worker.js': 'docs/education/flow_worker.js',
                'flow_field.js': 'docs/education/flow_field.js',
                'flow_volume.js': 'docs/education/flow_volume.js',
                'flow-source.rs': 'browser/solver/src/flow.rs'}
PACKET_FILES = ('rheon_solver.js', 'rheon_solver_bg.wasm',
                'native-flow-reference.json', 'native-flow-fields.bin', 'build-receipt.json')


def require(condition, message):
    if not condition:
        raise RuntimeError('Flow browser: ' + message)


def close(actual, expected, label, *, tolerance=3e-12, absolute=0):
    require(type(actual) in (int, float) and math.isfinite(actual), 'nonfinite '+label)
    require(abs(actual-expected) <= max(absolute, tolerance*abs(expected)),
            f'{label}: observed {actual!r}, expected {expected!r}')


def source_hashes(repo=ROOT):
    return {name: hashlib.sha256((Path(repo)/path).read_bytes()).hexdigest()
            for name, path in SOURCE_PATHS.items()}


def field_counts(grid):
    cells = tuple(grid['counts'])
    return {name: tuple(n+int(axis == d) for d, n in enumerate(cells))
            for axis, name in enumerate(FIELDS[:3])} | {'tracer': cells}


def index(counts, i, j, k):
    return i+counts[0]*(j+counts[1]*k)


def f32_bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def axis_rms(values, counts):
    """Adjacent-plane differences; a 1D extrusion has two exact zero entries."""
    totals = [0., 0., 0.]
    pairs = [0, 0, 0]
    for k in range(counts[2]):
        for j in range(counts[1]):
            for i in range(counts[0]):
                p = (i, j, k)
                value = values[index(counts, *p)]
                for d in range(3):
                    if p[d]+1 < counts[d]:
                        q = list(p)
                        q[d] += 1
                        totals[d] += (values[index(counts, *q)]-value)**2
                        pairs[d] += 1
    return [math.sqrt(total/count) if count else 0 for total, count in zip(totals, pairs)]


def field_diagnostics(state, dt=None, density=DENSITY):
    """Differentiate each stored face pair, including the gauge cell and walls."""
    grid = state['grid']
    n, h = grid['counts'], grid['spacing']
    counts = field_counts(grid)
    divergence = []
    for k in range(n[2]):
        for j in range(n[1]):
            for i in range(n[0]):
                p = (i, j, k)
                value = 0.
                for d, name in enumerate(FIELDS[:3]):
                    q = list(p)
                    q[d] += 1
                    value += (state[name][index(counts[name], *q)]-
                              state[name][index(counts[name], *p)])/h[d]
                divergence.append(value)
    weight = sum(state['tracer'])
    centre = spread = outside_fraction = None
    if weight > 0:
        coordinates = [[grid['origin'][d]+(p+.5)*h[d] for p in range(n[d])] for d in range(3)]
        first = [0., 0., 0.]
        outside = 0.
        points = []
        for k in range(n[2]):
            for j in range(n[1]):
                for i in range(n[0]):
                    amount = state['tracer'][index(n, i, j, k)]
                    point = [coordinates[d][p] for d, p in enumerate((i, j, k))]
                    points.append((amount, point))
                    for d in range(3): first[d] += amount*point[d]
                    if not all(lo <= p < hi for lo, p, hi in zip((.30,.08,.38), point, (.50,.28,.58))):
                        outside += amount
        centre = [v/weight for v in first]
        spread = [math.sqrt(sum(amount*(point[d]-centre[d])**2 for amount, point in points)/weight) for d in range(3)]
        outside_fraction = outside/weight
    volume = math.prod(h)
    energies = [.5*density*volume*sum(v*v for v in state[name]) for name in FIELDS[:3]]
    maxima = [max(map(abs, state[name])) for name in FIELDS[:3]]
    walls = []
    for d, name in enumerate(FIELDS[:3]):
        shape = counts[name]
        for k in range(shape[2]):
            for j in range(shape[1]):
                for i in range(shape[0]):
                    if (i, j, k)[d] in (0, n[d]):
                        walls.append(state[name][index(shape, i, j, k)])
    return {'actual_divergence_max': max(map(abs, divergence)),
            'kinetic_energy': sum(energies), 'component_energy': energies,
            'velocity_max': maxima, 'wall_max': max(map(abs, walls)),
            'courant': (REQUESTED_DT if dt is None else dt)*sum(v/s for v, s in zip(maxima, h)),
            'tracer_integral': sum(state['tracer'])*volume,
            'tracer_min': min(state['tracer']), 'tracer_max': max(state['tracer']),
            'tracer_axis_rms': axis_rms(state['tracer'], n),
            'tracer_centroid': centre, 'tracer_axis_stddev': spread,
            'tracer_outside_source_fraction': outside_fraction}


def check_state(state, *, report=None, full_3d=True):
    require(isinstance(state, dict), 'missing native 3D snapshot')
    require(state.get('schema') == 1 and state.get('fixture') == FIXTURE, 'native 3D snapshot identity')
    require(state.get('method') in METHODS, 'native 3D pressure method')
    require(state.get('grid') == GRID, 'fixed genuine 24-cubed grid')
    require(state.get('parameters') == PARAMETERS, 'fixed source, force and native acceptance contract')
    require(state.get('source') == {'lower':PARAMETERS['source_lower'],'upper':PARAMETERS['source_upper'],
            'tracer_rate':2.,'vertical_acceleration':0.}, 'actual source marker contract')
    require(type(state.get('managed_bytes')) is int and 0 < state['managed_bytes'] <= PARAMETERS['memory_limit'],
            'bounded retained native buffers, not a whole-process memory claim')
    steps = state.get('accepted_steps')
    require(type(steps) is int and 0 <= steps <= MAX_STEPS, 'bounded accepted step count')
    require(state.get('max_steps') == MAX_STEPS and state.get('source_steps') == SOURCE_STEPS,
            'bounded trajectory and source pulse identity')
    require(state.get('next_source_active') is (steps < SOURCE_STEPS), 'source switches after step 30')
    require(state.get('requested_dt') == REQUESTED_DT, 'requested timestep identity')
    require(type(state.get('generation')) is int and state['generation'] >= steps and
            state.get('generation_u64') == str(state['generation']), 'native state generation mirror')
    require(type(state.get('time')) in (float, int) and math.isfinite(state['time']) and state['time'] >= 0,
            'accepted physical time')
    require(state.get('time_bits') == struct.pack('>d', state['time']).hex(), 'accepted time bit mirror')
    counts = field_counts(state['grid'])
    for name in FIELDS:
        values = state.get(name)
        bits = state.get('bits', {}).get(name)
        require(isinstance(values, (list, array)) and len(values) == math.prod(counts[name]), 'complete tensor shape '+name)
        require(isinstance(bits, (list, array)) and len(bits) == len(values), 'complete tensor bits '+name)
        for offset, (value, bit) in enumerate(zip(values, bits)):
            require(type(value) in (float, int) and math.isfinite(value), f'finite {name}[{offset}]')
            require(type(bit) is int and 0 <= bit < 2**32 and bit == f32_bits(value), f'float32 bit mirror {name}[{offset}]')
            require(struct.unpack('<f', struct.pack('<I', bit))[0] == value, f'exact binary32 {name}[{offset}]')
    derived = field_diagnostics(state, report['dt'] if report else state.get('last_accepted_dt') or 0)
    require(derived['wall_max'] == 0, 'all six stationary wall-normal face planes remain exactly zero')
    require(derived['actual_divergence_max'] <= 1e-5, 'independent all-cell divergence acceptance')
    require(0 <= derived['tracer_min'] <= derived['tracer_max'] <= 1, 'passive tracer bounds')
    if steps == 0:
        require(state['time'] == 0 and all(value == 0 for name in FIELDS for value in state[name]), 'reset/rest fields')
    if full_3d and steps >= 10:
        require(all(value > 1e-5 for value in derived['component_energy']), 'all three velocity components have measurable energy')
        require(all(value > 1e-3 for value in derived['tracer_axis_rms']), 'tracer varies measurably along all three axes')
    check_metrics(state, derived)
    if report is not None:
        check_report(report, state, derived)
    return derived


def check_report(report, state, derived=None):
    require(isinstance(report, dict), 'native accepted report')
    dt = report.get('dt')
    require(type(dt) in (int, float) and math.isfinite(dt) and 0 < dt <= REQUESTED_DT, 'actual adaptive dt')
    derived = field_diagnostics(state, dt) if derived is None else derived
    for key in ('time', 'generation'):
        close(report.get(key), state[key], 'reported '+key)
    for key in ('actual_divergence_max', 'kinetic_energy', 'tracer_integral', 'courant'):
        close(report.get(key), derived[key], 'independently recomputed '+key, absolute=2e-16 if key == 'actual_divergence_max' else 0)
    require(0 <= report['courant'] <= 1, 'actual stored velocity Courant acceptance')
    pressure = report.get('pressure', {})
    require(type(pressure.get('iterations')) is int and 0 < pressure['iterations'] <= 2000, 'native pressure iteration count')
    for key in ('true_residual_l2', 'true_residual_max', 'predicted_divergence_max'):
        require(type(pressure.get(key)) in (int, float) and math.isfinite(pressure[key]) and pressure[key] >= 0,
                'finite pressure diagnostic '+key)
    require(pressure['predicted_divergence_max'] <= 1e-7, 'native predicted divergence acceptance')
    forces = report.get('forces')
    require(isinstance(forces, dict), 'actual localized force-stage report')
    for key in ('kinetic_energy_before', 'kinetic_energy_after', 'applied_work'):
        require(type(forces.get(key)) in (int, float) and math.isfinite(forces[key]), 'finite force diagnostic '+key)
    close(forces['applied_work'], forces['kinetic_energy_after']-forces['kinetic_energy_before'], 'force-stage energy difference')
    return derived


def decode_frame(frame, blob):
    """Decode native little-endian storage; reject gaps, overlap and truncation elsewhere."""
    require(isinstance(frame, dict) and isinstance(frame.get('state'), dict), 'native frame metadata')
    state = deepcopy(frame['state'])
    offset, size = frame.get('byte_offset'), frame.get('byte_length')
    counts = field_counts(state['grid'])
    required = 4*sum(math.prod(counts[name]) for name in FIELDS)
    require(type(offset) is int and offset >= 0 and offset % 4 == 0 and size == required and
            offset+size <= len(blob), 'complete aligned native tensor range')
    state['bits'] = {}
    for name in FIELDS:
        length = math.prod(counts[name])
        raw = blob[offset:offset+4*length]
        values = array('f'); values.frombytes(raw)
        bits = array('I'); bits.frombytes(raw)
        if sys.byteorder != 'little':
            values.byteswap(); bits.byteswap()
        state[name] = list(values)
        state['bits'][name] = list(bits)
        offset += 4*length
    return state


def state_equal(observed, expected, label):
    require(observed == expected, label+' differs from fresh native host state, including every stored bit')


def check_outcome(outcome, previous=None):
    require(isinstance(outcome, dict) and outcome.get('status') in ('accepted', 'rejected'), 'native outcome identity')
    accepted = outcome['status'] == 'accepted'
    require(outcome.get('code') == 'accepted' if accepted else isinstance(outcome.get('code'), str), 'native outcome code')
    if accepted:
        require(outcome.get('detail') is None, 'accepted outcome has no failure detail')
        derived = check_state(outcome.get('state'), report=outcome.get('report'))
        if previous is not None:
            current = outcome['state']
            require(current['accepted_steps'] == previous['accepted_steps']+1 and
                    current['generation'] == previous['generation']+1, 'exactly one native advance per accepted command')
            close(current['time'], previous['time']+outcome['report']['dt'], 'accepted time increment')
        return derived
    require(isinstance(outcome.get('detail'), str) and outcome['detail'] and outcome.get('report') is None,
            'rejection explanation without accepted report')
    check_state(outcome.get('state'))
    require(previous is not None, 'rejection needs a previous accepted state')
    state_equal(outcome['state'], previous, 'rejected candidate')
    return None


def check_metrics(state, derived):
    metrics = state.get('metrics')
    require(isinstance(metrics, dict), 'accepted tensor diagnostic summary')
    keys = {'max_u_v_w':'velocity_max', 'component_kinetic_energy':'component_energy',
            'tracer_adjacent_slice_rms':'tracer_axis_rms'}
    for key in ('actual_divergence_max', 'courant', 'tracer_min', 'tracer_max', 'tracer_integral',
                'tracer_centroid', 'tracer_axis_stddev', 'tracer_outside_source_fraction'):
        keys[key] = key
    for key, derived_key in keys.items():
        expected = derived[derived_key]
        observed = metrics.get(key)
        if isinstance(expected, list):
            require(isinstance(observed, list) and len(observed) == len(expected), 'metric vector '+key)
            for d, value in enumerate(expected):
                close(observed[d], value, f'independent metric {key}[{d}]', tolerance=2e-10)
        elif expected is None:
            require(observed is None, 'undefined rest metric '+key)
        else:
            close(observed, expected, 'independent metric '+key, tolerance=2e-10)
    cutoff = state.get('tracer_integral_at_source_cutoff')
    if state['accepted_steps'] < SOURCE_STEPS:
        require(cutoff is None and metrics.get('tracer_integral_change_since_cutoff') is None,
                'no source-off integral claim before cutoff')
    else:
        require(type(cutoff) in (float, int) and cutoff > 0, 'accepted source-off baseline')
        close(metrics.get('tracer_integral_change_since_cutoff'), derived['tracer_integral']-cutoff,
              'measured source-off integral change', absolute=2e-16)


def check_native_reference(manifest, blob):
    require(isinstance(manifest, dict) and manifest.get('schema') == 1 and manifest.get('fixture') == FIXTURE,
            'fresh native flow manifest identity')
    lengths = [math.prod(field_counts(GRID)[name]) for name in FIELDS]
    require(manifest.get('encoding') == {'format':'little-endian-f32', 'field_order':list(FIELDS),
            'field_lengths':lengths, 'bytes_per_state':4*sum(lengths)}, 'native tensor binary encoding')
    require(isinstance(blob, bytes) and manifest.get('binary_bytes') == len(blob), 'native binary byte length')
    runs = manifest.get('runs')
    require(isinstance(runs, list) and [run.get('method') for run in runs] == list(METHODS), 'both native pressure methods')
    spans = []
    for run in runs:
        frames = run.get('frames')
        require(isinstance(frames, list) and len(frames) == MAX_STEPS+1, 'all 81 native trajectory states')
        previous = None
        cutoff = None
        for step, frame in enumerate(frames):
            state = decode_frame(frame, blob)
            require(state['method'] == run['method'] and state['accepted_steps'] == step and
                    state['generation'] == step, 'native ordered trajectory identity')
            require(frame.get('kind') == ('baseline' if step == 0 else 'accepted'), 'native trajectory frame kind')
            if step == 0:
                require(frame.get('report') is None, 'native rest has no accepted report')
                check_state(state)
            else:
                check_outcome({'state':state, **{key:frame.get(key) for key in ('status','code','detail','report')}}, previous)
            if step == SOURCE_STEPS: cutoff = state['metrics']['tracer_integral']
            if step >= SOURCE_STEPS:
                close(state['tracer_integral_at_source_cutoff'], cutoff, 'preserved source-off integral')
            previous = state
            spans.append((frame['byte_offset'],frame['byte_length']))
        for key, step in [('cancelled_rest',0),('cancelled_nonzero',SOURCE_STEPS),('limit_rejection',MAX_STEPS)]:
            frame = run.get(key)
            require(isinstance(frame, dict) and frame.get('kind') == key, 'native transaction probe '+key)
            state = decode_frame(frame, blob)
            expected_code = 'step_limit' if key == 'limit_rejection' else 'cancelled_before_commit'
            require(frame.get('code') == expected_code and frame.get('status') == 'rejected', 'native rejection '+key)
            check_outcome({'state':state, **{name:frame.get(name) for name in ('status','code','detail','report')}},
                          decode_frame(frames[step],blob))
            spans.append((frame['byte_offset'],frame['byte_length']))
        reset = decode_frame(run.get('reset'),blob)
        check_state(reset)
        require(reset['accepted_steps'] == 0 and reset['generation'] == MAX_STEPS+1, 'native reset generation')
        after = run.get('after_reset')
        retried = decode_frame(after,blob)
        check_outcome({'state':retried, **{name:after.get(name) for name in ('status','code','detail','report')}},reset)
        first = decode_frame(frames[1],blob)
        for name in (*FIELDS,'bits','time','time_bits'):
            require(retried[name] == first[name], 'native repeatability after reset '+name)
        for frame in (run['reset'], after): spans.append((frame['byte_offset'],frame['byte_length']))
    offset = 0
    for start, length in sorted(spans):
        require(start == offset, 'native binary has a missing, duplicate or overlapping tensor range')
        offset += length
    require(offset == len(blob), 'native binary has unreferenced trailing bytes')
    return {run['method']:run for run in runs}


OBSERVER = r'''(() => {
  const RealWorker=window.Worker;
  window.__flowEvidence={messages:[],requests:[],workers:0,pauseAt:null};
  window.Worker=class extends RealWorker {
    constructor(url,options){
      super(url,options);
      if(!new URL(url,location.href).pathname.endsWith('/flow_worker.js'))return;
      const evidence=window.__flowEvidence, starts=new Map();evidence.workers++;
      const post=this.postMessage.bind(this);
      this.postMessage=(data,...args)=>{starts.set(data.id,performance.now());
        evidence.requests.push({id:data.id,action:data.action,method:data.method});return post(data,...args);};
      this.addEventListener('message',({data})=>{
        const outcome=data.outcome?{...data.outcome,state:null}:null;
        evidence.messages.push({id:data.id,state:data.state,outcome,runtime:data.runtime,error:data.error,
          durationMs:performance.now()-(starts.get(data.id)||performance.now())});
        starts.delete(data.id);
        // Qualification alone schedules an ordinary Pause button click at an
        // exact accepted boundary. The runtime never reads this observer.
        if(data.outcome?.status==='accepted'&&data.state?.accepted_steps===evidence.pauseAt){
          evidence.pauseAt=null;document.querySelector('#flow-run').click();
        }
      });
    }
  };
})();'''


def wait_ready(page, *, stopped=True):
    # Poll synchronously after module import. A Promise itself is truthy in
    # Playwright's polling implementation, even when its value resolves false.
    page.evaluate(r'''async()=>{
      const reader=[...document.scripts].find(s=>/\/reader\.js$/.test(s.src));
      window.__flowModule=await import(new URL('labs.js',reader.src).href);
    }''')
    handle=page.wait_for_function(r'''stopped => {
      const s=window.__flowModule.snapshot(),r=s.reference;
      return s.lab==='flow'&&r&&(r.error||r.renderError||r.included===false||
        (r.state&&!r.busy&&(!stopped||!r.running)))?s:false;
    }''',arg=stopped,timeout=120000)
    result=handle.json_value();handle.dispose()
    require(result.get('lab')=='flow' and result.get('webgl'),'actual 3D WebGL lesson selected')
    reference=result.get('reference')
    require(isinstance(reference,dict) and not reference.get('error') and not reference.get('renderError'),
            'native 3D worker/renderer failed: '+str(reference and (reference.get('error') or reference.get('renderError'))))
    if reference.get('included') is not False:
        require(reference.get('state') is not None and reference.get('busy') is False,'settled accepted state readiness')
    return result


def observer_message(page):
    return page.evaluate('window.__flowEvidence.messages.shift() || null')


def requests(page):
    return page.evaluate('window.__flowEvidence.requests')


def check_renderer(observed):
    reference=observed['reference']
    state=reference['state']
    rendered=reference.get('renderer')
    require(isinstance(rendered,dict), 'actual full-tensor renderer inspection')
    require(rendered.get('grid') == GRID, 'renderer physical 3D grid')
    require(rendered.get('extinction') == 32 and rendered.get('maxSamples') == 512,
            'fixed disclosed optical appearance, without field normalization')
    require(rendered.get('interpolation') in ('hardware-linear','manual-trilinear'), 'volume trilinear sampling path')
    close(rendered.get('nominalStep'), 1/48, 'physical ray integration spacing')
    # The renderer's tensor extrema are checked against the actual worker field.
    close(rendered.get('tracerLength'),24**3,'rendered tensor length')
    require(rendered.get('tracerRange') == [min(state['tracer']),max(state['tracer'])], 'rendered absolute concentration range')
    require(rendered.get('source') == {key:state['source'][key] for key in ('lower','upper')}, 'actual source geometry marker')
    require(rendered.get('sourceTime')==state['time'] and rendered.get('sourceGeneration')==state['generation'],
            'derived display identifies its accepted state')
    require(rendered.get('lighting') in ('lit','unlit') and rendered.get('lightExtinction')==12 and
            rendered.get('lightAmbient')==.28 and rendered.get('lightInterpolation')=='cell-center-prefix-trilinear',
            'fixed disclosed approximate depth-light appearance')
    require(rendered.get('streamlineGeometry')=='frozen-instantaneous-arclength-rk4' and
            rendered.get('streamlinePositionUnits')=='m' and rendered.get('streamlineSpeedUnits')=='m/s' and
            rendered.get('streamlineSpeedMax')==1 and rendered.get('streamlineOverlay') is True,
            'instantaneous spatial overlays retain physical speed units')



def project_domain_corners(camera,target,width,height):
    """Independent perspective projection for the source-bound 42-degree camera.

    OrbitControls keeps +Y as up. FlowVolume centers the physical unit cube;
    projection checks geometry containment, not document scroll overflow.
    """
    require(width>0 and height>0,'positive camera viewport')
    forward=[b-a for a,b in zip(camera,target)]
    length=math.sqrt(sum(v*v for v in forward))
    require(length>0,'camera outside its target')
    forward=[v/length for v in forward]
    right=[-forward[2],0.,forward[0]]
    length=math.sqrt(sum(v*v for v in right))
    require(length>0,'camera has a valid up direction')
    right=[v/length for v in right]
    up=[right[1]*forward[2]-right[2]*forward[1],right[2]*forward[0]-right[0]*forward[2],right[0]*forward[1]-right[1]*forward[0]]
    tangent=math.tan(math.radians(42/2))
    projected=[]
    for x in (-.5,.5):
        for y in (-.5,.5):
            for z in (-.5,.5):
                offset=[v-c for v,c in zip((x,y,z),camera)]
                depth=sum(v*d for v,d in zip(offset,forward))
                require(depth>.01,'all domain corners are in front of camera')
                projected.append([sum(v*r for v,r in zip(offset,right))/(depth*tangent*width/height),
                                  sum(v*u for v,u in zip(offset,up))/(depth*tangent)])
    return projected


def check_default_framing(page):
    observed=settled_camera(page)
    canvas=page.locator('#scene canvas').bounding_box()
    points=project_domain_corners(observed['camera'],observed['target'],canvas['width'],canvas['height'])
    require(all(abs(v)<=1 for p in points for v in p),'default camera must frame all eight domain corners')
    return {'width':page.viewport_size['width'],'canvas':[canvas['width'],canvas['height']],
            'max_abs_ndc':max(abs(v) for p in points for v in p),'all_eight_corners_visible':True}


def render_oracles(page):
    """Exercise shipped CPU indexing and the actual GPU shader on analytic tensors."""
    result=page.evaluate(r'''async () => {
      const reader=[...document.scripts].find(s=>/\/reader\.js$/.test(s.src));
      const {FlowVolume,beerLambertOpacity,sampleTracerTrilinear}=await import(new URL('flow_volume.js',reader.src));
      const THREE=await import('three');
      const state={grid:{counts:[3,4,5],spacing:[.2,.3,.4],origin:[.7,-.4,1.1]},tracer:[]};
      for(let k=0;k<5;k++)for(let j=0;j<4;j++)for(let i=0;i<3;i++)state.tracer.push(.001*(i+1)+.002*(j+1)+.004*(k+1));
      const probes=[[.8,-.25,1.3],[1.,.2,2.1],[1.2,.65,2.9],[.5,-.9,.8],[1.4,.9,3.5]];
      const samples=probes.map(p=>sampleTracerTrilinear(state,p));
      const extremes=[[-1e308,0,0],[1e308,0,0]].map(p=>sampleTracerTrilinear({...state,tracer:Array(60).fill(.2)},p));
      const opacities=[[.02,2,32],[.1,.25,32],[0,1,32]].map(p=>beerLambertOpacity(...p));
      const renderer=new THREE.WebGLRenderer({alpha:true,antialias:false});renderer.setSize(65,65);
      renderer.setClearColor(0,0);renderer.outputColorSpace=THREE.LinearSRGBColorSpace;
      const scene=new THREE.Scene(),group=new THREE.Group();scene.add(group);
      const camera=new THREE.PerspectiveCamera(20,1,.01,100);
      const target=new THREE.WebGLRenderTarget(65,65,{format:THREE.RGBAFormat,type:THREE.UnsignedByteType});
      const pixels=[],modes=[],lightingPixels=[];
      try {
        for(const forceManualInterpolation of [false,true]){
          const volume=new FlowVolume({group,renderer},{forceManualInterpolation,boxVisible:false,sourceVisible:false,
            lighting:'unlit',streamlines:false});
          try {
            function capture(kind,concentration,original=true){
              const captures=[];
              for(const lighting of ['unlit','lit']){
                volume.setDisplay({lighting,streamlines:false});
                renderer.setRenderTarget(target);renderer.clear();renderer.render(scene,camera);
                const rgba=new Uint8Array(65*65*4);renderer.readRenderTargetPixels(target,0,0,65,65,rgba);
                captures.push(rgba);
              }
              const center=4*(32+65*32), unlit=[...captures[0].slice(center,center+4)],lit=[...captures[1].slice(center,center+4)];
              let maxAlphaDifference=0;
              for(let i=3;i<captures[0].length;i+=4)maxAlphaDifference=Math.max(maxAlphaDifference,Math.abs(captures[0][i]-captures[1][i]));
              const metadata={mode:volume.snapshot().interpolation,kind,concentration,path:2};
              if(original)pixels.push({...metadata,rgba:unlit});
              lightingPixels.push({...metadata,unlit,lit,maxAlphaDifference});
            }
            for(const concentration of [.01,.05]){
              volume.update({...state,tracer:Array(60).fill(concentration)});
              camera.position.set(0,0,5);camera.lookAt(0,0,0);camera.updateMatrixWorld();
              capture('constant',concentration);
            }
            volume.update(state);
            // Off-centre axis-aligned ray through first x/y cell centres.
            camera.position.set(-.2,-.45,5);camera.lookAt(-.2,-.45,0);camera.updateMatrixWorld();
            capture('affine-low',.015);
            camera.position.set(-.2,.45,5);camera.lookAt(-.2,.45,0);camera.updateMatrixWorld();
            capture('affine-high',.021,false);
            modes.push(volume.snapshot().interpolation);
          } finally {volume.dispose();}
        }
      } finally {target.dispose();renderer.dispose();renderer.forceContextLoss();}
      return {probes,samples,extremes,opacities,pixels,modes,lightingPixels};
    }''')
    for actual,expected in zip(result['samples'],(.007,.019,.031,.007,.031)):
        close(actual,expected,'independent x-fastest affine interpolation',absolute=2e-8)
    for actual in result['extremes']: close(actual,.2,'extreme finite coordinate clamps before division',absolute=2e-8)
    for actual,parameters in zip(result['opacities'],((.02,2,32),(.1,.25,32),(0,1,32))):
        close(actual,-math.expm1(-math.prod(parameters)),'Beer-Lambert helper')
    require(len(result['pixels']) == 6 and 'manual-trilinear' in result['modes'], 'both GPU interpolation paths')
    for item in result['pixels']:
        expected=-math.expm1(-32*item['concentration']*item['path'])
        close(item['rgba'][3]/255,expected,'actual GPU physical-path opacity',absolute=2/255)
    require(len(result['lightingPixels'])==8,'all actual lit/unlit GPU pairs')
    tint=[value/12.92 if value<=.04045 else ((value+.055)/1.055)**2.4 for value in (22/255,127/255,136/255)]
    for item in result['lightingPixels']:
        require(item['maxAlphaDifference']==0,'lighting preserves every rendered pixel alpha exactly')
        accumulated=radiance=0.
        ray=[]
        for sample in range(20):
            if item['kind']=='constant':
                concentration=item['concentration'];light_depth=12*concentration*.6
            else:
                z_index=max(0.,min(4.,((sample+.5)*.1)/.4-.5))
                concentration=(.003 if item['kind']=='affine-low' else .009)+.004*(z_index+1)
                light_depth=12*.3*(3.5*concentration+.012 if item['kind']=='affine-low' else .5*concentration)
            # The camera sees +Z first. Reverse-order accumulation below uses
            # the same physical column integral but different front weights.
            # Collect samples before integrating in camera-to-domain order.
            ray.append((concentration,light_depth))
        for concentration,light_depth in reversed(ray):
            weight=math.exp(-32*accumulated)*(-math.expm1(-32*concentration*.1))
            radiance+=weight*(.28+.72*math.exp(-light_depth));accumulated+=concentration*.1
        alpha=-math.expm1(-32*accumulated)
        close(item['lit'][3]/255,alpha,'lit actual GPU opacity',absolute=2/255)
        for observed,channel in zip(item['lit'][:3],tint):
            close(observed/255,channel*radiance,'actual GPU fixed +Y depth-light RGB',absolute=2/255)
        require(sum(item['lit'][:3])<sum(item['unlit'][:3]),'disclosed lighting visibly changes RGB')
    return {'affine_indexing_probes':5,'beer_lambert_probes':3,'gpu_alpha_probes':6,
            'gpu_paths':result['modes'],'actual_shader_alpha':True,'extreme_coordinate_probes':2,
            'gpu_lighting_alpha_pairs':8,'gpu_lighting_gradient_probes':2,
            'lit_unlit_alpha_unchanged':True,'rgb_matches_disclosed_depth_law':True}


def field_oracles(page):
    cases=fixture_cases()
    url=page.evaluate("new URL('flow_field.js',[...document.scripts].find(s=>/\\/reader\\.js$/.test(s.src)).src).href")
    result=page.evaluate(JS_PROBE,{'url':url,'cases':cases})
    return {**check_field_results(cases,result),'pure_module_exercised_in_browser':True}


def derived_snapshot(page):
    return page.evaluate(r'''async()=>{
      const reader=[...document.scripts].find(s=>/\/reader\.js$/.test(s.src));
      return (await import(new URL('labs.js',reader.src))).snapshot({flowDerived:true});
    }''')


def display_controls(page):
    before=settled_camera(page)
    before_requests=requests(page)
    before_pixels=canvas_pixels(page)
    page.locator('#flow-diagnostics').evaluate('node=>node.open=true')
    legend=page.locator('#flow-velocity-legend')
    require(legend.is_visible() and 'm/s' in legend.inner_text(),'fixed physical speed legend is visible')
    require(legend.evaluate("node=>CSS.supports('border-image','linear-gradient(90deg in srgb-linear,#a96629,#f5ce80) 1') && getComputedStyle(node).borderImageSource!=='none'"),
            'browser renders the fixed linear-colour speed gradient')
    page.locator('#flow-streamlines').uncheck();settle(page)
    hidden=snapshot(page)
    require(hidden['reference']['renderer']['streamlinesVisible'] is False and not legend.is_visible(),
            'velocity control hides actual overlay and its legend')
    bare_lit=canvas_pixels(page)
    require(pixel_difference(before_pixels,bare_lit)>.05,'actual instantaneous overlays change canvas pixels')
    page.locator('#flow-lighting').uncheck();settle(page)
    unlit=snapshot(page)
    require(unlit['reference']['renderer']['lighting']=='unlit','lighting control changes actual renderer mode')
    require(pixel_difference(bare_lit,canvas_pixels(page))>.05,'actual accepted field changes colour with depth lighting')
    page.locator('#lab').dispatch_event('change')
    remounted=wait_ready(page)
    require(remounted['reference']['renderer']['lighting']=='unlit' and
            remounted['reference']['renderer']['streamlinesVisible'] is False and
            not page.locator('#flow-lighting').is_checked() and not page.locator('#flow-streamlines').is_checked(),
            'display choices survive an accepted-state remount')
    page.locator('#flow-diagnostics').evaluate('node=>node.open=true')
    page.locator('#flow-lighting').check();page.locator('#flow-streamlines').check();settle(page)
    after=settled_camera(page)
    state_equal(after['reference']['state'],before['reference']['state'],'display toggles preserve accepted full fields')
    same_camera(before,after,'display toggles/remount preserve manual camera')
    require(requests(page)==before_requests,'display toggles/remount never submit native requests')
    require(pixel_difference(before_pixels,canvas_pixels(page))<.01,'restoring display choices restores actual pixels')
    page.locator('#flow-diagnostics').evaluate('node=>node.open=false')
    page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
    return {'lighting_changes_rgb':True,'streamlines_change_pixels':True,'toggle_state_survives_remount':True,
            'fields_and_camera_unchanged':True,'no_native_requests':True,'restored_image_matches':True,
            'physical_speed_gradient_supported':True}


def legend_below_canvas(page):
    canvas=page.locator('#scene canvas').bounding_box()
    legend=page.locator('#plot-legend').bounding_box()
    require(canvas is not None and legend is not None and legend['y']>=canvas['y']+canvas['height']-.5,
            'flow legend remains outside the computed volume canvas')
    require(legend['x']>=0 and legend['x']+legend['width']<=page.viewport_size['width']+.5,
            'physical speed legend fits the reading viewport')


def canvas_pixels(page):
    """Read a lossless actual rendered canvas capture; used only for comparison."""
    from io import BytesIO
    from PIL import Image
    return Image.open(BytesIO(page.locator('#scene canvas').screenshot(type='png'))).convert('RGB')


def pixel_difference(first,second):
    from PIL import ImageChops, ImageStat
    require(first.size == second.size, 'compared canvas dimensions')
    return sum(ImageStat.Stat(ImageChops.difference(first,second)).mean)/3


def readout(page,state):
    text=page.locator('#flow-time').inner_text()
    require(str(state['accepted_steps']) in page.locator('#flow-progress').inner_text(), 'visible accepted step count')
    require(any(unit in text for unit in (' s','seconds')), 'visible physical time unit')
    values=page.locator('.flow-diagnostic-values').evaluate("node=>Object.fromEntries([...node.querySelectorAll('dt')].map(dt=>[dt.textContent,dt.nextElementSibling.textContent]))")
    report=snapshot(page)['reference']['lastReport']
    close(float(text.split()[0]),state['time'],'visible rounded physical time',absolute=.00051)
    for label,key in [('Accepted dt','dt'),('Stored divergence max','actual_divergence_max'),
                      ('Courant number','courant'),('Kinetic energy','kinetic_energy'),('Tracer integral','tracer_integral')]:
        close(float(values[label].split()[0]),report[key],'visible '+label,tolerance=5.1e-5)
    close(float(values['Force-stage work'].split()[0]),report['forces']['applied_work'],'visible native force work',tolerance=5.1e-5)
    close(float(values['Pressure residual max'].split()[0]),report['pressure']['true_residual_max'],'visible pressure residual',tolerance=5.1e-5)
    require(int(values['Pressure iterations']) == report['pressure']['iterations'],'visible pressure iterations')
    for value,expected in zip(values['Tracer range'].split(' … '),(min(state['tracer']),max(state['tracer']))):
        close(float(value),expected,'visible absolute tracer range',tolerance=5.1e-5)
    for value,expected in zip(values['Maximum |u / v / w|'].removesuffix(' m/s').split(' / '),state['metrics']['max_u_v_w']):
        close(float(value),expected,'visible 3D component maxima',tolerance=5.1e-5)
    cutoff=state['tracer_integral_at_source_cutoff']
    close(float(values['Change since source stopped'].removesuffix('%')),100*(report['tracer_integral']/cutoff-1),
          'visible measured source-off integral loss',tolerance=5.1e-5)
    return values


def inspector_lifecycle(page):
    """Moving shared evidence into the inspector must preserve DOM ownership."""
    before=wait_ready(page)
    before_requests=requests(page)
    for notes_open,sources_open in ((False,True),(True,False)):
        page.locator('#lab').select_option('cap');settle(page)
        page.evaluate('''([notesOpen,sourcesOpen])=>{
          document.querySelector('.example-notes').open=notesOpen;
          document.querySelector('.example-sources').open=sourcesOpen;
          window.__flowBorrowedEvidence=['#lab-description','.example-notes','.example-sources','#chapter-target'].map(selector=>{
            const node=document.querySelector(selector);
            return {selector,node,parent:node.parentNode,next:node.nextSibling,
                    open:node instanceof HTMLDetailsElement?node.open:null};
          });
          const legend=document.querySelector('#plot-legend');
          window.__flowLegendHome={node:legend,parent:legend.parentNode,next:legend.nextSibling};
        }''',[notes_open,sources_open])
        page.locator('#lab').select_option('flow')
        active=wait_ready(page)
        state_equal(active['reference']['state'],before['reference']['state'],'inspector remount preserves native fields')
        require(page.evaluate('''()=>{
          const details=document.querySelector('#flow-diagnostics');
          return !details.open && window.__flowBorrowedEvidence.every(({selector,node})=>
            document.querySelectorAll(selector).length===1 && document.querySelector(selector)===node &&
            details.contains(node) && (!(node instanceof HTMLDetailsElement)||node.open)) &&
            document.querySelector('#flow-time').closest('.flow-actions') &&
            document.querySelector('#flow-progress').closest('.flow-actions') &&
            document.querySelectorAll('.flow-actions button').length===2 &&
            document.querySelector('#metrics').children.length===0 &&
            document.querySelector('#plot-legend')===window.__flowLegendHome.node &&
            document.querySelector('#scene').nextElementSibling===window.__flowLegendHome.node &&
            document.querySelector('#flow-sources').hidden===false &&
            ['reference','transport','timestep'].every(kind=>document.querySelector('#'+kind+'-sources').hidden);
        }'''),'compact inspector retains unique shared evidence and primary controls')
        legend_below_canvas(page)
        page.locator('#flow-diagnostics > summary').click()
        for selector in ('#lab-description','#lab-expected','#lab-limit','#chapter-target','#flow-sources'):
            require(page.locator(selector).is_visible(),'inspector exposes '+selector)
        require('14-guidance-rendering' in page.locator('#chapter-target').get_attribute('href'),'flow mechanism citation destination')
        page.locator('#lab').select_option('cap');settle(page)
        require(page.evaluate('''()=>window.__flowBorrowedEvidence.every(({selector,node,parent,next,open})=>
          document.querySelectorAll(selector).length===1 && document.querySelector(selector)===node &&
          node.parentNode===parent && node.nextSibling===next &&
          !node.classList.contains('flow-inspector-content') &&
          (!(node instanceof HTMLDetailsElement)||node.open===open)) &&
          document.querySelectorAll('#flow-diagnostics,.flow-inspector-content').length===0 &&
          document.querySelector('#plot-legend')===window.__flowLegendHome.node &&
          window.__flowLegendHome.node.parentNode===window.__flowLegendHome.parent &&
          window.__flowLegendHome.node.nextSibling===window.__flowLegendHome.next &&
          document.querySelector('#reference-sources').hidden===false &&
          document.querySelector('#flow-sources').hidden===true'''),'other lesson restores evidence nodes, positions and disclosure states')
        require('23-wetting-and-adhesion' in page.locator('#chapter-target').get_attribute('href'),'restored cap mechanism destination')
    page.locator('#lab').select_option('flow')
    after=wait_ready(page)
    state_equal(after['reference']['state'],before['reference']['state'],'inspector round trips preserve accepted owner')
    require(requests(page)==before_requests,'inspector and cross-lesson navigation never submit native commands')
    page.evaluate('delete window.__flowBorrowedEvidence; delete window.__flowLegendHome')
    return {'round_trips':2,'node_identity_and_positions':True,'disclosure_states_restored':True,
            'citations_and_limits_visible':True,'primary_controls_compact':True,'no_native_requests':True,
            'legend_identity_and_parent_restored':True}


def qualify(page,mobile,load,check_errors,artifact_dir=None):
    """Run both actual UI methods, inspect every worker state, then test lifecycle."""
    initial_sources=source_hashes()
    screenshots=[]
    capture_states=[]
    responses=[]
    contexts=list(dict.fromkeys([page.context,mobile.context]))
    def response_received(response):
        if response.url.endswith('/solver/rheon_solver_bg.wasm'):
            responses.append({'url':response.url,'status':response.status})
    for context in contexts: context.on('response',response_received)
    for target in (page,mobile): target.add_init_script(OBSERVER)
    def picture(name,target=page):
        if artifact_dir is not None:
            destination=evidence_path(artifact_dir,name)
            target.screenshot(path=str(destination),type='jpeg',quality=85,full_page=False)
            screenshots.append(str(destination))
            observed=snapshot(target);reference=observed['reference'];state=reference['state']
            require(not reference['busy'] and not reference['running'],'capture shows one paused accepted state')
            packed=b''.join(struct.pack('<I',bit) for name in FIELDS for bit in state['bits'][name])
            capture_states.append({'path':str(destination),'sha256':hashlib.sha256(destination.read_bytes()).hexdigest(),
                'method':state['method'],'accepted_steps':state['accepted_steps'],'time':state['time'],
                'generation':state['generation'],'field_sha256':hashlib.sha256(packed).hexdigest(),
                'camera':observed['camera'],'viewport':target.viewport_size})
    try:
        load(page,'flow-lab.html')
        initial=wait_ready(page)
        root_url=page.evaluate("new URL('.', [...document.scripts].find(s=>/\\/reader\\.js$/.test(s.src)).src).href")
        def fetch(path):
            response=page.request.get(urljoin(root_url,path),timeout=120000)
            require(response.ok,'served flow source/artifact '+path)
            return response.body()
        served={name:hashlib.sha256(fetch(name)).hexdigest() for name in SOURCE_PATHS}
        require(served == initial_sources,'served flow modules/native source differ from reviewed bytes')
        if initial['reference'].get('included') is False:
            require(all(page.locator('#'+name).is_disabled() for name in ('flow-run','flow-reset','flow-method')),
                    'source-only controls disabled')
            require(initial['reference'].get('state') is None and not page.workers,
                    'source-only build must not fabricate native fields')
            require('no compiled flow solver' in page.locator('#flow-status').inner_text().lower(),'missing packet explained')
            check_errors()
            return {'schema':SCHEMA,'included':False,'source_sha256':served,'packet_hashes':{},
                    'fluid_advances':0,'numerical_states':0,'unavailable_controls_disabled':True,
                    'no_synthetic_state':True,'screenshots':[]}
        packet={name:fetch('solver/'+name) for name in PACKET_FILES}
        hashes={name:hashlib.sha256(data).hexdigest() for name,data in packet.items()}
        receipt=json.loads(packet['build-receipt.json'])
        for name in PACKET_FILES[:-1]:
            require(receipt.get('artifact_sha256',{}).get(name) == hashes[name], 'actual solver artifact hash '+name)
        blob=packet['native-flow-fields.bin']
        host=check_native_reference(json.loads(packet['native-flow-reference.json']),blob)
        require(responses and all(response['status'] == 200 for response in responses),'actual WASM asset loaded in worker')
        rendering=render_oracles(page)
        fields=field_oracles(page)
        numerical_states=0
        fluid_advances=0
        durations=[]
        method_results=[]
        derived_curves=[]
        def consume(target,method,*,initialise=False):
            nonlocal numerical_states,fluid_advances
            accepted=[]
            while True:
                message=observer_message(target)
                if message is None: break
                require(not message.get('error'),'observed native worker error '+str(message.get('error')))
                require(message.get('runtime') == 'wasm-worker','actual accepted worker runtime')
                state=message.get('state')
                if not state: continue
                require(state['method'] == method,'observed requested pressure method')
                if message.get('outcome'):
                    outcome=message['outcome'];outcome['state']=state
                    check_outcome(outcome)
                    step=state['accepted_steps']
                    require(outcome['status'] == 'accepted','UI run unexpectedly rejected')
                    expected=decode_frame(host[method]['frames'][step],blob)
                    state_equal(state,expected,'actual browser accepted full tensor')
                    require(outcome['report'] == host[method]['frames'][step]['report'], 'browser report matches native report')
                    fluid_advances+=1;accepted.append(step);durations.append(message['durationMs'])
                else:
                    check_state(state)
                    require(state['accepted_steps'] == 0,'configure starts at rest')
                    state_equal(state,decode_frame(host[method]['frames'][0],blob),'browser configured state')
                numerical_states+=1
            check_errors()
            return accepted
        for method in METHODS:
            if snapshot(page)['reference']['state']['method'] != method:
                page.locator('#flow-diagnostics').evaluate('node=>node.open=true')
                page.locator('#flow-method').select_option(method)
                wait_ready(page)
            consume(page,method,initialise=True)
            before=wait_ready(page)
            require(before['reference']['state']['accepted_steps'] == 0,'explicit method run starts at rest')
            rest_pixels=canvas_pixels(page)
            page.locator('#flow-run').click()
            page.wait_for_function('window.__flowEvidence.messages.some(m=>m.state?.accepted_steps>=2)',timeout=120000)
            page.locator('#flow-run').click()
            paused=wait_ready(page)
            require(2 <= paused['reference']['state']['accepted_steps'] < MAX_STEPS,'pause between actual accepted timesteps')
            observed_steps=consume(page,method)
            stable_requests=requests(page)
            settle(page);settle(page)
            require(requests(page) == stable_requests,'pause stops new native requests')
            page.evaluate('window.__flowEvidence.pauseAt=30')
            page.locator('#flow-run').click()
            pulse_captured=False
            deadline=time.monotonic()+600
            while True:
                observed_steps.extend(consume(page,method))
                status=page.evaluate(r'''async()=>{
                  const reader=[...document.scripts].find(s=>/\/reader\.js$/.test(s.src));
                  const r=(await import(new URL('labs.js',reader.src))).snapshot().reference;
                  return {busy:r.busy,running:r.running,steps:r.state?.accepted_steps,error:r.error};
                }''')
                require(not status['error'],'native run error '+str(status['error']))
                if status['steps']==SOURCE_STEPS and not status['busy'] and not status['running'] and not pulse_captured:
                    observed_steps.extend(consume(page,method))
                    pulse=wait_ready(page)
                    require(pulse['reference']['state']['accepted_steps']==SOURCE_STEPS,'exact accepted source cutoff capture')
                    page.locator('#flow-diagnostics').evaluate('node=>node.open=false')
                    page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
                    picture('flow-'+method+'-source-cutoff.jpg')
                    pulse_captured=True;page.locator('#flow-run').click()
                if status['steps'] == MAX_STEPS and not status['busy'] and not status['running']:
                    observed_steps.extend(consume(page,method));break
                require(time.monotonic()<deadline,'bounded native run exceeded ten-minute qualification budget')
                page.wait_for_timeout(10)
            require(pulse_captured,'actual source cutoff was captured at step30')
            require(observed_steps == list(range(1,MAX_STEPS+1)),'every actual accepted worker state captured in order')
            final=wait_ready(page)
            check_renderer(final)
            derived=check_state(final['reference']['state'],report=final['reference']['lastReport'])
            require(pixel_difference(rest_pixels,canvas_pixels(page)) > .15,'actual computed tracer changes visible canvas pixels')
            require(page.locator('#flow-run').is_disabled(),'completed 80-step run cannot secretly continue')
            readout(page,final['reference']['state'])
            actual_display=derived_snapshot(page)['reference']
            derived_curves.append({'method':method,**check_native_curves(actual_display['state'],actual_display['renderer'])})
            method_results.append({'method':method,'accepted_states':MAX_STEPS,'final_time':final['reference']['state']['time'],
                 'final_divergence_max':derived['actual_divergence_max'],'final_component_energy':derived['component_energy'],
                 'final_tracer_axis_rms':derived['tracer_axis_rms'],
                 'source_off_integral_change':final['reference']['state']['metrics']['tracer_integral_change_since_cutoff']})
        stable=wait_ready(page)
        stable_requests=requests(page)
        display=display_controls(page)
        page.locator('#flow-diagnostics').evaluate('node=>node.open=false')
        page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
        picture('flow-desktop-accepted-front.jpg')
        default_framing=[check_default_framing(page)]
        rendered_before=canvas_pixels(page)
        scene=page.locator('#scene canvas').bounding_box()
        require(scene['width']>=290 and scene['height']>=280,'large desktop 3D canvas')
        page.mouse.move(scene['x']+scene['width']*.4,scene['y']+scene['height']*.45)
        page.mouse.down();page.mouse.move(scene['x']+scene['width']*.68,scene['y']+scene['height']*.6,steps=8);page.mouse.up()
        camera=settled_camera(page)
        require(math.dist(stable['camera'],camera['camera'])>.05,'manual 3D camera rotates')
        require(pixel_difference(rendered_before,canvas_pixels(page))>.15,'actual volume view changes with camera depth')
        state_equal(camera['reference']['state'],stable['reference']['state'],'camera preserves accepted fields')
        for anchor in ('flow','flow-depth','flow-limits'):
            scroll_to(page,'#'+anchor)
            observed=wait_ready(page)
            state_equal(observed['reference']['state'],stable['reference']['state'],'reading scroll preserves state')
            same_camera(camera,observed,'flow scrolling preserves manual camera')
        require(requests(page) == stable_requests,'camera and scroll do not submit solver commands')
        page.locator('#follow-reading').click()
        for anchor in ('flow','flow-depth','flow-limits'):
            scroll_to(page,'#'+anchor)
            observed=wait_ready(page)
            state_equal(observed['reference']['state'],stable['reference']['state'],'following preserves accepted state')
            require(page.locator('#'+anchor).evaluate('node=>node.classList.contains("current-lesson")'),'flow stop follows reading '+anchor)
            same_camera(camera,observed,'flow following preserves manual camera')
        no_overflow(page,'3D desktop book')
        page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
        picture('flow-desktop-accepted.jpg')
        accepted_mobile=[]
        for width in (390,320):
            page.set_viewport_size({'width':width,'height':844})
            page.locator('button[data-reading-mode=example]').click();settle(page)
            page.locator('#reset-view').click()
            default_framing.append(check_default_framing(page))
            visible=wait_ready(page)
            state_equal(visible['reference']['state'],stable['reference']['state'],'responsive nonzero example')
            check_renderer(visible)
            legend_below_canvas(page)
            no_overflow(page,'responsive accepted volume example')
            canvas=page.locator('#scene canvas').bounding_box()
            require(canvas['width']>=width-56 and canvas['height']>=190,'responsive computed-volume canvas')
            picture(f'flow-phone-{width}-accepted-example.jpg')
            page.locator('#scene').press('ArrowRight')
            moved=settled_camera(page)
            require(math.dist(visible['camera'],moved['camera'])>.01,'responsive volume camera works')
            state_equal(moved['reference']['state'],stable['reference']['state'],'responsive camera preserves accepted field')
            page.locator('button[data-reading-mode=split]').click();scroll_to(page,'#flow-depth')
            split=wait_ready(page)
            state_equal(split['reference']['state'],stable['reference']['state'],'responsive nonzero Split')
            same_camera(moved,split,'responsive Split keeps camera')
            require(page.locator('.reading-text').is_visible() and page.locator('.example-workspace').is_visible(),
                    'nonzero phone Split displays text and volume')
            no_overflow(page,'responsive accepted volume split')
            picture(f'flow-phone-{width}-accepted-split.jpg')
            require(requests(page)==stable_requests,'responsive camera and modes submit no native requests')
            accepted_mobile.append({'width':width,'accepted_steps':80,'method':METHODS[-1],
                'full_fields_unchanged':True,'camera_works':True,'example_and_split':True,'no_new_requests':True,
                'desktop_context_viewport_emulation':True,'legend_separate_from_canvas':True})
        page.set_viewport_size({'width':1440,'height':1050})
        page.locator('button[data-reading-mode=text]').evaluate('node=>node.click()');settle(page)
        # A same-option change remounts the actual lesson volume while preserving
        # its nonzero owner. Chapter navigation below tests a different lesson.
        page.locator('#lab').dispatch_event('change')
        remounted=wait_ready(page)
        state_equal(remounted['reference']['state'],stable['reference']['state'],'remount preserves nonzero owned state')
        require(page.evaluate('window.__flowEvidence.workers') == 1,'one persistent flow worker')
        page.locator('#flow-reset').click()
        reset=wait_ready(page)
        state_equal(reset['reference']['state'],decode_frame(host[METHODS[-1]]['reset'],blob),'UI native reset')
        require(not reset['reference']['running'],'native reset stays paused')
        chapter='chapters/14-guidance-rendering.html'
        load(page,chapter+'#surface-and-volume-outputs')
        chapter_state=wait_ready(page)
        check_state(chapter_state['reference']['state'])
        page.locator('#lab').select_option('projection');settle(page)
        page.locator('#lab').select_option('flow')
        remounted=wait_ready(page)
        state_equal(remounted['reference']['state'],chapter_state['reference']['state'],'cross-lesson remount preserves owner')
        require(page.evaluate('window.__flowEvidence.workers')==1,'cross-lesson selection keeps one native owner')
        inspector=inspector_lifecycle(page)
        page.locator('#follow-reading').click()
        for anchor in ('surface-and-volume-outputs','metrics-and-integration'):
            scroll_to(page,'#'+anchor)
            observed=wait_ready(page)
            state_equal(observed['reference']['state'],chapter_state['reference']['state'],'chapter scroll preserves native state')
            require(page.locator('#'+anchor).evaluate('node=>node.classList.contains("current-lesson")'),'chapter reading stop '+anchor)
        for path in ['flow-lab.html#'+anchor for anchor in ('flow','flow-depth','flow-limits')]+[
                chapter+'#surface-and-volume-outputs',chapter+'#metrics-and-integration']:
            load(page,path)
            linked=wait_ready(page)
            require(linked['reference']['state']['accepted_steps']==0 and all(r['action']=='configure' for r in requests(page)),
                    'flow deep link never executes a timestep')
        mobile_results=[]
        for width in (390,320):
            mobile.set_viewport_size({'width':width,'height':844})
            load(mobile,'flow-lab.html#flow-depth')
            rest=wait_ready(mobile)
            require(mobile.locator('body').get_attribute('data-reading-mode') == 'text','phone readable default')
            require(mobile.locator('.reading-text').is_visible() and
                    mobile.locator('.reading-text').bounding_box()['width'] >= width-48,'phone text width')
            require(mobile.locator('.reading-text').evaluate('n=>parseFloat(getComputedStyle(n).fontSize)')>=16,'phone text size')
            no_overflow(mobile,'flow phone text')
            position=mobile.evaluate('scrollY')
            for mode in ('example','split','text'):
                mobile.locator('button[data-reading-mode='+mode+']').click();settle(mobile)
                observed=wait_ready(mobile)
                state_equal(observed['reference']['state'],rest['reference']['state'],'phone mode preserves native state')
                no_overflow(mobile,'flow phone '+mode)
            require(abs(mobile.evaluate('scrollY')-position)<=2,'phone text reading position restored')
            mobile.locator('button[data-reading-mode=example]').click()
            require(mobile.locator('#flow-run').is_visible() and mobile.locator('#flow-reset').is_visible(),'phone compact commands visible')
            mobile.locator('button[data-reading-mode=split]').click();scroll_to(mobile,'#flow-limits')
            require(mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),'phone split text and example')
            state_equal(wait_ready(mobile)['reference']['state'],rest['reference']['state'],'phone split scroll preserves state')
            require(all(request['action']=='configure' for request in requests(mobile)),'phone modes/scroll never step')
            picture(f'flow-phone-{width}-split.jpg',mobile)
            # Different documents force a fresh owner; a same-document hash
            # navigation intentionally preserves the reader's selected mode.
            load(mobile,chapter+'#metrics-and-integration')
            linked=wait_ready(mobile)
            require(linked['reference']['state']['accepted_steps']==0,'phone chapter link never steps')
            mobile_results.append({'width':width,'modes':['text','split','example'],'readable_text':True,
                'no_horizontal_overflow':True,'mode_preserves_state_and_scroll':True,'compact_controls':True})
        compact=[]
        for target,width in ((page,1440),(mobile,390)):
            target.set_viewport_size({'width':width,'height':1050 if width>500 else 844})
            load(target,'flow-lab.html?view=example')
            rest=wait_ready(target)
            require(target.locator('body').evaluate('node=>node.classList.contains("flow-focus")'),'compact example entry point')
            canvas=target.locator('#scene canvas').bounding_box()
            require(canvas['height']>=300 and canvas['width']>=(width*.7 if width<500 else 800),
                    'compact entry gives the real volume a large canvas')
            require(target.locator('#flow-run').is_visible() and target.locator('#flow-reset').is_visible(),'compact entry commands accessible')
            require(rest['reference']['state']['accepted_steps']==0 and all(r['action']=='configure' for r in requests(target)),
                    'compact URL never starts native simulation')
            no_overflow(target,'compact real 3D flow')
            legend_below_canvas(target)
            compact.append({'width':width,'large_canvas':True,'controls_accessible':True,'no_implicit_advance':True,
                            'legend_separate_from_canvas':True})
        require(source_hashes()==initial_sources,'flow sources changed during qualification')
        check_errors()
        require(fluid_advances == 160,'exactly both bounded actual UI trajectories')
        return {'schema':SCHEMA,'included':True,'fixture':FIXTURE,
            'source_sha256':served,'packet_hashes':hashes,'methods':list(METHODS),
            'native_reference_states':172,'numerical_states':numerical_states,'fluid_advances':fluid_advances,
            'accepted_states_per_method':80,'method_results':method_results,
            'full_field_and_bit_comparison':True,'independent_tensor_diagnostics':True,
            'genuine_three_axis_variation':True,'fresh_host_native_comparison':True,'actual_wasm_worker':True,
            'every_accepted_worker_state':True,'pause_and_reset':True,'persistent_single_worker':True,
            'camera_scroll_and_modes_do_not_advance':True,'actual_canvas_changes_with_computation_and_camera':True,
            'render_oracles':rendering,'field_oracles':fields,'derived_curves':derived_curves,'display_controls':display,
            'inspector_lifecycle':inspector,'default_framing':default_framing,'accepted_mobile':accepted_mobile,'capture_states':capture_states,'desktop_scroll_stops':3,'chapter_scroll_stops':2,'desktop_direct_links':5,'mobile':mobile_results,'compact':compact,
            'worker_roundtrip_ms':{'minimum':min(durations),'maximum':max(durations),'mean':sum(durations)/len(durations),
                                  'includes_transfer_and_ui_scheduling':True,'real_time_claimed':False},
            'screenshots':screenshots}
    finally:
        for context in contexts: context.remove_listener('response',response_received)


def validate_receipt(observed,included):
    try:
        _validate_receipt(observed,included)
    except (RuntimeError,KeyError,TypeError,ValueError) as error:
        raise ValueError(str(error)) from error


def _validate_receipt(observed,included):
    require(isinstance(observed,dict) and observed.get('schema')==SCHEMA, 'missing current 3D flow qualification receipt')
    require(observed.get('included') is included, 'flow receipt inclusion identity')
    require(observed.get('source_sha256')==source_hashes(), 'stale flow module/native-source bindings')
    require(isinstance(observed.get('screenshots'),list), 'flow screenshot list')
    if not included:
        require(observed.get('packet_hashes')=={}, 'source-only flow cannot claim native artifacts')
        for key in ('fluid_advances','numerical_states'):
            require(type(observed.get(key)) is int and observed[key]==0, 'source-only flow cannot claim '+key)
        for key in ('unavailable_controls_disabled','no_synthetic_state'):
            require(observed.get(key) is True, 'source-only flow check '+key)
        return
    hashes=observed.get('packet_hashes')
    require(isinstance(hashes,dict) and set(hashes)==set(PACKET_FILES), 'all actual served flow artifacts are bound')
    require(all(isinstance(value,str) and len(value)==64 and all(c in '0123456789abcdef' for c in value)
                for value in hashes.values()), 'valid flow artifact SHA256 values')
    require(observed.get('fixture')==FIXTURE and observed.get('methods')==list(METHODS), 'actual fixed 3D fixture and both methods')
    for key,expected in [('native_reference_states',172),('numerical_states',162),('fluid_advances',160),
                         ('accepted_states_per_method',80),('desktop_scroll_stops',3),('chapter_scroll_stops',2),('desktop_direct_links',5)]:
        require(type(observed.get(key)) is int and observed[key]==expected, 'complete bounded flow coverage '+key)
    for key in ('full_field_and_bit_comparison','independent_tensor_diagnostics','genuine_three_axis_variation',
                'fresh_host_native_comparison','actual_wasm_worker','every_accepted_worker_state','pause_and_reset',
                'persistent_single_worker','camera_scroll_and_modes_do_not_advance',
                'actual_canvas_changes_with_computation_and_camera'):
        require(observed.get(key) is True, 'flow qualification check '+key)
    methods=observed.get('method_results')
    require(isinstance(methods,list) and [r.get('method') for r in methods]==list(METHODS), 'both measured actual trajectories')
    for method in methods:
        require(method.get('accepted_states')==80, 'all eighty accepted method states')
        for key in ('final_time','final_divergence_max','source_off_integral_change'):
            require(type(method.get(key)) in (float,int) and math.isfinite(method[key]), 'finite measured flow result '+key)
        require(0<method['final_time']<=1.6 and 0<=method['final_divergence_max']<=1e-5, 'accepted final time/divergence')
        for key,lower in [('final_component_energy',1e-5),('final_tracer_axis_rms',1e-3)]:
            values=method.get(key)
            require(isinstance(values,list) and len(values)==3 and all(type(v) in (float,int) and math.isfinite(v) and v>lower for v in values),
                    'genuine three-axis measured result '+key)
    rendering=observed.get('render_oracles',{})
    for key,expected in [('affine_indexing_probes',5),('beer_lambert_probes',3),('gpu_alpha_probes',6),('extreme_coordinate_probes',2),
                         ('gpu_lighting_alpha_pairs',8),('gpu_lighting_gradient_probes',2)]:
        require(type(rendering.get(key)) is int and rendering[key]==expected, 'actual renderer analytic coverage '+key)
    require(rendering.get('actual_shader_alpha') is True and len(rendering.get('gpu_paths',[]))==2 and
            'manual-trilinear' in rendering['gpu_paths'], 'actual GPU physical-opacity checks and fallback')
    for key in ('lit_unlit_alpha_unchanged','rgb_matches_disclosed_depth_law'):
        require(rendering.get(key) is True,'actual GPU lighting coverage '+key)
    fields=observed.get('field_oracles',{})
    for key,expected in [('mac_queries',14),('sampler_paths',2),('optical_depth_values',96),('streamline_branches',20),('invalid_inputs_rejected',4)]:
        require(type(fields.get(key)) is int and fields[key]==expected,'independent display-field coverage '+key)
    for key in ('physical_speed_scale','input_fields_unchanged','pure_module_exercised_in_browser'):
        require(fields.get(key) is True,'independent display-field check '+key)
    errors=fields.get('rotation_endpoint_errors');ratios=fields.get('rotation_refinement_ratios')
    require(isinstance(errors,list) and len(errors)==3 and all(type(v) in (int,float) and math.isfinite(v) for v in errors) and
            0<errors[2]<errors[1]<errors[0] and errors[2]<1e-6,'measured analytical rotation endpoint convergence')
    require(isinstance(ratios,list) and len(ratios)==2,'both fourth-order refinement ratios')
    for i,ratio in enumerate(ratios):
        close(ratio,errors[i]/errors[i+1],'measured RK4 refinement ratio')
        require(10<ratio<24,'fourth-order RK4 convergence at matched arclength')
    derived=observed.get('derived_curves')
    require(isinstance(derived,list) and [row.get('method') for row in derived]==list(METHODS),'actual derived curves for both native methods')
    for row in derived:
        require(row.get('curves')==6 and row.get('branches')==12,'six bidirectional native-field curves')
        require(type(row.get('rk4_segments')) is int and 0<row['rk4_segments']<=1200 and
                row.get('speed_values')==row['rk4_segments']+12,'all actual native curve vertices/speeds checked')
        for key in ('accepted_state_identity','physical_units','independent_mac_reintegration'):
            require(row.get(key) is True,'actual native curve coverage '+key)
    display=observed.get('display_controls',{})
    for key in ('lighting_changes_rgb','streamlines_change_pixels','toggle_state_survives_remount',
                'fields_and_camera_unchanged','no_native_requests','restored_image_matches','physical_speed_gradient_supported'):
        require(display.get(key) is True,'actual display-control coverage '+key)
    inspector=observed.get('inspector_lifecycle',{})
    require(type(inspector.get('round_trips')) is int and inspector['round_trips']==2,'both inspector disclosure-state round trips')
    for key in ('node_identity_and_positions','disclosure_states_restored','citations_and_limits_visible',
                'primary_controls_compact','no_native_requests','legend_identity_and_parent_restored'):
        require(inspector.get(key) is True,'inspector lifecycle coverage '+key)
    mobile=observed.get('mobile')
    require(isinstance(mobile,list) and [r.get('width') for r in mobile]==[390,320], 'both phone widths')
    for row in mobile:
        require(row.get('modes')==['text','split','example'], 'all phone modes')
        for key in ('readable_text','no_horizontal_overflow','mode_preserves_state_and_scroll','compact_controls'):
            require(row.get(key) is True, 'flow phone coverage '+key)
    framing=observed.get('default_framing')
    require(isinstance(framing,list) and [r.get('width') for r in framing]==[1440,390,320],'default camera desktop and phone framing')
    for row in framing:
        require(row.get('all_eight_corners_visible') is True and type(row.get('max_abs_ndc')) in (float,int) and
                math.isfinite(row['max_abs_ndc']) and 0<row['max_abs_ndc']<=1,'all eight domain corners visible')
    nonzero=observed.get('accepted_mobile')
    require(isinstance(nonzero,list) and [r.get('width') for r in nonzero]==[390,320], 'both nonzero responsive viewports')
    for row in nonzero:
        require(row.get('accepted_steps')==80 and row.get('method')==METHODS[-1], 'actual accepted responsive volume')
        for key in ('full_fields_unchanged','camera_works','example_and_split','no_new_requests','desktop_context_viewport_emulation','legend_separate_from_canvas'):
            require(row.get(key) is True, 'accepted responsive volume coverage '+key)
    compact=observed.get('compact')
    require(isinstance(compact,list) and [r.get('width') for r in compact]==[1440,390], 'desktop and phone compact entry')
    for row in compact:
        for key in ('large_canvas','controls_accessible','no_implicit_advance','legend_separate_from_canvas'):
            require(row.get(key) is True, 'compact flow coverage '+key)
    duration=observed.get('worker_roundtrip_ms',{})
    for key in ('minimum','maximum','mean'):
        require(type(duration.get(key)) in (int,float) and math.isfinite(duration[key]) and duration[key]>0, 'measured worker duration '+key)
    require(duration['minimum']<=duration['mean']<=duration['maximum'] and
            duration.get('includes_transfer_and_ui_scheduling') is True and duration.get('real_time_claimed') is False,
            'worker duration scope, without a real-time claim')
