"""Frozen source/input checks only; no evaluator or ELF invocation."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib,json,re,struct
P=Path(__file__).resolve().parent;ROOT=P.parents[1];OLD=ROOT/'evidence/case41-paired-observation-v1'
def require(ok,msg):
    if not ok:raise ValueError(msg)
def run():
    policy=json.loads((P/'protocol.json').read_text())
    for f,h in policy['source_sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'frozen source '+f)
    original=(OLD/'native/candidate.rs').read_bytes()
    require((P/'native/candidate.rs').read_bytes()==original+b'\ninclude!("../forecast_probe.rs");\n','unchanged complete numerical source prefix')
    changed={'candidate.rs','harness.rs','library-overlay.rs','forecast_bits.rs'}
    for f in (P/'native').glob('*.rs'):
        if f.name not in changed:require(f.read_bytes()==(OLD/'native'/f.name).read_bytes(),'unchanged native '+f.name)
    require((P/'paired_probe.rs').read_bytes()==(OLD/'paired_probe.rs').read_bytes(),'same numerical boundary source')
    inputs=json.loads((P/'forecast-inputs.json').read_text());prediction=json.loads((ROOT/'evidence/case41-captured-linear-predictions-v1/analysis-80-normal.json').read_text())
    base=inputs['original_fixture']['final_authorized_unknowns']
    bits=lambda x:struct.pack('>d',x).hex()
    projected=[Q(x) for x in prediction['projection']['exact_displacement']]
    forecast=[float(Q(x)+Q(float(s))) for x,s in zip(base,projected)]
    require([bits(x) for x in forecast]==inputs['forecast_unknown_bits'],'exact prior rounded-coordinate forecast')
    require([bits(x) for x in base]==inputs['baseline_unknown_bits'],'original baseline coordinates')
    require([float(Q(y)-Q(x)) for x,y in zip(base,forecast)]==prediction['projection']['coordinate_lattice_increments'],'same recorded lattice increments')
    rust=[x.lower() for x in re.findall(r'0x([0-9a-fA-F]{16})',(P/'native/forecast_bits.rs').read_text())]
    require(rust==inputs['forecast_unknown_bits'],'all22 literal native forecast bits')
    require(all(x==0 or abs(x)>=2**-1022 for x in forecast),'fixed checked input normal range')
    source=(P/'forecast_probe.rs').read_text();harness=(P/'native/harness.rs').read_text()
    require('for observation in 0usize..2' in source and source.count('work.case41_paired_equation(')==1,'two calls through one unchanged evaluator site')
    require(source.index('assert_eq!(e.rate.map(f64::to_bits), FROZEN_E2_RATE_BITS)')<source.index('work.forecast_serialize('),'native bit assertion before serialization/next call')
    require('return;' in source[source.index('Err(error)'):],'refusal terminal')
    require('RHEON_CASE41_TWO_OBSERVATION_ALLOW' in harness and harness.index('RHEON_CASE41_TWO_OBSERVATION_ALLOW')<harness.index('candidate::case41_forecast_capture('),'guard dominates numerical test')
    require('case41_paired_candidate_roster' not in harness and 'fixed_capture_e2_experiment' not in harness,'closed new numerical test roster')
    require(not (P/'execution-authorization.json').exists() and not (P/'capture-before.json').exists() and not (P/'native.log').exists(),'no authorization/journal/numerical launch')
    return dict(status='PASS_EXACT_SOURCE_AND_SINGLE_FORECAST_INPUTS',original_equation_unchanged=True,original_fixture_unchanged=True,forecast_all22_bits_exact=True,baseline_bit_guard=True,evaluator_sites=1,proposed_observations=2,native_equations=0,controller_corrections=0,owner_advances=0,historical_memory_qualified=False)
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
