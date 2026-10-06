"""Build a test-only native copy and exact-bit inputs from the frozen packet."""
from pathlib import Path
import difflib
import hashlib
import json
import struct

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
original = (ROOT/'src/coupled_discrete.rs').read_text()
source = original.replace('struct Work {', "struct Work<'a> {\n    research: Option<&'a mut scalar::ChartWorkspace>,", 1)
source = source.replace('work: Work,', "work: Work<'static>,", 1).replace('impl Work {', "impl Work<'_> {", 1)
source = source.replace('pressure: [0.; P],', 'pressure: [0.; P],\n            research: None,', 1)
source = source.replace('let values = linear(block, rhs)?;', '''let values = if let Some(research) = self.research.as_deref_mut() {
            let values = research.solve(&p.d, &p.z)?;
            for i in 0..15 {
                if add(dot(&block[i], &values)?, -rhs[i])?.abs() > LIMIT {
                    return Err(CoupledDiscreteError::LinearFailure);
                }
            }
            values
        } else { linear(block, rhs)? };''', 1)
source = source.replace('#![allow(clippy::needless_range_loop)]', '#![allow(clippy::needless_range_loop, dead_code)]\n#[path="scalar.rs"] mod scalar;', 1)
source += '\ninclude!("native_tests.rs");\n'
(HERE/'native.rs').write_text(source)
(HERE/'native-transform.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True),source.splitlines(True),fromfile='frozen/src/coupled_discrete.rs',tofile='research/native.rs',n=0)))
packet = json.loads((ROOT/'evidence/forcing-increment-contract-supplement/selected-capture.json').read_text())
c, e = packet['input_row'], packet['fixed_native_candidate']
def rust(x):
    if isinstance(x, list): return '['+','.join(rust(y) for y in x)+']'
    return 'f64::from_bits(0x'+struct.pack('>d', float(x)).hex()+')'
types = {'Q0':'[f64;3]','ETA0':'[f64;6]','UNKNOWN0':'[f64;22]','OLD':'[[f64;3];16]','M0':'[f64;16]',
         'Z0':'[f64;22]','Z1':'[f64;22]','END_U':'[[f64;3];16]','END_M':'[f64;16]','RATE':'[f64;22]',
         'DIRECT':'[f64;22]','FORCE':'[[f64;3];16]','R_EXPECT':'[[[f64;22];2];16]',
         'D_EXPECT':'[[f64;22];24]','B_EXPECT':'[[f64;22];16]','PLUS':'[f64;40]','MINUS':'[f64;40]'}
data = dict(Q0=c['q'],ETA0=c['eta'],UNKNOWN0=c['unknown'],OLD=c['old'],M0=c['mass'],Z0=e['start_z'],Z1=e['end_z'],
            END_U=e['end_velocity'],END_M=e['end_mass'],RATE=e['rate'],DIRECT=e['direct_rate'],FORCE=e['end_force'],
            R_EXPECT=e['r'],D_EXPECT=e['end_d'],B_EXPECT=e['end_b'],PLUS=e['plus'],MINUS=e['minus'])
out='// Exact bit patterns from the pinned refused second step; no owner state mutation.\n'
for name, value in data.items():out += f'const {name}: {types[name]} = {rust(value)};\n'
out+='const H: f64 = '+rust(c['h'])+';\nconst EXPECTED_NORM: f64 = '+rust(c['expected_norm'])+';\n'
out+='const PAIRS: [[usize;2];40] = '+str(e['pairs']).replace(' ','')+';\n'
(HERE/'native_inputs.rs').write_text(out)
patch='\n#[cfg(test)]\n#[path = "../evidence/forcing-increment-comparison/native.rs"]\nmod research_increment_comparison;\n'
(HERE/'library-test-overlay.rs').write_text(patch)
(HERE/'baseline-binding.json').write_text(json.dumps(dict(base='704842c368953e47df279728935e20cfda89f0c5',
    production_sha256={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in ['src/lib.rs','src/coupled_discrete.rs','src/translated_viscous.rs','src/fitted_height.rs']},
    frozen_policy_sha256=hashlib.sha256((ROOT/'evidence/forcing-increment-contract-supplement/comparison-policy.json').read_bytes()).hexdigest(),
    selected_capture_sha256=hashlib.sha256((ROOT/'evidence/forcing-increment-contract-supplement/selected-capture.json').read_bytes()).hexdigest()),indent=2)+'\n')
