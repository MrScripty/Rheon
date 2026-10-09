#!/usr/bin/env python3
"""Independent wire-v1 lab for caller-declared rest; no PDE solve or oracle input.
The SHA references bind explicit files created here, not verified physical truth.
Outputs must be a fresh directory outside the repository; never relax the cap.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'research'))
from check_proof_audit import require_external_output

CAP = 16_000_000

def require(condition, message="oracle validation failed"):
    if not condition:
        raise ValueError(message)

class Wire:
    def __init__(self, data):
        self.data, self.pos = data, 0
    def take(self, n):
        require(self.pos + n <= len(self.data), "truncated wire")
        value = self.data[self.pos:self.pos+n]
        self.pos += n
        return value
    def u8(self):
        return self.take(1)[0]
    def u32(self):
        return struct.unpack('<I', self.take(4))[0]
    def u64(self):
        return struct.unpack('<Q', self.take(8))[0]
    def f64(self):
        return struct.unpack('<d', self.take(8))[0]
    def reference(self):
        return self.u64(), self.u64(), self.take(32).hex()
    def array(self, count, read):
        require(self.u64() == count, "incorrect represented array length")
        return [read() for _ in range(count)]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run(command, success=True):
    result = subprocess.run([str(v) for v in command], capture_output=True, text=True)
    if success:
        require(result.returncode == 0, result.stderr)
    else:
        require(result.returncode != 0, f"unexpected acceptance: {command}")
    return result

def decode(path, n, refs):
    raw = path.read_bytes()
    r = Wire(raw)
    require(r.take(8) == b'RHEONOS1' and r.u32() == 1)
    require([r.u64() for _ in range(3)] == [n]*3)
    h = 3 / n
    for expected in ([0.]*3, [h]*3, [1.]*3, [2.]*3):
        require([r.f64() for _ in range(3)] == expected)
    require((r.u64(), r.u64()) == (73, 1))
    tolerance = r.f64()
    require(math.isfinite(tolerance) and tolerance > 0)
    vertices = r.array(8, lambda: [r.f64() for _ in range(3)])
    require(vertices == [[2. if corner & (1 << d) else 1. for d in range(3)] for corner in range(8)])
    triangles = r.array(12, lambda: [r.u64() for _ in range(3)])
    require(triangles == [[0,2,3],[0,3,1],[4,5,7],[4,7,6],[0,1,5],[0,5,4],[2,6,7],[2,7,3],[0,4,6],[0,6,2],[1,3,7],[1,7,5]])
    require(r.u64() == 1)
    def coord(index, counts):
        return index % counts[0], (index // counts[0]) % counts[1], index // (counts[0]*counts[1])
    lo, hi = n//3, 2*n//3
    solid = [all(lo <= c < hi for c in coord(index, [n]*3)) for index in range(n**3)]
    require(r.array(n**3, r.f64) == [0. if s else h**3 for s in solid])
    lengths = []
    for axis in range(3):
        counts = [n]*3
        counts[axis] += 1
        length = math.prod(counts)
        lengths.append(length)
        expected = []
        for index in range(length):
            c = coord(index, counts)
            blocked = lo <= c[axis] <= hi and all(lo <= c[d] < hi for d in range(3) if d != axis)
            expected.append(0. if blocked else h*h)
        require(r.array(length, r.f64) == expected)
    require(r.array(n**3, r.u64) == [(2**64-1) if s else 0 for s in solid])
    metadata_start = r.pos
    require([r.u8() for _ in range(5)] == [1,2,1,0,0])
    require([r.f64() for _ in range(4)] == [1000.,0.001,0.,0.])
    require(r.u64() == 0)
    require(r.reference() == (1,0,refs[0]))
    require(r.reference() == (2,0,refs[1]))
    require(r.u8() == 1 and r.reference() == (3,0,refs[2]))
    require(r.u8() == 1 and r.reference() == (4,0,refs[3]))
    require((r.f64(), r.f64()) == (0.,0.))
    require(r.u8() == 1 and r.u8() == 9)
    require([r.u8() for _ in range(9)] == [0]*9)
    field_header = r.pos
    require([r.u64() for _ in range(3)] == lengths)
    values_start = r.pos
    require(r.take(sum(lengths)*8) == b'\0'*(sum(lengths)*8), "caller-declared rest payload mismatch")
    require(r.pos == len(raw), "trailing bytes")
    return dict(n=n, bytes=len(raw), sha256=sha(path), metadata_start=metadata_start,
                field_header=field_header, values_start=values_start, face_lengths=lengths,
                independent_geometry_arrays_match=True, all_inputs_file_bound=True,
                structural_qualification='Unqualified', errors='all_unknown', physical_dynamics_qualified=False)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--repo', required=True, type=Path)
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--out', required=True, type=Path)
    args = p.parse_args()
    repo, binary, out = args.repo.resolve(), args.binary.resolve(), args.out.resolve()
    out = require_external_output(out,repo)
    out.mkdir(parents=True, exist_ok=False)
    declarations = [
        'Explicit example problem: transient incompressible Stokes; SI; domain [0,3]^3; retained solid [1,2]^3; density 1000 kg/m^3; dynamic viscosity 0.001 Pa s; initial time 0 s. No physical solution is certified.\n',
        'Explicit boundary declaration: stationary no-slip retained solid; sealed free-slip outer walls. No independently measured boundary error is available.\n',
        'Explicit initial condition: every full-face velocity component is exactly 0 m/s at time 0 s; known rest. This is caller-specified input, not oracle substitution.\n',
        'Explicit forcing history: no body/external forcing on [0,0] s. No evolution is attempted.\n',
    ]
    names = ['problem', 'boundary', 'initial', 'forcing']
    refs = []
    for name, text in zip(names, declarations):
        file = out / (name+'.txt')
        file.write_text(text)
        refs.append(sha(file))
    cases = []
    for n in [3,6,12]:
        folder = out / f'n{n}'
        result = run([binary,folder,n,*refs])
        native = json.loads(result.stdout)
        (out/f'n{n}-native.json').write_text(json.dumps(native,indent=2)+'\n')
        require(native['cap_bytes'] == CAP)
        require(native['coexisting_example_payload_bound_bytes'] <= CAP)
        case = decode(folder/'state.rheon-os1',n,refs)
        require((folder/'state.rheon-os1').read_bytes() == (folder/'roundtrip.rheon-os1').read_bytes())
        require(json.loads(run([binary,'check',n,folder/'state.rheon-os1']).stdout)['structurally_valid'])
        cases.append(dict(independent=case,native=native,bitwise_roundtrip=True))
    original = (out/'n3/state.rheon-os1').read_bytes()
    locations = cases[0]['independent']
    m, header, values = (locations[k] for k in ['metadata_start','field_header','values_start'])
    mutated = []
    def challenge(name, data):
        file = out/(name+'.malformed-os1')
        file.write_bytes(data)
        result = run([binary,'check',3,file],False)
        mutated.append(dict(name=name,sha256=sha(file),native_exit=result.returncode,native_error=result.stderr.strip()))
    def change(name, offset, payload):
        data = bytearray(original)
        data[offset:offset+len(payload)] = payload
        challenge(name,data)
    for name,offset,payload in [
        ('certified_flag',m+3,b'\1'),('pressure_flag',m+4,b'\1'),
        ('missing_problem_digest',m+61,b'\0'*32),('nan_density',m+5,struct.pack('<d',math.nan)),
        ('uncovered_time',m+29,struct.pack('<d',1.)),('unknown_error_tag',m+257,b'\2'),
        ('unknown_model',m+1,b'\xff'),('wrong_spacing',60,struct.pack('<d',0.5)),
        ('inflated_face_count',header+16,struct.pack('<Q',2**64-1)),
        ('nonstationary_outer_velocity',values,struct.pack('<d',1.)),
        ('subnormal_velocity',values,struct.pack('<Q',1)),
    ]:
        change(name,offset,payload)
    for length in [0,8,12,m,header,values,len(original)-1]:
        challenge('truncated_'+str(length),original[:length])
    challenge('trailing_byte',original+b'\0')
    # Missing CLI physical references must fail before output creation.
    missing = run([binary,out/'missing-inputs',3],False)
    require(not (out/'missing-inputs').exists())
    sources = ['src/obstacle_state.rs','src/lib.rs','src/aligned_strain.rs',
               'examples/owned_obstacle_state_checkpoint.rs','tests/owned_obstacle_state_contract.rs',
               'tools/check_owned_obstacle_state_checkpoint.py',
               'docs/research-book/implementation/owned-obstacle-state-foundation-20261009.md']
    report = dict(scope='opt-in owned unqualified snapshot foundation; no physical solve/step/pressure/coupling',
                  source_head=run(['git','-C',repo,'rev-parse','HEAD']).stdout.strip(),
                  sources={file:sha(repo/file) for file in sources},
                  binary_sha256=sha(binary),cases=cases,adversarial_native_refusals=mutated,
                  missing_physical_cli_inputs_refused=missing.returncode != 0,
                  all_checks_passed=True,physical_dynamics_qualified=False,accepted_step_authorized=False)
    (out/'checkpoint-report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    print(json.dumps({'report':str(out/'checkpoint-report.json'),'sha256':sha(out/'checkpoint-report.json'),
                      'cases':len(cases),'native_refusals':len(mutated),'all_checks_passed':True}))
if __name__ == '__main__':
    main()
